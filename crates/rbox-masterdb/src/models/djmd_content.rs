// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Track records, relationship access, and local import defaults.

use chrono::Utc;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;
use std::path::PathBuf;
use uuid::Uuid;

use super::agent_registry::AgentRegistry;
use super::djmd_album::DjmdAlbum;
use super::djmd_artist::DjmdArtist;
use super::djmd_color::DjmdColor;
use super::djmd_device::DjmdDevice;
use super::djmd_genre::DjmdGenre;
use super::djmd_key::DjmdKey;
use super::djmd_label::DjmdLabel;
use super::{Date, DateString, RandomIdGenerator};
use crate::enums::{Analyzed, FileType};
use crate::error::Error as MasterDbError;
use crate::schema::{
    djmdAlbum, djmdArtist, djmdColor, djmdContent, djmdGenre, djmdKey, djmdLabel, djmdProperty,
};
use rbox_core::{PathExt, Track, TrackMetadata, TrackSource};
use rbox_db::{load_related, Model, ModelInsert, ModelRelations, ModelUpdate};

/// Represents the `djmdContent` table in the Rekordbox database.
///
/// This struct maps to the `djmdContent` table in the SQLite database used by Rekordbox.
/// It contains metadata and attributes for each track in the collection, including file info,
/// artist, album, genre, and more.
///
/// # Referenced by
/// * [`ContentActiveCensor`] via `content_id` foreign key.
/// * [`ContentCue`] via `content_id` foreign key.
/// * [`ContentFile`] via `content_id` foreign key.
/// * [`DjmdActiveCensor`] via `content_id` and `content_uuid` foreign keys.
/// * [`DjmdSongHistory`] via `content_id` foreign key.
/// * [`DjmdSongHotCueBanklist`] via `content_id` foreign key.
/// * [`DjmdMixerParam`] via `content_id` foreign key.
/// * [`DjmdSongMyTag`] via `content_id` foreign key.
/// * [`DjmdSongPlaylist`] via `content_id` foreign key.
/// * [`DjmdRecommendLike`] via `content_id1` and `content_id2` foreign keys.
/// * [`DjmdSongRelatedTracks`] via `content_id` foreign key.
/// * [`DjmdSongSampler`] via `content_id` foreign key.
/// * [`DjmdSongTagList`] via `content_id` foreign key.
///
/// # References
/// * [`DjmdArtist`] via `artist_id`, `remixer_id`, `original_artist_id`, `composer_id` and `lyricist` foreign keys.
/// * [`DjmdAlbum`] via `album_id` foreign key.
/// * [`DjmdGenre`] via `genre_id` foreign key.
/// * [`DjmdLabel`] via `label_id` foreign key.
/// * [`DjmdKey`] via `key_id` foreign key.
/// * [`DjmdColor`] via `color_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = djmdContent)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdArtist, foreign_key = artist_id))]
#[diesel(belongs_to(DjmdAlbum, foreign_key = album_id))]
#[diesel(belongs_to(DjmdGenre, foreign_key = genre_id))]
#[diesel(belongs_to(DjmdLabel, foreign_key = label_id))]
#[diesel(belongs_to(DjmdKey, foreign_key = key_id))]
#[diesel(belongs_to(DjmdColor, foreign_key = color_id))]
#[diesel(belongs_to(DjmdDevice, foreign_key = device_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdContent {
    /// A unique identifier for the entry.
    pub id: String,
    /// A unique universal identifier for the entry.
    pub uuid: String,
    /// An integer representing the data status in Rekordbox.
    pub rb_data_status: i32,
    /// An integer representing the local data status in Rekordbox.
    pub rb_local_data_status: i32,
    /// An integer indicating whether the entry is locally deleted.
    pub rb_local_deleted: i32,
    /// An integer indicating whether the entry is locally synced.
    pub rb_local_synced: i32,
    /// An optional integer representing the update sequence number.
    pub usn: Option<i32>,
    /// An optional integer representing the local update sequence number.
    pub rb_local_usn: Option<i32>,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    /// An optional string representing the folder path of the track.
    pub folder_path: Option<String>,
    /// An optional string representing the long file name of the track.
    pub file_name_l: Option<String>,
    /// An optional string representing the short file name of the track.
    pub file_name_s: Option<String>,
    /// An optional string representing the title of the track.
    pub title: Option<String>,
    /// An optional string representing the ID of the associated artist in [`DjmdArtist`].
    pub artist_id: Option<String>,
    /// An optional string representing the ID of the associated album in [`DjmdAlbum`].
    pub album_id: Option<String>,
    /// An optional string representing the ID of the associated genre in [`DjmdGenre`].
    pub genre_id: Option<String>,
    /// Track tempo in BPM multiplied by 100.
    pub bpm: Option<i64>,
    /// An optional integer representing the length of the track in rounded seconds.
    pub length: Option<i32>,
    /// An optional integer representing the track number in the album.
    pub track_no: Option<i64>,
    /// Stored bitrate, normally kb/s.
    pub bit_rate: Option<i64>,
    /// An optional integer representing the bit depth of the track.
    pub bit_depth: Option<i32>,
    /// An optional string representing comments about the track.
    pub commnt: Option<String>,
    /// An optional integer representing the file type of the track.
    pub file_type: i32,
    /// An optional integer representing the rating of the track.
    pub rating: Option<i32>,
    /// An optional integer representing the release year of the track.
    pub release_year: Option<i32>,
    /// An optional string representing the ID of the remixer in [`DjmdArtist`].
    pub remixer_id: Option<String>,
    /// An optional string representing the ID of the label in [`DjmdLabel`].
    pub label_id: Option<String>,
    /// An optional string representing the ID of the original artist in [`DjmdArtist`].
    pub org_artist_id: Option<String>,
    /// An optional string representing the ID of the key in [`DjmdKey`].
    pub key_id: Option<String>,
    /// An optional string representing the stock date of the track.
    pub stock_date: Option<String>,
    /// An optional string representing the ID of the color in [`DjmdColor`].
    pub color_id: Option<String>,
    /// An optional integer representing the DJ play count of the track.
    pub dj_play_count: Option<i32>,
    /// An optional string representing the path to the track's image.
    pub image_path: Option<String>,
    /// An optional string representing the ID of the master database.
    pub master_db_id: Option<String>,
    /// An optional string representing the ID of the master song.
    pub master_song_id: Option<String>,
    /// An optional string representing the path to the analysis data.
    pub analysis_data_path: Option<String>,
    /// An optional string used for searching the track.
    pub search_str: Option<String>,
    /// An optional integer representing the file size of the track.
    pub file_size: Option<i64>,
    /// An optional integer representing the disc number of the track.
    pub disc_no: Option<i32>,
    /// An optional string representing the ID of the composer in [`DjmdArtist`].
    pub composer_id: Option<String>,
    /// An optional string representing the subtitle of the track.
    pub subtitle: Option<String>,
    /// Sample rate in Hz.
    pub sample_rate: Option<i64>,
    /// An optional integer indicating whether quantization is disabled for the track.
    pub disable_quantize: Option<i32>,
    /// An optional integer indicating whether the track has been analyzed.
    pub analysed: Option<i32>,
    /// An optional string representing the release date of the track.
    pub release_date: Option<String>,
    /// An optional string representing the creation date of the track.
    pub date_created: Option<String>,
    /// An optional integer representing the content link of the track.
    pub content_link: Option<i32>,
    /// An optional string representing tags associated with the track.
    pub tag: Option<String>,
    /// An optional string indicating whether the track was modified by Rekordbox.
    pub modified_by_rbm: Option<String>,
    /// An optional string representing the auto-load hot cue setting.
    pub hot_cue_auto_load: Option<String>,
    /// An optional string representing the delivery control setting.
    pub delivery_control: Option<String>,
    /// An optional string representing comments about the delivery.
    pub delivery_comment: Option<String>,
    /// An optional string indicating whether the cue points were updated.
    pub cue_updated: Option<String>,
    /// An optional string indicating whether the analysis data was updated.
    pub analysis_updated: Option<String>,
    /// An optional string indicating whether the track info was updated.
    pub track_info_updated: Option<String>,
    /// An optional string representing the lyricist of the track.
    pub lyricist: Option<String>,
    /// An optional string representing the ISRC (International Standard Recording Code) of the track.
    pub isrc: Option<String>,
    /// An optional integer representing sampler track information.
    pub sampler_track_info: Option<i32>,
    /// An optional integer representing the sampler play offset.
    pub sampler_play_offset: Option<i32>,
    /// An optional float representing the sampler gain.
    pub sampler_gain: Option<f64>,
    /// An optional string representing the associated video.
    pub video_associate: Option<String>,
    /// An optional integer representing the lyric status of the track.
    pub lyric_status: Option<i32>,
    /// An optional integer representing the service ID.
    pub service_id: Option<i32>,
    /// An optional string representing the original folder path of the track.
    pub org_folder_path: Option<String>,
    /// An optional string for reserved data.
    pub reserved1: Option<String>,
    /// An optional string for reserved data.
    pub reserved2: Option<String>,
    /// An optional string for reserved data.
    pub reserved3: Option<String>,
    /// An optional string for reserved data.
    pub reserved4: Option<String>,
    /// An optional string for extended information.
    pub ext_info: Option<String>,
    /// An optional string representing the Rekordbox file ID.
    pub rb_file_id: Option<String>,
    /// An optional string representing the ID of the associated device in [`DjmdDevice`].
    pub device_id: Option<String>,
    /// An optional string representing the local folder path in Rekordbox.
    pub rb_local_folder_path: Option<String>,
    /// An optional string representing the source ID.
    pub src_id: Option<String>,
    /// An optional string representing the source title.
    pub src_title: Option<String>,
    /// An optional string representing the source artist name.
    pub src_artist_name: Option<String>,
    /// An optional string representing the source album name.
    pub src_album_name: Option<String>,
    /// An optional integer representing the source length in milliseconds.
    pub src_length: Option<i32>,
}

/// Track snapshot with direct metadata relationships and the album artist resolved.
///
/// Field access performs no queries, and editing this view does not persist changes.
/// Related records are plain rows without recursive expansion. `None` means an
/// absent foreign key or a missing target; the content row retains the original key.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdContentView {
    /// Original content row, including its foreign keys.
    pub content: DjmdContent,
    /// Main artist.
    pub artist: Option<DjmdArtist>,
    /// Artist credited as remixer.
    pub remixer: Option<DjmdArtist>,
    /// Artist credited as original artist.
    pub original_artist: Option<DjmdArtist>,
    /// Artist credited as composer.
    pub composer: Option<DjmdArtist>,
    /// Associated album.
    pub album: Option<DjmdAlbum>,
    /// Artist referenced by the associated album's `album_artist_id`.
    pub album_artist: Option<DjmdArtist>,
    /// Associated genre.
    pub genre: Option<DjmdGenre>,
    /// Associated label.
    pub label: Option<DjmdLabel>,
    /// Associated key.
    pub key: Option<DjmdKey>,
    /// Associated color.
    pub color: Option<DjmdColor>,
}

/// Read row fields directly while keeping relationship resolution explicit.
impl std::ops::Deref for DjmdContentView {
    type Target = DjmdContent;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl ModelRelations for DjmdContent {
    type Resolved = DjmdContentView;

    fn resolve_many(
        conn: &mut SqliteConnection,
        models: Vec<Self>,
    ) -> QueryResult<Vec<Self::Resolved>> {
        if models.is_empty() {
            return Ok(Vec::new());
        }
        conn.transaction(|conn| {
            let albums = load_related(
                conn,
                models.iter().filter_map(|row| row.album_id.clone()),
                |conn, ids| {
                    DjmdAlbum::query()
                        .filter(djmdAlbum::id.eq_any(ids))
                        .load(conn)
                },
                |row| row.id.clone(),
            )?;
            let artists = load_related(
                conn,
                models
                    .iter()
                    .flat_map(|row| {
                        [
                            row.artist_id.clone(),
                            row.remixer_id.clone(),
                            row.org_artist_id.clone(),
                            row.composer_id.clone(),
                        ]
                        .into_iter()
                        .flatten()
                    })
                    .chain(
                        albums
                            .values()
                            .filter_map(|album| album.album_artist_id.clone()),
                    ),
                |conn, ids| {
                    DjmdArtist::query()
                        .filter(djmdArtist::id.eq_any(ids))
                        .load(conn)
                },
                |row| row.id.clone(),
            )?;
            let genres = load_related(
                conn,
                models.iter().filter_map(|row| row.genre_id.clone()),
                |conn, ids| {
                    DjmdGenre::query()
                        .filter(djmdGenre::id.eq_any(ids))
                        .load(conn)
                },
                |row| row.id.clone(),
            )?;
            let labels = load_related(
                conn,
                models.iter().filter_map(|row| row.label_id.clone()),
                |conn, ids| {
                    DjmdLabel::query()
                        .filter(djmdLabel::id.eq_any(ids))
                        .load(conn)
                },
                |row| row.id.clone(),
            )?;
            let keys = load_related(
                conn,
                models.iter().filter_map(|row| row.key_id.clone()),
                |conn, ids| DjmdKey::query().filter(djmdKey::id.eq_any(ids)).load(conn),
                |row| row.id.clone(),
            )?;
            let colors = load_related(
                conn,
                models.iter().filter_map(|row| row.color_id.clone()),
                |conn, ids| {
                    DjmdColor::query()
                        .filter(djmdColor::id.eq_any(ids))
                        .load(conn)
                },
                |row| row.id.clone(),
            )?;
            Ok(models
                .into_iter()
                .map(|content| DjmdContentView {
                    artist: content
                        .artist_id
                        .as_ref()
                        .and_then(|id| artists.get(id))
                        .cloned(),
                    remixer: content
                        .remixer_id
                        .as_ref()
                        .and_then(|id| artists.get(id))
                        .cloned(),
                    original_artist: content
                        .org_artist_id
                        .as_ref()
                        .and_then(|id| artists.get(id))
                        .cloned(),
                    composer: content
                        .composer_id
                        .as_ref()
                        .and_then(|id| artists.get(id))
                        .cloned(),
                    album_artist: content
                        .album_id
                        .as_ref()
                        .and_then(|id| albums.get(id))
                        .and_then(|album| album.album_artist_id.as_ref())
                        .and_then(|id| artists.get(id))
                        .cloned(),
                    album: content
                        .album_id
                        .as_ref()
                        .and_then(|id| albums.get(id))
                        .cloned(),
                    genre: content
                        .genre_id
                        .as_ref()
                        .and_then(|id| genres.get(id))
                        .cloned(),
                    label: content
                        .label_id
                        .as_ref()
                        .and_then(|id| labels.get(id))
                        .cloned(),
                    key: content.key_id.as_ref().and_then(|id| keys.get(id)).cloned(),
                    color: content
                        .color_id
                        .as_ref()
                        .and_then(|id| colors.get(id))
                        .cloned(),
                    content,
                })
                .collect())
        })
    }
}

impl Model for DjmdContent {
    type Id = str;

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }
}

impl ModelUpdate for DjmdContent {
    fn update(mut self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        // Todo: count changes
        self.updated_at = Utc::now();
        self.rb_local_usn = Some(AgentRegistry::increment_local_usn_by(conn, 1)?);
        diesel::update(djmdContent::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl DjmdContent {
    /// Load all tracks and their direct metadata using batched relationship queries.
    /// Content and related rows are read within the same transaction.
    pub fn all_with_relations(conn: &mut SqliteConnection) -> QueryResult<Vec<DjmdContentView>> {
        Ok(conn.transaction::<_, Error, _>(|conn| {
            let contents = DjmdContent::all(conn)?;
            Self::resolve_many(conn, contents)
        })?)
    }

    /// Load a track and all direct metadata relationships in one read transaction.
    /// Returns a detached snapshot; missing relationship targets remain `None`.
    pub fn find_with_relations(
        conn: &mut SqliteConnection,
        id: &str,
    ) -> crate::Result<Option<DjmdContentView>> {
        Ok(conn.transaction::<_, Error, _>(|conn| {
            Self::find(conn, id)?
                .map(|content| content.resolve(conn))
                .transpose()
        })?)
    }

    /// Queries all records from the `djmdContent` table matching the given `ids`.
    pub fn by_ids(conn: &mut SqliteConnection, ids: &[&str]) -> QueryResult<Vec<Self>> {
        Self::query().filter(djmdContent::id.eq_any(ids)).load(conn)
    }

    /// Load matching rows and their direct metadata in one read transaction.
    /// Missing IDs are omitted, duplicates are collapsed, and order is unspecified.
    pub fn by_ids_with_relations(
        conn: &mut SqliteConnection,
        ids: &[&str],
    ) -> crate::Result<Vec<DjmdContentView>> {
        conn.transaction::<_, MasterDbError, _>(|conn| {
            let contents = Self::by_ids(conn, ids)?;
            Ok(Self::resolve_many(conn, contents)?)
        })
    }

    /// Queries a record from the `djmdContent` table by its `folder_path`.
    pub fn find_by_path(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdContent::folder_path.eq(name))
            .first(conn)
            .optional()
    }

    /// Queries a record from the `djmdContent` table by its `folder_path` and loads all direct metadata relationships in one read transaction.
    pub fn find_by_path_with_relations(
        conn: &mut SqliteConnection,
        id: &str,
    ) -> crate::Result<Option<DjmdContentView>> {
        Ok(conn.transaction::<_, Error, _>(|conn| {
            Self::find_by_path(conn, id)?
                .map(|content| content.resolve(conn))
                .transpose()
        })?)
    }

    /// Checks if a record with the given `folder_path` exists in the `djmdContent` table.
    pub fn path_exists(conn: &mut SqliteConnection, path: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdContent::folder_path.eq(path));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Checks if a record with the given `rb_file_id` exists in the `djmdContent` table.
    pub fn file_id_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdContent::rb_file_id.eq(id));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Queries the path to the analysis data for a record with the given `id`.
    pub fn find_anlz_path(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<String>> {
        djmdContent::table
            .find(id)
            .select(djmdContent::analysis_data_path)
            .first(conn)
    }

    /// Updates an existing record in the `djmdContent` table.
    pub fn update(mut self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        self.updated_at = Utc::now();
        self.rb_local_usn = Some(AgentRegistry::increment_local_usn_by(conn, 1)?);
        diesel::update(djmdContent::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }

    /// Sets the `analysis_data_path` field of a record in the `djmdContent` table matching the given `id`.
    pub fn set_album_id(
        conn: &mut SqliteConnection,
        id: &str,
        album_id: &str,
    ) -> QueryResult<usize> {
        diesel::update(djmdContent::table.find(id))
            .set(djmdContent::album_id.eq(album_id))
            .execute(conn)
    }

    /// Sets the `artist_id` field of a record in the `djmdContent` table matching the given `id`.
    pub fn set_artist_id(
        conn: &mut SqliteConnection,
        id: &str,
        artist_id: &str,
    ) -> QueryResult<usize> {
        diesel::update(djmdContent::table.find(id))
            .set(djmdContent::artist_id.eq(artist_id))
            .execute(conn)
    }

    /// Sets the `remixer_id` field of a record in the `djmdContent` table matching the given `id`.
    pub fn set_remixer_id(
        conn: &mut SqliteConnection,
        id: &str,
        artist_id: &str,
    ) -> QueryResult<usize> {
        diesel::update(djmdContent::table.find(id))
            .set(djmdContent::remixer_id.eq(artist_id))
            .execute(conn)
    }

    /// Sets the `org_artist_id` field of a record in the `djmdContent` table matching the given `id`.
    pub fn set_original_artist_id(
        conn: &mut SqliteConnection,
        id: &str,
        artist_id: &str,
    ) -> QueryResult<usize> {
        diesel::update(djmdContent::table.find(id))
            .set(djmdContent::org_artist_id.eq(artist_id))
            .execute(conn)
    }

    /// Sets the `composer_id` field of a record in the `djmdContent` table matching the given `id`.
    pub fn set_composer_id(
        conn: &mut SqliteConnection,
        id: &str,
        artist_id: &str,
    ) -> QueryResult<usize> {
        diesel::update(djmdContent::table.find(id))
            .set(djmdContent::composer_id.eq(artist_id))
            .execute(conn)
    }

    /// Sets the `genre_id` field of a record in the `djmdContent` table matching the given `id`.
    pub fn set_genre_id(
        conn: &mut SqliteConnection,
        id: &str,
        genre_id: &str,
    ) -> QueryResult<usize> {
        diesel::update(djmdContent::table.find(id))
            .set(djmdContent::genre_id.eq(genre_id))
            .execute(conn)
    }

    /// Sets the `label_id` field of a record in the `djmdContent` table matching the given `id`.
    pub fn set_label_id(
        conn: &mut SqliteConnection,
        id: &str,
        label_id: &str,
    ) -> QueryResult<usize> {
        diesel::update(djmdContent::table.find(id))
            .set(djmdContent::label_id.eq(label_id))
            .execute(conn)
    }

    /// Sets the `key_id` field of a record in the `djmdContent` table matching the given `id`.
    pub fn set_key_id(conn: &mut SqliteConnection, id: &str, key_id: &str) -> QueryResult<usize> {
        diesel::update(djmdContent::table.find(id))
            .set(djmdContent::key_id.eq(key_id))
            .execute(conn)
    }

    /// Sets the `folder_path` field of a record in the `djmdContent` table matching the given `id`.
    pub fn set_path(conn: &mut SqliteConnection, id: &str, path: &str) -> QueryResult<usize> {
        diesel::update(djmdContent::table.find(id))
            .set(djmdContent::folder_path.eq(path))
            .execute(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdContent` table.
    fn generate_id(conn: &mut SqliteConnection) -> QueryResult<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                if !Self::id_exists(conn, &tmp_id)? {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Generates a new unique identifier for a record in the `djmdContent` table.
    fn generate_file_id(conn: &mut SqliteConnection) -> QueryResult<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                if !Self::file_id_exists(conn, &tmp_id)? {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }
}

/// Represents a new record insertable into the `djmdContent` table.
///
/// Insertion uses the library identity from `djmdProperty` and initializes local
/// playback, sampler and empty-text fields using observed import defaults.
/// Explicit builder values take precedence. `ExtInfo` defaults to the text
/// `"null"`; `ContentLink` is left absent unless supplied by the caller.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox_masterdb::models::NewDjmdContent;
///
/// let new = NewDjmdContent::new("path/to/file").title("Title").artist_id("1");
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdContent {
    /// The path of the track file.
    pub folder_path: String,
    /// An optional string representing the title of the track.
    pub title: Option<String>,
    /// An optional string representing the ID of the associated artist in [`DjmdArtist`].
    pub artist_id: Option<String>,
    /// An optional string representing the ID of the associated album in [`DjmdAlbum`].
    pub album_id: Option<String>,
    /// An optional string representing the ID of the associated genre in [`DjmdGenre`].
    pub genre_id: Option<String>,
    /// Track tempo in BPM multiplied by 100.
    pub bpm: Option<i64>,
    /// Track length in rounded seconds.
    pub length: Option<i32>,
    /// An optional integer representing the track number in the album.
    pub track_no: Option<i64>,
    /// Stored bitrate, normally kb/s.
    pub bit_rate: Option<i64>,
    /// An optional integer representing the bit depth of the track.
    pub bit_depth: Option<i32>,
    /// An optional string representing comments about the track.
    pub commnt: Option<String>,
    /// Explicit Rekordbox file type, or `None` to infer it from the path extension.
    pub file_type: Option<i32>,
    /// An optional integer representing the rating of the track.
    pub rating: Option<i32>,
    /// An optional integer representing the release year of the track.
    pub release_year: Option<i32>,
    /// An optional string representing the ID of the remixer in [`DjmdArtist`].
    pub remixer_id: Option<String>,
    /// An optional string representing the ID of the label in [`DjmdLabel`].
    pub label_id: Option<String>,
    /// An optional string representing the ID of the original artist in [`DjmdArtist`].
    pub org_artist_id: Option<String>,
    /// An optional string representing the ID of the key in [`DjmdKey`].
    pub key_id: Option<String>,
    /// An optional string representing the stock date of the track.
    pub stock_date: Option<String>,
    /// An optional string representing the ID of the color in [`DjmdColor`].
    pub color_id: Option<String>,
    /// An optional integer representing the DJ play count of the track.
    pub dj_play_count: Option<i32>,
    /// An optional string representing the path to the track's image.
    pub image_path: Option<String>,
    /// An optional string representing the path to the analysis data.
    pub analysis_data_path: Option<String>,
    /// An optional string used for searching the track.
    pub search_str: Option<String>,
    /// An optional integer representing the disc number of the track.
    pub disc_no: Option<i32>,
    /// An optional string representing the ID of the composer in [`DjmdArtist`].
    pub composer_id: Option<String>,
    /// An optional string representing the subtitle of the track.
    pub subtitle: Option<String>,
    /// Sample rate in Hz.
    pub sample_rate: Option<i64>,
    /// An optional string representing the release date of the track.
    pub release_date: Option<String>,
    /// An optional string representing the creation date of the track.
    pub date_created: Option<String>,
    /// An optional integer representing the content link of the track.
    pub content_link: Option<i32>,
    /// An optional string representing tags associated with the track.
    pub tag: Option<String>,
    // /// An optional string representing the auto-load hot cue setting.
    // pub hot_cue_auto_load: Option<String>,
    // /// An optional string representing the delivery control setting.
    //  delivery_control: Option<String>,
    /// An optional string representing comments about the delivery.
    pub delivery_comment: Option<String>,
    /// An optional string representing the lyricist of the track.
    pub lyricist: Option<String>,
    /// An optional string representing the ISRC (International Standard Recording Code) of the track.
    pub isrc: Option<String>,
    /// An optional integer representing sampler track information.
    pub sampler_track_info: Option<i32>,
    /// An optional integer representing the sampler play offset.
    pub sampler_play_offset: Option<i32>,
    /// An optional float representing the sampler gain.
    pub sampler_gain: Option<f64>,
    /// An optional string representing the associated video.
    pub video_associate: Option<String>,
    /// An optional integer representing the lyric status of the track.
    pub lyric_status: Option<i32>,
    /// An optional string representing the original folder path of the track.
    pub org_folder_path: Option<String>,
    /// An optional string for reserved data.
    pub reserved1: Option<String>,
    /// An optional string for reserved data.
    pub reserved2: Option<String>,
    /// An optional string for reserved data.
    pub reserved3: Option<String>,
    /// An optional string for reserved data.
    pub reserved4: Option<String>,
    /// An optional string for extended information.
    pub ext_info: Option<String>,
    /// An optional string representing the ID of the associated device in [`DjmdDevice`].
    pub device_id: Option<String>,
    /// An optional string representing the local folder path in Rekordbox.
    pub rb_local_folder_path: Option<String>,
    /// An optional string representing the source ID.
    pub src_id: Option<String>,
    /// An optional string representing the source title.
    pub src_title: Option<String>,
    /// An optional string representing the source artist name.
    pub src_artist_name: Option<String>,
    /// An optional string representing the source album name.
    pub src_album_name: Option<String>,
    /// An optional integer representing the source length in milliseconds.
    pub src_length: Option<i32>,
}

impl ModelInsert for NewDjmdContent {
    type Model = DjmdContent;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let path = PathBuf::from(&self.folder_path);
        let rb_path_str = path
            .normalize_unix_string()
            .expect("Failed to convert path to string");
        if !path.is_file() || !path.exists() {
            let e = Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                Box::new(format!("File does not exist: {}", rb_path_str)),
            );
            return Err(e);
        }
        if Self::Model::path_exists(conn, &rb_path_str)? {
            let e = Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                Box::new(format!("Path is not unique: {}", rb_path_str)),
            );
            return Err(e);
        }

        // Validate input and read metadata before changing the local registry.
        let invalid_input = |message: String| {
            Error::SerializationError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                message,
            )))
        };
        let file_type = match self.file_type {
            Some(file_type) => file_type,
            None => {
                let ext = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .ok_or_else(|| {
                        invalid_input(format!(
                            "Missing or invalid file extension: {}",
                            rb_path_str
                        ))
                    })?;
                FileType::try_from_extension(ext)
                    .map_err(invalid_input)?
                    .into()
            }
        };
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| invalid_input(format!("Invalid file name: {}", rb_path_str)))?
            .to_string();
        let meta =
            std::fs::metadata(&path).map_err(|err| Error::SerializationError(Box::new(err)))?;
        let file_size =
            i64::try_from(meta.len()).map_err(|err| Error::SerializationError(Box::new(err)))?;
        let (db_id, library_device_id): (String, String) = djmdProperty::table
            .select((djmdProperty::db_id, djmdProperty::device_id))
            .first(conn)?;
        // ContentLink is track-state metadata, not a menu USN or a maximum
        // borrowed from another track. Leave it absent unless supplied.
        let content_link = self.content_link;
        let id = Self::Model::generate_id(conn)?;
        let file_id = Self::Model::generate_file_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        let now = Utc::now();

        let item = Self::Model {
            id: id.clone(),
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            folder_path: Some(rb_path_str.to_string()),
            file_name_l: Some(file_name.clone()),
            file_name_s: Some("".to_string()),
            file_type: file_type as i32,
            master_db_id: Some(db_id),
            master_song_id: Some(id),
            file_size: Some(file_size),
            content_link,
            hot_cue_auto_load: Some("on".to_string()),
            delivery_control: Some(String::new()),
            modified_by_rbm: Some(String::new()),
            service_id: Some(0),
            rb_file_id: Some(file_id),
            title: self.title,
            artist_id: self.artist_id,
            album_id: self.album_id,
            genre_id: self.genre_id,
            bpm: self.bpm,
            length: self.length,
            track_no: self.track_no,
            bit_rate: self.bit_rate,
            bit_depth: self.bit_depth,
            commnt: self.commnt,
            rating: self.rating.or(Some(0)),
            release_year: self.release_year,
            remixer_id: self.remixer_id,
            label_id: self.label_id,
            org_artist_id: self.org_artist_id,
            key_id: self.key_id,
            stock_date: self.stock_date,
            color_id: Some(self.color_id.unwrap_or_else(|| "0".into())),
            dj_play_count: self.dj_play_count.or(Some(0)),
            image_path: self.image_path,
            analysis_data_path: self.analysis_data_path,
            search_str: self.search_str,
            disc_no: self.disc_no.or(Some(0)),
            composer_id: self.composer_id,
            subtitle: Some(self.subtitle.unwrap_or_default()),
            sample_rate: self.sample_rate,
            release_date: self.release_date,
            date_created: self.date_created,
            tag: self.tag,
            delivery_comment: Some(self.delivery_comment.unwrap_or_default()),
            lyricist: Some(self.lyricist.unwrap_or_default()),
            isrc: self.isrc,
            sampler_track_info: self.sampler_track_info.or(Some(0)),
            sampler_play_offset: self.sampler_play_offset.or(Some(0)),
            sampler_gain: self.sampler_gain.or(Some(0.0)),
            video_associate: Some(self.video_associate.unwrap_or_else(|| "0".into())),
            lyric_status: self.lyric_status.or(Some(0)),
            org_folder_path: Some(self.org_folder_path.unwrap_or_default()),
            reserved1: Some(self.reserved1.unwrap_or_default()),
            reserved2: self.reserved2,
            reserved3: self.reserved3,
            reserved4: self.reserved4,
            ext_info: Some(self.ext_info.unwrap_or_else(|| "null".into())),
            device_id: Some(self.device_id.unwrap_or(library_device_id)),
            rb_local_folder_path: self.rb_local_folder_path,
            src_id: self.src_id,
            src_title: self.src_title,
            src_artist_name: self.src_artist_name,
            src_album_name: self.src_album_name,
            src_length: self.src_length,
            analysed: Some(Analyzed::NotAnalyzed as i32),
            ..Default::default()
        };
        diesel::insert_into(djmdContent::table)
            .values(item)
            .get_result(conn)
    }
}

/// Represents a new record insertale to the `djmdContent` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox_masterdb::models::NewDjmdContent;
///
/// let new = NewDjmdContent::new("path/to/file.mp3").title("Title");
/// println!("{:?}", new);
/// ```
impl NewDjmdContent {
    /// Creates a new record with the required fields.
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self {
            folder_path: path.into(),
            ..Default::default()
        }
    }

    /// Use an explicit Rekordbox file type instead of inferring it from the extension.
    ///
    /// This is useful for containers such as M4A, whose codec cannot be determined
    /// from the extension alone, and for files with non-standard extensions.
    pub fn file_type(mut self, file_type: i32) -> Self {
        self.file_type = Some(file_type);
        self
    }

    /// Builder for `title`.
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Builder for `artist_id`.
    pub fn artist_id<S: Into<String>>(mut self, artist_id: S) -> Self {
        self.artist_id = Some(artist_id.into());
        self
    }

    /// Builder for `album_id`.
    pub fn album_id<S: Into<String>>(mut self, album_id: S) -> Self {
        self.album_id = Some(album_id.into());
        self
    }

    /// Builder for `genre_id`.
    pub fn genre_id<S: Into<String>>(mut self, genre_id: S) -> Self {
        self.genre_id = Some(genre_id.into());
        self
    }

    /// Builder for `bpm`.
    pub fn bpm(mut self, bpm: i64) -> Self {
        self.bpm = Some(bpm);
        self
    }

    /// Builder for `length`.
    pub fn length(mut self, length: i32) -> Self {
        self.length = Some(length);
        self
    }

    /// Builder for `track_no`.
    pub fn track_no(mut self, track_no: i64) -> Self {
        self.track_no = Some(track_no);
        self
    }

    /// Builder for `bit_rate`.
    pub fn bit_rate(mut self, bit_rate: i64) -> Self {
        self.bit_rate = Some(bit_rate);
        self
    }

    /// Builder for `bit_depth`.
    pub fn bit_depth(mut self, bit_depth: i32) -> Self {
        self.bit_depth = Some(bit_depth);
        self
    }

    /// Builder for `commnt`.
    pub fn commnt<S: Into<String>>(mut self, commnt: S) -> Self {
        self.commnt = Some(commnt.into());
        self
    }

    /// Builder for `rating`.
    pub fn rating(mut self, rating: i32) -> Self {
        self.rating = Some(rating);
        self
    }

    /// Builder for `release_year`.
    pub fn release_year(mut self, release_year: i32) -> Self {
        self.release_year = Some(release_year);
        self
    }

    /// Builder for `remixer_id`.
    pub fn remixer_id<S: Into<String>>(mut self, remixer_id: S) -> Self {
        self.remixer_id = Some(remixer_id.into());
        self
    }

    /// Builder for `label_id`.
    pub fn label_id<S: Into<String>>(mut self, label_id: S) -> Self {
        self.label_id = Some(label_id.into());
        self
    }

    /// Builder for `org_artist_id`.
    pub fn org_artist_id<S: Into<String>>(mut self, org_artist_id: S) -> Self {
        self.org_artist_id = Some(org_artist_id.into());
        self
    }

    /// Builder for `key_id`.
    pub fn key_id<S: Into<String>>(mut self, key_id: S) -> Self {
        self.key_id = Some(key_id.into());
        self
    }

    /// Builder for `stock_date`.
    pub fn stock_date<S: Into<String>>(mut self, stock_date: S) -> Self {
        self.stock_date = Some(stock_date.into());
        self
    }

    /// Builder for `search_str`.
    pub fn search_str<S: Into<String>>(mut self, search_str: S) -> Self {
        self.search_str = Some(search_str.into());
        self
    }

    /// Builder for `disc_no`.
    pub fn disc_no(mut self, disc_no: i32) -> Self {
        self.disc_no = Some(disc_no);
        self
    }

    /// Builder for `composer_id`.
    pub fn composer_id<S: Into<String>>(mut self, composer_id: S) -> Self {
        self.composer_id = Some(composer_id.into());
        self
    }

    /// Builder for `subtitle`.
    pub fn subtitle<S: Into<String>>(mut self, subtitle: S) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Builder for `sample_rate`.
    pub fn sample_rate(mut self, sample_rate: i64) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    /// Builder for `release_date`.
    pub fn release_date<S: Into<String>>(mut self, release_date: S) -> Self {
        self.release_date = Some(release_date.into());
        self
    }

    /// Builder for `date_created`.
    pub fn date_created<S: Into<String>>(mut self, date_created: S) -> Self {
        self.date_created = Some(date_created.into());
        self
    }

    /// Builder for `tag`.
    pub fn tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Builder for `lyricist`.
    pub fn lyricist<S: Into<String>>(mut self, lyricist: S) -> Self {
        self.lyricist = Some(lyricist.into());
        self
    }

    /// Builder for `isrc`.
    pub fn isrc<S: Into<String>>(mut self, isrc: S) -> Self {
        self.isrc = Some(isrc.into());
        self
    }
}

/// Convert a resolved view without querying the database. Missing paths become empty paths.
/// Negative or out-of-range numeric metadata is rejected.
/// `master_content_id` is the row ID and `source` is `MasterDb`. The owning
/// library ID cannot be inferred from a content row: `master_db_id` stays `None`.
/// [`crate::MasterDb::get_tracks`] supplies it from `DjmdProperty`.
impl TryFrom<&DjmdContentView> for Track {
    type Error = MasterDbError;

    fn try_from(view: &DjmdContentView) -> Result<Self, Self::Error> {
        let input = &view.content;
        let id = input.id.parse::<u32>().map_err(|_| {
            MasterDbError::InvalidOperation("content ID must be a decimal u32".into())
        })?;
        if input.rating.is_some_and(|value| !(0..=5).contains(&value)) {
            return Err(MasterDbError::InvalidOperation(
                "rating must be between 0 and 5".into(),
            ));
        }
        if input.bpm.is_some_and(|value| value < 0) || input.length.is_some_and(|value| value < 0) {
            return Err(MasterDbError::InvalidOperation(
                "bpm and length must be nonnegative".into(),
            ));
        }
        Ok(Self {
            id: Some(id),
            master_content_id: Some(id),
            master_db_id: None,
            source: TrackSource::MasterDb,
            uuid: input.uuid.clone(),
            artwork_path: input.image_path.as_ref().map(PathBuf::from),
            analysis_data_path: input.analysis_data_path.as_ref().map(PathBuf::from),
            color: input
                .color_id
                .as_ref()
                .map(|value| value.parse::<u8>())
                .transpose()
                .map_err(|_| {
                    MasterDbError::InvalidOperation(
                        "color_id is outside the Track u8 range or is not numeric".into(),
                    )
                })?,
            play_count: input
                .dj_play_count
                .map(u32::try_from)
                .transpose()
                .map_err(|_| {
                    MasterDbError::InvalidOperation(
                        "dj_play_count is outside the Track u32 range".into(),
                    )
                })?,
            hot_cue_auto_load: input.hot_cue_auto_load.clone(),
            delivery_control: input.delivery_control.clone(),
            delivery_comment: input.delivery_comment.clone(),
            path: input
                .folder_path
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_default(),
            file_type: Some(input.file_type),
            bpm: input.bpm.map(|value| value as f64 / 100.0),
            length: input.length.map(f64::from),
            title: input.title.clone(),
            search_str: input.search_str.clone(),
            tag: input.tag.clone(),
            commnt: input.commnt.clone(),
            stock_date: input.stock_date.clone(),
            subtitle: input.subtitle.clone(),
            release_date: input.release_date.clone(),
            date_created: input.date_created.clone(),
            lyricist: input.lyricist.clone(),
            isrc: input.isrc.clone(),
            rating: input.rating.map(u8::try_from).transpose().map_err(|_| {
                MasterDbError::InvalidOperation("rating is outside the Track u8 range".into())
            })?,
            track_no: input.track_no.map(u32::try_from).transpose().map_err(|_| {
                MasterDbError::InvalidOperation("track_no is outside the Track u32 range".into())
            })?,
            bit_rate: input.bit_rate.map(u32::try_from).transpose().map_err(|_| {
                MasterDbError::InvalidOperation("bit_rate is outside the Track u32 range".into())
            })?,
            bit_depth: input
                .bit_depth
                .map(u16::try_from)
                .transpose()
                .map_err(|_| {
                    MasterDbError::InvalidOperation(
                        "bit_depth is outside the Track u16 range".into(),
                    )
                })?,
            release_year: input
                .release_year
                .map(u16::try_from)
                .transpose()
                .map_err(|_| {
                    MasterDbError::InvalidOperation(
                        "release_year is outside the Track u16 range".into(),
                    )
                })?,
            file_size: input
                .file_size
                .map(u64::try_from)
                .transpose()
                .map_err(|_| {
                    MasterDbError::InvalidOperation(
                        "file_size is outside the Track u64 range".into(),
                    )
                })?,
            disc_no: input.disc_no.map(u32::try_from).transpose().map_err(|_| {
                MasterDbError::InvalidOperation("disc_no is outside the Track u32 range".into())
            })?,
            sample_rate: input
                .sample_rate
                .map(u32::try_from)
                .transpose()
                .map_err(|_| {
                    MasterDbError::InvalidOperation(
                        "sample_rate is outside the Track u32 range".into(),
                    )
                })?,
            artist: view.artist.as_ref().map(|row| row.name.clone()),
            album: view.album.as_ref().map(|row| row.name.clone()),
            genre: view.genre.as_ref().map(|row| row.name.clone()),
            remixer: view.remixer.as_ref().map(|row| row.name.clone()),
            org_artist: view.original_artist.as_ref().map(|row| row.name.clone()),
            composer: view.composer.as_ref().map(|row| row.name.clone()),
            key: view.key.as_ref().map(|row| row.name.clone()),
            label: view.label.as_ref().map(|row| row.name.clone()),
            album_artist: view.album_artist.as_ref().map(|row| row.name.clone()),
        })
    }
}

/// Build an insertion row without resolving names or writing to the database.
/// BPM is rounded to hundredths and duration to seconds. File size is read from
/// the actual file by `insert`; the metadata size hint is not copied.
impl TryFrom<&TrackMetadata> for NewDjmdContent {
    type Error = MasterDbError;

    fn try_from(input: &TrackMetadata) -> Result<Self, Self::Error> {
        if input.rating.is_some_and(|value| value > 5) {
            return Err(MasterDbError::InvalidOperation(
                "rating must be between 0 and 5".into(),
            ));
        }
        Ok(Self {
            folder_path: input
                .path
                .normalize_unix_string()
                .ok_or_else(|| MasterDbError::InvalidOperation("Track path is not UTF-8".into()))?,
            file_type: input.file_type,
            title: input.title.clone(),
            search_str: input.search_str.clone(),
            tag: input.tag.clone(),
            commnt: input.commnt.clone(),
            stock_date: input.stock_date.clone(),
            subtitle: input.subtitle.clone(),
            release_date: input.release_date.clone(),
            date_created: input.date_created.clone(),
            lyricist: input.lyricist.clone(),
            isrc: input.isrc.clone(),
            bpm: input
                .bpm
                .map(|value| {
                    let rounded = (value * 100.0).round();
                    if !value.is_finite() || value < 0.0 || rounded >= i64::MAX as f64 {
                        return Err(MasterDbError::InvalidOperation(
                            "bpm must be finite, nonnegative, and fit the database's i64 range"
                                .into(),
                        ));
                    }
                    Ok(rounded as i64)
                })
                .transpose()?,
            length: input
                .length
                .map(|value| {
                    let rounded = (value * 1.0).round();
                    if !value.is_finite() || value < 0.0 || rounded > f64::from(i32::MAX) {
                        return Err(MasterDbError::InvalidOperation(
                            "length must be finite, nonnegative, and fit the database's i32 range"
                                .into(),
                        ));
                    }
                    Ok(rounded as i32)
                })
                .transpose()?,
            rating: input.rating.map(i32::from),
            bit_depth: input.bit_depth.map(i32::from),
            release_year: input.release_year.map(i32::from),
            track_no: input.track_no.map(i64::from),
            bit_rate: input.bit_rate.map(i64::from),
            disc_no: input.disc_no.map(i32::try_from).transpose().map_err(|_| {
                MasterDbError::InvalidOperation("disc_no exceeds the database i32 range".into())
            })?,
            sample_rate: input.sample_rate.map(i64::from),
            ..Default::default()
        })
    }
}

impl TryFrom<DjmdContentView> for Track {
    type Error = MasterDbError;

    fn try_from(input: DjmdContentView) -> Result<Self, Self::Error> {
        Self::try_from(&input)
    }
}

impl TryFrom<TrackMetadata> for NewDjmdContent {
    type Error = MasterDbError;

    fn try_from(input: TrackMetadata) -> Result<Self, Self::Error> {
        Self::try_from(&input)
    }
}
