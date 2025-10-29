// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::djmd_content::DjmdContent;
use super::schema;
use super::util::{Date, DateString};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `DjmdRelatedTracks` table in the Rekordbox database.
///
/// This struct maps to the `DjmdRelatedTracks` table in the SQLite database used by Rekordbox.
/// It stores information about related tracks, including metadata such as sequence, name,
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
#[diesel(table_name = schema::djmdRelatedTracks)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdRelatedTracks, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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

    /// An optional integer representing the sequence/order of the related track entry.
    pub seq: Option<i32>,
    /// An optional string representing the name of the related track entry.
    pub name: Option<String>,
    /// An optional integer representing additional attributes of the related track entry.
    pub attribute: Option<i32>,
    /// An optional string representing the ID of the parent [`DjmdRelatedTracks`] entry.
    pub parent_id: Option<String>,
    /// An optional string containing criteria for the related track entry.
    pub criteria: Option<String>,
}

/// Represents the `DjmdSongRelatedTracks` table in the Rekordbox database.
///
/// This struct maps to the `DjmdSongRelatedTracks` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and related tracks, including
/// metadata such as update sequence numbers, timestamps, and associated content or related tracks IDs.
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
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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

    /// An optional string representing the ID of the associated related tracks entry in [`DjmdSongRelatedTracks`].
    pub related_tracks_id: Option<String>,
    /// An optional string representing the ID of the associated content entry in [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional integer representing the track number in the related tracks entry.
    pub track_no: Option<i32>,
}
