// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Key snapshots and explicit read/insert operations over DeviceSQL.

use std::io::{Read, Seek};

use super::{read_fields, text, text_error, unsupported, validate_id};
use crate::binary::{KeyRow, PageType, Row, RowRecord, StandardRow, TableKind};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached key from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Queries never load related
/// tables and reject extended database connections.
#[derive(Clone, PartialEq, Eq)]
pub struct Key {
    /// Export key ID.
    pub id: u32,
    /// Second stored ID, supplied independently; its role remains unknown.
    pub secondary_id: u32,
    /// Decoded display name, preserving empty text.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Key")
            .field("id", &self.id)
            .field("secondary_id", &self.secondary_id)
            .field("name", &self.name)
            .finish()
    }
}

impl Key {
    /// Standard database table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::MusicalKeys);

    /// Stream live rows in page-chain and slot order, decoding on demand.
    /// Creating the iterator does not read pages; row errors are not skipped.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live models-old without loading related tables.
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

impl TryFrom<RowRecord> for Key {
    type Error = Error;

    /// Decode a binary row while retaining its complete original read window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::Key(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            id: row.id,
            secondary_id: row.secondary_id,
            name: row
                .name
                .text()
                .map_err(|source| text_error(&record, "name", source))?,
            raw: record,
        })
    }
}

/// Input for a fresh key, without an existing raw row.
///
/// The second stored ID is explicit: it is not assumed to equal the primary
/// ID, and no musical-key numbering scheme is inferred.
/// Insertion commits to the connection's in-memory overlay; saving is explicit.
/// Other tables are not modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewKey {
    /// Unused, nonzero export key ID.
    pub id: u32,
    /// Second stored ID, supplied independently; its role remains unknown.
    pub secondary_id: u32,
    /// Decoded display name, preserving empty text.
    pub name: String,
}

impl NewKey {
    /// Prepare a new key with explicit identity and display name.
    pub fn new(id: u32, secondary_id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            secondary_id,
            name: name.into(),
        }
    }

    /// Insert atomically, rejecting duplicate IDs and leaving the connection unchanged on error.
    /// The returned model contains the engine-created row. This does not save to disk.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Key> {
        conn.check_kind(Key::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(Key::TABLE, Key::read_id)?
            .contains_key(&self.id)
        {
            return Err(Error::DuplicateId {
                table: Key::TABLE,
                id: self.id,
            });
        }
        let handle = tx.insert(Key::TABLE, &Row::Standard(StandardRow::Key(row)))?;
        let model = Key::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; the engine assigns placement.
    /// Direct binary inserts do not check model-ID uniqueness.
    pub fn to_binary(&self) -> Result<KeyRow> {
        validate_id(Key::TABLE, self.id)?;
        Ok(KeyRow {
            id: self.id,
            secondary_id: self.secondary_id,
            name: text(&self.name)?,
        })
    }
}
