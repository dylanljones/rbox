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

use std::collections::HashSet;
use std::io::{Read, Seek};

use super::{text, text_error, unsupported, update_export_count, Content};
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
/// The exact device/export name is the model's unique key, matching OneLibrary.
/// Duplicate names in existing binary data are reported as a constraint violation;
/// the low-level engine remains available to inspect or repair such files.
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
    /// Find a property by its exact device/export name, including an empty name.
    /// Rejects duplicate names instead of choosing an ambiguous row.
    pub fn find<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        device_name: &str,
    ) -> Result<Option<Self>> {
        Ok(Self::all(conn)?
            .into_iter()
            .find(|row| row.name == device_name))
    }

    /// Check whether the exact device/export name exists as a unique key.
    pub fn id_exists<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        device_name: &str,
    ) -> Result<bool> {
        Ok(Self::find(conn, device_name)?.is_some())
    }

    /// Set the count of the named property; return zero if the name is absent.
    /// Does not create properties or alter content rows. Saving remains explicit.
    pub fn set_number_of_contents<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        device_name: &str,
        number_of_contents: u32,
    ) -> Result<usize> {
        let mut tx = conn.transaction();
        let count = set_export_counts(&mut tx, Some(device_name), false, number_of_contents)?;
        tx.commit()?;
        Ok(count)
    }

    /// Set the first property's count in physical order; return zero for an empty table.
    pub fn set_number_of_contents_default<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        number_of_contents: u32,
    ) -> Result<usize> {
        let mut tx = conn.transaction();
        let count = set_export_counts(&mut tx, None, true, number_of_contents)?;
        tx.commit()?;
        Ok(count)
    }

    /// Set every property's count atomically and return the number of affected rows.
    pub fn set_number_of_contents_all<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        number_of_contents: u32,
    ) -> Result<usize> {
        let mut tx = conn.transaction();
        let count = set_export_counts(&mut tx, None, false, number_of_contents)?;
        tx.commit()?;
        Ok(count)
    }

    /// Set the count in every existing export-property row atomically.
    /// Does not change contents or create missing properties. Changes remain in
    /// memory until explicitly saved.
    pub fn set_content_count<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        count: u32,
    ) -> Result<()> {
        conn.check_kind(Self::TABLE)?;
        let mut tx = conn.transaction();
        update_export_count(&mut tx, count)?;
        tx.commit()
    }

    /// Count live content rows and update all existing export properties in one
    /// transaction. Returns the actual count without decoding track strings.
    /// Missing properties are not created. Saving remains explicit.
    pub fn update_content_count<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<u32> {
        conn.check_kind(Self::TABLE)?;
        let mut tx = conn.transaction();
        let mut count = 0u32;
        for record in tx.iter_raw(Content::TABLE)? {
            record?;
            count = count.checked_add(1).ok_or(Error::MutationOverflow)?;
        }
        update_export_count(&mut tx, count)?;
        tx.commit()?;
        Ok(count)
    }

    /// Database namespace and table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::ExportProperties);

    /// Stream live rows in physical order; creating the iterator reads no pages.
    /// Reports duplicate names as encountered. Consume the iterator fully to validate all keys.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        let mut names = HashSet::new();
        Ok(conn.iter_raw(Self::TABLE)?.map(move |record| {
            let row = Self::try_from(record?)?;
            if !names.insert(row.name.clone()) {
                return Err(duplicate_name());
            }
            Ok(row)
        }))
    }

    /// Read the first live row in physical order, or `None` if the table is empty.
    /// Validates all property names before returning a row; malformed or duplicate
    /// rows are never silently skipped.
    pub fn first<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Option<Self>> {
        Ok(Self::all(conn)?.into_iter().next())
    }

    /// Read all live rows in physical order without sorting or deduplication.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Update the property identified by the snapshot's original name.
    ///
    /// Only edited fields are encoded; unknown fields and unedited string frames
    /// are preserved. No date format or track-count recalculation is imposed.
    /// Content insertions/deletions maintain the count separately; re-fetch this
    /// snapshot after those operations. Missing or changed rows return
    /// [`Error::StaleModelRow`]. Renaming is allowed only to an unused name.
    /// Physical relocation alone does not change the property key.
    /// Errors roll back; changes commit in memory and saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw.clone())?;
        let mut tx = conn.transaction();
        let properties = collect_export_properties(tx.iter_raw(Self::TABLE)?)?;
        if self.name != original.name && properties.iter().any(|row| row.name == self.name) {
            return Err(duplicate_name());
        }
        let current = properties
            .iter()
            .find(|row| row.name == original.name)
            .ok_or(Error::StaleModelRow {
                table: Self::TABLE,
                page: self.raw.page(),
                slot: self.raw.slot().slot,
            })?;
        if current.raw.decode()? != self.raw.decode()? {
            return Err(Error::StaleModelRow {
                table: Self::TABLE,
                page: self.raw.page(),
                slot: self.raw.slot().slot,
            });
        }
        let handle = tx.locate(Self::TABLE, current.raw.page(), current.raw.slot().slot)?;
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

    /// Read the first live row in physical order, or `None` if the table is empty.
    /// Does not assume a singleton or decode subsequent rows. Errors from the
    /// first row are propagated; malformed rows are never skipped.
    pub fn first<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Option<Self>> {
        Self::iter(conn)?.next().transpose()
    }

    /// Read all live rows in physical order without sorting or deduplication.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Update the source My Tag library identity, preserving all reserved data.
    ///
    /// Zero is valid; this does not remap tags or change their associations.
    /// Targets this original occurrence, checking its original row location and
    /// fields. Returns a refreshed snapshot. Errors roll back;
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

// The extended property table has no name key; compare decoded rows rather than the
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

fn duplicate_name() -> Error {
    Error::UniqueConstraintViolation {
        table: ExportProperty::TABLE,
        fields: &["name"],
    }
}

/// Validate all export-property keys before a transaction changes any rows.
pub(super) fn collect_export_properties(
    records: impl Iterator<Item = Result<RowRecord>>,
) -> Result<Vec<ExportProperty>> {
    let mut names = HashSet::new();
    records
        .map(|record| {
            let row = ExportProperty::try_from(record?)?;
            if !names.insert(row.name.clone()) {
                return Err(duplicate_name());
            }
            Ok(row)
        })
        .collect()
}

/// Shared by explicit count setters and automatic content-count maintenance.
pub(super) fn set_export_counts<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    name: Option<&str>,
    first_only: bool,
    count: u32,
) -> Result<usize> {
    let properties = collect_export_properties(tx.iter_raw(ExportProperty::TABLE)?)?;
    let mut updated = 0;
    for property in properties
        .into_iter()
        .filter(|row| name.is_none_or(|name| row.name == name))
        .take(if first_only { 1 } else { usize::MAX })
    {
        let Row::Standard(StandardRow::ExportProperty(mut row)) = property.raw.decode()? else {
            return Err(unsupported(&property.raw));
        };
        row.track_count = count;
        let handle = tx.locate(
            ExportProperty::TABLE,
            property.raw.page(),
            property.raw.slot().slot,
        )?;
        tx.update(&handle, &Row::Standard(StandardRow::ExportProperty(row)))?;
        updated += 1;
    }
    Ok(updated)
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
