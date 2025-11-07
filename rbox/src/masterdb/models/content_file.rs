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
use super::schema::contentFile;
use super::util::{Date, DateString};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `contentFile` table in the Rekordbox database.
///
/// This struct maps to the `contentFile` table in the SQLite database used by Rekordbox.
/// It contains metadata and attributes of the additional file of contents. These include
/// the analysis and artwork files linked to a track
#[derive(
    Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = contentFile)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct ContentFile {
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

    /// An optional string representing the id of the associated [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional string representing the file path.
    pub path: Option<String>,
    /// An optional string representing the file hash.
    pub hash: Option<String>,
    /// An optional integer representing the file size.
    pub size: Option<i32>,
    /// An optional string representing the local file path.
    pub rb_local_path: Option<String>,
    /// An optional string representing the in-sync file hash.
    pub rb_insync_hash: Option<String>,
    /// An optional integer representing the in-sync local update sequence number.
    pub rb_insync_local_usn: Option<i32>,
    /// An optional integer indicating whether the file hash is dirty.
    pub rb_file_hash_dirty: Option<i32>,
    /// An optional integer representing the local file status.
    pub rb_local_file_status: Option<i32>,
    /// An optional integer indicating whether the file is in progress.
    pub rb_in_progress: Option<i32>,
    /// An optional integer representing the process type.
    pub rb_process_type: Option<i32>,
    /// An optional string representing the temporary file path.
    pub rb_temp_path: Option<String>,
    /// An optional integer representing the file priority.
    pub rb_priority: Option<i32>,
    /// An optional integer indicating whether the file size is dirty.
    pub rb_file_size_dirty: Option<i32>,
}

impl ContentFile {
    /// Queries all records from the `contentFile` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `contentFile` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }
}
