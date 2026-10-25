// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Artwork snapshots and explicit read/insert operations over DeviceSQL.

use std::io::{Read, Seek};

use super::{read_fields, text, text_error, unsupported, validate_id};
use crate::binary::{NamedRow, PageType, Row, RowRecord, StandardRow, TableKind};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached artwork from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Queries never load related
/// tables and reject extended database connections.
#[derive(Clone, PartialEq, Eq)]
pub struct Artwork {
    /// Export artwork ID.
    pub id: u32,

    /// Original artwork path, preserving stored separators.
    pub path: String,
    raw: RowRecord,
}

impl std::fmt::Debug for Artwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Artwork")
            .field("id", &self.id)
            .field("path", &self.path)
            .finish()
    }
}

impl Artwork {
    /// Standard database table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::ArtworkReferences);

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

impl TryFrom<RowRecord> for Artwork {
    type Error = Error;

    /// Decode a binary row while retaining its complete original read window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::Artwork(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            id: row.id,

            path: row
                .name
                .text()
                .map_err(|source| text_error(&record, "path", source))?,
            raw: record,
        })
    }
}

/// Input for a fresh artwork, without an existing raw row.
///
/// This is an image-path reference only. The caller copies the artwork file;
/// paths are stored as supplied without normalization or filesystem access.
/// Insertion commits to the connection's in-memory overlay; saving is explicit.
/// Other tables are not modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewArtwork {
    /// Unused, nonzero export artwork ID.
    pub id: u32,

    /// Original artwork path, preserving stored separators.
    pub path: String,
}

impl NewArtwork {
    /// Prepare a new artwork with explicit identity and image path.
    pub fn new(id: u32, path: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
        }
    }

    /// Insert atomically, rejecting duplicate IDs and leaving the connection unchanged on error.
    /// The returned model contains the engine-created row. This does not save to disk.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Artwork> {
        conn.check_kind(Artwork::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(Artwork::TABLE, Artwork::read_id)?
            .contains_key(&self.id)
        {
            return Err(Error::DuplicateId {
                table: Artwork::TABLE,
                id: self.id,
            });
        }
        let handle = tx.insert(Artwork::TABLE, &Row::Standard(StandardRow::Artwork(row)))?;
        let model = Artwork::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; the engine assigns placement.
    /// Direct binary inserts do not check model-ID uniqueness.
    pub fn to_binary(&self) -> Result<NamedRow> {
        validate_id(Artwork::TABLE, self.id)?;
        if self.path.is_empty() {
            return Err(Error::InvalidModelField {
                table: Artwork::TABLE,
                field: "path",
                reason: "artwork path must not be empty",
            });
        }
        Ok(NamedRow {
            id: self.id,
            name: text(&self.path)?,
        })
    }
}
