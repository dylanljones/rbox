// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Playlists, folders and their track occurrences in the standard DeviceSQL database.
//!
//! [Playlist] and [NewPlaylist] describe playlist-tree nodes in table 7.
//! [PlaylistContent] and [NewPlaylistContent] describe ordered track occurrences
//! in table 8. Queries do not load related rows automatically; repeated tracks
//! and position ties are retained.
//! Model insertions require unique `(playlist_id, content_id)` pairs; existing
//! duplicate binary rows remain readable for inspection and cleanup.
//! Updates validate changed parents. Deleting a folder removes its descendants
//! and their entries, while contents remain intact. Mutations commit in memory;
//! saving is explicit. Ordered mutations normalize affected sibling lists from zero
//! and playlist contents from one, writing only changed rows. Reads retain gaps/ties.
//!
//! ```
//! use rbox_devlib::{DatabaseKind, DeviceSqlConnection};
//! use rbox_devlib::models::{NewContent, NewPlaylist, NewPlaylistContent, Playlist};
//!
//! # fn main() -> rbox_devlib::Result<()> {
//! let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
//! NewPlaylist::folder(1, "Sets").insert(&mut conn)?;
//! NewPlaylist::playlist(2, "Warmup").parent_id(1).insert(&mut conn)?;
//! NewContent::new(10, "/Contents/track.wav").insert(&mut conn)?;
//! NewPlaylistContent::new(2, 10, None).insert(&mut conn)?;
//! assert_eq!(Playlist::get_contents(&mut conn, &2)?[0].id, 10);
//! Playlist::move_to(&mut conn, &2, Some(&0), Some(0))?;
//! // Changes are in memory; save the connection explicitly when ready.
//! # Ok(())
//! # }
//! ```

use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::{Read, Seek};

use super::ordering::{Change, Position};
use super::{
    ordering, read_fields, text, text_error, unsupported, validate_id, validate_nonzero_id, Content,
};
use crate::binary::{
    PageType, PlaylistEntryRow, PlaylistTreeRow, Row, RowRecord, StandardRow, TableKind,
    TrackHeader,
};
use crate::devicesql::{DeviceSqlConnection, Transaction};
use crate::{Error, Result};

/// Detached playlist tree node. Editing this snapshot does not persist changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistTreeNode {
    /// The playlist or folder and its original binary row.
    pub playlist: Playlist,
    /// Immediate children in sibling order.
    pub children: Vec<PlaylistTreeNode>,
}

/// A detached playlist or folder in the standard database's playlist tree.
///
/// IDs, positions and folder flags retain their stored values. Reads do not
/// traverse parents or load entries. Editing fields changes only this snapshot;
/// original bytes remain available through [`Self::raw_record`].
#[derive(Clone, PartialEq, Eq)]
pub struct Playlist {
    /// Export playlist/folder ID.
    pub id: u32,
    /// Parent folder ID; zero denotes the root.
    pub parent_id: u32,
    /// Stored sibling position; gaps and ties are retained.
    pub seq: u32,
    /// Raw folder flag; conventionally zero for playlists and nonzero for folders.
    pub attribute: u32,
    /// Decoded display name.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for Playlist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Playlist")
            .field("id", &self.id)
            .field("parent_id", &self.parent_id)
            .field("seq", &self.seq)
            .field("attribute", &self.attribute)
            .field("name", &self.name)
            .finish()
    }
}

impl Playlist {
    /// Standard database table containing playlist and folder nodes.
    pub const TABLE: TableKind = TableKind::Standard(PageType::PlaylistTree);
    /// First sibling position within each parent, including the root.
    pub const START_SEQ: u32 = 0;

    /// Stream live nodes in page-chain and slot order, decoding on demand.
    /// Creating the iterator does not read pages; malformed rows are not skipped.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live nodes in physical iteration order, without traversing the tree.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Find a playlist or folder by export ID using the engine's reusable index.
    /// Missing IDs return `None`; duplicate IDs are an error.
    pub fn find<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<Option<Self>> {
        let location = conn
            .index_rows(Self::TABLE, Self::read_id)?
            .get(id)
            .copied();
        location
            .map(|location| Self::try_from(conn.read_indexed_row(Self::TABLE, location)?))
            .transpose()
    }

    /// Check an ID without converting name frames to text.
    pub fn id_exists<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<bool> {
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(id))
    }

    /// Read immediate children sorted by stored sibling position; use zero for root.
    /// Ties retain physical iteration order. This scans the tree table and does not
    /// verify the parent exists, recurse, renumber positions or repair cycles.
    pub fn by_parent_id<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        parent_id: &u32,
    ) -> Result<Vec<Self>> {
        let mut children = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.parent_id == *parent_id {
                children.push(row);
            }
        }
        children.sort_by_key(|row| row.seq);
        Ok(children)
    }

    /// Return the stored flag; the root sentinel is treated as a folder.
    pub fn get_attribute<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<Option<u32>> {
        conn.check_kind(Self::TABLE)?;
        if *id == 0 {
            return Ok(Some(1));
        }
        Ok(Self::find(conn, id)?.map(|p| p.attribute))
    }

    /// Check whether an existing node is a regular playlist.
    pub fn is_playlist<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<bool> {
        Ok(Self::get_attribute(conn, id)? == Some(0))
    }

    /// Read contents in playlist order. Missing content references are errors;
    /// repeated occurrences in imported binary data remain repeated in the result.
    pub fn get_contents<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<Vec<Content>> {
        let entries = PlaylistContent::by_playlist_id(conn, id)?;
        let mut ids: Vec<_> = entries.iter().map(|e| e.content_id).collect();
        ids.sort_unstable();
        ids.dedup();
        let contents: HashMap<_, _> = Content::by_ids(conn, &ids)?
            .into_iter()
            .map(|c| (c.id, c))
            .collect();
        entries
            .into_iter()
            .map(|e| {
                contents
                    .get(&e.content_id)
                    .cloned()
                    .ok_or(Error::ModelNotFound {
                        table: Content::TABLE,
                        id: e.content_id,
                    })
            })
            .collect()
    }

    /// Find a node by exact path components. Empty, missing and ambiguous paths return `None`.
    /// Malformed tree ancestry or duplicate IDs return an error.
    pub fn find_by_path<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        path: &[&str],
    ) -> Result<Option<Self>> {
        let rows = Self::all(conn)?;
        validate_nodes(&rows)?;
        let mut parents = HashSet::from([0]);
        for name in path {
            parents = rows
                .iter()
                .filter(|r| parents.contains(&r.parent_id) && r.name == *name)
                .map(|r| r.id)
                .collect();
            if parents.is_empty() {
                return Ok(None);
            }
        }
        if path.is_empty() || parents.len() != 1 {
            return Ok(None);
        }
        Ok(rows.into_iter().find(|r| parents.contains(&r.id)))
    }

    /// Build a detached tree. Rejects duplicate IDs, missing/non-folder parents and
    /// cycles rather than silently dropping unreachable nodes. No contents are loaded.
    pub fn tree<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<Vec<PlaylistTreeNode>> {
        let rows = Self::all(conn)?;
        validate_nodes(&rows)?;
        // Build bottom-up without recursive traversal of untrusted ancestry.
        let mut depths = HashMap::from([(0, 0usize)]);
        let parents: HashMap<_, _> = rows.iter().map(|r| (r.id, r.parent_id)).collect();
        for row in &rows {
            let mut chain = Vec::new();
            let mut parent = row.id;
            while !depths.contains_key(&parent) {
                chain.push(parent);
                parent = parents[&parent];
            }
            let mut depth = depths[&parent];
            for id in chain.into_iter().rev() {
                depth += 1;
                depths.insert(id, depth);
            }
        }
        let mut rows = rows;
        rows.sort_by_key(|r| std::cmp::Reverse(depths[&r.id]));
        let mut groups: HashMap<u32, Vec<PlaylistTreeNode>> = HashMap::new();
        for row in rows {
            let mut children = groups.remove(&row.id).unwrap_or_default();
            children.sort_by_key(|n| n.playlist.seq);
            groups
                .entry(row.parent_id)
                .or_default()
                .push(PlaylistTreeNode {
                    playlist: row,
                    children,
                });
        }
        let mut roots = groups.remove(&0).unwrap_or_default();
        roots.sort_by_key(|n| n.playlist.seq);
        Ok(roots)
    }

    /// Rename an existing node, preserving its hierarchy, sequence and reserved data.
    pub fn rename<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
        name: &str,
    ) -> Result<usize> {
        let mut row = Self::find(conn, id)?.ok_or(Error::ModelNotFound {
            table: Self::TABLE,
            id: *id,
        })?;
        row.name = name.into();
        row.update(conn)?;
        Ok(1)
    }

    /// Count immediate children without decoding names.
    pub fn count_children<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        parent: &u32,
    ) -> Result<usize> {
        Ok(Self::get_seq_numbers(conn, parent)?.len())
    }

    /// Read stored sibling positions in ascending order without decoding names.
    pub fn get_seq_numbers<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        parent: &u32,
    ) -> Result<Vec<u32>> {
        let mut positions = Vec::new();
        for record in conn.iter_raw(Self::TABLE)? {
            let row = read_fields::<PlaylistTreeRow>(&record?)?;
            if row.parent_id == *parent {
                positions.push(row.position);
            }
        }
        positions.sort_unstable();
        Ok(positions)
    }

    /// Check contiguous zero-based sibling positions without changing the database.
    pub fn validate_seq_numbers<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        parent: &u32,
    ) -> Result<bool> {
        Ok(ordering::valid(
            &Self::get_seq_numbers(conn, parent)?,
            Self::START_SEQ,
        ))
    }

    /// Normalize one sibling list in stable sequence/physical order. Returns changed rows.
    pub fn reset_seq<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        parent: &u32,
    ) -> Result<usize> {
        let mut tx = conn.transaction();
        let rows = node_rows(&mut tx)?;
        validate_parent(&rows, *parent, &[])?;
        let changes = ordering::renumber(&node_positions(&rows), *parent, Self::START_SEQ)?;
        write_nodes(&mut tx, &rows, &changes)?;
        tx.commit()?;
        Ok(changes.len())
    }

    /// Move one node. `None` keeps the parent or appends within the destination.
    /// Positions are zero-based final positions; cycles and invalid folders are rejected.
    /// Returns one if the selected node changed parent/position, otherwise zero.
    pub fn move_to<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
        parent: Option<&u32>,
        seq: Option<u32>,
    ) -> Result<usize> {
        let mut tx = conn.transaction();
        let rows = node_rows(&mut tx)?;
        let index = node_index(&rows, *id)?;
        let count = move_nodes(
            &mut tx,
            &rows,
            &[*id],
            parent.copied().unwrap_or(rows[index].1.parent_id),
            seq,
        )?;
        tx.commit()?;
        Ok(count)
    }

    /// Move distinct nodes as a block in caller order. The destination is measured
    /// after removing selected siblings; `None` appends. The whole batch is atomic.
    /// Returns changed selected nodes, excluding shifts of unselected siblings.
    pub fn move_all_to<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        ids: &[u32],
        parent: &u32,
        seq: Option<u32>,
    ) -> Result<usize> {
        let mut tx = conn.transaction();
        let rows = node_rows(&mut tx)?;
        let count = move_nodes(&mut tx, &rows, ids, *parent, seq)?;
        tx.commit()?;
        Ok(count)
    }

    /// Interpret the conventional nonzero folder flag without changing its raw value.
    pub fn is_folder(&self) -> bool {
        self.attribute != 0
    }

    /// Persist edited name, parent and position; ID and attribute are immutable.
    ///
    /// Unedited fields, reserved values and the name frame are preserved.
    /// Edited fields use last-write-wins. A changed nonzero parent must exist and
    /// be a folder; self-parenting and cyclic or broken ancestry are rejected.
    /// Zero selects the root. Changed parent/position fields trigger an ordered move;
    /// changing only the parent appends there. Commits in memory only;
    /// errors roll back and saving remains explicit.
    /// String edits remain subject to the engine's supported allocation layouts;
    /// unsupported growth returns an error without discarding unknown data.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw)?;
        validate_id(Self::TABLE, self.id)?;
        for (field, changed) in [
            ("id", self.id != original.id),
            ("attribute", self.attribute != original.attribute),
        ] {
            if changed {
                return Err(Error::InvalidModelField {
                    table: Self::TABLE,
                    field,
                    reason: "existing playlist identity and folder status cannot be changed",
                });
            }
        }
        let mut tx = conn.transaction();
        if self.parent_id != original.parent_id || self.seq != original.seq {
            let rows = node_rows(&mut tx)?;
            let index = node_index(&rows, original.id)?;
            let parent = if self.parent_id != original.parent_id {
                self.parent_id
            } else {
                rows[index].1.parent_id
            };
            move_nodes(
                &mut tx,
                &rows,
                &[original.id],
                parent,
                (self.seq != original.seq).then_some(self.seq),
            )?;
        }
        let location = tx
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&original.id)
            .copied()
            .ok_or(Error::ModelNotFound {
                table: Self::TABLE,
                id: original.id,
            })?;
        let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
        let record = tx.read(&handle)?;
        let Row::Standard(StandardRow::PlaylistTree(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if self.name != original.name {
            row.name = text(&self.name)?;
        }
        let handle = tx.update(&handle, &Row::Standard(StandardRow::PlaylistTree(row)))?;
        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Delete a playlist/folder, its descendants and all their content associations.
    ///
    /// Descendants are followed by parent ID, even in malformed or cyclic trees;
    /// each row is removed once. Contents remain intact; surviving siblings are renumbered.
    /// Returns the total deleted row count. Missing IDs return zero without cleaning
    /// dangling associations; zero is rejected. One transaction covers both standard
    /// tables. Errors roll back; saving remains explicit.
    pub fn delete<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<usize> {
        Self::delete_nodes(conn, &[*id], false)
    }

    /// Delete selected subtrees atomically, counting overlapping descendants once.
    /// Duplicate input IDs or missing nodes reject the batch. Returns all deleted rows,
    /// including content associations, but excluding sibling sequence updates.
    pub fn delete_all<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        ids: &[u32],
    ) -> Result<usize> {
        Self::delete_nodes(conn, ids, true)
    }

    fn delete_nodes<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        ids: &[u32],
        strict: bool,
    ) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        let mut seen = HashSet::new();
        for &id in ids {
            validate_id(Self::TABLE, id)?;
            if !seen.insert(id) {
                return Err(ordering::invalid(
                    Self::TABLE,
                    "id",
                    "duplicate deletion identity",
                ));
            }
        }
        let mut tx = conn.transaction();
        let index = tx.index_rows(Self::TABLE, Self::read_id)?.clone();
        for id in ids {
            if !index.contains_key(id) {
                if strict {
                    return Err(Error::ModelNotFound {
                        table: Self::TABLE,
                        id: *id,
                    });
                }
                return Ok(0);
            }
        }
        let headers = playlist_rows(&mut tx)?;
        let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
        for header in headers.values() {
            children
                .entry(header.parent_id)
                .or_default()
                .push(header.id);
        }
        let mut removed = HashSet::new();
        let mut pending = ids.to_vec();
        while let Some(id) = pending.pop() {
            if removed.insert(id) {
                if let Some(ids) = children.get(&id) {
                    pending.extend(ids);
                }
            }
        }
        let parents: BTreeSet<_> = removed
            .iter()
            .map(|id| headers[id].parent_id)
            .filter(|parent| !removed.contains(parent))
            .collect();
        let mut count = remove_associations(&mut tx, |row| removed.contains(&row.playlist_id))?;
        for id in removed {
            let location = index.get(&id).ok_or(Error::ModelNotFound {
                table: Self::TABLE,
                id,
            })?;
            let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
            tx.delete(&handle)?;
            count = count.checked_add(1).ok_or(Error::MutationOverflow)?;
        }
        let rows = node_rows(&mut tx)?;
        let positions = node_positions(&rows);
        for parent in parents {
            write_nodes(
                &mut tx,
                &rows,
                &ordering::renumber(&positions, parent, Self::START_SEQ)?,
            )?;
        }
        tx.commit()?;
        Ok(count)
    }

    /// Original row window, including reserved fields and the source location.
    /// This snapshot is unchanged by public-field edits. Its location is not a
    /// stable mutation handle after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }

    fn read_id(record: &RowRecord) -> Result<u32> {
        if record.table() != Self::TABLE {
            return Err(unsupported(record));
        }
        Ok(read_fields::<PlaylistTreeRow>(record)?.id)
    }
}

fn invalid_parent(reason: &'static str) -> Error {
    Error::InvalidModelField {
        table: Playlist::TABLE,
        field: "parent_id",
        reason,
    }
}

fn playlist_rows<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
) -> Result<HashMap<u32, PlaylistTreeRow>> {
    let mut headers = HashMap::new();
    for record in tx.iter_raw(Playlist::TABLE)? {
        let record = record?;
        let id = Playlist::read_id(&record)?;
        if headers.insert(id, read_fields(&record)?).is_some() {
            return Err(Error::DuplicateId {
                table: Playlist::TABLE,
                id,
            });
        }
    }
    Ok(headers)
}

pub(super) fn remove_associations<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    matches: impl Fn(&PlaylistEntryRow) -> bool,
) -> Result<usize> {
    let mut locations = Vec::new();
    let mut parents = BTreeSet::new();
    for record in tx.iter_raw(PlaylistContent::TABLE)? {
        let record = record?;
        let Row::Standard(StandardRow::PlaylistEntry(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if matches(&row) {
            locations.push((record.page(), record.slot().slot));
            parents.insert(row.playlist_id);
        }
    }
    let count = locations.len();
    for (page, slot) in locations {
        let handle = tx.locate(PlaylistContent::TABLE, page, slot)?;
        tx.delete(&handle)?;
    }
    reset_entries(tx, &parents)?;
    Ok(count)
}

type NodeRows = Vec<(RowRecord, PlaylistTreeRow)>;
type EntryRows = Vec<(RowRecord, PlaylistEntryRow)>;

fn node_rows<R: Read + Seek>(tx: &mut Transaction<'_, R>) -> Result<NodeRows> {
    let mut ids = HashSet::new();
    tx.iter_raw(Playlist::TABLE)?
        .map(|record| {
            let record = record?;
            let row = read_fields::<PlaylistTreeRow>(&record)?;
            validate_id(Playlist::TABLE, row.id)?;
            if !ids.insert(row.id) {
                return Err(Error::DuplicateId {
                    table: Playlist::TABLE,
                    id: row.id,
                });
            }
            Ok((record, row))
        })
        .collect()
}

fn entry_rows<R: Read + Seek>(tx: &mut Transaction<'_, R>) -> Result<EntryRows> {
    tx.iter_raw(PlaylistContent::TABLE)?
        .map(|record| {
            let record = record?;
            let row = read_fields::<PlaylistEntryRow>(&record)?;
            Ok((record, row))
        })
        .collect()
}

fn node_positions(rows: &NodeRows) -> Vec<Position> {
    rows.iter()
        .map(|(_, r)| Position {
            parent: r.parent_id,
            seq: r.position,
        })
        .collect()
}

fn entry_positions(rows: &EntryRows) -> Vec<Position> {
    rows.iter()
        .map(|(_, r)| Position {
            parent: r.playlist_id,
            seq: r.position,
        })
        .collect()
}

fn node_index(rows: &NodeRows, id: u32) -> Result<usize> {
    rows.iter()
        .position(|(_, r)| r.id == id)
        .ok_or(Error::ModelNotFound {
            table: Playlist::TABLE,
            id,
        })
}

fn pair_conflict() -> Error {
    Error::UniqueConstraintViolation {
        table: PlaylistContent::TABLE,
        fields: &["playlist_id", "content_id"],
    }
}

fn entry_index(rows: &EntryRows, id: (u32, u32)) -> Result<usize> {
    let mut found = None;
    for (index, (_, row)) in rows.iter().enumerate() {
        if (row.playlist_id, row.track_id) == id {
            if found.replace(index).is_some() {
                return Err(pair_conflict());
            }
        }
    }
    found.ok_or(Error::ModelPairNotFound {
        table: PlaylistContent::TABLE,
        id,
    })
}

fn unique_entries(rows: &EntryRows, parent: u32) -> Result<()> {
    let mut ids = HashSet::new();
    for (_, row) in rows.iter().filter(|(_, r)| r.playlist_id == parent) {
        if !ids.insert(row.track_id) {
            return Err(pair_conflict());
        }
    }
    Ok(())
}

fn validate_parent(rows: &NodeRows, mut parent: u32, forbidden: &[u32]) -> Result<()> {
    let lookup: HashMap<_, _> = rows.iter().map(|(_, r)| (r.id, r)).collect();
    let mut visited: HashSet<_> = forbidden.iter().copied().collect();
    while parent != 0 {
        if !visited.insert(parent) {
            return Err(invalid_parent("parent ancestry must not contain a cycle"));
        }
        let node = lookup
            .get(&parent)
            .ok_or_else(|| invalid_parent("parent folder does not exist"))?;
        if node.folder_flag == 0 {
            return Err(invalid_parent("parent must be a folder"));
        }
        parent = node.parent_id;
    }
    Ok(())
}

fn validate_nodes(rows: &[Playlist]) -> Result<()> {
    let mut nodes = HashMap::new();
    for row in rows {
        validate_id(Playlist::TABLE, row.id)?;
        if nodes.insert(row.id, row).is_some() {
            return Err(Error::DuplicateId {
                table: Playlist::TABLE,
                id: row.id,
            });
        }
    }
    let mut completed = HashSet::from([0]);
    for row in rows {
        let mut current = row.id;
        let mut path = HashSet::new();
        while !completed.contains(&current) {
            if !path.insert(current) {
                return Err(invalid_parent("parent ancestry must not contain a cycle"));
            }
            let node = nodes[&current];
            current = node.parent_id;
            if current != 0 {
                let parent = nodes
                    .get(&current)
                    .ok_or_else(|| invalid_parent("parent folder does not exist"))?;
                if !parent.is_folder() {
                    return Err(invalid_parent("parent must be a folder"));
                }
            }
        }
        completed.extend(path);
    }
    Ok(())
}

fn validate_playlist<R: Read + Seek>(tx: &mut Transaction<'_, R>, id: u32) -> Result<()> {
    let rows = node_rows(tx)?;
    let index = node_index(&rows, id)?;
    if rows[index].1.folder_flag != 0 {
        return Err(ordering::invalid(
            PlaylistContent::TABLE,
            "playlist_id",
            "contents require a regular playlist",
        ));
    }
    validate_parent(&rows, rows[index].1.parent_id, &[id])
}

fn write_nodes<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    rows: &NodeRows,
    changes: &[Change],
) -> Result<()> {
    for change in changes {
        let (record, original) = &rows[change.index];
        let mut row = original.clone();
        row.parent_id = change.parent;
        row.position = change.seq;
        let handle = tx.locate(Playlist::TABLE, record.page(), record.slot().slot)?;
        tx.update(&handle, &Row::Standard(StandardRow::PlaylistTree(row)))?;
    }
    Ok(())
}

fn write_entries<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    rows: &EntryRows,
    changes: &[Change],
) -> Result<()> {
    for change in changes {
        let (record, original) = &rows[change.index];
        let mut row = original.clone();
        row.position = change.seq;
        let handle = tx.locate(PlaylistContent::TABLE, record.page(), record.slot().slot)?;
        tx.update(&handle, &Row::Standard(StandardRow::PlaylistEntry(row)))?;
    }
    Ok(())
}

fn move_nodes<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    rows: &NodeRows,
    ids: &[u32],
    parent: u32,
    seq: Option<u32>,
) -> Result<usize> {
    if ids.is_empty() {
        return Ok(0);
    }
    let selected = ids
        .iter()
        .map(|&id| node_index(rows, id))
        .collect::<Result<Vec<_>>>()?;
    validate_parent(rows, parent, ids)?;
    let changes = ordering::move_rows(
        Playlist::TABLE,
        &node_positions(rows),
        &selected,
        parent,
        seq,
        Playlist::START_SEQ,
    )?;
    let moved = changes
        .iter()
        .filter(|c| selected.contains(&c.index))
        .count();
    write_nodes(tx, rows, &changes)?;
    Ok(moved)
}

fn reset_entries<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    parents: &BTreeSet<u32>,
) -> Result<usize> {
    if parents.is_empty() {
        return Ok(0);
    }
    let rows = entry_rows(tx)?;
    let positions = entry_positions(&rows);
    let mut count = 0;
    for &parent in parents {
        let changes = ordering::renumber(&positions, parent, PlaylistContent::START_SEQ)?;
        count += changes.len();
        write_entries(tx, &rows, &changes)?;
    }
    Ok(count)
}

impl TryFrom<RowRecord> for Playlist {
    type Error = Error;

    /// Decode a playlist-tree row, retaining its original bytes.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::PlaylistTree(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            id: row.id,
            parent_id: row.parent_id,
            seq: row.position,
            attribute: row.folder_flag,
            name: row
                .name
                .text()
                .map_err(|source| text_error(&record, "name", source))?,
            raw: record,
        })
    }
}

/// Input for a fresh playlist or folder; no existing raw row is needed.
///
/// IDs are explicit. Model insertion validates the parent and maintains sibling
/// ordering; `None` appends. The reserved word defaults to zero and remains configurable.
/// Insertion commits in memory only and does not create entries or save to disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPlaylist {
    /// Unused, nonzero export playlist/folder ID.
    pub id: u32,
    /// Parent folder ID; zero denotes the root.
    pub parent_id: u32,
    /// Zero-based insertion position; `None` appends after the last sibling.
    pub seq: Option<u32>,
    /// Raw folder flag; constructors use zero for playlists and one for folders.
    pub attribute: u32,
    /// Display name.
    pub name: String,
    /// Reserved word at 0x04, defaulting to zero.
    pub unknown_04: u32,
}

impl NewPlaylist {
    /// Prepare a playlist with an explicit parent; use `None` for the position to append.
    pub fn new(id: u32, name: impl Into<String>, attribute: u32) -> Self {
        Self {
            id,
            parent_id: 0,
            seq: None,
            name: name.into(),
            attribute,
            unknown_04: 0,
        }
    }

    /// Prepare a regular playlist with an explicit ID and name.
    pub fn playlist(id: u32, name: impl Into<String>) -> Self {
        Self::new(id, name, 0)
    }

    /// Prepare a folder with an explicit ID and name.
    pub fn folder(id: u32, name: impl Into<String>) -> Self {
        Self::new(id, name, 1)
    }

    /// Set the insertion position among siblings; zero inserts first.
    pub fn seq(mut self, seq: u32) -> Self {
        self.seq = Some(seq);
        self
    }

    /// Builder for `parent_id`.
    pub fn parent_id(mut self, id: u32) -> Self {
        self.parent_id = id;
        self
    }

    /// Insert atomically, rejecting duplicate IDs and returning the engine-created row.
    /// Failures leave the connection unchanged; saving is explicit.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Playlist> {
        conn.check_kind(Playlist::TABLE)?;
        let mut row = self.to_binary()?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(Playlist::TABLE, Playlist::read_id)?
            .contains_key(&self.id)
        {
            return Err(Error::DuplicateId {
                table: Playlist::TABLE,
                id: self.id,
            });
        }
        let rows = node_rows(&mut tx)?;
        validate_parent(&rows, self.parent_id, &[self.id])?;
        let (position, changes) = ordering::insert(
            Playlist::TABLE,
            &node_positions(&rows),
            self.parent_id,
            self.seq,
            Playlist::START_SEQ,
        )?;
        write_nodes(&mut tx, &rows, &changes)?;
        row.position = position;
        let handle = tx.insert(
            Playlist::TABLE,
            &Row::Standard(StandardRow::PlaylistTree(row)),
        )?;
        let model = Playlist::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; the engine assigns placement.
    /// Direct binary insertions bypass uniqueness, parent and ordering checks.
    /// An omitted position encodes zero; append requires model insertion.
    pub fn to_binary(&self) -> Result<PlaylistTreeRow> {
        validate_id(Playlist::TABLE, self.id)?;
        Ok(PlaylistTreeRow {
            parent_id: self.parent_id,
            unknown_04: self.unknown_04,
            position: self.seq.unwrap_or(Playlist::START_SEQ),
            id: self.id,
            folder_flag: self.attribute,
            name: text(&self.name)?,
        })
    }
}

/// A detached track occurrence in a standard database playlist.
///
/// Entries have no unique scalar ID. Repeated tracks, identical occurrences and
/// position ties are retained. Editing fields changes only this snapshot; tracks
/// and playlist nodes are not loaded automatically.
#[derive(Clone, PartialEq, Eq)]
pub struct PlaylistContent {
    /// Export playlist ID.
    pub playlist_id: u32,
    /// Export track ID, distinct from the source/master content ID.
    pub content_id: u32,
    /// Stored position, commonly one-based; zero and gaps remain unmodified on reads.
    pub seq: u32,
    raw: RowRecord,
}

impl std::fmt::Debug for PlaylistContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaylistContent")
            .field("playlist_id", &self.playlist_id)
            .field("content_id", &self.content_id)
            .field("seq", &self.seq)
            .finish()
    }
}

impl PlaylistContent {
    /// Standard database table containing track occurrences.
    pub const TABLE: TableKind = TableKind::Standard(PageType::PlaylistEntries);
    /// First content position within a playlist.
    pub const START_SEQ: u32 = 1;

    /// Stream live occurrences in physical page-chain and slot order.
    /// Creating the iterator reads no pages. No deduplication or sorting occurs.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live occurrences in physical iteration order.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Read a playlist's occurrences sorted by stored position.
    /// Ties retain physical iteration order, including identical entries.
    /// This scans the entry table and does not load the playlist or referenced tracks.
    pub fn by_playlist_id<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        playlist_id: &u32,
    ) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.playlist_id == *playlist_id {
                entries.push(row);
            }
        }
        entries.sort_by_key(|row| row.seq);
        Ok(entries)
    }

    /// Read every association for a content ID, ordered by playlist then sequence.
    pub fn by_content_id<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<Vec<Self>> {
        let mut rows = Self::all(conn)?;
        rows.retain(|r| r.content_id == *id);
        rows.sort_by_key(|r| (r.playlist_id, r.seq));
        Ok(rows)
    }

    /// Find a unique `(playlist_id, content_id)` pair. Imported duplicates are errors.
    pub fn find<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &(u32, u32),
    ) -> Result<Option<Self>> {
        let mut found = None;
        for row in Self::iter(conn)? {
            let row = row?;
            if (row.playlist_id, row.content_id) == *id {
                if found.is_some() {
                    return Err(pair_conflict());
                }
                found = Some(row);
            }
        }
        Ok(found)
    }

    /// Check whether the unique playlist/content pair exists.
    pub fn id_exists<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &(u32, u32),
    ) -> Result<bool> {
        Ok(Self::find(conn, id)?.is_some())
    }

    /// Count live entries for a playlist without loading contents.
    pub fn count<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<usize> {
        Ok(Self::get_seq_numbers(conn, id)?.len())
    }

    /// Read the stored entry positions in ascending order.
    pub fn get_seq_numbers<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<Vec<u32>> {
        Ok(Self::by_playlist_id(conn, id)?
            .iter()
            .map(|r| r.seq)
            .collect())
    }

    /// Check contiguous one-based positions without changing imported values.
    pub fn validate_seq_numbers<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<bool> {
        Ok(ordering::valid(
            &Self::get_seq_numbers(conn, id)?,
            Self::START_SEQ,
        ))
    }

    /// Normalize positions in sequence/physical order, preserving all occurrences.
    /// Returns the number of changed rows. Duplicate pairs remain readable/repairable.
    pub fn reset_seq<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<usize> {
        let mut tx = conn.transaction();
        validate_playlist(&mut tx, *id)?;
        let count = reset_entries(&mut tx, &BTreeSet::from([*id]))?;
        tx.commit()?;
        Ok(count)
    }

    /// Move a unique pair within its playlist. Positions start at one; `None` appends.
    /// Returns one if its position changed, otherwise zero.
    pub fn move_to<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &(u32, u32),
        seq: Option<u32>,
    ) -> Result<usize> {
        Self::move_all_to(conn, &[*id], seq)
    }

    /// Move unique pairs from one playlist as a block in caller order. The destination
    /// is its first final position after removing selected entries; `None` appends.
    /// Returns changed selected entries, excluding shifts of unselected entries.
    pub fn move_all_to<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        ids: &[(u32, u32)],
        seq: Option<u32>,
    ) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        if ids.is_empty() {
            return Ok(0);
        }
        let parent = ids[0].0;
        if ids.iter().any(|id| id.0 != parent) {
            return Err(ordering::invalid(
                Self::TABLE,
                "playlist_id",
                "list moves require a single playlist",
            ));
        }
        let mut tx = conn.transaction();
        validate_playlist(&mut tx, parent)?;
        let rows = entry_rows(&mut tx)?;
        unique_entries(&rows, parent)?;
        let selected = ids
            .iter()
            .map(|id| entry_index(&rows, *id))
            .collect::<Result<Vec<_>>>()?;
        let changes = ordering::move_rows(
            Self::TABLE,
            &entry_positions(&rows),
            &selected,
            parent,
            seq,
            Self::START_SEQ,
        )?;
        let moved = changes
            .iter()
            .filter(|c| selected.contains(&c.index))
            .count();
        write_entries(&mut tx, &rows, &changes)?;
        tx.commit()?;
        Ok(moved)
    }

    /// Delete a unique pair, then close the gap. Missing pairs return zero.
    pub fn delete_by_id<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &(u32, u32),
    ) -> Result<usize> {
        match Self::find(conn, id)? {
            Some(row) => row.delete(conn),
            None => Ok(0),
        }
    }

    /// Delete distinct pairs atomically; missing or ambiguous pairs reject the batch.
    pub fn delete_all<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        ids: &[(u32, u32)],
    ) -> Result<usize> {
        let mut tx = conn.transaction();
        let rows = entry_rows(&mut tx)?;
        let mut selected = HashSet::new();
        for &id in ids {
            if !selected.insert(entry_index(&rows, id)?) {
                return Err(ordering::invalid(
                    Self::TABLE,
                    "id",
                    "duplicate deletion identity",
                ));
            }
        }
        let parents = selected.iter().map(|&i| rows[i].1.playlist_id).collect();
        for &i in &selected {
            let record = &rows[i].0;
            let handle = tx.locate(Self::TABLE, record.page(), record.slot().slot)?;
            tx.delete(&handle)?;
        }
        reset_entries(&mut tx, &parents)?;
        tx.commit()?;
        Ok(selected.len())
    }

    /// Remove all entries of a playlist, retaining the playlist and contents.
    pub fn delete_by_playlist_id<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<usize> {
        Self::delete_by_playlist_ids(conn, &[*id])
    }

    /// Remove all entries of the given playlists atomically; missing IDs are ignored.
    pub fn delete_by_playlist_ids<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        ids: &[u32],
    ) -> Result<usize> {
        let ids: HashSet<_> = ids.iter().copied().collect();
        let mut tx = conn.transaction();
        let count = remove_associations(&mut tx, |r| ids.contains(&r.playlist_id))?;
        tx.commit()?;
        Ok(count)
    }

    /// Remove every occurrence of a content ID and compact affected playlists.
    pub fn delete_by_content_id<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<usize> {
        Self::delete_by_content_ids(conn, &[*id])
    }

    /// Remove all occurrences of these content IDs atomically and compact affected lists.
    pub fn delete_by_content_ids<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        ids: &[u32],
    ) -> Result<usize> {
        let ids: HashSet<_> = ids.iter().copied().collect();
        let mut tx = conn.transaction();
        let count = remove_associations(&mut tx, |r| ids.contains(&r.track_id))?;
        tx.commit()?;
        Ok(count)
    }

    /// Delete this one original association, ignoring edits to public fields.
    ///
    /// Duplicate pairs are retained. Changed binary values or heap offsets return
    /// [`Error::StaleModelRow`]; missing locations return an engine error. Re-fetch
    /// after structural edits: an identical replacement at the same location cannot
    /// be distinguished. Remaining positions are normalized from one.
    /// Returns one on success; saving remains explicit.
    pub fn delete<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        let mut tx = conn.transaction();
        let handle = tx.locate(Self::TABLE, self.raw.page(), self.raw.slot().slot)?;
        let current = tx.read(&handle)?;
        if current.slot().heap_offset != self.raw.slot().heap_offset
            || current.decode()? != self.raw.decode()?
        {
            return Err(Error::StaleModelRow {
                table: Self::TABLE,
                page: self.raw.page(),
                slot: self.raw.slot().slot,
            });
        }
        tx.delete(&handle)?;
        let original = Self::try_from(self.raw)?;
        reset_entries(&mut tx, &BTreeSet::from([original.playlist_id]))?;
        tx.commit()?;
        Ok(1)
    }

    /// Remove every matching content/playlist pair, including duplicates.
    ///
    /// IDs must be nonzero; referenced rows need not exist, allowing dangling
    /// associations to be cleaned up. Returns the deleted count. Contents and playlists
    /// are retained. Errors roll back; changes remain in memory until saved.
    pub fn remove<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        content_id: &u32,
        playlist_id: &u32,
    ) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        validate_nonzero_id(Self::TABLE, "content_id", *content_id)?;
        validate_nonzero_id(Self::TABLE, "playlist_id", *playlist_id)?;
        let mut tx = conn.transaction();
        let count = remove_associations(&mut tx, |row| {
            row.track_id == *content_id && row.playlist_id == *playlist_id
        })?;
        tx.commit()?;
        Ok(count)
    }

    /// Original occurrence window and location, unaffected by public-field edits.
    /// The location distinguishes repeated occurrences in this snapshot but is not
    /// a stable mutation handle after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }
}

impl TryFrom<RowRecord> for PlaylistContent {
    type Error = Error;

    /// Decode an entry without discarding its original row window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::PlaylistEntry(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            playlist_id: row.playlist_id,
            content_id: row.track_id,
            seq: row.position,
            raw: record,
        })
    }
}

/// Input for a fresh playlist occurrence, without a synthetic ID or source row.
///
/// Insertion validates the playlist and content, rejects duplicate pairs, and
/// maintains one-based positions. `None` appends. Saving remains explicit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPlaylistContent {
    /// Existing regular playlist ID.
    pub playlist_id: u32,
    /// Nonzero export track ID.
    pub content_id: u32,
    /// One-based insertion position; `None` appends.
    pub seq: Option<u32>,
}

impl NewPlaylistContent {
    /// Prepare an occurrence with explicit playlist/track IDs; `None` appends.
    pub fn new(playlist_id: u32, content_id: u32, seq: impl Into<Option<u32>>) -> Self {
        Self {
            playlist_id,
            content_id,
            seq: seq.into(),
        }
    }

    /// Insert atomically, requiring a unique `(playlist_id, content_id)` pair.
    /// The same content may appear in different playlists. Affected positions are normalized.
    /// Returns the engine-created row and commits in memory only; saving is explicit.
    pub fn insert<R: Read + Seek>(
        self,
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<PlaylistContent> {
        conn.check_kind(PlaylistContent::TABLE)?;
        let mut row = self.to_binary()?;
        let mut tx = conn.transaction();
        for record in tx.iter_raw(PlaylistContent::TABLE)? {
            let record = record?;
            let existing = read_fields::<PlaylistEntryRow>(&record)?;
            if existing.playlist_id == self.playlist_id && existing.track_id == self.content_id {
                return Err(Error::UniqueConstraintViolation {
                    table: PlaylistContent::TABLE,
                    fields: &["playlist_id", "content_id"],
                });
            }
        }
        validate_playlist(&mut tx, self.playlist_id)?;
        if !tx
            .index_rows(Content::TABLE, |record| {
                Ok(read_fields::<TrackHeader>(record)?.id)
            })?
            .contains_key(&self.content_id)
        {
            return Err(Error::ModelNotFound {
                table: Content::TABLE,
                id: self.content_id,
            });
        }
        let rows = entry_rows(&mut tx)?;
        unique_entries(&rows, self.playlist_id)?;
        let (seq, changes) = ordering::insert(
            PlaylistContent::TABLE,
            &entry_positions(&rows),
            self.playlist_id,
            self.seq,
            PlaylistContent::START_SEQ,
        )?;
        write_entries(&mut tx, &rows, &changes)?;
        row.position = seq;
        let handle = tx.insert(
            PlaylistContent::TABLE,
            &Row::Standard(StandardRow::PlaylistEntry(row)),
        )?;
        let model = PlaylistContent::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode the binary fields for an explicit transaction; placement is assigned by the engine.
    /// Direct binary insertion bypasses uniqueness, reference and ordering checks.
    /// An omitted position encodes one; append requires model insertion.
    pub fn to_binary(&self) -> Result<PlaylistEntryRow> {
        validate_nonzero_id(PlaylistContent::TABLE, "playlist_id", self.playlist_id)?;
        validate_nonzero_id(PlaylistContent::TABLE, "content_id", self.content_id)?;
        Ok(PlaylistEntryRow {
            position: self.seq.unwrap_or(PlaylistContent::START_SEQ),
            track_id: self.content_id,
            playlist_id: self.playlist_id,
        })
    }
}
