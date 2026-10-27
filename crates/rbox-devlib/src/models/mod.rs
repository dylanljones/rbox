// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Application models with explicit queries over the DeviceSQL engine.
//!
//! Models are detached snapshots. Reads and insertions use the binary definitions
//! and engine directly. Content and album relationships are loaded explicitly into
//! [`DevContentView`] and [`DevAlbumView`]. Content updates preserve unedited binary data;
//! [crate::DeviceLibrary::delete_content] cleans known references across both files.
//! Debug output includes all public fields but omits the stored raw row; use
//! `raw_record()` to inspect the original binary data explicitly.
//!
//! Public models use a `Dev` prefix, with `NewDev*` insertion inputs and `Dev*View`
//! resolved views. [`DevContent`] represents a DeviceSQL track row; binary types
//! and tables retain their format-specific names.
//!
//! IDs and references use `i64`; track metadata uses signed integers compatible
//! with the SQL models. Binary rows keep their exact unsigned widths. Queries and
//! writes reject negative or out-of-range values with
//! [`Error::ModelIntegerOutOfRange`]; zero reference sentinels are preserved.
//! Sequence positions, counters outside track metadata, and raw flags retain their
//! original types.
//!
//! Models with insertion inputs keep them in the same file. The [`browse`] module
//! groups the three browse models; [`property`] groups the two property models. These
//! models have no insertion inputs; creation profiles initialize their rows.
//! Insertion builders allocate export IDs automatically using the highest live
//! table ID plus one (starting at 1). Failed insertions do not consume IDs;
//! deleted maximum IDs can be reused. Use the returned model's ID for relationships.
//! My Tag IDs remain explicit source-library identities; association models have
//! no independent ID to generate. Explicit imported IDs require binary operations.
//! Tables retain their standard or extended database namespace. Using the wrong connection returns
//! `WrongDatabaseKind`. Unknown tables remain accessible through binary rows.
//! Playlist and My Tag attributes use shared enums. Unknown attribute values are
//! rejected when decoding these models but remain readable through the binary engine.
//!
//! ```
//! use rbox_devlib::{binary::DatabaseKind, DeviceSqlConnection};
//! use rbox_devlib::models::{DevAlbum, NewDevAlbum, NewDevArtist, NewDevKey};
//!
//! # fn main() -> rbox_devlib::Result<()> {
//! let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
//! let artist = NewDevArtist::new("Artist").insert(&mut conn)?;
//! let album = NewDevAlbum::new("Album").artist_id(artist.id).insert(&mut conn)?;
//! // Only the secondary key ID is explicit; no musical-key numbering is inferred.
//! let key = NewDevKey::new("Am", 23).insert(&mut conn)?;
//! assert_eq!(DevAlbum::find(&mut conn, &album.id)?.unwrap().artist_id, artist.id);
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

pub use album::{DevAlbum, DevAlbumView, NewDevAlbum};
pub use artist::{DevArtist, NewDevArtist};
pub use artwork::{DevArtwork, NewDevArtwork};
pub use browse::{DevBrowseCategory, DevBrowseColumn, DevBrowseSort};
pub use color::DevColor;
pub use content::{DevContent, DevContentView, NewDevContent, TrackCreationOptions};
pub use genre::{DevGenre, NewDevGenre};
pub use history::{DevHistory, DevHistoryContent, NewDevHistory, NewDevHistoryContent};
pub use key::{DevKey, NewDevKey};
pub use label::{DevLabel, NewDevLabel};
pub use my_tag::{DevMyTag, DevMyTagContent, NewDevMyTag, NewDevMyTagContent};
pub use playlist::{DevPlaylist, DevPlaylistContent, NewDevPlaylist, NewDevPlaylistContent};
pub use property::{DevDatabaseProperty, DevExportProperty};
pub use recommend_like::{DevRecommendedLike, NewDevRecommendedLike};

use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Cursor, Read, Seek};

use binrw::BinRead;

use crate::binary::{DeviceSqlString, Row, RowRecord, StandardRow, StringEncoding, TableKind};
use crate::devicesql::{DeviceSqlConnection, Transaction};
use crate::{Error, Result};

/// Convert at a model/binary boundary without truncation or signed wrapping.
pub(crate) fn checked_number<T, U>(value: U, field: &'static str) -> Result<T>
where
    T: TryFrom<U>,
    U: Copy + Into<i128>,
{
    T::try_from(value).map_err(|_| Error::ModelIntegerOutOfRange {
        field,
        value: value.into(),
        target: std::any::type_name::<T>(),
    })
}

/// Generate a table-local ID from the engine's cached index of live rows.
fn next_id<R: Read + Seek>(
    conn: &mut DeviceSqlConnection<R>,
    table: TableKind,
    read_id: fn(&RowRecord) -> Result<u32>,
) -> Result<i64> {
    conn.index_rows(table, read_id)?
        .keys()
        .copied()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .map(i64::from)
        .ok_or(Error::InvalidModelField {
            table,
            field: "id",
            reason: "the highest live ID is u32::MAX; no next ID is available",
        })
}

/// Enforce exact decoded-name equality independently of the stored string encoding.
/// Updates exclude their own ID; callers validate ID uniqueness separately.
fn unique_name<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    table: TableKind,
    name: &str,
    exclude_id: Option<i64>,
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
        if Some(i64::from(id)) != exclude_id
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
fn validate_id(table: TableKind, id: i64) -> Result<()> {
    validate_nonzero_id(table, "id", id)
}

/// Validate a named identity/reference without resolving related rows.
fn validate_nonzero_id(table: TableKind, field: &'static str, id: i64) -> Result<()> {
    let _: u32 = checked_number(id, field)?;
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

/// Read a required property for model-specific JavaScript conversions.
#[cfg(feature = "napi")]
fn napi_required<T: napi::bindgen_prelude::FromNapiValue>(
    object: &napi::bindgen_prelude::Object<'_>,
    field: &str,
) -> napi::Result<T> {
    object.get(field)?.ok_or_else(|| {
        napi::Error::new(
            napi::Status::InvalidArg,
            format!("Missing required field `{field}`"),
        )
    })
}
