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

impl DjmdHistory {
    /// Queries all records from the `djmdHistory` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdHistory` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries all child records from the `djmdHistory` table.
    pub fn find_children(
        conn: &mut SqliteConnection,
        history_id: Option<String>,
    ) -> QueryResult<Vec<DjmdHistory>> {
        if let Some(history_id) = history_id {
            Self::query()
                .filter(djmdHistory::parent_id.eq(history_id))
                .load(conn)
        } else {
            Self::query()
                .filter(djmdHistory::parent_id.is_null())
                .load(conn)
        }
    }

    /// Queries the parent record from the `djmdHistory` table for the given history entry.
    pub fn parent(&self, conn: &mut SqliteConnection) -> QueryResult<Option<DjmdHistory>> {
        if let Some(parent_id) = &self.parent_id {
            Self::find(conn, parent_id)
        } else {
            Ok(None)
        }
    }

    /// Queries all child records from the `djmdHistory` table for the given history entry.
    pub fn children(&self, conn: &mut SqliteConnection) -> QueryResult<Vec<DjmdHistory>> {
        Self::find_children(conn, Some(self.id.clone()))
    }

    /// Queries all records from the `djmdSongHistory` table for the given history entry.
    pub fn songs(&self, conn: &mut SqliteConnection) -> QueryResult<Vec<DjmdSongHistory>> {
        DjmdSongHistory::find_by_history(conn, &self.id)
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

    /// An optional string representing the ID of the associated history entry in [`DjmdHistory`].
    pub history_id: Option<String>,
    /// An optional string representing the ID of the associated content entry in [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional integer representing the track number in the history entry.
    pub track_no: Option<i32>,
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
    pub fn find_by_history(
        conn: &mut SqliteConnection,
        history_id: &str,
    ) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongHistory::history_id.eq(history_id))
            .load(conn)
    }

    /// Queries all records from the `djmdContent` table by its `history_id`
    pub fn find_contents_by_history(
        conn: &mut SqliteConnection,
        history_id: &str,
    ) -> QueryResult<Vec<DjmdContent>> {
        djmdSongHistory::table
            .inner_join(
                djmdContent::table.on(djmdContent::id.nullable().eq(djmdSongHistory::content_id)),
            )
            .filter(djmdSongHistory::history_id.eq(history_id))
            .select(DjmdContent::as_select())
            .load(conn)
    }

    /// Queries the parent record from the `djmdHistory` table for the given history song entry.
    pub fn history(&self, conn: &mut SqliteConnection) -> QueryResult<DjmdHistory> {
        Ok(DjmdHistory::find(conn, self.history_id.as_ref().unwrap())?.unwrap())
    }
}
