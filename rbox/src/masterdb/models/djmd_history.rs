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
use super::schema::{djmdContent, djmdHistory, djmdSongHistory};
use super::util::{Date, DateString};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdHistory` table in the Rekordbox database.
///
/// This struct maps to the `djmdHistory` table in the SQLite database used by Rekordbox.
/// It stores information about history entries, including metadata such as sequence, name,
/// attributes, and parent relationships.
#[derive(Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = djmdHistory)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdHistory, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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

    /// The sequence/order number of the history (1-based index)
    pub seq: i32,
    /// The name of the history.
    pub name: String,
    /// The attribute of the tag, either history (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The ID of the parent [`DjmdHistory`], `'root'` for top-level my-tag records.
    pub parent_id: String,
    /// The creation date of the history.
    pub date_created: String,
}

impl DjmdHistory {
    /// Queries all records from the `djmdHistory` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdHistory` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries the records from the `djmdHistory` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdHistory::parent_id.eq(parent_id))
            .load(conn)
    }

    /// Queries the records from the `djmdContent` table associated with the given `djmdHistory`.
    pub fn get_contents(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<DjmdContent>> {
        djmdContent::table
            .inner_join(djmdSongHistory::table.on(djmdContent::id.eq(djmdSongHistory::content_id)))
            .filter(djmdSongHistory::history_id.eq(&id))
            .select(DjmdContent::as_select())
            .load(conn)
    }
}

/// Represents the `djmdSongHistory` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongHistory` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and history entries, including
/// metadata such as track number and associated history or content IDs.
#[derive(Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = djmdSongHistory)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdHistory, foreign_key = history_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
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

    /// The ID of the associated history entry in [`DjmdHistory`].
    pub history_id: String,
    /// The ID of the associated content entry in [`DjmdContent`].
    pub content_id: String,
    /// The track number in the history entry.
    pub track_no: i32,
}

impl DjmdSongHistory {
    /// Queries all records from the `djmdSongHistory` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdSongHistory` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries all records from the `djmdSongHistory` table by its `history_id`
    pub fn by_history_id(conn: &mut SqliteConnection, history_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongHistory::history_id.eq(history_id))
            .load(conn)
    }

    /// Queries a record from the `djmdSongHistory` table by its `content_id`.
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSongHistory::content_id.eq(cid))
            .first(conn)
            .optional()
    }
}
