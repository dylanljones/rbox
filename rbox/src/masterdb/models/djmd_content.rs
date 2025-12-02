// Author: Dylan Jones
// Date:   2025-09-02

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

use super::super::FileType;
use super::agent_registry::AgentRegistry;
use super::djmd_album::DjmdAlbum;
use super::djmd_artist::DjmdArtist;
use super::djmd_color::DjmdColor;
use super::djmd_device::DjmdDevice;
use super::djmd_genre::DjmdGenre;
use super::djmd_key::DjmdKey;
use super::djmd_label::DjmdLabel;
use super::schema::djmdContent;
use super::{Date, DateString, RandomIdGenerator};
use crate::model_traits::{Model, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};
use crate::NormalizePath;

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
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
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
    /// An optional integer representing the BPM of the track.
    pub bpm: Option<i32>,
    /// An optional integer representing the length of the track in milliseconds.
    pub length: Option<i32>,
    /// An optional integer representing the track number in the album.
    pub track_no: Option<i32>,
    /// An optional integer representing the bit rate of the track.
    pub bit_rate: Option<i32>,
    /// An optional integer representing the bit depth of the track.
    pub bit_depth: Option<i32>,
    /// An optional string representing comments about the track.
    pub commnt: Option<String>,
    /// An optional integer representing the file type of the track.
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
    /// An optional string representing the ID of the master database.
    pub master_db_id: Option<String>,
    /// An optional string representing the ID of the master song.
    pub master_song_id: Option<String>,
    /// An optional string representing the path to the analysis data.
    pub analysis_data_path: Option<String>,
    /// An optional string used for searching the track.
    pub search_str: Option<String>,
    /// An optional integer representing the file size of the track.
    pub file_size: Option<i32>,
    /// An optional integer representing the disc number of the track.
    pub disc_no: Option<i32>,
    /// An optional string representing the ID of the composer in [`DjmdArtist`].
    pub composer_id: Option<String>,
    /// An optional string representing the subtitle of the track.
    pub subtitle: Option<String>,
    /// An optional integer representing the sample rate of the track.
    pub sample_rate: Option<i32>,
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
    /// Queries all records from the `djmdContent` table matching the given `ids`.
    pub fn by_ids(conn: &mut SqliteConnection, ids: &[&str]) -> QueryResult<Vec<Self>> {
        Self::query().filter(djmdContent::id.eq_any(ids)).load(conn)
    }

    /// Queries a record from the `djmdContent` table by its `folder_path`.
    pub fn find_by_path(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdContent::folder_path.eq(name))
            .first(conn)
            .optional()
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

/// Represents a new record insertale to the `djmdContent` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdContent;
///
/// let new = NewDjmdContent::new("path/to/file".into()).title("Title").artist_id("1")
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdContent {
    /// An optional string representing the folder path of the track.
    pub folder_path: String,
    /// An optional string representing the title of the track.
    pub title: Option<String>,
    /// An optional string representing the ID of the associated artist in [`DjmdArtist`].
    pub artist_id: Option<String>,
    /// An optional string representing the ID of the associated album in [`DjmdAlbum`].
    pub album_id: Option<String>,
    /// An optional string representing the ID of the associated genre in [`DjmdGenre`].
    pub genre_id: Option<String>,
    /// An optional integer representing the BPM of the track.
    pub bpm: Option<i32>,
    /// An optional integer representing the length of the track in milliseconds.
    pub length: Option<i32>,
    /// An optional integer representing the track number in the album.
    pub track_no: Option<i32>,
    /// An optional integer representing the bit rate of the track.
    pub bit_rate: Option<i32>,
    /// An optional integer representing the bit depth of the track.
    pub bit_depth: Option<i32>,
    /// An optional string representing comments about the track.
    pub commnt: Option<String>,
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
    /// An optional string used for searching the track.
    pub search_str: Option<String>,
    /// An optional integer representing the disc number of the track.
    pub disc_no: Option<i32>,
    /// An optional string representing the ID of the composer in [`DjmdArtist`].
    pub composer_id: Option<String>,
    /// An optional string representing the subtitle of the track.
    pub subtitle: Option<String>,
    /// An optional integer representing the sample rate of the track.
    pub sample_rate: Option<i32>,
    /// An optional string representing the release date of the track.
    pub release_date: Option<String>,
    /// An optional string representing the creation date of the track.
    pub date_created: Option<String>,
    /// An optional string representing tags associated with the track.
    pub tag: Option<String>,
    /// An optional string representing the lyricist of the track.
    pub lyricist: Option<String>,
    /// An optional string representing the ISRC (International Standard Recording Code) of the track.
    pub isrc: Option<String>,
}

impl ModelInsert for NewDjmdContent {
    type Model = DjmdContent;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let path = PathBuf::from(&self.folder_path);
        let rb_path = path.normalize_sep("/");
        let rb_path_str = rb_path
            .as_os_str()
            .to_str()
            .expect("Failed to convert path to string");
        if !path.is_file() || !path.exists() {
            let e = Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                Box::new(format!("File does not exist: {}", rb_path_str)),
            );
            return Err(e);
        }
        if Self::Model::path_exists(conn, rb_path_str)? {
            let e = Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                Box::new(format!("Path is not unique: {}", rb_path_str)),
            );
            return Err(e);
        }

        let id = Self::Model::generate_id(conn)?;
        let file_id = Self::Model::generate_file_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        let now = Utc::now();
        // get file meta
        let meta = std::fs::metadata(&rb_path).expect("Failed to get file metadata");
        let file_size: u64 = meta.len();
        let ext: &str = path.extension().unwrap().to_str().unwrap();
        let file_type = FileType::try_from_extension(ext).expect("Failed to get file type");
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let db_id = DjmdDevice::master_db_id(conn)?;
        // TODO: No clue what content link should be, for now we just choose the last value used
        let content_link = djmdContent::table
            .select(djmdContent::content_link)
            .order(djmdContent::rb_local_usn.desc())
            .limit(1)
            .first(conn)?;

        let item = Self::Model {
            id: id.clone(),
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            folder_path: Some(rb_path_str.to_string()),
            file_name_l: Some(file_name.clone()),
            file_name_s: Some("".to_string()),
            file_type: Some(file_type as i32),
            master_db_id: Some(db_id),
            master_song_id: Some(id),
            file_size: Some(file_size as i32),
            content_link,
            hot_cue_auto_load: Some("on".to_string()),
            delivery_control: Some("on".to_string()),
            sampler_track_info: Some(0),
            sampler_play_offset: Some(0),
            sampler_gain: Some(0.0),
            lyric_status: Some(0),
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
            rating: self.rating,
            release_year: self.release_year,
            remixer_id: self.remixer_id,
            label_id: self.label_id,
            org_artist_id: self.org_artist_id,
            key_id: self.key_id,
            stock_date: self.stock_date,
            search_str: self.search_str,
            disc_no: self.disc_no,
            composer_id: self.composer_id,
            subtitle: self.subtitle,
            sample_rate: self.sample_rate,
            release_date: self.release_date,
            date_created: self.date_created,
            tag: self.tag,
            lyricist: self.lyricist,
            isrc: self.isrc,
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
/// use rbox::masterdb::models::NewDjmdContent;
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
    pub fn bpm(mut self, bpm: i32) -> Self {
        self.bpm = Some(bpm);
        self
    }

    /// Builder for `length`.
    pub fn length(mut self, length: i32) -> Self {
        self.length = Some(length);
        self
    }

    /// Builder for `track_no`.
    pub fn track_no(mut self, track_no: i32) -> Self {
        self.track_no = Some(track_no);
        self
    }

    /// Builder for `bit_rate`.
    pub fn bit_rate(mut self, bit_rate: i32) -> Self {
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
    pub fn sample_rate(mut self, sample_rate: i32) -> Self {
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
