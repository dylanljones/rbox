// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! History playlists and their track occurrences in the standard DeviceSQL database.
//!
//! [HistoryPlaylist] and [NewHistoryPlaylist] describe named histories in table 11.
//! [HistoryEntry] and [NewHistoryEntry] describe their ordered track occurrences
//! in table 12. History IDs are separate from regular playlist IDs. Queries load
//! relationships explicitly and retain repeated plays of the same track.

use std::io::{Read, Seek};

use super::{read_fields, text, text_error, unsupported, validate_id, validate_nonzero_id};
use crate::binary::{HistoryEntryRow, NamedRow, PageType, Row, RowRecord, StandardRow, TableKind};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached history playlist from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Queries never load related
/// tables and reject extended database connections.
#[derive(Clone, PartialEq, Eq)]
pub struct HistoryPlaylist {
    /// History-playlist ID, in a separate namespace from regular playlists.
    pub id: u32,

    /// Decoded display name, preserving empty text.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for HistoryPlaylist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryPlaylist")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl HistoryPlaylist {
    /// Standard database table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::HistoryPlaylists);

    /// Stream live rows in page-chain and slot order, decoding on demand.
    /// Creating the iterator does not read pages; row errors are not skipped.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live models without loading related tables.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Find by export ID using the engine's reusable index; missing IDs return `None`.
    /// Only the selected model's text is converted. Duplicate IDs are an error.
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

    /// Check an ID using the engine index without converting string frames to text.
    pub fn id_exists<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<bool> {
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(id))
    }

    /// Original row window and location, unaffected by edits to public fields.
    /// Locations are not stable mutation handles after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }

    fn read_id(record: &RowRecord) -> Result<u32> {
        if record.table() != Self::TABLE {
            return Err(unsupported(record));
        }
        read_fields::<u32>(record)
    }
}

impl TryFrom<RowRecord> for HistoryPlaylist {
    type Error = Error;

    /// Decode a binary row while retaining its complete original read window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::HistoryPlaylist(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            id: row.id,

            name: row
                .name
                .text()
                .map_err(|source| text_error(&record, "name", source))?,
            raw: record,
        })
    }
}

/// Input for a fresh history playlist, without an existing raw row.
///
/// IDs are explicit and nonzero. No existing row or database template is needed.
/// Insertion commits to the connection's in-memory overlay; saving is explicit.
/// Other tables are not modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewHistoryPlaylist {
    /// Unused, nonzero export history playlist ID.
    pub id: u32,

    /// Decoded display name, preserving empty text.
    pub name: String,
}

impl NewHistoryPlaylist {
    /// Prepare a new history playlist with explicit identity and display name.
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    /// Insert atomically, rejecting duplicate IDs and leaving the connection unchanged on error.
    /// The returned model contains the engine-created row. This does not save to disk.
    pub fn insert<R: Read + Seek>(
        self,
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<HistoryPlaylist> {
        conn.check_kind(HistoryPlaylist::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(HistoryPlaylist::TABLE, HistoryPlaylist::read_id)?
            .contains_key(&self.id)
        {
            return Err(Error::DuplicateId {
                table: HistoryPlaylist::TABLE,
                id: self.id,
            });
        }
        let handle = tx.insert(
            HistoryPlaylist::TABLE,
            &Row::Standard(StandardRow::HistoryPlaylist(row)),
        )?;
        let model = HistoryPlaylist::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; the engine assigns placement.
    /// Direct binary inserts do not check model-ID uniqueness.
    pub fn to_binary(&self) -> Result<NamedRow> {
        validate_id(HistoryPlaylist::TABLE, self.id)?;
        Ok(NamedRow {
            id: self.id,
            name: text(&self.name)?,
        })
    }
}

/// A detached track occurrence in a standard database history playlist.
///
/// Entries have no unique scalar ID. Repeated tracks, identical occurrences and
/// position ties are retained. Editing fields changes only this snapshot; tracks
/// and history playlists are not loaded automatically. No timestamp is inferred.
#[derive(Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    /// History-playlist ID; this does not reference the regular playlist table.
    pub playlist_id: u32,
    /// Export track ID, distinct from the source/master content ID.
    pub track_id: u32,
    /// Stored occurrence position; zero and gaps remain unmodified on reads.
    pub position: u32,
    raw: RowRecord,
}

impl std::fmt::Debug for HistoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryEntry")
            .field("playlist_id", &self.playlist_id)
            .field("track_id", &self.track_id)
            .field("position", &self.position)
            .finish()
    }
}

impl HistoryEntry {
    /// Standard database table containing track occurrences.
    pub const TABLE: TableKind = TableKind::Standard(PageType::HistoryEntries);

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

    /// Read a history playlist's occurrences sorted by stored position.
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

impl TryFrom<RowRecord> for HistoryEntry {
    type Error = Error;

    /// Decode an entry without discarding its original row window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::HistoryEntry(row)) = record.decode()? else {
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

/// Input for a fresh history occurrence, without a synthetic ID or source row.
///
/// Positions are explicit; insertion never shifts other entries or deduplicates
/// tracks. Nonzero playlist/track IDs are required, but referenced rows are not
/// resolved. The caller supplies valid references to a history playlist and an export track.
/// Saving, relationship validation and automatic ordering remain separate steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewHistoryEntry {
    /// Nonzero history-playlist ID; existence is not checked.
    pub playlist_id: u32,
    /// Nonzero export track ID.
    pub track_id: u32,
    /// Explicit occurrence position; other positions are untouched.
    pub position: u32,
}

impl NewHistoryEntry {
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
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<HistoryEntry> {
        conn.check_kind(HistoryEntry::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            HistoryEntry::TABLE,
            &Row::Standard(StandardRow::HistoryEntry(row)),
        )?;
        let model = HistoryEntry::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode the binary fields for an explicit transaction; placement is assigned by the engine.
    pub fn to_binary(&self) -> Result<HistoryEntryRow> {
        validate_nonzero_id(HistoryEntry::TABLE, "playlist_id", self.playlist_id)?;
        validate_nonzero_id(HistoryEntry::TABLE, "track_id", self.track_id)?;
        Ok(HistoryEntryRow {
            position: self.position,
            track_id: self.track_id,
            playlist_id: self.playlist_id,
        })
    }
}
