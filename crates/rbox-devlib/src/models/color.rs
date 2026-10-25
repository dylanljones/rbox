// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Color snapshots and explicit read/insert operations over DeviceSQL.

use std::io::{Read, Seek};

use super::{read_fields, text, text_error, unsupported, validate_id};
use crate::binary::{ColorRow, PageType, Row, RowRecord, StandardRow, TableKind};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached color from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Queries never load related
/// tables and reject extended database connections.
#[derive(Clone, PartialEq, Eq)]
pub struct Color {
    /// Export color ID, retaining both bytes of the binary field.
    pub id: u16,

    /// Decoded display name, preserving empty text.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Color")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl Color {
    /// Standard database table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::ColorLabels);

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
        id: &u16,
    ) -> Result<Option<Self>> {
        let location = conn
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&u32::from(*id))
            .copied();
        location
            .map(|location| Self::try_from(conn.read_indexed_row(Self::TABLE, location)?))
            .transpose()
    }

    /// Check an ID using the engine index without converting string frames to text.
    pub fn id_exists<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u16) -> Result<bool> {
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(&u32::from(*id)))
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
        Ok(u32::from(read_fields::<ColorRow>(record)?.id))
    }
}

impl TryFrom<RowRecord> for Color {
    type Error = Error;

    /// Decode a binary row while retaining its complete original read window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::Color(row)) = record.decode()? else {
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

/// Input for a fresh color, without an existing raw row.
///
/// The binary format exposes a 16-bit ID; conventional player colors are 1–8.
/// Reserved bytes default to an observed zero variant and remain configurable.
/// Their values are independent of the ID. Creating configuration defaults may
/// already have populated these IDs.
/// Insertion commits to the connection's in-memory overlay; saving is explicit.
/// Other tables are not modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewColor {
    /// Unused, nonzero export color ID.
    pub id: u16,

    /// Decoded display name, preserving empty text.
    pub name: String,
    /// Reserved word; defaults to the observed zero value.
    pub unknown_00: u32,
    /// Independent reserved byte; defaults to the observed zero variant, not the ID.
    pub unknown_04: u8,
    /// Reserved byte; defaults to the observed zero value.
    pub unknown_07: u8,
}

impl NewColor {
    /// Prepare a new color with explicit identity and display name.
    pub fn new(id: u16, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            unknown_00: 0,
            unknown_04: 0,
            unknown_07: 0,
        }
    }

    /// Insert atomically, rejecting duplicate IDs and leaving the connection unchanged on error.
    /// The returned model contains the engine-created row. This does not save to disk.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Color> {
        conn.check_kind(Color::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(Color::TABLE, Color::read_id)?
            .contains_key(&u32::from(self.id))
        {
            return Err(Error::DuplicateId {
                table: Color::TABLE,
                id: u32::from(self.id),
            });
        }
        let handle = tx.insert(Color::TABLE, &Row::Standard(StandardRow::Color(row)))?;
        let model = Color::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; the engine assigns placement.
    /// Direct binary inserts do not check model-ID uniqueness.
    pub fn to_binary(&self) -> Result<ColorRow> {
        validate_id(Color::TABLE, u32::from(self.id))?;
        Ok(ColorRow {
            unknown_00: self.unknown_00,
            unknown_04: self.unknown_04,
            id: self.id,
            unknown_07: self.unknown_07,
            name: text(&self.name)?,
        })
    }
}
