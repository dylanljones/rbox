// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Color snapshots and explicit queries and updates over DeviceSQL.
//!
//! The standard creation profile initializes the eight color labels. Models
//! update their names without inserting, deleting or reassigning color IDs.
//! Bare databases have no color rows until supplied through binary creation options.

use std::io::{Read, Seek};

#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::{pyclass_public, PyMutableMapping};

use super::{checked_number, read_fields, text, text_error, unique_name, unsupported, validate_id};
use crate::binary::{ColorRow, PageType, Row, RowRecord, StandardRow, TableKind};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached color from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Queries never load related
/// tables and reject extended database connections. Call [`Self::update`] to
/// persist an edited name; saving remains explicit.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass_public(from_py_object, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi)]
pub struct DevColor {
    /// Export color ID, retaining both bytes of the binary field.
    pub id: i64,

    /// Decoded display name, preserving empty text.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for DevColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DevColor")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl DevColor {
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

    /// Read all live colors without loading related tables.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Find by export ID using the engine's reusable index; missing IDs return `None`.
    /// Only the selected model's text is converted. Duplicate IDs are an error.
    pub fn find<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &i64,
    ) -> Result<Option<Self>> {
        let id = u32::from(checked_number::<u16, _>(*id, "id")?);
        let location = conn
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&id)
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
    pub fn id_exists<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &i64) -> Result<bool> {
        let id = u32::from(checked_number::<u16, _>(*id, "id")?);
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(&id))
    }

    /// Edited names must be unique within this table (exact, case-sensitive equality).
    /// Persist an edited name and return a refreshed color snapshot.
    ///
    /// Resolves the current row by its original ID; changing `id` is rejected.
    /// An unchanged name preserves the current name and string frame. Edited
    /// names use last-write-wins and may be empty. The engine handles growing
    /// names and row relocation. A deleted color returns [`Error::ModelNotFound`].
    /// Changes commit in memory or roll back on error; saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw)?;
        validate_id(Self::TABLE, self.id)?;
        if self.id != original.id {
            return Err(Error::InvalidModelField {
                table: Self::TABLE,
                field: "id",
                reason: "an existing color's ID cannot be changed",
            });
        }
        let mut tx = conn.transaction();
        let location = tx
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&u32::from(checked_number::<u16, _>(original.id, "id")?))
            .copied()
            .ok_or(Error::ModelNotFound {
                table: Self::TABLE,
                id: i64::from(original.id),
            })?;
        let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
        let record = tx.read(&handle)?;
        let Row::Standard(StandardRow::Color(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if self.name != original.name {
            unique_name(&mut tx, Self::TABLE, &self.name, Some(original.id))?;
            row.name = text(&self.name)?;
        }
        let handle = tx.update(&handle, &Row::Standard(StandardRow::Color(row)))?;
        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
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

impl TryFrom<RowRecord> for DevColor {
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
            id: i64::from(row.id),

            name: row
                .name
                .text()
                .map_err(|source| text_error(&record, "name", source))?,
            raw: record,
        })
    }
}
