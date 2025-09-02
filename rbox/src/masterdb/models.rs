// Author: Dylan Jones
// Date:   2025-05-01

//! # Rekordbox master.db database models
//!
//! This module defines the data models that map to the tables in the Rekordbox SQLite database.
//! It uses Diesel ORM for database interactions and provides structures for various tables,
//! including content, playlists, artists, albums, and more. These models facilitate reading,
//! writing, and updating data in the Rekordbox database.
//!
//! ## Features
//! - Comprehensive mapping of Rekordbox database tables to Rust structs.
//! - Support for Diesel ORM operations like querying, inserting, and updating records.
//! - Associations between related tables for seamless navigation of relationships.
//!
//! ## Usage
//! Import the necessary models and use them with Diesel's query builder to interact with the
//! database.

#![allow(non_snake_case)]

use diesel::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::schema;
use super::util::{Date, DateString};

/// Represents the `AgentRegistry` table in the Rekordbox database.
///
/// This struct maps to the `AgentRegistry` table in the SQLite database used by Rekordbox.
/// It contains various fields representing metadata and attributes of the Rekordbox collection.
/// Each entry has a unique name as `registry_id`. The local update sequence number (`USN`), for
/// example, is used to track changes made to the entry locally and if stored in the entry
/// with `registry_id="localUpdateCount"`.
/// Some important fields include:
/// * localUpdateCount
/// * SyncAnalysisDataRootPath
/// * SyncSettingsRootPath
/// * LangPath
///
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::agentRegistry)]
#[diesel(primary_key(registry_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AgentRegistry {
    /// A unique identifier for the registry entry.
    pub registry_id: String,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,
    /// An optional string field for additional identifier data.
    pub id_1: Option<String>,
    /// An optional string field for additional identifier data.
    pub id_2: Option<String>,
    /// An optional integer field for numerical data.
    pub int_1: Option<i32>,
    /// An optional integer field for numerical data.
    pub int_2: Option<i32>,
    /// An optional string field for textual data.
    pub str_1: Option<String>,
    /// An optional string field for textual data.
    pub str_2: Option<String>,
    /// An optional string field for date-related data.
    pub date_1: Option<String>,
    /// An optional string field for date-related data.
    pub date_2: Option<String>,
    /// An optional string field for extended text data.
    pub text_1: Option<String>,
    /// An optional string field for extended text data.
    pub text_2: Option<String>,
}

/// Represents the `CloudAgentRegistry` table in the Rekordbox database.
///
/// This struct maps to the `CloudAgentRegistry` table in the SQLite database used by Rekordbox.
/// It contains similar fields as the [`AgentRegistry`], but is specifically used for cloud-related
/// registry entries.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::cloudAgentRegistry)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CloudAgentRegistry {
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

    /// An optional integer field for numerical data.
    pub int_1: Option<i32>,
    /// An optional integer field for numerical data.
    pub int_2: Option<i32>,
    /// An optional string field for textual data.
    pub str_1: Option<String>,
    /// An optional string field for textual data.
    pub str_2: Option<String>,
    /// An optional string field for date-related data.
    pub date_1: Option<String>,
    /// An optional string field for date-related data.
    pub date_2: Option<String>,
    /// An optional string field for extended text data.
    pub text_1: Option<String>,
    /// An optional string field for extended text data.
    pub text_2: Option<String>,
}

/// Represents the `ContentActiveCensor` table in the Rekordbox database.
///
/// This struct maps to the `ContentActiveCensor` table in the SQLite database used by Rekordbox.
/// It contains the active censor entries [`DjmdActiveCensor`] of a file as stringified JSON data.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::contentActiveCensor)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ContentActiveCensor {
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

    /// An optional string representing the id of the associated [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional string representing the active censors.
    pub active_censors: Option<String>,
    /// An optional integer representing the count of active censors.
    pub rb_activecensor_count: Option<i32>,
}

/// Represents the `ContentCue` table in the Rekordbox database.
///
/// This struct maps to the `ContentCue` table in the SQLite database used by Rekordbox.
/// It contains the cue entry [`DjmdCue`] of a file as stringified JSON data.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::contentCue)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ContentCue {
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

    /// An optional string representing the id of the associated [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional string representing the cues.
    pub cues: Option<String>,
    /// An optional integer representing the count of cues.
    pub rb_cue_count: Option<i32>,
}

/// Represents the `ContentFile` table in the Rekordbox database.
///
/// This struct maps to the `ContentFile` table in the SQLite database used by Rekordbox.
/// It contains metadata and attributes of the additional file of contents. These include
/// the analysis and artwork files linked to a track
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::contentFile)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ContentFile {
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

    /// An optional string representing the id of the associated [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional string representing the file path.
    pub path: Option<String>,
    /// An optional string representing the file hash.
    pub hash: Option<String>,
    /// An optional integer representing the file size.
    pub size: Option<i32>,
    /// An optional string representing the local file path.
    pub rb_local_path: Option<String>,
    /// An optional string representing the in-sync file hash.
    pub rb_insync_hash: Option<String>,
    /// An optional integer representing the in-sync local update sequence number.
    pub rb_insync_local_usn: Option<i32>,
    /// An optional integer indicating whether the file hash is dirty.
    pub rb_file_hash_dirty: Option<i32>,
    /// An optional integer representing the local file status.
    pub rb_local_file_status: Option<i32>,
    /// An optional integer indicating whether the file is in progress.
    pub rb_in_progress: Option<i32>,
    /// An optional integer representing the process type.
    pub rb_process_type: Option<i32>,
    /// An optional string representing the temporary file path.
    pub rb_temp_path: Option<String>,
    /// An optional integer representing the file priority.
    pub rb_priority: Option<i32>,
    /// An optional integer indicating whether the file size is dirty.
    pub rb_file_size_dirty: Option<i32>,
}

/// Represents the `DjmdActiveCensor` table in the Rekordbox database.
///
/// This struct maps to the `DjmdActiveCensor` table in the SQLite database used by Rekordbox.
/// It contains information for actively censoring explicit content of tracks in the Rekordbox
/// collection. Active Censor items behave like two cue points, between which an effect is applied
/// to the audio of a track.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdActiveCensor)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdActiveCensor {
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

    /// An optional string representing the id of the associated [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional integer representing the start time of the censor in milliseconds.
    pub in_msec: Option<i32>,
    /// An optional integer representing the end time of the censor in milliseconds.
    pub out_msec: Option<i32>,
    /// An optional integer representing additional information about the censor.
    pub info: Option<i32>,
    /// An optional string representing the list of parameters for the censor.
    pub parameter_list: Option<String>,
    /// An optional string representing the uuid of the associated [`DjmdContent`].
    pub content_uuid: Option<String>,
}

/// Represents the `DjmdAlbum` table in the Rekordbox database.
///
/// This struct maps to the `DjmdAlbum` table in the SQLite database used by Rekordbox.
/// It stores album-related data, allowing multiple tracks to be associated with the same album.
/// This table includes metadata such as album name, artist, and compilation status.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdAlbum)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdArtist, foreign_key = album_artist_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdAlbum {
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

    /// An optional string representing the name of the album.
    pub name: Option<String>,
    /// An optional string representing the id of the album artist [`DjmdArtist`].
    pub album_artist_id: Option<String>,
    /// An optional string representing the path to the album's image.
    pub image_path: Option<String>,
    /// An optional integer indicating whether the album is a compilation.
    pub compilation: Option<i32>,
    /// An optional string used for searching the album.
    pub search_str: Option<String>,
}

impl DjmdAlbum {
    /// Creates a new instance of [`DjmdAlbum`].
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the album.
    /// * `uuid` - A unique universal identifier for the album.
    /// * `usn` - The update sequence number.
    /// * `now` - The current timestamp.
    /// * `name` - The name of the album.
    /// * `artist_id` - An optional identifier for the album artist.
    /// * `image_path` - An optional path to the album's image.
    /// * `compilation` - An optional integer indicating whether the album is a compilation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new [`DjmdAlbum`] instance or an error.
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        name: String,
        artist_id: Option<String>,
        image_path: Option<String>,
        compilation: Option<i32>,
    ) -> anyhow::Result<Self> {
        let comp = compilation.unwrap_or(0);
        Ok(Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            name: Some(name.clone()),
            album_artist_id: artist_id.clone(),
            image_path: image_path.clone(),
            compilation: Some(comp),
            search_str: None,
        })
    }
}

/// Represents the `DjmdArtist` table in the Rekordbox database.
///
/// This struct maps to the `DjmdArtist` table in the SQLite database used by Rekordbox.
/// It stores artist-related data, allowing multiple tracks to be associated with the same artist.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::djmdArtist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdArtist {
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

    /// An optional string representing the name of the artist.
    pub name: Option<String>,
    /// An optional string used for searching the artist.
    pub search_str: Option<String>,
}

impl DjmdArtist {
    /// Creates a new instance of [`DjmdArtist`].
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the artist.
    /// * `uuid` - A unique universal identifier for the artist.
    /// * `usn` - The update sequence number.
    /// * `now` - The current timestamp.
    /// * `name` - The name of the artist.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new [`DjmdArtist`] instance or an error.
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        name: String,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            name: Some(name.clone()),
            search_str: None,
        })
    }
}

/// Represents the `DjmdCategory` table in the Rekordbox database.
///
/// This struct maps to the `DjmdCategory` table in the SQLite database used by Rekordbox.
/// It stores category-related data, which can be associated with menu items.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdCategory)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdMenuItems, foreign_key = menu_item_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdCategory {
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

    /// An optional string representing the id of the associated [`DjmdMenuItems`].
    pub menu_item_id: Option<String>,
    /// An optional integer representing the sequence/order of the category.
    pub seq: Option<i32>,
    /// An optional integer indicating whether the category is disabled.
    pub disable: Option<i32>,
    /// An optional integer representing the info order for the category.
    pub info_order: Option<i32>,
}

/// Represents the `DjmdColor` table in the Rekordbox database.
///
/// This struct maps to the `DjmdColor` table in the SQLite database used by Rekordbox.
/// It stores color-related data, which can be associated with tracks and other entities.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::djmdColor)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdColor {
    /// A unique identifier for the color entry.
    pub id: String,
    /// A unique universal identifier for the color entry.
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

    /// An optional string representing the color code (e.g., hex value).
    pub color_code: Option<String>,
    /// An optional integer representing the sort key for ordering colors.
    pub sort_key: Option<i32>,
    /// An optional string for comments or additional information about the color.
    pub commnt: Option<String>,
}

/// Represents the `DjmdContent` table in the Rekordbox database.
///
/// This struct maps to the `DjmdContent` table in the SQLite database used by Rekordbox.
/// It contains metadata and attributes for each track in the collection, including file info,
/// artist, album, genre, and more.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdContent)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdArtist, foreign_key = artist_id))]
#[diesel(belongs_to(DjmdAlbum, foreign_key = album_id))]
#[diesel(belongs_to(DjmdGenre, foreign_key = genre_id))]
#[diesel(belongs_to(DjmdLabel, foreign_key = label_id))]
#[diesel(belongs_to(DjmdKey, foreign_key = key_id))]
#[diesel(belongs_to(DjmdColor, foreign_key = color_id))]
#[diesel(belongs_to(DjmdDevice, foreign_key = device_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
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
    ) -> anyhow::Result<Self> {
        let hot_cue_auto_load = "on".to_string();
        let deliver_control = "on".to_string();
        Ok(Self {
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
        })
    }
}

/// Represents the `DjmdCue` table in the Rekordbox database.
///
/// This struct maps to the `DjmdCue` table in the SQLite database used by Rekordbox.
/// It stores information about cue points for tracks, including their timing, type, and
/// additional metadata.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdCue)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdCue {
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

    /// An optional string representing the ID of the associated track in [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional integer representing the cue's start time in milliseconds.
    pub in_msec: Option<i32>,
    /// An optional integer representing the cue's start frame.
    pub in_frame: Option<i32>,
    /// An optional integer representing the cue's start MPEG frame.
    pub in_mpeg_frame: Option<i32>,
    /// An optional integer representing the cue's start MPEG absolute frame.
    pub in_mpeg_abs: Option<i32>,
    /// An optional integer representing the cue's end time in milliseconds.
    pub out_msec: Option<i32>,
    /// An optional integer representing the cue's end frame.
    pub out_frame: Option<i32>,
    /// An optional integer representing the cue's end MPEG frame.
    pub out_mpeg_frame: Option<i32>,
    /// An optional integer representing the cue's end MPEG absolute frame.
    pub out_mpeg_abs: Option<i32>,
    /// An optional integer representing the type of the cue (e.g., hot cue, memory cue).
    pub kind: Option<i32>,
    /// An optional integer representing the color of the cue.
    pub color: Option<i32>,
    /// An optional integer representing the index of the color in the color table.
    pub color_table_index: Option<i32>,
    /// An optional integer indicating whether the cue is part of an active loop.
    pub active_loop: Option<i32>,
    /// An optional string containing comments about the cue.
    pub comment: Option<String>,
    /// An optional integer representing the size of the beat loop.
    pub beat_loop_size: Option<i32>,
    /// An optional integer representing the cue's position in microseconds.
    pub cue_microsec: Option<i32>,
    /// An optional string containing seek information for the cue's start point.
    pub in_point_seek_info: Option<String>,
    /// An optional string containing seek information for the cue's end point.
    pub out_point_seek_info: Option<String>,
    /// An optional string representing the UUID of the associated track in [`DjmdContent`].
    pub content_uuid: Option<String>,
}

/// Represents the `DjmdDevice` table in the Rekordbox database.
///
/// This struct maps to the `DjmdDevice` table in the SQLite database used by Rekordbox.
/// It stores information about devices associated with the Rekordbox collection, including
/// metadata such as device name and master database ID.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::djmdDevice)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdDevice {
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

    /// An optional string representing the ID of the master database associated with the device.
    pub master_db_id: Option<String>,
    /// An optional string representing the name of the device.
    pub name: Option<String>,
}

/// Represents the `DjmdGenre` table in the Rekordbox database.
///
/// This struct maps to the `DjmdGenre` table in the SQLite database used by Rekordbox.
/// It stores genre-related data, allowing tracks to be categorized by genre.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::djmdGenre)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdGenre {
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

    /// An optional string representing the name of the genre.
    pub name: Option<String>,
}

impl DjmdGenre {
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        name: String,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            name: Some(name.clone()),
        })
    }
}

/// Represents the `DjmdHistory` table in the Rekordbox database.
///
/// This struct maps to the `DjmdHistory` table in the SQLite database used by Rekordbox.
/// It stores information about history entries, including metadata such as sequence, name,
/// attributes, and parent relationships.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdHistory)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdHistory, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdHistory {
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

    /// An optional integer representing the sequence/order of the history entry.
    pub seq: Option<i32>,
    /// An optional string representing the name of the history entry.
    pub name: Option<String>,
    /// An optional integer representing additional attributes of the history entry.
    pub attribute: Option<i32>,
    /// An optional string representing the ID of the parent [`DjmdHistory`] entry.
    pub parent_id: Option<String>,
    /// An optional string representing the creation date of the history entry.
    pub date_created: Option<String>,
}

/// Represents the `DjmdSongHistory` table in the Rekordbox database.
///
/// This struct maps to the `DjmdSongHistory` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and history entries, including
/// metadata such as track number and associated history or content IDs.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSongHistory)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdHistory, foreign_key = history_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongHistory {
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

    /// An optional string representing the ID of the associated history entry in [`DjmdHistory`].
    pub history_id: Option<String>,
    /// An optional string representing the ID of the associated content entry in [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional integer representing the track number in the history entry.
    pub track_no: Option<i32>,
}

/// Represents the `DjmdHotCueBanklist` table in the Rekordbox database.
///
/// This struct maps to the `DjmdHotCueBanklist` table in the SQLite database used by Rekordbox.
/// It stores information about hot cue bank lists, including metadata such as sequence, name,
/// image path, and parent relationships.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdHotCueBanklist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdHotCueBanklist, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdHotCueBanklist {
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

    /// An optional integer representing the sequence/order of the hot cue bank list.
    pub seq: Option<i32>,
    /// An optional string representing the name of the hot cue bank list.
    pub name: Option<String>,
    /// An optional string representing the path to the image associated with the hot cue bank list.
    pub image_path: Option<String>,
    /// An optional integer representing additional attributes of the hot cue bank list.
    pub attribute: Option<i32>,
    /// An optional string representing the ID of the parent [`DjmdHotCueBanklist`] entry.
    pub parent_id: Option<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSongHotCueBanklist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongHotCueBanklist {
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

    /// An optional string representing the ID of the associated content in [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional integer representing the track number in the hot cue bank list.
    pub track_no: Option<i32>,
    /// An optional string representing the ID of the associated [`DjmdCue`]
    pub cue_id: Option<String>,
    /// An optional integer representing the cue's start time in milliseconds.
    pub in_msec: Option<i32>,
    /// An optional integer representing the cue's start frame.
    pub in_frame: Option<i32>,
    /// An optional integer representing the cue's start MPEG frame.
    pub in_mpeg_frame: Option<i32>,
    /// An optional integer representing the cue's start MPEG absolute frame.
    pub in_mpeg_abs: Option<i32>,
    /// An optional integer representing the cue's end time in milliseconds.
    pub out_msec: Option<i32>,
    /// An optional integer representing the cue's end frame.
    pub out_frame: Option<i32>,
    /// An optional integer representing the cue's end MPEG frame.
    pub out_mpeg_frame: Option<i32>,
    /// An optional integer representing the cue's end MPEG absolute frame.
    pub out_mpeg_abs: Option<i32>,
    /// An optional integer representing the color of the cue.
    pub color: Option<i32>,
    /// An optional integer representing the index of the color in the color table.
    pub color_table_index: Option<i32>,
    /// An optional integer indicating whether the cue is part of an active loop.
    pub active_loop: Option<i32>,
    /// An optional string containing comments about the cue.
    pub comment: Option<String>,
    /// An optional integer representing the size of the beat loop.
    pub beat_loop_size: Option<i32>,
    /// An optional integer representing the cue's position in microseconds.
    pub cue_microsec: Option<i32>,
    /// An optional string containing seek information for the cue's start point.
    pub in_point_seek_info: Option<String>,
    /// An optional string containing seek information for the cue's end point.
    pub out_point_seek_info: Option<String>,
    /// An optional string representing the UUID of the associate [`DjmdHotCueBanklist`]
    pub hot_cue_banklist_uuid: Option<String>,
}

/// Represents the `HotCueBanklistCue` table in the Rekordbox database.
///
/// This struct maps to the `HotCueBanklistCue` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between hot cue bank lists and their associated cues,
/// including metadata such as update sequence numbers, timestamps, and cue data.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::hotCueBanklistCue)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdSongHotCueBanklist, foreign_key = hot_cue_banklist_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct HotCueBanklistCue {
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

    /// An optional string representing the ID of the associated [`DjmdSongHotCueBanklist`].
    pub hot_cue_banklist_id: Option<String>,
    /// An optional string containing the serialized cue data.
    pub cues: Option<String>,
    /// An optional integer representing the count of cues.
    pub rb_cue_count: Option<i32>,
}

/// Represents the `DjmdKey` table in the Rekordbox database.
///
/// This struct maps to the `DjmdKey` table in the SQLite database used by Rekordbox.
/// It stores information about musical keys, including their names and sequence order.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::djmdKey)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdKey {
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

    /// An optional string representing the name of the musical key.
    pub scale_name: Option<String>,
    /// An optional integer representing the sequence/order of the key.
    pub seq: Option<i32>,
}

impl DjmdKey {
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        name: String,
        seq: i32,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            scale_name: Some(name.clone()),
            seq: Some(seq),
        })
    }
}

/// Represents the `DjmdLabel` table in the Rekordbox database.
///
/// This struct maps to the `DjmdLabel` table in the SQLite database used by Rekordbox.
/// It stores information about labels, including their names and metadata.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::djmdLabel)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdLabel {
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

    /// An optional string representing the name of the label.
    pub name: Option<String>,
}

impl DjmdLabel {
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        name: String,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            name: Some(name.clone()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::djmdMenuItems)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdMenuItems {
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

    pub class: Option<i32>,
    pub name: Option<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdMixerParam)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdMixerParam {
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

    pub content_id: Option<String>,
    pub gain_high: Option<i32>,
    pub gain_low: Option<i32>,
    pub peak_high: Option<i32>,
    pub peak_low: Option<i32>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdMyTag)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdMyTag, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdMyTag {
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

    pub seq: Option<i32>,
    pub name: Option<String>,
    pub attribute: Option<i32>,
    pub parent_id: Option<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSongMyTag)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdMyTag, foreign_key = my_tag_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongMyTag {
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

    pub my_tag_id: Option<String>,
    pub content_id: Option<String>,
    pub track_no: Option<i32>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdPlaylist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdPlaylist, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdPlaylist {
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

    pub seq: Option<i32>,
    pub name: Option<String>,
    pub image_path: Option<String>,
    pub attribute: Option<i32>,
    pub parent_id: Option<String>,
    pub smart_list: Option<String>,
}

impl DjmdPlaylist {
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        name: String,
        attribute: i32,
        parent_id: String,
        seq: i32,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            seq: Some(seq),
            name: Some(name.clone()),
            image_path: image_path.clone(),
            attribute: Some(attribute),
            parent_id: Some(parent_id.clone()),
            smart_list: smart_list.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DjmdPlaylistTreeItem {
    pub id: String,
    pub uuid: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Date,
    pub updated_at: Date,

    pub seq: Option<i32>,
    pub name: Option<String>,
    pub image_path: Option<String>,
    pub attribute: Option<i32>,
    pub parent_id: Option<String>,
    pub smart_list: Option<String>,
    pub children: Vec<Rc<RefCell<Self>>>,
}

impl DjmdPlaylistTreeItem {
    pub fn from_playlist(playlist: DjmdPlaylist) -> Self {
        Self {
            id: playlist.id,
            uuid: playlist.uuid,
            rb_data_status: playlist.rb_data_status,
            rb_local_data_status: playlist.rb_local_data_status,
            rb_local_deleted: playlist.rb_local_deleted,
            rb_local_synced: playlist.rb_local_synced,
            usn: playlist.usn,
            rb_local_usn: playlist.rb_local_usn,
            created_at: playlist.created_at,
            updated_at: playlist.updated_at,

            seq: playlist.seq,
            name: playlist.name,
            image_path: playlist.image_path,
            attribute: playlist.attribute,
            parent_id: playlist.parent_id,
            smart_list: playlist.smart_list,

            children: Vec::new(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSongPlaylist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdPlaylist, foreign_key = playlist_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongPlaylist {
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

    pub playlist_id: Option<String>,
    pub content_id: Option<String>,
    pub track_no: Option<i32>,
}

impl DjmdSongPlaylist {
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        playlist_id: String,
        content_id: String,
        seq: i32,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            playlist_id: Some(playlist_id.clone()),
            content_id: Some(content_id.clone()),
            track_no: Some(seq),
        })
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdProperty)]
#[diesel(primary_key(db_id))]
#[diesel(belongs_to(DjmdDevice, foreign_key = device_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdProperty {
    pub db_id: String,
    pub db_version: Option<String>,
    pub base_db_drive: Option<String>,
    pub current_db_drive: Option<String>,
    pub device_id: Option<String>,
    pub reserved1: Option<String>,
    pub reserved2: Option<String>,
    pub reserved3: Option<String>,
    pub reserved4: Option<String>,
    pub reserved5: Option<String>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::djmdCloudProperty)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdCloudProperty {
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

    pub reserved1: Option<String>,
    pub reserved2: Option<String>,
    pub reserved3: Option<String>,
    pub reserved4: Option<String>,
    pub reserved5: Option<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdRecommendLike)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id1))]
// #[diesel(belongs_to(DjmdContent, foreign_key = content_id2))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdRecommendLike {
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

    pub content_id1: Option<String>,
    pub content_id2: Option<String>,
    pub like_rate: Option<i32>,
    pub data_created_h: Option<i32>,
    pub data_created_l: Option<i32>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdRelatedTracks)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdRelatedTracks, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdRelatedTracks {
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

    pub seq: Option<i32>,
    pub name: Option<String>,
    pub attribute: Option<i32>,
    pub parent_id: Option<String>,
    pub criteria: Option<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSongRelatedTracks)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdSongRelatedTracks, foreign_key = related_tracks_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongRelatedTracks {
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

    pub related_tracks_id: Option<String>,
    pub content_id: Option<String>,
    pub track_no: Option<i32>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSampler)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdSampler, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSampler {
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

    pub seq: Option<i32>,
    pub name: Option<String>,
    pub attribute: Option<i32>,
    pub parent_id: Option<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSongSampler)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdSongSampler, foreign_key = sampler_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongSampler {
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

    pub sampler_id: Option<String>,
    pub content_id: Option<String>,
    pub track_no: Option<i32>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSongTagList)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongTagList {
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

    pub content_id: Option<String>,
    pub track_no: Option<i32>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSort)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdMenuItems, foreign_key = menu_item_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSort {
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

    pub menu_item_id: Option<String>,
    pub seq: Option<i32>,
    pub disable: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::imageFile)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ImageFile {
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

    pub table_name: Option<String>,
    pub target_uuid: Option<String>,
    pub target_id: Option<String>,
    pub path: Option<String>,
    pub hash: Option<String>,
    pub size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_local_file_status: Option<i32>,
    pub rb_in_progress: Option<i32>,
    pub rb_process_type: Option<i32>,
    pub rb_temp_path: Option<String>,
    pub rb_priority: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::settingFile)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SettingFile {
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

    pub path: Option<String>,
    pub hash: Option<String>,
    pub size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::uuidIDMap)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UuidIDMap {
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

    pub table_name: Option<String>,
    pub target_uuid: Option<String>,
    pub current_id: Option<String>,
}
