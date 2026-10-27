// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Artwork snapshots and explicit read, insert, update, and delete operations.

use std::io::{Read, Seek};

#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::{pyclass_public, PyConstructor, PyMutableMapping};

use super::{
    checked_number, next_id, read_fields, text, text_error, unsupported, validate_id, DevContent,
};
use crate::binary::{NamedRow, PageType, Row, RowRecord, StandardRow, TableKind, TrackHeader};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached artwork from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Queries never load related
/// tables and reject extended database connections.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass_public(from_py_object, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi)]
pub struct DevArtwork {
    /// Export artwork ID.
    pub id: i64,

    /// Original artwork path, preserving stored separators.
    pub path: String,
    raw: RowRecord,
}

impl std::fmt::Debug for DevArtwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DevArtwork")
            .field("id", &self.id)
            .field("path", &self.path)
            .finish()
    }
}

impl DevArtwork {
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

    /// Read all live artwork references without loading related tables.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Find by export ID using the engine's reusable index; missing IDs return `None`.
    /// Only the selected model's text is converted. Duplicate IDs are an error.
    pub fn find<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &i64,
    ) -> Result<Option<Self>> {
        let id: u32 = checked_number(*id, "id")?;
        let location = conn
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&id)
            .copied();
        location
            .map(|location| Self::try_from(conn.read_indexed_row(Self::TABLE, location)?))
            .transpose()
    }

    /// Find the first exact, case-sensitive path match in physical order.
    ///
    /// Returns `None` when absent. Paths are compared as stored, without separator
    /// normalization or filesystem access. Duplicate paths return the first match;
    /// decoding errors encountered before that match propagate.
    pub fn find_by_path<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        path: &str,
    ) -> Result<Option<Self>> {
        for artwork in Self::iter(conn)? {
            let artwork = artwork?;
            if artwork.path == path {
                return Ok(Some(artwork));
            }
        }
        Ok(None)
    }

    /// Check an ID using the engine index without converting string frames to text.
    pub fn id_exists<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &i64) -> Result<bool> {
        let id: u32 = checked_number(*id, "id")?;
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(&id))
    }

    /// Persist an edited image path and return a refreshed artwork snapshot.
    ///
    /// Resolves the current row by its original ID; changing `id` is rejected.
    /// An unchanged path preserves the current path and string frame. Edited paths
    /// must be nonempty and use last-write-wins. Separators are stored as supplied;
    /// this does not copy, rename, validate the existence of, or delete image files.
    /// The engine handles growing paths and row relocation.
    ///
    /// Commits to the in-memory overlay or rolls back on error. A deleted artwork
    /// returns [`Error::ModelNotFound`]. Saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw)?;
        validate_id(Self::TABLE, self.id)?;
        if self.id != original.id {
            return Err(Error::InvalidModelField {
                table: Self::TABLE,
                field: "id",
                reason: "an existing artwork's ID cannot be changed",
            });
        }
        let mut tx = conn.transaction();
        let location = tx
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&checked_number(original.id, "id")?)
            .copied()
            .ok_or(Error::ModelNotFound {
                table: Self::TABLE,
                id: original.id,
            })?;
        let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
        let record = tx.read(&handle)?;
        let Row::Standard(StandardRow::Artwork(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if self.path != original.path {
            if self.path.is_empty() {
                return Err(Error::InvalidModelField {
                    table: Self::TABLE,
                    field: "path",
                    reason: "artwork path must not be empty",
                });
            }
            row.name = text(&self.path)?;
        }
        let handle = tx.update(&handle, &Row::Standard(StandardRow::Artwork(row)))?;
        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Delete an artwork reference and clear matching content references atomically.
    ///
    /// Content `artwork_id` values become zero (unassigned); all other fields and
    /// export track counts remain unchanged. Image files are never deleted.
    /// Returns the total number of affected rows: one artwork plus cleared content
    /// references. A missing artwork returns zero without changing dangling
    /// references; ID zero is rejected.
    /// Changes remain in memory until explicitly saved. Any error rolls back the
    /// artwork deletion and all content-reference updates.
    pub fn delete<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &i64) -> Result<usize> {
        let _: u32 = checked_number(*id, "id")?;
        conn.check_kind(Self::TABLE)?;
        validate_id(Self::TABLE, *id)?;
        let mut tx = conn.transaction();
        let Some(location) = tx
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&checked_number(*id, "id")?)
            .copied()
        else {
            return Ok(0);
        };
        let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
        tx.delete(&handle)?;

        let mut locations = Vec::new();
        for record in tx.iter_raw(DevContent::TABLE)? {
            let record = record?;
            let header: TrackHeader = read_fields(&record)?;
            if header.subtype != TrackHeader::SUBTYPE {
                return Err(unsupported(&record));
            }
            if i64::from(header.artwork_id) == *id {
                locations.push((record.page(), record.slot().slot));
            }
        }
        let mut affected = 1usize;
        for (page, slot) in locations {
            let handle = tx.locate(DevContent::TABLE, page, slot)?;
            let record = tx.read(&handle)?;
            let Row::Standard(StandardRow::Track(mut row)) = record.decode()? else {
                return Err(unsupported(&record));
            };
            row.header.artwork_id = 0;
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

    /// Generate the next export ID: the highest live ID plus one, or 1 when empty.
    fn generate_id<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<i64> {
        next_id(conn, Self::TABLE, Self::read_id)
    }
}

impl TryFrom<RowRecord> for DevArtwork {
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
            id: i64::from(row.id),

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
/// Insertion allocates the highest live table ID plus one, starting at 1.
/// Failed insertions do not consume an ID; deleting the highest ID permits reuse.
/// Use the returned model's ID when creating relationships.
///
/// This is an image-path reference only. The caller copies the artwork file;
/// paths are stored as supplied without normalization or filesystem access.
/// Insertion commits to the connection's in-memory overlay; saving is explicit.
/// Other tables are not modified.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping, PyConstructor))]
#[cfg_attr(feature = "napi", napi(object))]
#[cfg_attr(feature = "pyo3", py_constructor(required(path)))]
pub struct NewDevArtwork {
    /// Original artwork path, preserving stored separators.
    pub path: String,
}

impl NewDevArtwork {
    /// Prepare a new artwork with explicit image path.
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// Allocate an export ID and insert atomically; errors leave the connection unchanged.
    /// The returned model contains the engine-created row. This does not save to disk.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<DevArtwork> {
        conn.check_kind(DevArtwork::TABLE)?;
        let id = DevArtwork::generate_id(conn)?;
        let row = self.to_binary(id)?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(DevArtwork::TABLE, DevArtwork::read_id)?
            .contains_key(&checked_number(id, "id")?)
        {
            return Err(Error::DuplicateId {
                table: DevArtwork::TABLE,
                id: checked_number(id, "id")?,
            });
        }
        let handle = tx.insert(DevArtwork::TABLE, &Row::Standard(StandardRow::Artwork(row)))?;
        let model = DevArtwork::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields with the allocated ID; the engine assigns placement.
    fn to_binary(&self, id: i64) -> Result<NamedRow> {
        validate_id(DevArtwork::TABLE, id)?;
        if self.path.is_empty() {
            return Err(Error::InvalidModelField {
                table: DevArtwork::TABLE,
                field: "path",
                reason: "artwork path must not be empty",
            });
        }
        Ok(NamedRow {
            id: checked_number(id, "id")?,
            name: text(&self.path)?,
        })
    }
}
