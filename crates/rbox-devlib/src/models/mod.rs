// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Application models with explicit queries over the DeviceSQL engine.
//!
//! Models are detached snapshots. Reads and insertions use the binary definitions
//! and engine directly. Content and album relationships are loaded explicitly into
//! [`ContentView`] and [`AlbumView`]. Content updates preserve unedited binary data;
//! [crate::DeviceLibrary::delete_content] cleans known references across both files.
//! Debug output includes all public fields but omits the stored raw row; use
//! `raw_record()` to inspect the original binary data explicitly.
//!
//! [`Content`] represents a DeviceSQL track row. The model uses the name "content"
//! to match masterdb and OneLibrary; binary types and tables retain their track names.
//!
//! Models with insertion inputs keep them in the same file. The [`browse`] module
//! groups the three browse models; [`property`] groups the two property models. These
//! models have no insertion inputs; creation profiles initialize their rows.
//! Tables retain their standard or extended database namespace. Using the wrong connection returns
//! `WrongDatabaseKind`. Unknown tables remain accessible through binary rows.
//! Playlist and My Tag attributes use shared enums. Unknown attribute values are
//! rejected when decoding these models but remain readable through the binary engine.
//!
//! ```
//! use rbox_devlib::{binary::DatabaseKind, DeviceSqlConnection};
//! use rbox_devlib::models::{Album, NewAlbum, NewArtist, NewKey};
//!
//! # fn main() -> rbox_devlib::Result<()> {
//! let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
//! let artist = NewArtist::new(1, "Artist").insert(&mut conn)?;
//! let album = NewAlbum::new(2, artist.id, "Album").insert(&mut conn)?;
//! // Both stored key IDs are explicit; no numbering scheme is inferred.
//! let key = NewKey::new(3, 23, "Am").insert(&mut conn)?;
//! assert_eq!(Album::find(&mut conn, &album.id)?.unwrap().artist_id, artist.id);
//! # Ok(())
//! # }
//! ```

pub mod album;
pub mod artist;
pub mod artwork;
pub mod browse;
pub mod color;
pub mod content;
pub mod genre;
pub mod history;
pub mod key;
pub mod label;
pub mod my_tag;
mod ordering;
pub mod playlist;
pub mod property;
pub mod recommend_like;

pub use album::{Album, AlbumView, NewAlbum};
pub use artist::{Artist, NewArtist};
pub use artwork::{Artwork, NewArtwork};
pub use browse::{BrowseCategory, BrowseColumn, BrowseSort};
pub use color::Color;
pub use content::{Content, ContentView, NewContent, TrackCreationOptions};
pub use genre::{Genre, NewGenre};
pub use history::{History, HistoryContent, NewHistory, NewHistoryContent};
pub use key::{Key, NewKey};
pub use label::{Label, NewLabel};
pub use my_tag::{MyTag, MyTagContent, NewMyTag, NewMyTagContent};
pub use playlist::{NewPlaylist, NewPlaylistContent, Playlist, PlaylistContent};
pub use property::{DatabaseProperty, ExportProperty};
pub use recommend_like::{NewRecommendedLike, RecommendedLike};

use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Cursor, Read, Seek};

use binrw::BinRead;

use crate::binary::{DeviceSqlString, Row, RowRecord, StandardRow, StringEncoding, TableKind};
use crate::devicesql::{DeviceSqlConnection, Transaction};
use crate::{Error, Result};

/// Enforce exact decoded-name equality independently of the stored string encoding.
/// Updates exclude their own ID; callers validate ID uniqueness separately.
fn unique_name<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    table: TableKind,
    name: &str,
    exclude_id: Option<u32>,
) -> Result<()> {
    for record in tx.iter_raw(table)? {
        let record = record?;
        let (id, stored) = match record.decode()? {
            Row::Standard(StandardRow::Artist(row)) => (row.header.id, row.name),
            Row::Standard(StandardRow::Genre(row) | StandardRow::Label(row)) => (row.id, row.name),
            Row::Standard(StandardRow::Key(row)) => (row.id, row.name),
            Row::Standard(StandardRow::Color(row)) => (u32::from(row.id), row.name),
            _ => return Err(unsupported(&record)),
        };
        if Some(id) != exclude_id
            && stored
                .text()
                .map_err(|source| text_error(&record, "name", source))?
                == name
        {
            return Err(Error::UniqueConstraintViolation {
                table,
                fields: &["name"],
            });
        }
    }
    Ok(())
}

/// Load each distinct nonzero numeric foreign key once, skipping absent targets.
/// Sorting makes lookup/error order deterministic; empty sets perform no reads.
fn load_related<R, K, M>(
    conn: &mut DeviceSqlConnection<R>,
    ids: impl IntoIterator<Item = K>,
    mut find: impl FnMut(&mut DeviceSqlConnection<R>, &K) -> Result<Option<M>>,
) -> Result<HashMap<K, M>>
where
    R: Read + Seek,
    K: Copy + Default + Ord + Hash,
{
    let mut ids: Vec<_> = ids.into_iter().filter(|id| *id != K::default()).collect();
    ids.sort_unstable();
    ids.dedup();
    let mut rows = HashMap::with_capacity(ids.len());
    for id in ids {
        if let Some(row) = find(conn, &id)? {
            rows.insert(id, row);
        }
    }
    Ok(rows)
}

/// Read a binary prefix or row with its original location attached to errors.
fn read_fields<T: for<'a> BinRead<Args<'a> = ()>>(record: &RowRecord) -> Result<T> {
    T::read_le(&mut Cursor::new(record.as_bytes())).map_err(|source| Error::RowDecode {
        page: record.page(),
        table: record.table(),
        slot: record.slot().slot,
        source,
    })
}

/// Zero denotes an absent reference rather than an authored catalog record.
fn validate_id(table: TableKind, id: u32) -> Result<()> {
    validate_nonzero_id(table, "id", id)
}

/// Validate a named identity/reference without resolving related rows.
fn validate_nonzero_id(table: TableKind, field: &'static str, id: u32) -> Result<()> {
    if id == 0 {
        return Err(Error::InvalidModelField {
            table,
            field,
            reason: "export record IDs must be nonzero",
        });
    }
    Ok(())
}

/// Encode authored text using short ASCII, long ASCII or UTF-16 framing.
fn text(value: &str) -> Result<DeviceSqlString> {
    DeviceSqlString::encode(
        value,
        if !value.is_ascii() {
            StringEncoding::Utf16Le
        } else if value.len() <= 126 {
            StringEncoding::ShortAscii
        } else {
            StringEncoding::LongAscii
        },
    )
}

/// Update existing export-property rows inside the caller's transaction.
pub(crate) fn update_export_count<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    count: u32,
) -> Result<()> {
    property::set_export_counts(tx, None, false, count)?;
    Ok(())
}

fn unsupported(record: &RowRecord) -> Error {
    Error::UnsupportedModelRow {
        page: record.page(),
        table: record.table(),
        slot: record.slot().slot,
    }
}

fn text_error(record: &RowRecord, field: &'static str, source: Error) -> Error {
    Error::ModelText {
        page: record.page(),
        table: record.table(),
        slot: record.slot().slot,
        field,
        source: Box::new(source),
    }
}
