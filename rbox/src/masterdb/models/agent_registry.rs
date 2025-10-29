// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::schema;
use super::util::{Date, DateString};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `agentRegistry` table in the Rekordbox database.
///
/// This struct maps to the `agentRegistry` table in the SQLite database used by Rekordbox.
/// It contains various fields representing metadata and attributes of the Rekordbox collection.
/// Each entry has a unique name as `registry_id`. The local update sequence number (`USN`), for
/// example, is used to track changes made to the entry locally and if stored in the entry
/// with `registry_id="localUpdateCount"`.
/// Some important fields include:
/// * localUpdateCount
/// * SyncAnalysisDataRootPath
/// * SyncSettingsRootPath
/// * LangPath
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::agentRegistry)]
#[diesel(primary_key(registry_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct AgentRegistry {
    /// A unique identifier for the registry entry.
    #[cfg_attr(feature = "pyo3", repr_field)]
    pub registry_id: String,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,
    /// An optional string field for additional identifier data.
    pub id_1: Option<String>,
    /// An optional string field for additional identifier data.
    pub id_2: Option<String>,
    /// An optional integer field for numerical data.
    pub int_1: Option<i32>,
    /// An optional integer field for numerical data.
    pub int_2: Option<i32>,
    /// An optional string field for textual data.
    pub str_1: Option<String>,
    /// An optional string field for textual data.
    pub str_2: Option<String>,
    /// An optional string field for date-related data.
    pub date_1: Option<String>,
    /// An optional string field for date-related data.
    pub date_2: Option<String>,
    /// An optional string field for extended text data.
    pub text_1: Option<String>,
    /// An optional string field for extended text data.
    pub text_2: Option<String>,
}

/// Represents the `cloudAgentRegistry` table in the Rekordbox database.
///
/// This struct maps to the `cloudAgentRegistry` table in the SQLite database used by Rekordbox.
/// It contains similar fields as the [`AgentRegistry`], but is specifically used for cloud-related
/// registry entries.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::cloudAgentRegistry)]
#[diesel(primary_key(id))]
#[cfg_attr(feature = "pyo3", pyclass(unsendable, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct CloudAgentRegistry {
    /// A unique identifier for the entry.
    #[cfg_attr(feature = "pyo3", repr_field)]
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

    /// An optional integer field for numerical data.
    pub int_1: Option<i32>,
    /// An optional integer field for numerical data.
    pub int_2: Option<i32>,
    /// An optional string field for textual data.
    pub str_1: Option<String>,
    /// An optional string field for textual data.
    pub str_2: Option<String>,
    /// An optional string field for date-related data.
    pub date_1: Option<String>,
    /// An optional string field for date-related data.
    pub date_2: Option<String>,
    /// An optional string field for extended text data.
    pub text_1: Option<String>,
    /// An optional string field for extended text data.
    pub text_2: Option<String>,
}
