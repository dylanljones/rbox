// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Browse labels, category menus and sort options in the standard `export.pdb`.
//!
//! Rekordbox's Category and Sort settings select and order predefined options.
//! Three tables describe these menus:
//!
//! | Model | Table | Role |
//! | --- | --- | --- |
//! | [`BrowseColumn`] | 16 | Label catalog, including ARTIST, BPM and COLOR |
//! | [`BrowseCategory`] | 17 | Category menu visibility and display order |
//! | [`BrowseSort`] | 18 | Sort-option configuration |
//!
//! Category and sort rows refer to catalog entries through `column_id`. Resolve
//! labels explicitly with [`BrowseColumn::find`].
//! Use each snapshot's `update()` method to commit edits to the in-memory overlay;
//! saving remains explicit. No model insertion or deletion methods are provided.
//! Category/sort snapshots have no stable logical ID: reload them after rebuilding
//! or replacing the database. Updates check their original location and binary
//! fields, not just `column_id`; identical replacement rows cannot be distinguished.
//!
//! # Reading the settings
//!
//! Category `state` values 0 (visible) and 1 (hidden), and the `position` field,
//! have been tested on a CDJs.
//! Native values 2 and 3 remain unqualified. Preserve them rather than treating
//! every nonzero value as hidden. Hardware may not display every enabled category.
//!
//! Sort settings use a different layout. Comparing PDB and OneLibrary defaults
//! suggests that, in `raw_tail.to_le_bytes()`, byte 0 bit 0 means hidden, byte 0
//! bit 1 selects the additional display column, and byte 1 contains the position.
//! These are inferences, not confirmed edit semantics; bytes 2 and 3 and the full
//! flag/position widths remain unresolved. Keep the complete tail intact unless
//! the intended change has been validated against controlled Rekordbox exports.

use std::io::{Read, Seek};

use super::{checked_number, read_fields, text, text_error, unsupported};
use crate::binary::{PageType, Row, RowRecord, StandardRow, TableKind};
use crate::devicesql::{DeviceSqlConnection, RowHandle, Transaction};
use crate::{Error, Result};

/// A predefined browse label from standard table 16, such as ARTIST or BPM.
///
/// Category and sort settings refer to [`Self::id`] through their `column_id`.
/// This catalog does not determine which options are active. No Rekordbox UI for
/// arbitrary custom catalog entries is documented. Creation profiles initialize
/// these rows; preserve their IDs and codes when editing configuration.
///
/// Public-field edits affect only this snapshot. Unknown data and original framing
/// remain available through [`Self::raw_record`]. Related rows are never loaded implicitly.
#[derive(Clone, PartialEq, Eq)]
pub struct BrowseColumn {
    /// Column/menu-label ID.
    pub id: i64,
    /// Raw column code; unknown values are retained.
    pub code: u16,
    /// Display label with one enclosing U+FFFA/U+FFFB pair removed, if present.
    /// Unframed labels, whitespace and interior characters are preserved.
    pub name: String,
    raw: RowRecord,
}

impl std::fmt::Debug for BrowseColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowseColumn")
            .field("id", &self.id)
            .field("code", &self.code)
            .field("name", &self.name)
            .finish()
    }
}

impl BrowseColumn {
    /// Database namespace and table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::BrowseColumns);

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

    /// Find the first exact, case-sensitive match for a plain-text display label.
    /// Matches the decoded `name`, without its enclosing U+FFFA/U+FFFB markers.
    /// Scans physical row order, loading no related tables. Missing names return
    /// `None`; decoding errors encountered before a match are propagated.
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

    /// Check whether an exact, case-sensitive plain-text display label exists.
    /// Stops at the first match, with the same errors as [`Self::find_by_name`].
    pub fn name_exists<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        name: &str,
    ) -> Result<bool> {
        Ok(Self::find_by_name(conn, name)?.is_some())
    }

    /// Find a column by its 16-bit ID using the engine index; duplicate IDs are an error.
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
    /// Check an ID without converting label frames to text.
    pub fn id_exists<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &i64) -> Result<bool> {
        let id = u32::from(checked_number::<u16, _>(*id, "id")?);
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(&id))
    }

    fn read_id(record: &RowRecord) -> Result<u32> {
        if record.table() != Self::TABLE {
            return Err(unsupported(record));
        }
        Ok(u32::from(read_fields::<u16>(record)?))
    }

    /// Persist an edited plain-text label, preserving its current enclosing markers.
    ///
    /// Catalog IDs and codes are fixed; changing them is rejected. The current row
    /// is resolved by ID, so relocation is supported. An unchanged name preserves
    /// the current string frame; an edited name uses last-write-wins. Wrapped labels
    /// retain one outer U+FFFA/U+FFFB pair, and unwrapped labels remain unwrapped.
    /// Returns a refreshed snapshot. Errors roll back; saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw)?;
        unchanged(Self::TABLE, "id", self.id, original.id)?;
        unchanged(Self::TABLE, "code", self.code, original.code)?;
        let mut tx = conn.transaction();
        let id = u32::from(checked_number::<u16, _>(original.id, "id")?);
        let location = tx
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&id)
            .copied()
            .ok_or(Error::ModelNotFound {
                table: Self::TABLE,
                id: i64::from(id),
            })?;
        let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
        let record = tx.read(&handle)?;
        let Row::Standard(StandardRow::BrowseColumn(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if self.name != original.name {
            let current = row
                .name
                .text()
                .map_err(|source| text_error(&record, "name", source))?;
            let wrapped = current.starts_with('\u{fffa}') && current.ends_with('\u{fffb}');
            row.name = text(&if wrapped {
                format!("\u{fffa}{}\u{fffb}", self.name)
            } else {
                self.name
            })?;
        }
        let handle = tx.update(&handle, &Row::Standard(StandardRow::BrowseColumn(row)))?;
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

impl TryFrom<RowRecord> for BrowseColumn {
    type Error = Error;

    /// Decode the binary fields while retaining the original row window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::BrowseColumn(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        let name = row
            .name
            .text()
            .map_err(|source| text_error(&record, "name", source))?;
        Ok(Self {
            id: i64::from(row.id),
            code: row.code,
            name: label_text(&name).to_owned(),
            raw: record,
        })
    }
}

/// Remove only a complete enclosing pair; do not strip interior or unmatched markers.
fn label_text(name: &str) -> &str {
    name.strip_prefix('\u{fffa}')
        .and_then(|inner| inner.strip_suffix('\u{fffb}'))
        .unwrap_or(name)
}

fn unchanged<T: PartialEq>(
    table: TableKind,
    field: &'static str,
    value: T,
    original: T,
) -> Result<()> {
    if value != original {
        return Err(Error::InvalidModelField {
            table,
            field,
            reason: "browse catalog identity cannot be changed",
        });
    }
    Ok(())
}

/// Fixed-size settings have no unique key. Check the original occurrence rather
/// than searching by column ID or comparing the row window's unrelated tail.
fn settings_handle<R: Read + Seek>(
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

/// Category menu visibility and order from standard table 17.
///
/// [`Self::column_id`] references a [`BrowseColumn`] label. Use [`Self::state`]
/// for the stored visibility value and [`Self::position`] for display order;
/// physical row order and row presence do not determine active menu membership.
/// See the [module documentation](self) for interpretation and update guidance.
///
/// Public-field edits affect only this snapshot. Unknown data and original framing
/// remain available through [`Self::raw_record`]. Related rows are never loaded implicitly.
/// No unique row ID or singleton constraint is assumed; repeated rows are retained.
#[derive(Clone, PartialEq, Eq)]
pub struct BrowseCategory {
    /// Column/menu-label ID.
    pub column_id: i64,
    /// Content/menu selector, not a PDB table ID.
    pub selector: u16,
    /// Raw attribute byte; its meaning is not inferred.
    pub unknown_04: u8,
    /// Visibility/state: 0 visible and 1 hidden in CDJ-350 experiments.
    /// Values 2 and 3 also occur in native exports and remain unqualified.
    /// Preserve unknown values without boolean coercion.
    pub state: u8,
    /// Stored display order; zero is a valid first position.
    pub position: u16,
    raw: RowRecord,
}

impl std::fmt::Debug for BrowseCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowseCategory")
            .field("column_id", &self.column_id)
            .field("selector", &self.selector)
            .field("unknown_04", &self.unknown_04)
            .field("state", &self.state)
            .field("position", &self.position)
            .finish()
    }
}

impl BrowseCategory {
    /// Database namespace and table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::BrowseCategorySettings);

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

    /// Scan rows for a column ID, retaining physical order and repetitions.
    /// Referenced rows are not loaded or validated.
    pub fn by_column<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        column_id: &i64,
    ) -> Result<Vec<Self>> {
        let _: u16 = checked_number(*column_id, "column_id")?;
        let mut rows = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.column_id == *column_id {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    /// Update this occurrence's state, position and explicit raw attribute byte.
    ///
    /// `column_id` and `selector` cannot change. No state coercion or automatic
    /// reordering is performed. The original location must still contain the same
    /// binary fields, otherwise the update fails (including concurrent edits).
    /// Repeated rows remain distinct. Reload snapshots after rebuild/replacement;
    /// there is no stable row identity. Returns the refreshed snapshot and commits
    /// atomically to memory; saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw.clone())?;
        unchanged(Self::TABLE, "column_id", self.column_id, original.column_id)?;
        unchanged(Self::TABLE, "selector", self.selector, original.selector)?;
        let mut tx = conn.transaction();
        let handle = settings_handle(&mut tx, &self.raw)?;
        let record = tx.read(&handle)?;
        let Row::Standard(StandardRow::BrowseCategory(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        row.unknown_04 = self.unknown_04;
        row.state = self.state;
        row.position = self.position;
        let handle = tx.update(&handle, &Row::Standard(StandardRow::BrowseCategory(row)))?;
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

impl TryFrom<RowRecord> for BrowseCategory {
    type Error = Error;

    /// Decode the binary fields while retaining the original row window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::BrowseCategory(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            column_id: i64::from(row.column_id),
            selector: row.selector,
            unknown_04: row.unknown_04,
            state: row.state,
            position: row.position,
            raw: record,
        })
    }
}

/// Sort-option configuration from standard table 18.
///
/// [`Self::column_id`] references a [`BrowseColumn`] label. Enabled state and
/// position appear to be packed into [`Self::raw_tail`], but their complete
/// semantics are not established. The layout differs from [`BrowseCategory`].
/// See the [module documentation](self) for the inferred mapping and its limits.
///
/// Public-field edits affect only this snapshot. Unknown data and original framing
/// remain available through [`Self::raw_record`]. Related rows are never loaded implicitly.
/// No unique row ID or singleton constraint is assumed; repeated rows are retained.
#[derive(Clone, PartialEq, Eq)]
pub struct BrowseSort {
    /// Column/menu-label ID.
    pub column_id: i64,
    /// Sort-item selector; zero is retained.
    pub selector: u16,
    /// Complete little-endian tail word, preserving all four bytes.
    /// Flag and position interpretations are provisional; see the module docs.
    pub raw_tail: u32,
    raw: RowRecord,
}

impl std::fmt::Debug for BrowseSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowseSort")
            .field("column_id", &self.column_id)
            .field("selector", &self.selector)
            .field("raw_tail", &self.raw_tail)
            .finish()
    }
}

impl BrowseSort {
    /// Database namespace and table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::BrowseSortSettings);

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

    /// Scan rows for a column ID, retaining physical order and repetitions.
    /// Referenced rows are not loaded or validated.
    pub fn by_column<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        column_id: &i64,
    ) -> Result<Vec<Self>> {
        let _: u16 = checked_number(*column_id, "column_id")?;
        let mut rows = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.column_id == *column_id {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    /// Update this occurrence's complete raw tail without interpreting its bits.
    ///
    /// `column_id` and `selector` cannot change. The original location must still
    /// contain the same binary fields, otherwise the update fails. Repeated rows
    /// remain distinct. Reload snapshots after rebuild/replacement; there is no
    /// stable row identity. Returns a refreshed snapshot and commits atomically
    /// to memory; saving remains explicit. See the module's tail-format caveats.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw.clone())?;
        unchanged(Self::TABLE, "column_id", self.column_id, original.column_id)?;
        unchanged(Self::TABLE, "selector", self.selector, original.selector)?;
        let mut tx = conn.transaction();
        let handle = settings_handle(&mut tx, &self.raw)?;
        let record = tx.read(&handle)?;
        let Row::Standard(StandardRow::BrowseSort(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        row.raw_tail = self.raw_tail;
        let handle = tx.update(&handle, &Row::Standard(StandardRow::BrowseSort(row)))?;
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

impl TryFrom<RowRecord> for BrowseSort {
    type Error = Error;

    /// Decode the binary fields while retaining the original row window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::BrowseSort(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            column_id: i64::from(row.column_id),
            selector: row.selector,
            raw_tail: row.raw_tail,
            raw: record,
        })
    }
}
