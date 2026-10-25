// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Export metadata and source-library properties initialized by creation profiles.
//!
//! [`ExportProperty`] represents standard table 19 in `export.pdb`: the stored
//! track count, device name, export date, version text and background color.
//! [`DatabaseProperty`] represents extended table 7 in `exportExt.pdb`: the source
//! My Tag library identity, plus reserved fields retained in the binary row.
//! Neither model represents an ordinary content item to append during an export.
//!
//! # Initialization
//!
//! [`CreateOptions::standard`](crate::devicesql::CreateOptions::standard) supplies
//! one export-property row with zero tracks and the supplied date and device name.
//! [`CreateOptions::extended`](crate::devicesql::CreateOptions::extended) supplies
//! one database-property row with the supplied My Tag library identity, including
//! zero. Pass these profiles to [`DeviceSqlConnection::create_with_options`].
//! Bare [`DeviceSqlConnection::create`] initializes empty tables without properties.
//! These models therefore provide no insertion inputs; custom initialization can
//! still supply binary rows through the engine's creation options.

use std::io::{Read, Seek};

use super::{text, text_error, unsupported};
use crate::binary::{ExtPageType, ExtendedRow, PageType, Row, RowRecord, StandardRow, TableKind};
use crate::devicesql::{DeviceSqlConnection, RowHandle, Transaction};
use crate::{Error, Result};

/// Export metadata from standard table 19, initialized by the standard creation profile.
///
/// Update the existing metadata when the export changes. See the [module
/// documentation](self) for initialization, count maintenance and binary updates.
///
/// Public-field edits affect only this snapshot. Unknown data and original framing
/// remain available through [`Self::raw_record`]. Related rows are never loaded implicitly.
/// No unique row ID or singleton constraint is assumed; repeated rows are retained.
#[derive(Clone, PartialEq, Eq)]
pub struct ExportProperty {
    /// Stored track count after the export update.
    pub track_count: u32,
    /// Raw Device Library background-color value.
    pub background_color: u8,
    /// Export/creation date as stored text; no date format is imposed.
    pub date: String,
    /// Stored version text.
    pub version: String,
    /// Device/export name.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for ExportProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportProperty")
            .field("track_count", &self.track_count)
            .field("background_color", &self.background_color)
            .field("date", &self.date)
            .field("version", &self.version)
            .field("name", &self.name)
            .finish()
    }
}

impl ExportProperty {
    /// Database namespace and table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::ExportProperties);

    /// Stream live rows in physical order; creating the iterator reads no pages.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live rows in physical order without sorting or deduplication.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Update this existing occurrence and return its refreshed snapshot.
    ///
    /// Only edited fields are encoded; unknown fields and unedited string frames
    /// are preserved. No date format or track-count recalculation is imposed.
    /// Content insertions/deletions maintain the count separately; re-fetch this
    /// snapshot after those operations. With no unique row ID, a changed row or
    /// heap offset returns [`Error::StaleModelRow`], and a missing location returns
    /// an engine error. Re-fetch after structural edits. An identical replacement
    /// at the same location cannot be distinguished from the original occurrence.
    /// Errors roll back; changes commit in memory and saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw.clone())?;
        let mut tx = conn.transaction();
        let handle = property_handle(&mut tx, &self.raw)?;
        let Row::Standard(StandardRow::ExportProperty(mut row)) = tx.read(&handle)?.decode()?
        else {
            return Err(unsupported(&self.raw));
        };
        if self.track_count != original.track_count {
            row.track_count = self.track_count;
        }
        if self.background_color != original.background_color {
            row.background_color = self.background_color;
        }
        if self.date != original.date {
            row.date = text(&self.date)?;
        }
        if self.version != original.version {
            row.version = text(&self.version)?;
        }
        if self.name != original.name {
            row.name = text(&self.name)?;
        }
        let handle = tx.update(&handle, &Row::Standard(StandardRow::ExportProperty(row)))?;
        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Original row window, including reserved fields. Locations are not stable
    /// mutation handles after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }
}

impl TryFrom<RowRecord> for ExportProperty {
    type Error = Error;

    /// Decode the binary fields while retaining the original row window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::ExportProperty(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            track_count: row.track_count,
            background_color: row.background_color,
            date: row
                .date
                .text()
                .map_err(|source| text_error(&record, "date", source))?,
            version: row
                .version
                .text()
                .map_err(|source| text_error(&record, "version", source))?,
            name: row
                .name
                .text()
                .map_err(|source| text_error(&record, "name", source))?,
            raw: record,
        })
    }
}

/// Source My Tag library metadata from extended table 7, initialized by the extended profile.
///
/// The exposed identity is used verbatim, without a DBID conversion. Reserved
/// words and string frames remain in the original binary row. See the [module
/// documentation](self) for initialization and updates that preserve this data.
///
/// Public-field edits affect only this snapshot. Unknown data and original framing
/// remain available through [`Self::raw_record`]. Related rows are never loaded implicitly.
/// No unique row ID or singleton constraint is assumed; repeated rows are retained.
#[derive(Clone, PartialEq, Eq)]
pub struct DatabaseProperty {
    /// Source/master My Tag library identity, not a row ID; zero is valid.
    pub my_tag_master_db_id: u32,
    raw: RowRecord,
}

impl std::fmt::Debug for DatabaseProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseProperty")
            .field("my_tag_master_db_id", &self.my_tag_master_db_id)
            .finish()
    }
}

impl DatabaseProperty {
    /// Database namespace and table containing these rows.
    pub const TABLE: TableKind = TableKind::Extended(ExtPageType::DatabasePropertyOptions);

    /// Stream live rows in physical order; creating the iterator reads no pages.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live rows in physical order without sorting or deduplication.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Update the source My Tag library identity, preserving all reserved data.
    ///
    /// Zero is valid; this does not remap tags or change their associations.
    /// Targets this original occurrence, with the same stale-row checks as
    /// [`ExportProperty::update`]. Returns a refreshed snapshot. Errors roll back;
    /// changes commit in memory and saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let mut tx = conn.transaction();
        let handle = property_handle(&mut tx, &self.raw)?;
        let Row::Extended(ExtendedRow::DatabaseProperty(mut row)) = tx.read(&handle)?.decode()?
        else {
            return Err(unsupported(&self.raw));
        };
        row.header.my_tag_master_db_id = self.my_tag_master_db_id;
        let handle = tx.update(&handle, &Row::Extended(ExtendedRow::DatabaseProperty(row)))?;
        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Original row window, including reserved fields. Locations are not stable
    /// mutation handles after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }
}

// Property tables have no unique IDs; compare decoded rows rather than the
// read window's unrelated trailing bytes before modifying an occurrence.
fn property_handle<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    original: &RowRecord,
) -> Result<RowHandle> {
    let handle = tx.locate(original.table(), original.page(), original.slot().slot)?;
    let current = tx.read(&handle)?;
    if current.slot().heap_offset != original.slot().heap_offset
        || current.decode()? != original.decode()?
    {
        return Err(Error::StaleModelRow {
            table: original.table(),
            page: original.page(),
            slot: original.slot().slot,
        });
    }
    Ok(handle)
}

impl TryFrom<RowRecord> for DatabaseProperty {
    type Error = Error;

    /// Decode the binary fields while retaining the original row window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Extended(ExtendedRow::DatabaseProperty(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            my_tag_master_db_id: row.header.my_tag_master_db_id,
            raw: record,
        })
    }
}
