// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Playlists, folders and their track occurrences in the standard DeviceSQL database.
//!
//! [Playlist] and [NewPlaylist] describe playlist-tree nodes in table 7.
//! [PlaylistEntry] and [NewPlaylistEntry] describe ordered track occurrences
//! in table 8. Queries do not load related rows automatically; repeated tracks
//! and position ties are retained.

use std::io::{Read, Seek};

use super::{read_fields, text, text_error, unsupported, validate_id, validate_nonzero_id};
use crate::binary::{
    PageType, PlaylistEntryRow, PlaylistTreeRow, Row, RowRecord, StandardRow, TableKind,
};
use crate::devicesql::DeviceSqlConnection;
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
    pub position: u32,
    /// Raw folder flag; conventionally zero for playlists and nonzero for folders.
    pub folder_flag: u32,
    /// Decoded display name.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for Playlist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Playlist")
            .field("id", &self.id)
            .field("parent_id", &self.parent_id)
            .field("position", &self.position)
            .field("folder_flag", &self.folder_flag)
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
        children.sort_by_key(|row| row.position);
        Ok(children)
    }

    /// Interpret the conventional nonzero folder flag without changing its raw value.
    pub fn is_folder(&self) -> bool {
        self.folder_flag != 0
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
            position: row.position,
            folder_flag: row.folder_flag,
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
    pub position: u32,
    /// Raw folder flag; constructors use zero for playlists and one for folders.
    pub folder_flag: u32,
    /// Display name.
    pub name: String,
    /// Reserved word at 0x04, defaulting to zero.
    pub unknown_04: u32,
}

impl NewPlaylist {
    /// Prepare a playlist with an explicit parent and sibling position.
    pub fn new(id: u32, parent_id: u32, position: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            parent_id,
            position,
            name: name.into(),
            folder_flag: 0,
            unknown_04: 0,
        }
    }

    /// Prepare a folder with an explicit parent and sibling position.
    pub fn folder(id: u32, parent_id: u32, position: u32, name: impl Into<String>) -> Self {
        Self {
            folder_flag: 1,
            ..Self::new(id, parent_id, position, name)
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
            position: self.position,
            id: self.id,
            folder_flag: self.folder_flag,
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
pub struct PlaylistEntry {
    /// Export playlist ID.
    pub playlist_id: u32,
    /// Export track ID, distinct from the source/master content ID.
    pub track_id: u32,
    /// Stored position, commonly one-based; zero and gaps remain unmodified on reads.
    pub position: u32,
    raw: RowRecord,
}

impl std::fmt::Debug for PlaylistEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaylistEntry")
            .field("playlist_id", &self.playlist_id)
            .field("track_id", &self.track_id)
            .field("position", &self.position)
            .finish()
    }
}

impl PlaylistEntry {
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
        entries.sort_by_key(|row| row.position);
        Ok(entries)
    }

    /// Original occurrence window and location, unaffected by public-field edits.
    /// The location distinguishes repeated occurrences in this snapshot but is not
    /// a stable mutation handle after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }
}

impl TryFrom<RowRecord> for PlaylistEntry {
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
            track_id: row.track_id,
            position: row.position,
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
pub struct NewPlaylistEntry {
    /// Nonzero export playlist ID; existence and folder status are not inferred.
    pub playlist_id: u32,
    /// Nonzero export track ID.
    pub track_id: u32,
    /// Explicit position, conventionally one-based; other positions are untouched.
    pub position: u32,
}

impl NewPlaylistEntry {
    /// Prepare an occurrence with an explicit playlist, track and position.
    pub fn new(playlist_id: u32, track_id: u32, position: u32) -> Self {
        Self {
            playlist_id,
            track_id,
            position,
        }
    }

    /// Insert one occurrence atomically, allowing repeated tracks and position ties.
    /// Returns the engine-created row and commits in memory only; saving is explicit.
    pub fn insert<R: Read + Seek>(
        self,
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<PlaylistEntry> {
        conn.check_kind(PlaylistEntry::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            PlaylistEntry::TABLE,
            &Row::Standard(StandardRow::PlaylistEntry(row)),
        )?;
        let model = PlaylistEntry::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode the binary fields for an explicit transaction; placement is assigned by the engine.
    pub fn to_binary(&self) -> Result<PlaylistEntryRow> {
        validate_nonzero_id(PlaylistEntry::TABLE, "playlist_id", self.playlist_id)?;
        validate_nonzero_id(PlaylistEntry::TABLE, "track_id", self.track_id)?;
        Ok(PlaylistEntryRow {
            position: self.position,
            track_id: self.track_id,
            playlist_id: self.playlist_id,
        })
    }
}
