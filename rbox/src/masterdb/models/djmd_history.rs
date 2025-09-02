// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;
use super::util::{Date, DateString};
use super::DjmdContent;

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
