// Author: Dylan Jones
// Date:   2025-05-01
//
// Rekordbox master.db database models

#![allow(non_snake_case)]

use chrono::{DateTime, Utc};
use diesel::backend::Backend;
use diesel::deserialize::{FromSql, Result as DResult};
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, Result as SResult, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize, Serializer};

use std::cell::RefCell;
use std::rc::Rc;

use super::schema;
use super::util::{format_datetime, parse_datetime};

type Date = DateTime<Utc>;

/// Wrapper for DateTime<Utc> for diesel serialization/deserialization
#[derive(Debug, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub struct DateString(pub DateTime<Utc>);

impl From<DateString> for DateTime<Utc> {
    fn from(s: DateString) -> Self {
        s.0
    }
}

impl From<DateTime<Utc>> for DateString {
    fn from(dt: DateTime<Utc>) -> Self {
        DateString(dt)
    }
}

impl<B> FromSql<Text, B> for DateString
where
    B: Backend,
    String: FromSql<Text, B>,
{
    fn from_sql(bytes: B::RawValue<'_>) -> DResult<Self> {
        let s = <String as FromSql<Text, B>>::from_sql(bytes)?;
        Ok(parse_datetime(&s).map(DateString).map_err(|_| {
            let msg = format!("Invalid datetime string: {}", s);
            diesel::result::Error::DeserializationError(msg.into())
        })?)
    }
}

impl ToSql<Text, diesel::sqlite::Sqlite> for DateString
where
    str: ToSql<Text, diesel::sqlite::Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> SResult {
        let s = format_datetime(&self.0); // Format the DateTime<Utc> as a string
        out.set_value(s); // Set the value in the output buffer
        Ok(IsNull::No)
    }
}

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
/// # Fields
///
/// * `registry_id` - A unique identifier for the registry entry.
/// * `created_at` - The timestamp when the entry was created, serialized/deserialized as `DateString`.
/// * `updated_at` - The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
/// * `id_1` - An optional string field for additional identifier data.
/// * `id_2` - An optional string field for additional identifier data.
/// * `int_1` - An optional integer field for numerical data.
/// * `int_2` - An optional integer field for numerical data.
/// * `str_1` - An optional string field for textual data.
/// * `str_2` - An optional string field for textual data.
/// * `date_1` - An optional string field for date-related data.
/// * `date_2` - An optional string field for date-related data.
/// * `text_1` - An optional string field for extended text data.
/// * `text_2` - An optional string field for extended text data.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::agentRegistry)]
#[diesel(primary_key(registry_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AgentRegistry {
    pub registry_id: String,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,
    pub id_1: Option<String>,
    pub id_2: Option<String>,
    pub int_1: Option<i32>,
    pub int_2: Option<i32>,
    pub str_1: Option<String>,
    pub str_2: Option<String>,
    pub date_1: Option<String>,
    pub date_2: Option<String>,
    pub text_1: Option<String>,
    pub text_2: Option<String>,
}

/// Represents the `CloudAgentRegistry` table in the Rekordbox database.
///
/// This struct maps to the `CloudAgentRegistry` table in the SQLite database used by Rekordbox.
/// It contains similar fields as the [`AgentRegistry`], but is specifically used for cloud-related
/// registry entries.
///
/// # Fields
///
/// * `ID` - A unique identifier for the entry.
/// * `UUID` - A unique universal identifier for the entry.
/// * `rb_data_status` - An integer representing the data status in Rekordbox.
/// * `rb_local_data_status` - An integer representing the local data status in Rekordbox.
/// * `rb_local_deleted` - An integer indicating whether the entry is locally deleted.
/// * `rb_local_synced` - An integer indicating whether the entry is locally synced.
/// * `usn` - An optional integer representing the update sequence number.
/// * `rb_local_usn` - An optional integer representing the local update sequence number.
/// * `created_at` - The timestamp when the entry was created, serialized/deserialized as `DateString`.
/// * `updated_at` - The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
/// * `int_1` - An optional integer field for numerical data.
/// * `int_2` - An optional integer field for numerical data.
/// * `str_1` - An optional string field for textual data.
/// * `str_2` - An optional string field for textual data.
/// * `date_1` - An optional string field for date-related data.
/// * `date_2` - An optional string field for date-related data.
/// * `text_1` - An optional string field for extended text data.
/// * `text_2` - An optional string field for extended text data.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::cloudAgentRegistry)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CloudAgentRegistry {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub int_1: Option<i32>,
    pub int_2: Option<i32>,
    pub str_1: Option<String>,
    pub str_2: Option<String>,
    pub date_1: Option<String>,
    pub date_2: Option<String>,
    pub text_1: Option<String>,
    pub text_2: Option<String>,
}

/// Represents the `ContentActiveCensor` table in the Rekordbox database.
///
/// This struct maps to the `ContentActiveCensor` table in the SQLite database used by Rekordbox.
/// It contains the active censor entries [`DjmdActiveCensor`] of a file as stringified JSON data.
///
/// # Fields
///
/// * `ID` - A unique identifier for the entry.
/// * `UUID` - A unique universal identifier for the entry.
/// * `rb_data_status` - An integer representing the data status in Rekordbox.
/// * `rb_local_data_status` - An integer representing the local data status in Rekordbox.
/// * `rb_local_deleted` - An integer indicating whether the entry is locally deleted.
/// * `rb_local_synced` - An integer indicating whether the entry is locally synced.
/// * `usn` - An optional integer representing the update sequence number.
/// * `rb_local_usn` - An optional integer representing the local update sequence number.
/// * `created_at` - The timestamp when the entry was created, serialized/deserialized as `DateString`.
/// * `updated_at` - The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
/// * `ContentID` - An optional string representing the ID of the associated [`DjmdContent`].
/// * `ActiveCensors` - An optional string representing the active censors.
/// * `rb_activecensor_count` - An optional integer representing the count of active censors.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::contentActiveCensor)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ContentActiveCensor {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ContentID: Option<String>,
    pub ActiveCensors: Option<String>,
    pub rb_activecensor_count: Option<i32>,
}

/// Represents the `ContentCue` table in the Rekordbox database.
///
/// This struct maps to the `ContentCue` table in the SQLite database used by Rekordbox.
/// It contains the cue entry [`DjmdCue`] of a file as stringified JSON data.
///
/// # Fields
///
/// * `ID` - A unique identifier for the entry.
/// * `UUID` - A unique universal identifier for the entry.
/// * `rb_data_status` - An integer representing the data status in Rekordbox.
/// * `rb_local_data_status` - An integer representing the local data status in Rekordbox.
/// * `rb_local_deleted` - An integer indicating whether the entry is locally deleted.
/// * `rb_local_synced` - An integer indicating whether the entry is locally synced.
/// * `usn` - An optional integer representing the update sequence number.
/// * `rb_local_usn` - An optional integer representing the local update sequence number.
/// * `created_at` - The timestamp when the entry was created, serialized/deserialized as `DateString`.
/// * `updated_at` - The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
/// * `ContentID` - An optional string representing the ID of the associated [`DjmdContent`].
/// * `Cues` - An optional string representing the cues.
/// * `rb_cue_count` - An optional integer representing the count of cues.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::contentCue)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ContentCue {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ContentID: Option<String>,
    pub Cues: Option<String>,
    pub rb_cue_count: Option<i32>,
}

/// Represents the `ContentFile` table in the Rekordbox database.
///
/// This struct maps to the `ContentFile` table in the SQLite database used by Rekordbox.
/// It contains metadata and attributes of the additional file of contents. These include
/// the analysis and artwork files linked to a track
///
/// # Fields
///
/// * `ID` - A unique identifier for the entry.
/// * `UUID` - A unique universal identifier for the entry.
/// * `rb_data_status` - An integer representing the data status in Rekordbox.
/// * `rb_local_data_status` - An integer representing the local data status in Rekordbox.
/// * `rb_local_deleted` - An integer indicating whether the entry is locally deleted.
/// * `rb_local_synced` - An integer indicating whether the entry is locally synced.
/// * `usn` - An optional integer representing the update sequence number.
/// * `rb_local_usn` - An optional integer representing the local update sequence number.
/// * `created_at` - The timestamp when the entry was created, serialized/deserialized as `DateString`.
/// * `updated_at` - The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
/// * `ContentID` - An optional string representing the ID of the associated [`DjmdContent`].
/// * `Path` - An optional string representing the file path.
/// * `Hash` - An optional string representing the file hash.
/// * `Size` - An optional integer representing the file size.
/// * `rb_local_path` - An optional string representing the local file path.
/// * `rb_insync_hash` - An optional string representing the in-sync file hash.
/// * `rb_insync_local_usn` - An optional integer representing the in-sync local update sequence number.
/// * `rb_file_hash_dirty` - An optional integer indicating whether the file hash is dirty.
/// * `rb_local_file_status` - An optional integer representing the local file status.
/// * `rb_in_progress` - An optional integer indicating whether the file is in progress.
/// * `rb_process_type` - An optional integer representing the process type.
/// * `rb_temp_path` - An optional string representing the temporary file path.
/// * `rb_priority` - An optional integer representing the file priority.
/// * `rb_file_size_dirty` - An optional integer indicating whether the file size is dirty.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::contentFile)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ContentFile {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ContentID: Option<String>,
    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
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

/// Represents the `DjmdActiveCensor` table in the Rekordbox database.
///
/// This struct maps to the `DjmdActiveCensor` table in the SQLite database used by Rekordbox.
/// It contains information for actively censoring explicit content of tracks in the Rekordbox
/// collection. Active Censor items behave like two cue points, between which an effect is applied
/// to the audio of a track.
///
/// # Fields
///
/// * `ID` - A unique identifier for the entry.
/// * `UUID` - A unique universal identifier for the entry.
/// * `rb_data_status` - An integer representing the data status in Rekordbox.
/// * `rb_local_data_status` - An integer representing the local data status in Rekordbox.
/// * `rb_local_deleted` - An integer indicating whether the entry is locally deleted.
/// * `rb_local_synced` - An integer indicating whether the entry is locally synced.
/// * `usn` - An optional integer representing the update sequence number.
/// * `rb_local_usn` - An optional integer representing the local update sequence number.
/// * `created_at` - The timestamp when the entry was created, serialized/deserialized as `DateString`.
/// * `updated_at` - The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
/// * `ContentID` - An optional string representing the ID of the associated [`DjmdContent`].
/// * `InMsec` - An optional integer representing the start time of the censor in milliseconds.
/// * `OutMsec` - An optional integer representing the end time of the censor in milliseconds.
/// * `Info` - An optional integer representing additional information about the censor.
/// * `ParameterList` - An optional string representing the list of parameters for the censor.
/// * `ContentUUID` - An optional string representing the UUID of the associated [`DjmdContent`].

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdActiveCensor)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdActiveCensor {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ContentID: Option<String>,
    pub InMsec: Option<i32>,
    pub OutMsec: Option<i32>,
    pub Info: Option<i32>,
    pub ParameterList: Option<String>,
    pub ContentUUID: Option<String>,
}

/// Represents the `DjmdAlbum` table in the Rekordbox database.
///
/// This struct maps to the `DjmdAlbum` table in the SQLite database used by Rekordbox.
/// It stores album-related data, allowing multiple tracks to be associated with the same album.
/// This table includes metadata such as album name, artist, and compilation status.
///
/// # Fields
///
/// * `ID` - A unique identifier for the album.
/// * `UUID` - A unique universal identifier for the album.
/// * `rb_data_status` - An integer representing the data status in Rekordbox.
/// * `rb_local_data_status` - An integer representing the local data status in Rekordbox.
/// * `rb_local_deleted` - An integer indicating whether the album is locally deleted.
/// * `rb_local_synced` - An integer indicating whether the album is locally synced.
/// * `usn` - An optional integer representing the update sequence number.
/// * `rb_local_usn` - An optional integer representing the local update sequence number.
/// * `created_at` - The timestamp when the album was created, serialized/deserialized as `DateString`.
/// * `updated_at` - The timestamp when the album was last updated, serialized/deserialized as `DateString`.
/// * `Name` - An optional string representing the name of the album.
/// * `AlbumArtistID` - An optional string representing the ID of the album artist [`DjmdArtist`].
/// * `ImagePath` - An optional string representing the path to the album's image.
/// * `Compilation` - An optional integer indicating whether the album is a compilation.
/// * `SearchStr` - An optional string used for searching the album.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdAlbum)]
#[diesel(primary_key(ID))]
pub struct DjmdAlbum {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Name: Option<String>,
    pub AlbumArtistID: Option<String>,
    pub ImagePath: Option<String>,
    pub Compilation: Option<i32>,
    pub SearchStr: Option<String>,
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
            ID: id.clone(),
            UUID: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
            Name: Some(name.clone()),
            AlbumArtistID: artist_id.clone(),
            ImagePath: image_path.clone(),
            Compilation: Some(comp),
            SearchStr: None,
        })
    }
}

/// Represents the `DjmdArtist` table in the Rekordbox database.
///
/// This struct maps to the `DjmdArtist` table in the SQLite database used by Rekordbox.
/// It stores artist-related data, allowing multiple tracks to be associated with the same artist.
///
/// # Fields
///
/// * `ID` - A unique identifier for the artist.
/// * `UUID` - A unique universal identifier for the artist.
/// * `rb_data_status` - An integer representing the data status in Rekordbox.
/// * `rb_local_data_status` - An integer representing the local data status in Rekordbox.
/// * `rb_local_deleted` - An integer indicating whether the artist is locally deleted.
/// * `rb_local_synced` - An integer indicating whether the artist is locally synced.
/// * `usn` - An optional integer representing the update sequence number.
/// * `rb_local_usn` - An optional integer representing the local update sequence number.
/// * `created_at` - The timestamp when the artist entry was created, serialized/deserialized as `DateString`.
/// * `updated_at` - The timestamp when the artist entry was last updated, serialized/deserialized as `DateString`.
/// * `Name` - An optional string representing the name of the artist.
/// * `SearchStr` - An optional string used for searching the artist.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdArtist)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdArtist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Name: Option<String>,
    pub SearchStr: Option<String>,
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
            ID: id.clone(),
            UUID: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
            Name: Some(name.clone()),
            SearchStr: None,
        })
    }
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdCategory)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdCategory {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub MenuItemID: Option<String>,
    pub Seq: Option<i32>,
    pub Disable: Option<i32>,
    pub InfoOrder: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdColor)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdColor {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,
    pub ColorCode: Option<String>,
    pub SortKey: Option<i32>,
    pub Commnt: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdContent)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdContent {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub FolderPath: Option<String>,
    pub FileNameL: Option<String>,
    pub FileNameS: Option<String>,
    pub Title: Option<String>,
    pub ArtistID: Option<String>,
    pub AlbumID: Option<String>,
    pub GenreID: Option<String>,
    pub BPM: Option<i32>,
    pub Length: Option<i32>,
    pub TrackNo: Option<i32>,
    pub BitRate: Option<i32>,
    pub BitDepth: Option<i32>,
    pub Commnt: Option<String>,
    pub FileType: Option<i32>,
    pub Rating: Option<i32>,
    pub ReleaseYear: Option<i32>,
    pub RemixerID: Option<String>,
    pub LabelID: Option<String>,
    pub OrgArtistID: Option<String>,
    pub KeyID: Option<String>,
    pub StockDate: Option<String>,
    pub ColorID: Option<String>,
    pub DJPlayCount: Option<i32>,
    pub ImagePath: Option<String>,
    pub MasterDBID: Option<String>,
    pub MasterSongID: Option<String>,
    pub AnalysisDataPath: Option<String>,
    pub SearchStr: Option<String>,
    pub FileSize: Option<i32>,
    pub DiscNo: Option<i32>,
    pub ComposerID: Option<String>,
    pub Subtitle: Option<String>,
    pub SampleRate: Option<i32>,
    pub DisableQuantize: Option<i32>,
    pub Analysed: Option<i32>,
    pub ReleaseDate: Option<String>,
    pub DateCreated: Option<String>,
    pub ContentLink: Option<i32>,
    pub Tag: Option<String>,
    pub ModifiedByRBM: Option<String>,
    pub HotCueAutoLoad: Option<String>,
    pub DeliveryControl: Option<String>,
    pub DeliveryComment: Option<String>,
    pub CueUpdated: Option<String>,
    pub AnalysisUpdated: Option<String>,
    pub TrackInfoUpdated: Option<String>,
    pub Lyricist: Option<String>,
    pub ISRC: Option<String>,
    pub SamplerTrackInfo: Option<i32>,
    pub SamplerPlayOffset: Option<i32>,
    pub SamplerGain: Option<f64>,
    pub VideoAssociate: Option<String>,
    pub LyricStatus: Option<i32>,
    pub ServiceID: Option<i32>,
    pub OrgFolderPath: Option<String>,
    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub ExtInfo: Option<String>,
    pub rb_file_id: Option<String>,
    pub DeviceID: Option<String>,
    pub rb_LocalFolderPath: Option<String>,
    pub SrcID: Option<String>,
    pub SrcTitle: Option<String>,
    pub SrcArtistName: Option<String>,
    pub SrcAlbumName: Option<String>,
    pub SrcLength: Option<i32>,
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
            ID: id.clone(),
            UUID: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),

            FolderPath: Some(path.clone()),
            FileNameL: Some(file_name.clone()),
            FileNameS: Some("".to_string()),
            Title: None,
            ArtistID: None,
            AlbumID: None,
            GenreID: None,
            BPM: None,
            Length: None,
            TrackNo: None,
            BitRate: None,
            BitDepth: None,
            Commnt: None,
            FileType: Some(file_type),
            Rating: None,
            ReleaseYear: None,
            RemixerID: None,
            LabelID: None,
            OrgArtistID: None,
            KeyID: None,
            StockDate: None,
            ColorID: None,
            DJPlayCount: None,
            ImagePath: None,
            MasterDBID: Some(db_id),
            MasterSongID: Some(id),
            AnalysisDataPath: None,
            SearchStr: None,
            FileSize: Some(size),
            DiscNo: None,
            ComposerID: None,
            Subtitle: None,
            SampleRate: None,
            DisableQuantize: None,
            Analysed: None,
            ReleaseDate: None,
            DateCreated: None,
            ContentLink: content_link,
            Tag: None,
            ModifiedByRBM: None,
            HotCueAutoLoad: Some(hot_cue_auto_load),
            DeliveryControl: Some(deliver_control),
            DeliveryComment: None,
            CueUpdated: None,
            AnalysisUpdated: None,
            TrackInfoUpdated: None,
            Lyricist: None,
            ISRC: None,
            SamplerTrackInfo: Some(0),
            SamplerPlayOffset: Some(0),
            SamplerGain: Some(0.0),
            VideoAssociate: None,
            LyricStatus: Some(0),
            ServiceID: Some(0),
            OrgFolderPath: None,
            Reserved1: None,
            Reserved2: None,
            Reserved3: None,
            Reserved4: None,
            ExtInfo: None,
            rb_file_id: Some(file_id),
            DeviceID: None,
            rb_LocalFolderPath: None,
            SrcID: None,
            SrcTitle: None,
            SrcArtistName: None,
            SrcAlbumName: None,
            SrcLength: None,
        })
    }
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdCue)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdCue {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ContentID: Option<String>,
    pub InMsec: Option<i32>,
    pub InFrame: Option<i32>,
    pub InMpegFrame: Option<i32>,
    pub InMpegAbs: Option<i32>,
    pub OutMsec: Option<i32>,
    pub OutFrame: Option<i32>,
    pub OutMpegFrame: Option<i32>,
    pub OutMpegAbs: Option<i32>,
    pub Kind: Option<i32>,
    pub Color: Option<i32>,
    pub ColorTableIndex: Option<i32>,
    pub ActiveLoop: Option<i32>,
    pub Comment: Option<String>,
    pub BeatLoopSize: Option<i32>,
    pub CueMicrosec: Option<i32>,
    pub InPointSeekInfo: Option<String>,
    pub OutPointSeekInfo: Option<String>,
    pub ContentUUID: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdDevice)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdDevice {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub MasterDBID: Option<String>,
    pub Name: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdGenre)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdGenre {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Name: Option<String>,
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
            ID: id.clone(),
            UUID: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
            Name: Some(name.clone()),
        })
    }
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdHistory)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdHistory {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub DateCreated: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdSongHistory)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongHistory {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub HistoryID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdHotCueBanklist)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdHotCueBanklist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdSongHotCueBanklist)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongHotCueBanklist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
    pub CueID: Option<String>,
    pub InMsec: Option<i32>,
    pub InFrame: Option<i32>,
    pub InMpegFrame: Option<i32>,
    pub InMpegAbs: Option<i32>,
    pub OutMsec: Option<i32>,
    pub OutFrame: Option<i32>,
    pub OutMpegFrame: Option<i32>,
    pub OutMpegAbs: Option<i32>,
    pub Color: Option<i32>,
    pub ColorTableIndex: Option<i32>,
    pub ActiveLoop: Option<i32>,
    pub Comment: Option<String>,
    pub BeatLoopSize: Option<i32>,
    pub CueMicrosec: Option<i32>,
    pub InPointSeekInfo: Option<String>,
    pub OutPointSeekInfo: Option<String>,
    pub HotCueBanklistUUID: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::hotCueBanklistCue)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct HotCueBanklistCue {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub HotCueBanklistID: Option<String>,
    pub Cues: Option<String>,
    pub rb_cue_count: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdKey)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdKey {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ScaleName: Option<String>,
    pub Seq: Option<i32>,
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
            ID: id.clone(),
            UUID: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
            ScaleName: Some(name.clone()),
            Seq: Some(seq.clone()),
        })
    }
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdLabel)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdLabel {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Name: Option<String>,
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
            ID: id.clone(),
            UUID: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
            Name: Some(name.clone()),
        })
    }
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdMenuItems)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdMenuItems {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Class: Option<i32>,
    pub Name: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdMixerParam)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdMixerParam {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ContentID: Option<String>,
    pub GainHigh: Option<i32>,
    pub GainLow: Option<i32>,
    pub PeakHigh: Option<i32>,
    pub PeakLow: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdMyTag)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdMyTag {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdSongMyTag)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongMyTag {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub MyTagID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdPlaylist)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdPlaylist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub SmartList: Option<String>,
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
            ID: id.clone(),
            UUID: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
            Seq: Some(seq.clone()),
            Name: Some(name.clone()),
            ImagePath: image_path.clone(),
            Attribute: Some(attribute.clone()),
            ParentID: Some(parent_id.clone()),
            SmartList: smart_list.clone(),
        })
    }
}

fn serialize_rc_vec<S>(
    vec: &Vec<Rc<RefCell<DjmdPlaylistTreeItem>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let serializable_vec: Vec<DjmdPlaylistTreeItem> =
        vec.iter().map(|item| item.borrow().clone()).collect();
    serializable_vec.serialize(serializer)
}

#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct DjmdPlaylistTreeItem {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Date,
    pub updated_at: Date,

    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub SmartList: Option<String>,

    #[serde(serialize_with = "serialize_rc_vec")]
    pub Children: Vec<Rc<RefCell<Self>>>,
}

impl DjmdPlaylistTreeItem {
    pub fn from_playlist(playlist: DjmdPlaylist) -> Self {
        Self {
            ID: playlist.ID,
            UUID: playlist.UUID,
            rb_data_status: playlist.rb_data_status,
            rb_local_data_status: playlist.rb_local_data_status,
            rb_local_deleted: playlist.rb_local_deleted,
            rb_local_synced: playlist.rb_local_synced,
            usn: playlist.usn,
            rb_local_usn: playlist.rb_local_usn,
            created_at: playlist.created_at,
            updated_at: playlist.updated_at,

            Seq: playlist.Seq,
            Name: playlist.Name,
            ImagePath: playlist.ImagePath,
            Attribute: playlist.Attribute,
            ParentID: playlist.ParentID,
            SmartList: playlist.SmartList,

            Children: Vec::new(),
        }
    }
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdSongPlaylist)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongPlaylist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub PlaylistID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
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
            ID: id.clone(),
            UUID: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
            PlaylistID: Some(playlist_id.clone()),
            ContentID: Some(content_id.clone()),
            TrackNo: Some(seq.clone()),
        })
    }
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdProperty)]
#[diesel(primary_key(DBID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdProperty {
    pub DBID: String,
    pub DBVersion: Option<String>,
    pub BaseDBDrive: Option<String>,
    pub CurrentDBDrive: Option<String>,
    pub DeviceID: Option<String>,
    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub Reserved5: Option<String>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdCloudProperty)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdCloudProperty {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub Reserved5: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdRecommendLike)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdRecommendLike {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ContentID1: Option<String>,
    pub ContentID2: Option<String>,
    pub LikeRate: Option<i32>,
    pub DataCreatedH: Option<i32>,
    pub DataCreatedL: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdRelatedTracks)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdRelatedTracks {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub Criteria: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdSongRelatedTracks)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongRelatedTracks {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub RelatedTracksID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdSampler)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSampler {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdSongSampler)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongSampler {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub SamplerID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdSongTagList)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongTagList {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::djmdSort)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSort {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub MenuItemID: Option<String>,
    pub Seq: Option<i32>,
    pub Disable: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::imageFile)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ImageFile {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub TableName: Option<String>,
    pub TargetUUID: Option<String>,
    pub TargetID: Option<String>,
    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
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

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::settingFile)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SettingFile {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
)]
#[diesel(table_name = schema::uuidIDMap)]
#[diesel(primary_key(ID))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UuidIDMap {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    pub TableName: Option<String>,
    pub TargetUUID: Option<String>,
    pub CurrentID: Option<String>,
}
