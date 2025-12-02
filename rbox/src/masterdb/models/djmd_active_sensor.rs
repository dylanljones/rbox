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
use super::schema::djmdActiveCensor;
use super::{Date, DateString};
use crate::model_traits::Model;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdActiveCensor` table in the Rekordbox database.
///
/// This struct maps to the `djmdActiveCensor` table in the SQLite database used by Rekordbox.
/// It contains information for actively censoring explicit content of tracks in the Rekordbox
/// collection. Active Censor items behave like two cue points, between which an effect is applied
/// to the audio of a track.
///
/// # References
/// * [`DjmdContent`] via `content_id` and `content_uuid` foreign keys.
#[derive(
    Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdActiveCensor)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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

    /// The id of the associated [`DjmdContent`].
    pub content_id: String,
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

impl Model for DjmdActiveCensor {
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
