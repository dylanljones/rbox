// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Album snapshots and explicit read/insert operations over DeviceSQL.

use std::io::{Read, Seek};

use super::{load_related, read_fields, text, text_error, unsupported, validate_id, Artist};
use crate::binary::{
    AlbumHeader, AlbumRow, PageType, Row, RowRecord, StandardRow, StringOffsets, TableKind,
};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached album from the standard database.
///
/// Changing public fields changes only this snapshot. Original bytes, reserved
/// values and framing remain in [`Self::raw_record`]. Plain queries never load related
/// tables. Use [`Self::resolve`] or [`Self::all_with_relations`] for an [`AlbumView`].
/// All queries reject extended database connections.
#[derive(Clone, PartialEq, Eq)]
pub struct Album {
    /// Export album ID.
    pub id: u32,
    /// Album artist ID, independent of the track artist; zero means unassigned.
    pub artist_id: u32,
    /// Decoded display name, preserving empty text.
    pub name: String,
    raw: RowRecord,
}

/// An album snapshot with its album artist explicitly resolved.
///
/// [`Self::artist`] resolves [`Album::artist_id`], the same relationship exposed
/// as [`super::ContentView::album_artist`] on a content view. It is independent
/// of the content's main artist; the album row has only this one artist reference.
///
/// Field access performs no queries. A zero artist ID or missing target resolves
/// to `None`; the original ID and raw row remain in [`Self::album`]. Editing this
/// view changes only the snapshot, without saving or refreshing relationships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumView {
    /// Original album snapshot, including its artist ID.
    pub album: Album,
    /// Album artist resolved from `album.artist_id`, without recursive expansion.
    /// Equivalent to `ContentView::album_artist`, not the content's main artist.
    pub artist: Option<Artist>,
}

/// Access album fields without performing queries.
impl std::ops::Deref for AlbumView {
    type Target = Album;

    fn deref(&self) -> &Self::Target {
        &self.album
    }
}

impl std::fmt::Debug for Album {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Album")
            .field("id", &self.id)
            .field("artist_id", &self.artist_id)
            .field("name", &self.name)
            .finish()
    }
}

impl Album {
    /// Standard database table containing these rows.
    pub const TABLE: TableKind = TableKind::Standard(PageType::Albums);

    /// Resolve this snapshot's album artist without reloading the album.
    /// Zero and missing IDs become `None`; decoding and index errors propagate.
    pub fn resolve<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<AlbumView> {
        conn.check_kind(Self::TABLE)?;
        let artist = if self.artist_id == 0 {
            None
        } else {
            Artist::find(conn, &self.artist_id)?
        };
        Ok(AlbumView {
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
    ) -> Result<Vec<AlbumView>> {
        conn.check_kind(Self::TABLE)?;
        let artists = load_related(
            conn,
            albums.iter().map(|album| album.artist_id),
            Artist::find,
        )?;
        Ok(albums
            .into_iter()
            .map(|album| AlbumView {
                artist: artists.get(&album.artist_id).cloned(),
                album,
            })
            .collect())
    }

    /// Find an album and explicitly resolve its album artist.
    /// A missing album ID returns `None` without reading the artist table.
    pub fn find_with_relations<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<Option<AlbumView>> {
        Self::find(conn, id)?
            .map(|album| album.resolve(conn))
            .transpose()
    }

    /// Read all albums in physical order and resolve shared artists in a batch.
    pub fn all_with_relations<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<Vec<AlbumView>> {
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
        let header: AlbumHeader = read_fields(record)?;
        if !matches!(
            header.subtype,
            AlbumHeader::NEAR_SUBTYPE | AlbumHeader::FAR_SUBTYPE
        ) {
            return Err(unsupported(record));
        }
        Ok(header.id)
    }
}

impl TryFrom<RowRecord> for Album {
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
            id: row.header.id,
            artist_id: row.header.artist_id,
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
/// New rows use the near-offset subtype and marker 3. Reserved words default
/// to observed zero values and remain configurable. Artist IDs are stored as
/// supplied; no artist row is created or loaded.
/// Insertion commits to the connection's in-memory overlay; saving is explicit.
/// Other tables are not modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAlbum {
    /// Unused, nonzero export album ID.
    pub id: u32,
    /// Album artist ID, independent of the track artist; zero means unassigned.
    pub artist_id: u32,
    /// Decoded display name, preserving empty text.
    pub name: String,
    /// Reserved word at 0x04; defaults to the observed zero authoring value.
    pub unknown_04: u32,
    /// Reserved word at 0x10; defaults to the observed zero authoring value.
    pub unknown_10: u32,
}

impl NewAlbum {
    /// Prepare a new album with explicit identity and display name.
    pub fn new(id: u32, artist_id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            artist_id,
            name: name.into(),
            unknown_04: 0,
            unknown_10: 0,
        }
    }

    /// Insert atomically, rejecting duplicate IDs and leaving the connection unchanged on error.
    /// The returned model contains the engine-created row. This does not save to disk.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Album> {
        conn.check_kind(Album::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(Album::TABLE, Album::read_id)?
            .contains_key(&self.id)
        {
            return Err(Error::DuplicateId {
                table: Album::TABLE,
                id: self.id,
            });
        }
        let handle = tx.insert(Album::TABLE, &Row::Standard(StandardRow::Album(row)))?;
        let model = Album::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; the engine assigns placement.
    /// Direct binary inserts do not check model-ID uniqueness.
    pub fn to_binary(&self) -> Result<AlbumRow> {
        validate_id(Album::TABLE, self.id)?;
        Ok(AlbumRow {
            header: AlbumHeader {
                subtype: AlbumHeader::NEAR_SUBTYPE,
                index_shift: 0,
                unknown_04: self.unknown_04,
                artist_id: self.artist_id,
                id: self.id,
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
