// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::djmd_content::DjmdContent;
use super::schema::{djmdContent, djmdHotCueBanklist, djmdSongHotCueBanklist, hotCueBanklistCue};
use super::{Date, DateString};
use crate::model_traits::Model;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdHotCueBanklist` table in the Rekordbox database.
///
/// This struct maps to the `djmdHotCueBanklist` table in the SQLite database used by Rekordbox.
/// It stores information about hot cue bank lists, including metadata such as sequence, name,
/// image path, and parent relationships.
///
/// # Referenced by
/// * [`DjmdHotCueBanklist`] via `parent_id` foreign key.
/// * [`DjmdSongHotCueBanklist`] via `hot_cue_banklist_id` and `hot_cue_banklist_uuid` foreign keys.
/// * [`HotCueBanklistCue`] via `hot_cue_banklist_id` foreign key.
///
/// # References
/// * [`DjmdHotCueBanklist`] via `parent_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdHotCueBanklist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdHotCueBanklist, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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

    /// The sequence/order of the hot cue bank list (1-based index)
    pub seq: i32,
    /// The name of the hot cue bank list.
    pub name: String,
    /// An optional path to the image associated with the hot cue bank list.
    pub image_path: Option<String>,
    /// The attribute of the hot cue bank list, either list (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The ID of the parent [`DjmdHotCueBanklist`], `'root'` for top-level records.
    pub parent_id: String,
}

impl Model for DjmdHotCueBanklist {
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

impl DjmdHotCueBanklist {
    /// Queries the records from the `djmdHotCueBanklist` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdHotCueBanklist::parent_id.eq(parent_id))
            .load(conn)
    }

    /// Queries the records from the `djmdContent` table associated with the given `djmdHotCueBanklist`.
    pub fn get_contents(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<DjmdContent>> {
        djmdContent::table
            .inner_join(
                djmdSongHotCueBanklist::table
                    .on(djmdContent::id.eq(djmdSongHotCueBanklist::content_id)),
            )
            .filter(djmdSongHotCueBanklist::hot_cue_banklist_id.eq(&id))
            .select(DjmdContent::as_select())
            .load(conn)
    }
}

/// Represents the `djmdSongHotCueBanklist` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongHotCueBanklist` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between hot cue bank lists and their associated songs,
/// including the actual cue data
///
/// # Notes
/// Rekordbox internally represents time in “frames”, each being 1/150th of a second (6.666ms).
/// The InFrame and OutFrame values use this unit of time. However, when a track is encoded with
/// variable bit-rate (VBR) or average bit-rate (ABR), the InMpegFrame and OutMpegFrame values are
/// filled out to assist with correct seeking. Despite the names, these values are not the frame
/// indices within the MPEG file, but instead use an alternative timing scheme that is typically
/// around 1/75th of a second (13.333ms) per frame, i.e. about half the granularity of normal frames.
///
/// # References
/// * [`DjmdHotCueBanklist`] via `hot_cue_banklist_id` and `hot_cue_banklist_uuid` foreign keys.
/// * [`DjmdContent`] via `content_id` foreign key.
/// * [`DjmdCue`] via `cue_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdSongHotCueBanklist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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

    /// The ID of the associated [`DjmdHotCueBanklist`].
    pub hot_cue_banklist_id: String,
    /// The ID of the associated content in [`DjmdContent`].
    pub content_id: String,
    /// The track number in the hot cue bank list (1-based index)
    pub track_no: i32,
    /// The ID of the associated [`DjmdCue`]
    pub cue_id: String,
    /// The cue's start time in milliseconds.
    pub in_msec: i32,
    /// The cue's start frame.
    ///
    /// One frame is 1/150th of a second
    pub in_frame: i32,
    /// The cue's start MPEG frame or 0 if not a VBR/ABR MPEG file (see note).
    pub in_mpeg_frame: i32,
    /// The cue's start MPEG absolute frame or 0 if not a VBR/ABR MPEG file.
    pub in_mpeg_abs: i32,
    /// The cue's end time in milliseconds or -1 if not a loop.
    pub out_msec: i32,
    /// The cue's end frame or -1 if not a loop.
    ///
    /// One frame is 1/150th of a second
    pub out_frame: i32,
    /// The cue's end MPEG frame or 0 if not a loop or not a VBR/ABR MPEG file (see note)
    pub out_mpeg_frame: i32,
    /// The cue's end MPEG absolute frame or 0 if not a loop or not a VBR/ABR MPEG file
    pub out_mpeg_abs: i32,
    /// The color ID of the cue or -1 if no color.
    pub color: i32,
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

impl Model for DjmdSongHotCueBanklist {
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

impl DjmdSongHotCueBanklist {
    /// Queries all records from the `djmdSongHotCueBanklist` table by its `hot_cue_banklist_id`
    pub fn by_hot_cue_banklist_id(
        conn: &mut SqliteConnection,
        history_id: &str,
    ) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongHotCueBanklist::hot_cue_banklist_id.eq(history_id))
            .load(conn)
    }

    /// Queries a record from the `djmdSongHotCueBanklist` table by its `content_id`.
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSongHotCueBanklist::content_id.eq(cid))
            .first(conn)
            .optional()
    }
}

/// Represents the `hotCueBanklistCue` table in the Rekordbox database.
///
/// This struct maps to the `hotCueBanklistCue` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between hot cue bank lists and their associated cues,
/// including metadata such as update sequence numbers, timestamps, and cue data as JSON string.
///
/// # References
/// * [`DjmdHotCueBanklist`] via `hot_cue_banklist_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = hotCueBanklistCue)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdSongHotCueBanklist, foreign_key = hot_cue_banklist_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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

    /// The ID of the associated [`DjmdSongHotCueBanklist`].
    pub hot_cue_banklist_id: String,
    /// An optional string containing the serialized cue data.
    pub cues: Option<String>,
    /// An optional integer representing the count of cues.
    pub rb_cue_count: Option<i32>,
}

impl Model for HotCueBanklistCue {
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

impl HotCueBanklistCue {
    /// Queries all records from the `hotCueBanklistCue` table by its `hot_cue_banklist_id`
    pub fn by_hot_cue_banklist_id(
        conn: &mut SqliteConnection,
        history_id: &str,
    ) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(hotCueBanklistCue::hot_cue_banklist_id.eq(history_id))
            .load(conn)
    }
}
