// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Playlists, folders and their track occurrences in the standard DeviceSQL database.
//!
//! [Playlist] and [NewPlaylist] describe playlist-tree nodes in table 7.
//! [PlaylistContent] and [NewPlaylistContent] describe ordered track occurrences
//! in table 8. Queries do not load related rows automatically; repeated tracks
//! and position ties are retained.
//! Updates validate changed parents. Deleting a folder removes its descendants
//! and their entries, while contents remain intact. Mutations commit in memory;
//! saving is explicit and surviving sequence values are not renumbered.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};

use super::{read_fields, text, text_error, unsupported, validate_id, validate_nonzero_id};
use crate::binary::{
    PageType, PlaylistEntryRow, PlaylistTreeRow, Row, RowRecord, StandardRow, TableKind,
};
use crate::devicesql::{DeviceSqlConnection, Transaction};
use crate::{Error, Result};

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
    pub fn by_parent<R: Read + Seek>(
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

    /// Interpret the conventional nonzero folder flag without changing its raw value.
    pub fn is_folder(&self) -> bool {
        self.attribute != 0
    }

    /// Persist edited name, parent and position; ID and attribute are immutable.
    ///
    /// Unedited fields, reserved values and the name frame are preserved.
    /// Edited fields use last-write-wins. A changed nonzero parent must exist and
    /// be a folder; self-parenting and cyclic or broken ancestry are rejected.
    /// Zero selects the root. Siblings are not renumbered. Commits in memory only;
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
        if self.parent_id != original.parent_id {
            let headers = playlist_rows(&mut tx)?;
            let mut visited = HashSet::from([original.id]);
            let mut parent = self.parent_id;
            while parent != 0 {
                if !visited.insert(parent) {
                    return Err(invalid_parent("parent ancestry must not contain a cycle"));
                }
                let header = headers
                    .get(&parent)
                    .ok_or_else(|| invalid_parent("parent folder does not exist"))?;
                if header.folder_flag == 0 {
                    return Err(invalid_parent("parent must be a folder"));
                }
                parent = header.parent_id;
            }
            row.parent_id = self.parent_id;
        }
        if self.seq != original.seq {
            row.position = self.seq;
        }
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
    /// each row is removed once. Contents and surviving sequence values are untouched.
    /// Returns the total deleted row count. Missing IDs return zero without cleaning
    /// dangling associations; zero is rejected. One transaction covers both standard
    /// tables. Errors roll back; saving remains explicit.
    pub fn delete<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        validate_id(Self::TABLE, *id)?;
        let mut tx = conn.transaction();
        let index = tx.index_rows(Self::TABLE, Self::read_id)?.clone();
        if !index.contains_key(id) {
            return Ok(0);
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
        let mut pending = vec![*id];
        while let Some(id) = pending.pop() {
            if removed.insert(id) {
                if let Some(ids) = children.get(&id) {
                    pending.extend(ids);
                }
            }
        }
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

fn remove_associations<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    matches: impl Fn(&PlaylistEntryRow) -> bool,
) -> Result<usize> {
    let mut locations = Vec::new();
    for record in tx.iter_raw(PlaylistContent::TABLE)? {
        let record = record?;
        let Row::Standard(StandardRow::PlaylistEntry(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if matches(&row) {
            locations.push((record.page(), record.slot().slot));
        }
    }
    let count = locations.len();
    for (page, slot) in locations {
        let handle = tx.locate(PlaylistContent::TABLE, page, slot)?;
        tx.delete(&handle)?;
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
/// IDs and sibling positions are explicit. The reserved word defaults to the
/// observed zero authoring value and remains configurable. Parent IDs are stored
/// as supplied; hierarchy validation, traversal and automatic ordering are deferred.
/// Insertion commits in memory only and does not create entries or save to disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPlaylist {
    /// Unused, nonzero export playlist/folder ID.
    pub id: u32,
    /// Parent folder ID; zero denotes the root.
    pub parent_id: u32,
    /// Explicit sibling position; other nodes are not renumbered.
    pub seq: u32,
    /// Raw folder flag; constructors use zero for playlists and one for folders.
    pub attribute: u32,
    /// Display name.
    pub name: String,
    /// Reserved word at 0x04, defaulting to zero.
    pub unknown_04: u32,
}

impl NewPlaylist {
    /// Prepare a playlist with an explicit parent and sibling position.
    pub fn new(id: u32, parent_id: u32, seq: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            parent_id,
            seq,
            name: name.into(),
            attribute: 0,
            unknown_04: 0,
        }
    }

    /// Prepare a folder with an explicit parent and sibling position.
    pub fn folder(id: u32, parent_id: u32, seq: u32, name: impl Into<String>) -> Self {
        Self {
            attribute: 1,
            ..Self::new(id, parent_id, seq, name)
        }
    }

    /// Insert atomically, rejecting duplicate IDs and returning the engine-created row.
    /// Failures leave the connection unchanged; saving is explicit.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Playlist> {
        conn.check_kind(Playlist::TABLE)?;
        let row = self.to_binary()?;
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
        let handle = tx.insert(
            Playlist::TABLE,
            &Row::Standard(StandardRow::PlaylistTree(row)),
        )?;
        let model = Playlist::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; the engine assigns placement.
    /// Direct binary insertions do not check model-ID uniqueness.
    pub fn to_binary(&self) -> Result<PlaylistTreeRow> {
        validate_id(Playlist::TABLE, self.id)?;
        Ok(PlaylistTreeRow {
            parent_id: self.parent_id,
            unknown_04: self.unknown_04,
            position: self.seq,
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
    pub fn by_playlist<R: Read + Seek>(
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

    /// Delete this one original association, ignoring edits to public fields.
    ///
    /// Duplicate pairs are retained. Changed binary values or heap offsets return
    /// [`Error::StaleModelRow`]; missing locations return an engine error. Re-fetch
    /// after structural edits: an identical replacement at the same location cannot
    /// be distinguished. Returns one on success; saving remains explicit.
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
/// Positions are explicit; insertion never shifts other entries or deduplicates
/// tracks. Nonzero playlist/track IDs are required, but referenced rows are not
/// resolved. The caller supplies valid references to a playlist and an export track.
/// Saving, relationship validation and automatic ordering remain separate steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPlaylistContent {
    /// Nonzero export playlist ID; existence and folder status are not inferred.
    pub playlist_id: u32,
    /// Nonzero export track ID.
    pub content_id: u32,
    /// Explicit position, conventionally one-based; other positions are untouched.
    pub seq: u32,
}

impl NewPlaylistContent {
    /// Prepare an occurrence with an explicit playlist, track and position.
    pub fn new(playlist_id: u32, content_id: u32, seq: u32) -> Self {
        Self {
            playlist_id,
            content_id,
            seq,
        }
    }

    /// Insert one occurrence atomically, allowing repeated tracks and position ties.
    /// Returns the engine-created row and commits in memory only; saving is explicit.
    pub fn insert<R: Read + Seek>(
        self,
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<PlaylistContent> {
        conn.check_kind(PlaylistContent::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            PlaylistContent::TABLE,
            &Row::Standard(StandardRow::PlaylistEntry(row)),
        )?;
        let model = PlaylistContent::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode the binary fields for an explicit transaction; placement is assigned by the engine.
    pub fn to_binary(&self) -> Result<PlaylistEntryRow> {
        validate_nonzero_id(PlaylistContent::TABLE, "playlist_id", self.playlist_id)?;
        validate_nonzero_id(PlaylistContent::TABLE, "content_id", self.content_id)?;
        Ok(PlaylistEntryRow {
            position: self.seq,
            track_id: self.content_id,
            playlist_id: self.playlist_id,
        })
    }
}
