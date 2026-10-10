// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use chrono::Utc;
use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::album::Album;
use super::artist::{Artist, Composer, Lyricist, OriginalArtist, Remixer};
use super::color::Color;
use super::cue::Cue;
use super::genre::Genre;
use super::history::HistoryContent;
use super::image::Image;
use super::key::Key;
use super::label::Label;
use super::my_tag::MyTagContent;
use super::playlist::PlaylistContent;
use super::property::Property;
use super::recommend_like::RecommendedLike;
use super::schema::{album, artist, color, content, genre, image, key, label};
use super::{Date, DateString};
// use crate::enums::FileType;
use crate::model_traits::{
    load_related, Model, ModelDelete, ModelInsert, ModelRelations, ModelUpdate,
};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `content` table in the Rekordbox One Library database.
///
/// This struct maps to the `content` table in the One Library export database.
/// It contains metadata and attributes for each track in the collection, including file info,
/// artist, album, genre, and more.
///
/// # Referenced by
/// * [`Cue`] via `content_id` foreign key.
/// * [`HistoryContent`] via `content_id` foreign key.
/// * [`MyTagContent`] via `content_id` foreign key.
/// * [`PlaylistContent`] via `content_id` foreign key.
/// * [`RecommendedLike`] via `content_id_1` and `content_id_2` foreign keys.
///
/// # References
/// * [`Artist`] via `artist_id`, `remixer_id`, `original_artist_id`, `composer_id` and `lyricist_id` foreign keys.
/// * [`Album`] via `album_id` foreign key.
/// * [`Genre`] via `genre_id` foreign key.
/// * [`Label`] via `label_id` foreign key.
/// * [`Key`] via `key_id` foreign key.
/// * [`Color`] via `color_id` foreign key.
/// * [`Image`] via `image_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = content)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(Artist, foreign_key = artist_id))]
#[diesel(belongs_to(Remixer, foreign_key = remixer_id))]
#[diesel(belongs_to(OriginalArtist, foreign_key = original_artist_id))]
#[diesel(belongs_to(Composer, foreign_key = composer_id))]
#[diesel(belongs_to(Lyricist, foreign_key = lyricist_id))]
#[diesel(belongs_to(Album, foreign_key = album_id))]
#[diesel(belongs_to(Genre, foreign_key = genre_id))]
#[diesel(belongs_to(Label, foreign_key = label_id))]
#[diesel(belongs_to(Key, foreign_key = key_id))]
#[diesel(belongs_to(Color, foreign_key = color_id))]
#[diesel(belongs_to(Image, foreign_key = image_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Content {
    /// The unique identifier of the content.
    pub id: i32,
    /// The title of the track.
    pub title: Option<String>,
    /// An optional title used for searching.
    pub title_for_search: Option<String>,
    /// The subtitle of the track
    pub subtitle: Option<String>,
    /// The beats per minute multiplied by 100.
    pub bpmx100: Option<i32>,
    /// The length of the track in seconds.
    pub length: Option<i32>,
    /// The track number of the track
    pub track_no: Option<i32>,
    /// The disc number of the track
    pub disc_no: Option<i32>,
    /// Optional foreign key referencing the artist [`Artist`].
    pub artist_id: Option<i32>,
    /// Optional foreign key referencing the remixer [`Artist`].
    pub remixer_id: Option<i32>,
    /// Optional foreign key referencing the original artist [`Artist`].
    pub original_artist_id: Option<i32>,
    /// Optional foreign key referencing the composer [`Artist`].
    pub composer_id: Option<i32>,
    /// Optional foreign key referencing the lyricist [`Artist`].
    pub lyricist_id: Option<i32>,
    /// Optional foreign key referencing the [`Album`].
    pub album_id: Option<i32>,
    /// Optional foreign key referencing the [`Genre`].
    pub genre_id: Option<i32>,
    /// Optional foreign key referencing the [`Label`].
    pub label_id: Option<i32>,
    /// Optional foreign key referencing the [`Key`].
    pub key_id: Option<i32>,
    /// Optional foreign key referencing the [`Color`].
    pub color_id: Option<i32>,
    /// Optional foreign key referencing the artwork [`Image`].
    pub image_id: Option<i32>,
    /// The ccomment of the track.
    pub dj_comment: Option<String>,
    /// The rating of the track.
    pub rating: Option<i32>,
    /// The release year of the track.
    pub release_year: Option<i32>,
    /// The release date of the track
    pub release_date: Option<String>,
    /// The date the record was created
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub date_created: Date,
    /// The date the record was added
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub date_added: Date,
    /// The relative file path of the track (usually `'/Contents/<Artist>/<Title>'`)
    pub path: String,
    /// The file name of the track.
    pub file_name: Option<String>,
    /// The file size of the track
    pub file_size: Option<i32>,
    /// The file type of the track
    pub file_type: i32,
    /// The bitrate of the trackk
    pub bitrate: Option<i32>,
    /// The bit depth of the track
    pub bit_depth: Option<i32>,
    /// The sample rate of the track.
    pub sampling_rate: Option<i32>,
    /// The ISRC of the track
    pub isrc: Option<String>,
    /// A flag if hot-cue auto load is enabled
    pub is_hot_cue_auto_load_on: Option<i32>,
    /// A flag if kuvo delivery is enabled
    pub is_kuvo_deliver_status_on: Option<i32>,
    /// A kuvo delivery comment
    pub kuvo_delivery_comment: Option<String>,
    /// The masterdb ID
    pub master_db_id: Option<i32>,
    /// The ID of the contnent in the masterdb
    pub master_content_id: Option<i32>,
    /// The relative path to the analysis data
    pub analysis_data_file_path: Option<String>,
    /// The number of bits analysed (?)
    pub analysed_bits: Option<i32>,
    /// The ID of the content in the content link database (seems to always be 788224)
    pub content_link: Option<i32>,
    /// A flag if the content has been modified
    pub has_modified: Option<i32>,
    /// The number of updates to the associated cues
    pub cue_update_count: Option<i32>,
    /// The number of updates to the analysis data
    pub analysis_data_update_count: Option<i32>,
    /// The number of updates to the content record
    pub information_update_count: Option<i32>,
}

/// Track snapshot with all supported direct metadata relationships resolved.
///
/// Field access performs no queries, and editing this view does not persist changes.
/// Related records are plain rows without recursive expansion. `None` means an
/// absent foreign key or a missing target; the content row retains the original key.
#[derive(Debug, Clone, PartialEq)]
pub struct ContentView {
    /// Original content row, including its foreign keys.
    pub content: Content,
    /// Main artist.
    pub artist: Option<Artist>,
    /// Artist credited as remixer.
    pub remixer: Option<Artist>,
    /// Artist credited as original artist.
    pub original_artist: Option<Artist>,
    /// Artist credited as composer.
    pub composer: Option<Artist>,
    /// Artist credited as lyricist.
    pub lyricist: Option<Artist>,
    /// Associated album.
    pub album: Option<Album>,
    /// Associated genre.
    pub genre: Option<Genre>,
    /// Associated label.
    pub label: Option<Label>,
    /// Associated key.
    pub key: Option<Key>,
    /// Associated color.
    pub color: Option<Color>,
    /// Associated image.
    pub image: Option<Image>,
}

impl ModelRelations for Content {
    type Resolved = ContentView;

    fn resolve_many(
        conn: &mut SqliteConnection,
        models: Vec<Self>,
    ) -> QueryResult<Vec<Self::Resolved>> {
        if models.is_empty() {
            return Ok(Vec::new());
        }
        conn.transaction(|conn| {
            let artists = load_related(
                conn,
                models.iter().flat_map(|row| {
                    [
                        row.artist_id,
                        row.remixer_id,
                        row.original_artist_id,
                        row.composer_id,
                        row.lyricist_id,
                    ]
                    .into_iter()
                    .flatten()
                }),
                |conn, ids| Artist::query().filter(artist::id.eq_any(ids)).load(conn),
                |row| row.id,
            )?;
            let albums = load_related(
                conn,
                models.iter().filter_map(|row| row.album_id),
                |conn, ids| Album::query().filter(album::id.eq_any(ids)).load(conn),
                |row| row.id,
            )?;
            let genres = load_related(
                conn,
                models.iter().filter_map(|row| row.genre_id),
                |conn, ids| Genre::query().filter(genre::id.eq_any(ids)).load(conn),
                |row| row.id,
            )?;
            let labels = load_related(
                conn,
                models.iter().filter_map(|row| row.label_id),
                |conn, ids| Label::query().filter(label::id.eq_any(ids)).load(conn),
                |row| row.id,
            )?;
            let keys = load_related(
                conn,
                models.iter().filter_map(|row| row.key_id),
                |conn, ids| Key::query().filter(key::id.eq_any(ids)).load(conn),
                |row| row.id,
            )?;
            let colors = load_related(
                conn,
                models.iter().filter_map(|row| row.color_id),
                |conn, ids| Color::query().filter(color::id.eq_any(ids)).load(conn),
                |row| row.id,
            )?;
            let images = load_related(
                conn,
                models.iter().filter_map(|row| row.image_id),
                |conn, ids| Image::query().filter(image::id.eq_any(ids)).load(conn),
                |row| row.id,
            )?;
            Ok(models
                .into_iter()
                .map(|content| ContentView {
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
                        .original_artist_id
                        .as_ref()
                        .and_then(|id| artists.get(id))
                        .cloned(),
                    composer: content
                        .composer_id
                        .as_ref()
                        .and_then(|id| artists.get(id))
                        .cloned(),
                    lyricist: content
                        .lyricist_id
                        .as_ref()
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
                    image: content
                        .image_id
                        .as_ref()
                        .and_then(|id| images.get(id))
                        .cloned(),
                    content,
                })
                .collect())
        })
    }
}

impl Model for Content {
    type Id = i32;

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

impl ModelUpdate for Content {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(content::table.find(self.id))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Content {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let mut result = diesel::delete(content::table.find(id)).execute(conn)?;
            result += Self::cascade_delete(conn, *id)?;
            Self::update_property(conn)?;
            Ok(result)
        })
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let mut result =
                diesel::delete(content::table.filter(content::id.eq_any(&ids))).execute(conn)?;
            let ids: Vec<i32> = ids.iter().map(|id| *(*id)).collect();
            result += Self::cascade_deletes(conn, &ids)?;
            Self::update_property(conn)?;
            Ok(result)
        })
    }
}

impl Content {
    /// Queries all records from the `content` table matching the given `ids`.
    pub fn by_ids(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<Vec<Self>> {
        Self::query().filter(content::id.eq_any(ids)).load(conn)
    }

    /// Queries a record from the `content` table by its `path`.
    pub fn find_by_path(conn: &mut SqliteConnection, path: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(content::path.eq(path))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `path` exists in the `content` table.
    pub fn path_exists(conn: &mut SqliteConnection, path: &str) -> QueryResult<bool> {
        let query = Self::query().filter(content::path.eq(path));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Sets the `artist_id` field of a record in the `content` table matching the given `id`.
    pub fn set_artist_id(
        conn: &mut SqliteConnection,
        id: i32,
        artist_id: i32,
    ) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::artist_id.eq(artist_id))
            .execute(conn)
    }

    /// Sets the `artist_id` field of a record in the `content` table matching the given `id`.
    pub fn set_remixer_id(
        conn: &mut SqliteConnection,
        id: i32,
        artist_id: i32,
    ) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::remixer_id.eq(artist_id))
            .execute(conn)
    }

    /// Sets the `original_artist_id` field of a record in the `content` table matching the given `id`.
    pub fn set_original_artist_id(
        conn: &mut SqliteConnection,
        id: i32,
        artist_id: i32,
    ) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::original_artist_id.eq(artist_id))
            .execute(conn)
    }

    /// Sets the `original_artist_id` field of a record in the `content` table matching the given `id`.
    pub fn set_composer_id(
        conn: &mut SqliteConnection,
        id: i32,
        artist_id: i32,
    ) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::composer_id.eq(artist_id))
            .execute(conn)
    }

    /// Sets the `lyricist_id` field of a record in the `content` table matching the given `id`.
    pub fn set_lyricist_id(
        conn: &mut SqliteConnection,
        id: i32,
        artist_id: i32,
    ) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::lyricist_id.eq(artist_id))
            .execute(conn)
    }

    /// Sets the `album_id` field of a record in the `content` table matching the given `id`.
    pub fn set_album_id(conn: &mut SqliteConnection, id: i32, album_id: i32) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::album_id.eq(album_id))
            .execute(conn)
    }

    /// Sets the `genre_id` field of a record in the `content` table matching the given `id`.
    pub fn set_genre_id(conn: &mut SqliteConnection, id: i32, genre_id: i32) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::genre_id.eq(genre_id))
            .execute(conn)
    }

    /// Sets the `label_id` field of a record in the `content` table matching the given `id`.
    pub fn set_label_id(conn: &mut SqliteConnection, id: i32, label_id: i32) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::label_id.eq(label_id))
            .execute(conn)
    }

    /// Sets the `key_id` field of a record in the `content` table matching the given `id`.
    pub fn set_key_id(conn: &mut SqliteConnection, id: i32, key_id: i32) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::key_id.eq(key_id))
            .execute(conn)
    }

    /// Sets the `color_id` field of a record in the `content` table matching the given `id`.
    pub fn set_color_id(conn: &mut SqliteConnection, id: i32, color_id: i32) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::color_id.eq(color_id))
            .execute(conn)
    }

    /// Sets the `image_id` field of a record in the `content` table matching the given `id`.
    pub fn set_image_id(conn: &mut SqliteConnection, id: i32, image_id: i32) -> QueryResult<usize> {
        diesel::update(content::table.find(id))
            .set(content::image_id.eq(image_id))
            .execute(conn)
    }

    /// Cascade deletion of a single record
    fn cascade_delete(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        let mut result = 0;
        // Remove all cues referencing the content
        result += Cue::delete_by_content_id(conn, id)?;
        // Remove all history_content records referencing the content
        result += HistoryContent::delete_by_content_id(conn, id)?;
        // Remove all playlist_content records referencing the content
        result += PlaylistContent::delete_by_content_id(conn, id)?;
        // Remove all myTag_content records referencing the content
        result += MyTagContent::delete_by_content_id(conn, id)?;
        // Remove all recommendedLike records referencing the content
        result += RecommendedLike::delete_by_content_id(conn, id)?;
        Ok(result)
    }

    /// Cascade deletion of multiple records
    fn cascade_deletes(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        let mut result = 0;
        // Remove all cues referencing the contents
        result += Cue::delete_by_content_ids(conn, ids)?;
        // Remove all history_content records referencing the contents
        result += HistoryContent::delete_by_content_ids(conn, ids)?;
        // Remove all playlist_content records referencing the contents
        result += PlaylistContent::delete_by_content_ids(conn, ids)?;
        // Remove all myTag_content records referencing the contents
        result += MyTagContent::delete_by_content_ids(conn, ids)?;
        // Remove all recommendedLike records referencing the contents
        result += RecommendedLike::delete_by_content_ids(conn, ids)?;
        Ok(result)
    }

    fn update_property(conn: &mut SqliteConnection) -> QueryResult<usize> {
        let count: i64 = content::table.count().get_result(conn)?;
        Property::set_number_of_contents_all(conn, count as i32)
    }
}

/// Represents a new record insertale to the `content` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::one_library::models::NewContent;
///
/// let new = NewContent::new("Content/Artist/Title.mp3");
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = content)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewContent {
    /// The date the record was created
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub date_created: Date,
    /// The date the record was added
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub date_added: Date,
    /// The relative file path of the track
    pub path: String,
    /// The title of the track.
    pub title: Option<String>,
    /// An optional title used for searching.
    pub title_for_search: Option<String>,
    /// The subtitle of the track
    pub subtitle: Option<String>,
    /// The beats per minute multiplied by 100.
    pub bpmx100: Option<i32>,
    /// The length of the track in seconds.
    pub length: Option<i32>,
    /// The track number of the track
    pub track_no: Option<i32>,
    /// The disc number of the track
    pub disc_no: Option<i32>,
    /// Optional foreign key referencing the artist [`Artist`].
    pub artist_id: Option<i32>,
    /// Optional foreign key referencing the remixer [`Artist`].
    pub remixer_id: Option<i32>,
    /// Optional foreign key referencing the original artist [`Artist`].
    pub original_artist_id: Option<i32>,
    /// Optional foreign key referencing the composer [`Artist`].
    pub composer_id: Option<i32>,
    /// Optional foreign key referencing the lyricist [`Artist`].
    pub lyricist_id: Option<i32>,
    /// Optional foreign key referencing the [`Album`].
    pub album_id: Option<i32>,
    /// Optional foreign key referencing the [`Genre`].
    pub genre_id: Option<i32>,
    /// Optional foreign key referencing the [`Label`].
    pub label_id: Option<i32>,
    /// Optional foreign key referencing the [`Key`].
    pub key_id: Option<i32>,
    /// Optional foreign key referencing the [`Color`].
    pub color_id: Option<i32>,
    /// Optional foreign key referencing the artwork [`Image`].
    pub image_id: Option<i32>,
    /// The ccomment of the track.
    pub dj_comment: Option<String>,
    /// The rating of the track.
    pub rating: Option<i32>,
    /// The release year of the track.
    pub release_year: Option<i32>,
    /// The release date of the track
    pub release_date: Option<String>,
    /// The file name of the track.
    pub file_name: Option<String>,
    /// The file size of the track
    pub file_size: Option<i32>,
    /// The file typo of the track
    pub file_type: Option<i32>,
    /// The bitrate of the trackk
    pub bitrate: Option<i32>,
    /// The bit depth of the track
    pub bit_depth: Option<i32>,
    /// The sample rate of the track.
    pub sampling_rate: Option<i32>,
    /// The ISRC of the track
    pub isrc: Option<String>,
    /// A flag if hot-cue auto load is enabled
    pub is_hot_cue_auto_load_on: Option<i32>,
    /// A flag if kuvo delivery is enabled
    pub is_kuvo_deliver_status_on: Option<i32>,
    /// A kuvo delivery comment
    pub kuvo_delivery_comment: Option<String>,
    /// The masterdb ID
    pub master_db_id: Option<i32>,
    /// The ID of the contnent in the masterdb
    pub master_content_id: Option<i32>,
    /// The relative path to the analysis data
    pub analysis_data_file_path: Option<String>,
    /// The number of bits analysed (?)
    pub analysed_bits: Option<i32>,
    /// The ID of the content in the content link database (seems to always be 788224)
    pub content_link: Option<i32>,
    /// A flag if the content has been modified
    pub has_modified: Option<i32>,
    /// The number of updates to the associated cues
    pub cue_update_count: Option<i32>,
    /// The number of updates to the analysis data
    pub analysis_data_update_count: Option<i32>,
    /// The number of updates to the content record
    pub information_update_count: Option<i32>,
}

impl ModelInsert for NewContent {
    type Model = Content;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let result = diesel::insert_into(content::table)
            .values(self)
            .get_result(conn)?;
        Content::update_property(conn)?;
        Ok(result)
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        let results = diesel::insert_into(content::table)
            .values(items)
            .get_results(conn)?;
        Content::update_property(conn)?;
        Ok(results)
    }
}

impl NewContent {
    /// Creates a new content record with the required fields.
    pub fn new<S: Into<String>>(path: S) -> Self {
        let now = Utc::now();
        Self {
            date_created: now.clone(),
            date_added: now.clone(),
            path: path.into(),
            ..Default::default()
        }
    }
}
