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
use crate::py_mapping::{PyItemsIter, PyObjectIter, PyStrIter};

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
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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
