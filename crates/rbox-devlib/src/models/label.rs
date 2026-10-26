// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Label snapshots and explicit read, insert, update, and delete operations.

use std::io::{Read, Seek};

use super::{read_fields, text, text_error, unique_name, unsupported, validate_id, Content};
use crate::binary::{NamedRow, PageType, Row, RowRecord, StandardRow, TableKind, TrackHeader};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached label from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Queries never load related
/// tables and reject extended database connections. Call [`Self::update`] to
/// persist edited fields to the connection's in-memory overlay.
#[derive(Clone, PartialEq, Eq)]
pub struct Label {
    /// Export label ID.
    pub id: u32,

    /// Decoded display name, preserving empty text.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Label")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl Label {
    /// Standard database table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::Labels);

    /// Stream live rows in page-chain and slot order, decoding on demand.
    /// Creating the iterator does not read pages; row errors are not skipped.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live labels without loading related tables.
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

    /// Find the first row with an exactly matching, case-sensitive name.
    /// Scans in physical row order and returns `None` when absent. Existing
    /// duplicate names return the first match. Related tables are not loaded;
    /// decoding errors encountered before the match are propagated.
    pub fn find_by_name<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        name: &str,
    ) -> Result<Option<Self>> {
        for model in Self::iter(conn)? {
            let model = model?;
            if model.name == name {
                return Ok(Some(model));
            }
        }
        Ok(None)
    }

    /// Check whether an exact, case-sensitive name exists, without loading relations.
    /// Stops at the first match and propagates the same errors as `find_by_name`.
    pub fn name_exists<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        name: &str,
    ) -> Result<bool> {
        Ok(Self::find_by_name(conn, name)?.is_some())
    }
    /// Check an ID using the engine index without converting string frames to text.
    pub fn id_exists<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<bool> {
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(id))
    }

    /// Edited names must be unique within this table (exact, case-sensitive equality).
    /// Persist an edited name and return a refreshed label snapshot.
    ///
    /// Resolves the current row by its original ID; changing `id` is rejected.
    /// An unchanged name preserves the current name and string frame. Edited
    /// names use last-write-wins and may be empty. The engine handles growing
    /// names and row relocation. A deleted label returns [`Error::ModelNotFound`].
    /// Changes commit in memory or roll back on error; saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw)?;
        validate_id(Self::TABLE, self.id)?;
        if self.id != original.id {
            return Err(Error::InvalidModelField {
                table: Self::TABLE,
                field: "id",
                reason: "an existing label's ID cannot be changed",
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
        let Row::Standard(StandardRow::Label(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if self.name != original.name {
            unique_name(&mut tx, Self::TABLE, &self.name, Some(original.id))?;
            row.name = text(&self.name)?;
        }
        let handle = tx.update(&handle, &Row::Standard(StandardRow::Label(row)))?;
        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Delete a label and clear matching content references atomically.
    ///
    /// Content `label_id` values become zero (unassigned); all other fields and
    /// export track counts remain unchanged. Returns the total number of affected
    /// rows: one label plus cleared content references. A missing label returns
    /// zero without changing dangling references; ID zero is rejected.
    /// Changes remain in memory until explicitly saved. Any error rolls back the
    /// label deletion and all content-reference updates.
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
        for record in tx.iter_raw(Content::TABLE)? {
            let record = record?;
            let header: TrackHeader = read_fields(&record)?;
            if header.subtype != TrackHeader::SUBTYPE {
                return Err(unsupported(&record));
            }
            if header.label_id == *id {
                locations.push((record.page(), record.slot().slot));
            }
        }
        let mut affected = 1usize;
        for (page, slot) in locations {
            let handle = tx.locate(Content::TABLE, page, slot)?;
            let record = tx.read(&handle)?;
            let Row::Standard(StandardRow::Track(mut row)) = record.decode()? else {
                return Err(unsupported(&record));
            };
            row.header.label_id = 0;
            tx.update(&handle, &Row::Standard(StandardRow::Track(row)))?;
            affected = affected.checked_add(1).ok_or(Error::MutationOverflow)?;
        }
        tx.commit()?;
        Ok(affected)
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

impl TryFrom<RowRecord> for Label {
    type Error = Error;

    /// Decode a binary row while retaining its complete original read window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::Label(row)) = record.decode()? else {
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

/// Input for a fresh label, without an existing raw row.
///
/// IDs are explicit and nonzero. No existing row or database template is needed.
/// Insertion commits to the connection's in-memory overlay; saving is explicit.
/// Other tables are not modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLabel {
    /// Unused, nonzero export label ID.
    pub id: u32,

    /// Decoded display name, preserving empty text.
    pub name: String,
}

impl NewLabel {
    /// Prepare a new label with explicit identity and display name.
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    /// Names must be unique within this table (exact, case-sensitive text equality).
    /// Insert atomically, rejecting duplicate IDs and leaving the connection unchanged on error.
    /// The returned model contains the engine-created row. This does not save to disk.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Label> {
        conn.check_kind(Label::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(Label::TABLE, Label::read_id)?
            .contains_key(&self.id)
        {
            return Err(Error::DuplicateId {
                table: Label::TABLE,
                id: self.id,
            });
        }
        unique_name(&mut tx, Label::TABLE, &self.name, None)?;
        let handle = tx.insert(Label::TABLE, &Row::Standard(StandardRow::Label(row)))?;
        let model = Label::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; the engine assigns placement.
    /// Direct binary inserts bypass model ID and name uniqueness constraints.
    pub fn to_binary(&self) -> Result<NamedRow> {
        validate_id(Label::TABLE, self.id)?;
        Ok(NamedRow {
            id: self.id,
            name: text(&self.name)?,
        })
    }
}
