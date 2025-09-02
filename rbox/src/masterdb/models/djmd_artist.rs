// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;
use super::util::{Date, DateString};

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
