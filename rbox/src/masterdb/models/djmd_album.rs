// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::djmd_artist::DjmdArtist;
use super::schema;
use super::util::{Date, DateString};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

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
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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
