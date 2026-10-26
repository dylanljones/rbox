// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! History playlists and their track occurrences in the standard DeviceSQL database.
//!
//! [History] and [NewHistory] describe named histories in table 11.
//! [HistoryContent] and [NewHistoryContent] describe their ordered track occurrences
//! in table 12. History IDs are separate from regular playlist IDs. Queries load
//! relationships explicitly and retain repeated plays of the same track.
//! History names can be updated; deleting a history removes its occurrences.
//! Individual occurrences support deletion but not updates. Mutations commit to
//! the connection's in-memory overlay; saving remains explicit.

use std::io::{Read, Seek};

use super::{
    next_id, read_fields, text, text_error, unsupported, validate_id, validate_nonzero_id,
};
use crate::binary::{HistoryEntryRow, NamedRow, PageType, Row, RowRecord, StandardRow, TableKind};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached history playlist from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Queries never load related
/// tables and reject extended database connections. Use [`Self::update`] to
/// persist an edited name.
#[derive(Clone, PartialEq, Eq)]
pub struct History {
    /// History-playlist ID, in a separate namespace from regular playlists.
    pub id: u32,

    /// Decoded display name, preserving empty text.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("History")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl History {
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

    /// Persist an edited name and return a refreshed history snapshot.
    ///
    /// Resolves the current row by its original ID; changing `id` is rejected.
    /// An unchanged name preserves the current name and string frame. Edited
    /// names use last-write-wins and may be empty. The engine handles growing
    /// names and row relocation. A deleted history returns [`Error::ModelNotFound`].
    /// Changes commit in memory or roll back on error; saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw)?;
        validate_id(Self::TABLE, self.id)?;
        if self.id != original.id {
            return Err(Error::InvalidModelField {
                table: Self::TABLE,
                field: "id",
                reason: "an existing history's ID cannot be changed",
            });
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
        let Row::Standard(StandardRow::HistoryPlaylist(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if self.name != original.name {
            row.name = text(&self.name)?;
        }
        let handle = tx.update(&handle, &Row::Standard(StandardRow::HistoryPlaylist(row)))?;
        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Delete a history and all its occurrences in one in-memory transaction.
    ///
    /// Returns the number of deleted rows, including the history. A missing ID
    /// returns zero and leaves dangling entries untouched; zero is rejected.
    /// Contents, regular playlists, and other histories are unchanged. Any error
    /// rolls back the whole operation. Saving remains explicit.
    pub fn delete<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        validate_id(Self::TABLE, *id)?;
        let mut tx = conn.transaction();
        let Some(location) = tx.index_rows(Self::TABLE, Self::read_id)?.get(id).copied() else {
            return Ok(0);
        };
        let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
        tx.delete(&handle)?;
        let mut locations = Vec::new();
        for record in tx.iter_raw(HistoryContent::TABLE)? {
            let record = record?;
            let Row::Standard(StandardRow::HistoryEntry(row)) = record.decode()? else {
                return Err(unsupported(&record));
            };
            if row.playlist_id == *id {
                locations.push((record.page(), record.slot().slot));
            }
        }
        let count = locations
            .len()
            .checked_add(1)
            .ok_or(Error::MutationOverflow)?;
        for (page, slot) in locations {
            let handle = tx.locate(HistoryContent::TABLE, page, slot)?;
            tx.delete(&handle)?;
        }
        tx.commit()?;
        Ok(count)
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

    /// Generate the next export ID: the highest live ID plus one, or 1 when empty.
    fn generate_id<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<u32> {
        next_id(conn, Self::TABLE, Self::read_id)
    }
}

impl TryFrom<RowRecord> for History {
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
/// Insertion allocates the highest live table ID plus one, starting at 1.
/// Failed insertions do not consume an ID; deleting the highest ID permits reuse.
/// Use the returned model's ID when creating relationships.
///
/// No existing row or database template is needed.
/// Insertion commits to the connection's in-memory overlay; saving is explicit.
/// Other tables are not modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewHistory {
    /// Decoded display name, preserving empty text.
    pub name: String,
}

impl NewHistory {
    /// Prepare a new history playlist with explicit display name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Allocate an export ID and insert atomically; errors leave the connection unchanged.
    /// The returned model contains the engine-created row. This does not save to disk.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<History> {
        conn.check_kind(History::TABLE)?;
        let id = History::generate_id(conn)?;
        let row = self.to_binary(id)?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(History::TABLE, History::read_id)?
            .contains_key(&id)
        {
            return Err(Error::DuplicateId {
                table: History::TABLE,
                id,
            });
        }
        let handle = tx.insert(
            History::TABLE,
            &Row::Standard(StandardRow::HistoryPlaylist(row)),
        )?;
        let model = History::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields with the allocated ID; the engine assigns placement.
    fn to_binary(&self, id: u32) -> Result<NamedRow> {
        validate_id(History::TABLE, id)?;
        Ok(NamedRow {
            id,
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
pub struct HistoryContent {
    /// History-playlist ID; this does not reference the regular playlist table.
    pub history_id: u32,
    /// Export track ID, distinct from the source/master content ID.
    pub content_id: u32,
    /// Stored occurrence position; zero and gaps remain unmodified on reads.
    pub seq: u32,
    raw: RowRecord,
}

impl std::fmt::Debug for HistoryContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryContent")
            .field("history_id", &self.history_id)
            .field("content_id", &self.content_id)
            .field("seq", &self.seq)
            .finish()
    }
}

impl HistoryContent {
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
    pub fn by_history_id<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        history_id: &u32,
    ) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.history_id == *history_id {
                entries.push(row);
            }
        }
        entries.sort_by_key(|row| row.seq);
        Ok(entries)
    }

    /// Delete this single occurrence, preserving repeated plays and sequence gaps.
    ///
    /// Uses the original row location and binary values, ignoring public-field
    /// edits. The history and referenced content are retained. A changed row or
    /// heap offset returns [`Error::StaleModelRow`]; a missing location returns an
    /// engine error. Re-fetch after structural edits: identical replacements at
    /// the same location cannot be distinguished from the original occurrence.
    /// Commits in memory only and returns one on success; errors roll back.
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

    /// Original occurrence window and location, unaffected by public-field edits.
    /// The location distinguishes repeated occurrences in this snapshot but is not
    /// a stable mutation handle after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }
}

impl TryFrom<RowRecord> for HistoryContent {
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
            history_id: row.playlist_id,
            content_id: row.track_id,
            seq: row.position,
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
pub struct NewHistoryContent {
    /// Nonzero history-playlist ID; existence is not checked.
    pub history_id: u32,
    /// Nonzero export track ID.
    pub content_id: u32,
    /// Explicit occurrence position; other positions are untouched.
    pub seq: u32,
}

impl NewHistoryContent {
    /// Prepare an occurrence with an explicit playlist, track and position.
    pub fn new(history_id: u32, content_id: u32, seq: u32) -> Self {
        Self {
            history_id,
            content_id,
            seq,
        }
    }

    /// Insert one occurrence atomically, allowing repeated tracks and position ties.
    /// Returns the engine-created row and commits in memory only; saving is explicit.
    pub fn insert<R: Read + Seek>(
        self,
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<HistoryContent> {
        conn.check_kind(HistoryContent::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            HistoryContent::TABLE,
            &Row::Standard(StandardRow::HistoryEntry(row)),
        )?;
        let model = HistoryContent::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Validate references and encode fields; placement is assigned by the engine.
    fn to_binary(&self) -> Result<HistoryEntryRow> {
        validate_nonzero_id(HistoryContent::TABLE, "history_id", self.history_id)?;
        validate_nonzero_id(HistoryContent::TABLE, "content_id", self.content_id)?;
        Ok(HistoryEntryRow {
            position: self.seq,
            track_id: self.content_id,
            playlist_id: self.history_id,
        })
    }
}
