// Author: Dylan Jones
// Date:   2025-09-02

use chrono::Utc;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
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
use super::schema::djmdContent;
use super::{Date, DateString, RandomIdGenerator};
use crate::masterdb::FileType;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};
use crate::NormalizePath;

/// Represents the `djmdContent` table in the Rekordbox database.
///
/// This struct maps to the `djmdContent` table in the SQLite database used by Rekordbox.
/// It contains metadata and attributes for each track in the collection, including file info,
/// artist, album, genre, and more.
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

impl DjmdContent {
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        path: String,
        file_name: String,
        file_type: i32,
        size: i32,
        db_id: String,
        file_id: String,
        content_link: Option<i32>,
    ) -> Self {
        let hot_cue_auto_load = "on".to_string();
        let deliver_control = "on".to_string();
        Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            folder_path: Some(path.clone()),
            file_name_l: Some(file_name.clone()),
            file_name_s: Some("".to_string()),
            file_type: Some(file_type),
            master_db_id: Some(db_id),
            master_song_id: Some(id),
            file_size: Some(size),
            content_link: content_link,
            hot_cue_auto_load: Some(hot_cue_auto_load),
            delivery_control: Some(deliver_control),
            sampler_track_info: Some(0),
            sampler_play_offset: Some(0),
            sampler_gain: Some(0.0),
            lyric_status: Some(0),
            service_id: Some(0),
            rb_file_id: Some(file_id),
            ..Default::default()
        }
    }

    /// Queries all records from the `djmdContent` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdContent` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

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

    /// Checks if a record with the given `id` exists in the `djmdContent` table.
    pub fn id_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
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

impl NewDjmdContent {
    /// Creates a new record with the required fields.
    pub fn new(path: String) -> Self {
        Self {
            folder_path: path,
            ..Default::default()
        }
    }

    /// Inserts the new record into the database.
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<DjmdContent> {
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
        if DjmdContent::path_exists(conn, rb_path_str)? {
            let e = Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                Box::new(format!("Path is not unique: {}", rb_path_str)),
            );
            return Err(e);
        }

        let id = DjmdContent::generate_id(conn)?;
        let file_id = DjmdContent::generate_file_id(conn)?;
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

        let item = DjmdContent {
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
