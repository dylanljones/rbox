// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::djmd_content::DjmdContent;
use super::schema;
use super::util::{Date, DateString};

/// Represents the `DjmdSongTagList` table in the Rekordbox database.
///
/// This struct maps to the `DjmdSongTagList` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and tag lists, including
/// metadata such as update sequence numbers, timestamps, and associated content IDs.
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

    /// An optional string representing the ID of the associated content in [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional integer representing the track number in the tag list.
    pub track_no: Option<i32>,
}
