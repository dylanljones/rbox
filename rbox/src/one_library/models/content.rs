// Author: Dylan Jones
// Date:   2025-09-02

use chrono::Utc;
use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

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
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

type JoinTuple = (
    Content,
    Option<Artist>,
    Option<Remixer>,
    Option<OriginalArtist>,
    Option<Composer>,
    Option<Lyricist>,
    Option<Album>,
    Option<Genre>,
    Option<Label>,
    Option<Key>,
    Option<Color>,
    Option<Image>,
);

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
        let mut result = diesel::delete(content::table.find(id)).execute(conn)?;
        result += Self::cascade_delete(conn, *id)?;
        Self::update_property(conn)?;
        Ok(result)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let mut result =
            diesel::delete(content::table.filter(content::id.eq_any(&ids))).execute(conn)?;
        let ids: Vec<i32> = ids.iter().map(|id| *(*id)).collect();
        result += Self::cascade_deletes(conn, &ids)?;
        Self::update_property(conn)?;
        Ok(result)
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

/// Represents a complete `content` record with all associations.
#[derive(Debug, Clone, PartialEq)]
pub struct ContentJoin {
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
    pub date_created: Date,
    /// The date the record was added
    pub date_added: Date,
    /// The relative file path of the track
    pub path: String,
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

    pub artist: Option<Artist>,
    pub remixer: Option<Remixer>,
    pub original_artist: Option<OriginalArtist>,
    pub composer: Option<Composer>,
    pub lyricist: Option<Lyricist>,
    pub album: Option<Album>,
    pub genre: Option<Genre>,
    pub label: Option<Label>,
    pub key: Option<Key>,
    pub color: Option<Color>,
    pub image: Option<Image>,
}

impl Model for ContentJoin {
    type Id = i32;

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        let (main_artist, remixer, original_artist, composer, lyricist) = diesel::alias!(
            artist as main_artist,
            artist as remixer,
            artist as original_artist,
            artist as composer,
            artist as lyricist
        );

        let results: Vec<JoinTuple> = content::table
            .left_join(
                main_artist.on(main_artist
                    .field(artist::id)
                    .nullable()
                    .eq(content::artist_id)),
            )
            .left_join(remixer.on(remixer.field(artist::id).nullable().eq(content::remixer_id)))
            .left_join(
                original_artist.on(original_artist
                    .field(artist::id)
                    .nullable()
                    .eq(content::original_artist_id)),
            )
            .left_join(
                composer.on(composer
                    .field(artist::id)
                    .nullable()
                    .eq(content::composer_id)),
            )
            .left_join(
                lyricist.on(lyricist
                    .field(artist::id)
                    .nullable()
                    .eq(content::lyricist_id)),
            )
            .left_join(album::table.on(album::id.nullable().eq(content::album_id)))
            .left_join(genre::table.on(genre::id.nullable().eq(content::genre_id)))
            .left_join(label::table.on(label::id.nullable().eq(content::label_id)))
            .left_join(key::table.on(key::id.nullable().eq(content::key_id)))
            .left_join(color::table.on(color::id.nullable().eq(content::color_id)))
            .left_join(image::table.on(image::id.nullable().eq(content::image_id)))
            .load(conn)?;
        Ok(results.into_iter().map(Self::from_join).collect())
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        let (main_artist, remixer, original_artist, composer, lyricist) = diesel::alias!(
            artist as main_artist,
            artist as remixer,
            artist as original_artist,
            artist as composer,
            artist as lyricist
        );

        let result: Option<JoinTuple> = content::table
            .find(id)
            .left_join(
                main_artist.on(main_artist
                    .field(artist::id)
                    .nullable()
                    .eq(content::artist_id)),
            )
            .left_join(remixer.on(remixer.field(artist::id).nullable().eq(content::remixer_id)))
            .left_join(
                original_artist.on(original_artist
                    .field(artist::id)
                    .nullable()
                    .eq(content::original_artist_id)),
            )
            .left_join(
                composer.on(composer
                    .field(artist::id)
                    .nullable()
                    .eq(content::composer_id)),
            )
            .left_join(
                lyricist.on(lyricist
                    .field(artist::id)
                    .nullable()
                    .eq(content::lyricist_id)),
            )
            .left_join(album::table.on(album::id.nullable().eq(content::album_id)))
            .left_join(genre::table.on(genre::id.nullable().eq(content::genre_id)))
            .left_join(label::table.on(label::id.nullable().eq(content::label_id)))
            .left_join(key::table.on(key::id.nullable().eq(content::key_id)))
            .left_join(color::table.on(color::id.nullable().eq(content::color_id)))
            .left_join(image::table.on(image::id.nullable().eq(content::image_id)))
            .first(conn)
            .optional()?;
        Ok(result.map(Self::from_join))
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        Content::id_exists(conn, id)
    }
}

impl ContentJoin {
    fn from_join(tuple: JoinTuple) -> Self {
        let mut result = ContentJoin::from(tuple.0);
        result.artist = tuple.1;
        result.remixer = tuple.2;
        result.original_artist = tuple.3;
        result.composer = tuple.4;
        result.lyricist = tuple.5;
        result.album = tuple.6;
        result.genre = tuple.7;
        result.label = tuple.8;
        result.key = tuple.9;
        result.color = tuple.10;
        result.image = tuple.11;
        result
    }

    /// Queries a record from the `content` table by its `path` with joined associations.
    pub fn find_by_path(conn: &mut SqliteConnection, path: &str) -> QueryResult<Option<Self>> {
        let (main_artist, remixer, original_artist, composer, lyricist) = diesel::alias!(
            artist as main_artist,
            artist as remixer,
            artist as original_artist,
            artist as composer,
            artist as lyricist
        );

        let result: Option<JoinTuple> = content::table
            .filter(content::path.eq(path))
            .left_join(
                main_artist.on(main_artist
                    .field(artist::id)
                    .nullable()
                    .eq(content::artist_id)),
            )
            .left_join(remixer.on(remixer.field(artist::id).nullable().eq(content::remixer_id)))
            .left_join(
                original_artist.on(original_artist
                    .field(artist::id)
                    .nullable()
                    .eq(content::original_artist_id)),
            )
            .left_join(
                composer.on(composer
                    .field(artist::id)
                    .nullable()
                    .eq(content::composer_id)),
            )
            .left_join(
                lyricist.on(lyricist
                    .field(artist::id)
                    .nullable()
                    .eq(content::lyricist_id)),
            )
            .left_join(album::table.on(album::id.nullable().eq(content::album_id)))
            .left_join(genre::table.on(genre::id.nullable().eq(content::genre_id)))
            .left_join(label::table.on(label::id.nullable().eq(content::label_id)))
            .left_join(key::table.on(key::id.nullable().eq(content::key_id)))
            .left_join(color::table.on(color::id.nullable().eq(content::color_id)))
            .left_join(image::table.on(image::id.nullable().eq(content::image_id)))
            .first(conn)
            .optional()?;
        Ok(result.map(Self::from_join))
    }
}

impl Into<Content> for ContentJoin {
    fn into(self) -> Content {
        Content {
            id: self.id,
            title: self.title,
            title_for_search: self.title_for_search,
            subtitle: self.subtitle,
            bpmx100: self.bpmx100,
            length: self.length,
            track_no: self.track_no,
            disc_no: self.disc_no,
            artist_id: self.artist_id,
            remixer_id: self.remixer_id,
            original_artist_id: self.original_artist_id,
            composer_id: self.composer_id,
            lyricist_id: self.lyricist_id,
            album_id: self.album_id,
            genre_id: self.genre_id,
            label_id: self.label_id,
            key_id: self.key_id,
            color_id: self.color_id,
            image_id: self.image_id,
            dj_comment: self.dj_comment,
            rating: self.rating,
            release_year: self.release_year,
            release_date: self.release_date,
            date_created: self.date_created,
            date_added: self.date_added,
            path: self.path,
            file_name: self.file_name,
            file_size: self.file_size,
            file_type: self.file_type,
            bitrate: self.bitrate,
            bit_depth: self.bit_depth,
            sampling_rate: self.sampling_rate,
            isrc: self.isrc,
            is_hot_cue_auto_load_on: self.is_hot_cue_auto_load_on,
            is_kuvo_deliver_status_on: self.is_kuvo_deliver_status_on,
            kuvo_delivery_comment: self.kuvo_delivery_comment,
            master_db_id: self.master_db_id,
            master_content_id: self.master_content_id,
            analysis_data_file_path: self.analysis_data_file_path,
            analysed_bits: self.analysed_bits,
            content_link: self.content_link,
            has_modified: self.has_modified,
            cue_update_count: self.cue_update_count,
            analysis_data_update_count: self.analysis_data_update_count,
            information_update_count: self.information_update_count,
        }
    }
}

impl From<Content> for ContentJoin {
    fn from(raw: Content) -> Self {
        Self {
            id: raw.id,
            title: raw.title,
            title_for_search: raw.title_for_search,
            subtitle: raw.subtitle,
            bpmx100: raw.bpmx100,
            length: raw.length,
            track_no: raw.track_no,
            disc_no: raw.disc_no,
            artist_id: raw.artist_id,
            remixer_id: raw.remixer_id,
            original_artist_id: raw.original_artist_id,
            composer_id: raw.composer_id,
            lyricist_id: raw.lyricist_id,
            album_id: raw.album_id,
            genre_id: raw.genre_id,
            label_id: raw.label_id,
            key_id: raw.key_id,
            color_id: raw.color_id,
            image_id: raw.image_id,
            dj_comment: raw.dj_comment,
            rating: raw.rating,
            release_year: raw.release_year,
            release_date: raw.release_date,
            date_created: raw.date_created,
            date_added: raw.date_added,
            path: raw.path,
            file_name: raw.file_name,
            file_size: raw.file_size,
            file_type: raw.file_type,
            bitrate: raw.bitrate,
            bit_depth: raw.bit_depth,
            sampling_rate: raw.sampling_rate,
            isrc: raw.isrc,
            is_hot_cue_auto_load_on: raw.is_hot_cue_auto_load_on,
            is_kuvo_deliver_status_on: raw.is_kuvo_deliver_status_on,
            kuvo_delivery_comment: raw.kuvo_delivery_comment,
            master_db_id: raw.master_db_id,
            master_content_id: raw.master_content_id,
            analysis_data_file_path: raw.analysis_data_file_path,
            analysed_bits: raw.analysed_bits,
            content_link: raw.content_link,
            has_modified: raw.has_modified,
            cue_update_count: raw.cue_update_count,
            analysis_data_update_count: raw.analysis_data_update_count,
            information_update_count: raw.information_update_count,
            artist: None,
            remixer: None,
            original_artist: None,
            composer: None,
            lyricist: None,
            album: None,
            genre: None,
            label: None,
            key: None,
            color: None,
            image: None,
        }
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
