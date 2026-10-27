// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Album snapshots and explicit read, insert, update, and delete operations.

use std::io::{Read, Seek};

#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::{pyclass_public, PyMutableMapping};

use super::{
    checked_number, load_related, next_id, read_fields, text, text_error, unsupported, validate_id,
    DevArtist, DevContent,
};
use crate::binary::{
    AlbumHeader, AlbumRow, PageType, Row, RowRecord, StandardRow, StringOffsets, TableKind,
    TrackHeader,
};
use crate::devicesql::{DeviceSqlConnection, Transaction};
use crate::{Error, Result};

/// A detached album from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Plain queries never load related
/// tables. Use [`Self::resolve`] or [`Self::all_with_relations`] for a [`DevAlbumView`].
/// All queries reject extended database connections.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass_public(from_py_object, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi)]
pub struct DevAlbum {
    /// Export album ID.
    pub id: i64,
    /// Album artist ID, independent of the track artist; zero means unassigned.
    pub artist_id: i64,
    /// Decoded display name, preserving empty text.
    pub name: String,
    raw: RowRecord,
}

/// An album snapshot with its album artist explicitly resolved.
///
/// [`Self::artist`] resolves [`DevAlbum::artist_id`], the same relationship exposed
/// as [`super::DevContentView::album_artist`] on a content view. It is independent
/// of the content's main artist; the album row has only this one artist reference.
///
/// Field access performs no queries. A zero artist ID or missing target resolves
/// to `None`; the original ID and raw row remain in [`Self::album`]. Editing this
/// view changes only the snapshot, without saving or refreshing relationships.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all))]
#[cfg_attr(feature = "napi", napi(object, object_from_js = false))]
pub struct DevAlbumView {
    /// Original album snapshot, including its artist ID.
    pub album: DevAlbum,
    /// Album artist resolved from `album.artist_id`, without recursive expansion.
    /// Equivalent to `DevContentView::album_artist`, not the content's main artist.
    pub artist: Option<DevArtist>,
}

/// Access album fields without performing queries.
impl std::ops::Deref for DevAlbumView {
    type Target = DevAlbum;

    fn deref(&self) -> &Self::Target {
        &self.album
    }
}

impl std::fmt::Debug for DevAlbum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DevAlbum")
            .field("id", &self.id)
            .field("artist_id", &self.artist_id)
            .field("name", &self.name)
            .finish()
    }
}

impl DevAlbum {
    /// Standard database table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::Albums);

    /// Resolve this snapshot's album artist without reloading the album.
    /// Zero and missing IDs become `None`; decoding and index errors propagate.
    pub fn resolve<R: Read + Seek>(
        self,
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<DevAlbumView> {
        conn.check_kind(Self::TABLE)?;
        let artist = if self.artist_id == 0 {
            None
        } else {
            DevArtist::find(conn, &self.artist_id)?
        };
        Ok(DevAlbumView {
            album: self,
            artist,
        })
    }

    /// Resolve a batch, decoding each distinct artist once per call.
    /// Preserves input order, repeated albums and local snapshot edits. Empty input
    /// performs no reads; extended connections are always rejected.
    pub fn resolve_many<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        albums: Vec<Self>,
    ) -> Result<Vec<DevAlbumView>> {
        conn.check_kind(Self::TABLE)?;
        let artists = load_related(
            conn,
            albums.iter().map(|album| album.artist_id),
            DevArtist::find,
        )?;
        Ok(albums
            .into_iter()
            .map(|album| DevAlbumView {
                artist: artists.get(&album.artist_id).cloned(),
                album,
            })
            .collect())
    }

    /// Find an album and explicitly resolve its album artist.
    /// A missing album ID returns `None` without reading the artist table.
    pub fn find_with_relations<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &i64,
    ) -> Result<Option<DevAlbumView>> {
        let _: u32 = checked_number(*id, "id")?;
        Self::find(conn, id)?
            .map(|album| album.resolve(conn))
            .transpose()
    }

    /// Read all albums in physical order and resolve shared artists in a batch.
    pub fn all_with_relations<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<Vec<DevAlbumView>> {
        let albums = Self::all(conn)?;
        Self::resolve_many(conn, albums)
    }

    /// Stream live rows in page-chain and slot order, decoding on demand.
    /// Creating the iterator does not read pages; row errors are not skipped.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live albums without loading related tables.
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

    /// Find the first row with an exactly matching, case-sensitive name.
    /// Albums with different album artists may share a name; this query does
    /// not filter by artist and returns only the first matching album.
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
        let id: u32 = checked_number(*id, "id")?;
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(&id))
    }

    /// Persist fields edited since this snapshot was read, returning a refreshed album.
    /// The resulting `(name, artist_id)` must be unique when either field changes.
    /// Names use exact, case-sensitive equality; zero is a valid unassigned artist.
    ///
    /// Resolves the current row by its original ID; changing `id` is rejected.
    /// Unedited fields, reserved values and unchanged string frames are preserved
    /// from the current row. Concurrent edits to the same field use last-write-wins;
    /// unrelated intervening changes survive. Artist IDs are stored as supplied,
    /// consistently with insertion; this does not create or resolve an artist.
    ///
    /// The entire operation commits to the in-memory overlay or rolls back on error.
    /// A deleted album returns [`Error::ModelNotFound`]. Saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw)?;
        validate_id(Self::TABLE, self.id)?;
        if self.id != original.id {
            return Err(Error::InvalidModelField {
                table: Self::TABLE,
                field: "id",
                reason: "an existing album's ID cannot be changed",
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
        let Row::Standard(StandardRow::Album(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if self.artist_id != original.artist_id {
            row.header.artist_id = checked_number(self.artist_id, "artist_id")?;
        }
        if self.name != original.name {
            row.name = text(&self.name)?;
        }
        if self.name != original.name || self.artist_id != original.artist_id {
            unique_album(&mut tx, &row, Some(original.id))?;
        }
        let handle = tx.update(&handle, &Row::Standard(StandardRow::Album(row)))?;

        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;

        Ok(model)
    }

    /// Delete an album and clear its content references in one transaction.
    ///
    /// Matching content rows keep all other fields; their `album_id` becomes zero
    /// (unassigned). Artists, media files and export track counts are unchanged.
    /// Returns the total number of affected rows: one album plus cleared content
    /// references, following OneLibrary's convention. A missing album returns zero
    /// without modifying dangling references; ID zero is rejected.
    /// Changes remain in memory until explicitly saved. Any error rolls back both
    /// the album deletion and all reference updates.
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

        // Scan only fixed prefixes to select references, then decode affected
        // rows. Unrelated content text is never converted or rewritten.
        let mut locations = Vec::new();
        for record in tx.iter_raw(DevContent::TABLE)? {
            let record = record?;
            let header: TrackHeader = read_fields(&record)?;
            if header.subtype != TrackHeader::SUBTYPE {
                return Err(unsupported(&record));
            }
            if i64::from(header.album_id) == *id {
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
            row.header.album_id = 0;
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
        let header: AlbumHeader = read_fields(record)?;
        if !matches!(
            header.subtype,
            AlbumHeader::NEAR_SUBTYPE | AlbumHeader::FAR_SUBTYPE
        ) {
            return Err(unsupported(record));
        }
        Ok(header.id)
    }

    /// Generate the next export ID: the highest live ID plus one, or 1 when empty.
    fn generate_id<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<i64> {
        next_id(conn, Self::TABLE, Self::read_id)
    }
}

/// Compare the final merged row, rather than potentially stale public fields.
fn unique_album<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    candidate: &AlbumRow,
    exclude_id: Option<i64>,
) -> Result<()> {
    let name = candidate.name.text()?;
    for record in tx.iter_raw(DevAlbum::TABLE)? {
        let record = record?;
        let Row::Standard(StandardRow::Album(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if Some(i64::from(row.header.id)) != exclude_id
            && row.header.artist_id == candidate.header.artist_id
            && row
                .name
                .text()
                .map_err(|source| text_error(&record, "name", source))?
                == name
        {
            return Err(Error::UniqueConstraintViolation {
                table: DevAlbum::TABLE,
                fields: &["name", "artist_id"],
            });
        }
    }
    Ok(())
}

impl TryFrom<RowRecord> for DevAlbum {
    type Error = Error;

    /// Decode a binary row while retaining its complete original read window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::Album(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            id: i64::from(row.header.id),
            artist_id: i64::from(row.header.artist_id),
            name: row
                .name
                .text()
                .map_err(|source| text_error(&record, "name", source))?,
            raw: record,
        })
    }
}

/// Input for a fresh album, without an existing raw row.
///
/// Insertion allocates the highest live table ID plus one, starting at 1.
/// Failed insertions do not consume an ID; deleting the highest ID permits reuse.
/// Use the returned model's ID when creating relationships.
///
/// New rows use the near-offset subtype and marker 3. Reserved words default
/// to observed zero values and remain configurable.
/// Insertion commits to the connection's in-memory overlay; saving is explicit.
/// Other tables are not modified.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDevAlbum {
    /// Album artist ID, independent of the track artist; zero means unassigned.
    pub artist_id: i64,
    /// Decoded display name, preserving empty text.
    pub name: String,
    /// Reserved word at 0x04; defaults to the observed zero authoring value.
    pub unknown_04: u32,
    /// Reserved word at 0x10; defaults to the observed zero authoring value.
    pub unknown_10: u32,
}

impl NewDevAlbum {
    /// Prepare an album with no artist assigned; set one with [`Self::artist_id`].
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            artist_id: 0,
            name: name.into(),
            unknown_04: 0,
            unknown_10: 0,
        }
    }

    /// Builder for `artist_id`.
    pub fn artist_id(mut self, id: i64) -> Self {
        self.artist_id = id;
        self
    }

    /// Allocate an export ID and insert atomically, rejecting duplicate `(name, artist_id)` pairs.
    /// Names use exact, case-sensitive text equality; failures leave the connection unchanged.
    /// The returned model contains the engine-created row. This does not save to disk.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<DevAlbum> {
        conn.check_kind(DevAlbum::TABLE)?;
        let id = DevAlbum::generate_id(conn)?;
        let row = self.to_binary(id)?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(DevAlbum::TABLE, DevAlbum::read_id)?
            .contains_key(&checked_number(id, "id")?)
        {
            return Err(Error::DuplicateId {
                table: DevAlbum::TABLE,
                id: checked_number(id, "id")?,
            });
        }
        unique_album(&mut tx, &row, None)?;
        let handle = tx.insert(DevAlbum::TABLE, &Row::Standard(StandardRow::Album(row)))?;
        let model = DevAlbum::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields with the allocated ID; the engine assigns placement.
    fn to_binary(&self, id: i64) -> Result<AlbumRow> {
        validate_id(DevAlbum::TABLE, id)?;
        Ok(AlbumRow {
            header: AlbumHeader {
                subtype: AlbumHeader::NEAR_SUBTYPE,
                index_shift: 0,
                unknown_04: self.unknown_04,
                artist_id: checked_number(self.artist_id, "artist_id")?,
                id: checked_number(id, "id")?,
                unknown_10: self.unknown_10,
            },
            strings: StringOffsets::Near {
                marker: 3,
                offsets: [0],
            },
            name: text(&self.name)?,
        })
    }
}
