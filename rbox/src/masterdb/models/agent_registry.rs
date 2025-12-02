// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::schema::{agentRegistry, cloudAgentRegistry};
use super::{Date, DateString};
use crate::model_traits::Model;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `agentRegistry` table in the Rekordbox database.
///
/// This struct maps to the `agentRegistry` table in the SQLite database used by Rekordbox.
/// It contains various fields representing metadata and attributes of the Rekordbox collection.
/// Each entry has a unique name as `registry_id`. The local update sequence number (USN), for
/// example, is used to track changes made to the entry locally and if stored in the entry
/// with `registry_id="localUpdateCount"`.
/// Some important fields include:
/// * localUpdateCount
/// * SyncAnalysisDataRootPath
/// * SyncSettingsRootPath
/// * LangPath
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = agentRegistry)]
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

impl Model for AgentRegistry {
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

impl AgentRegistry {
    /// Queries the local update sequence number (USN) from the `agentRegistry` table.
    ///
    /// Returns a [`diesel::result::Error::NotFound`] error if the `localUpdateCount` entry is not
    /// found or the `int_1` column is `NULL`.
    pub fn local_usn(conn: &mut SqliteConnection) -> QueryResult<i32> {
        let result = agentRegistry::table
            .find("localUpdateCount")
            .select(agentRegistry::int_1)
            .first(conn)?;
        if let Some(value) = result {
            Ok(value)
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }

    /// Updates the local update sequence number (USN) by a value and returns the new value.
    ///
    /// # Arguments
    /// * `n` - An integer value representing the change to apply to the local USN.
    ///
    /// Returns a [`diesel::result::Error::NotFound`] error if the `localUpdateCount` entry is not
    /// found or the `int_1` column is `NULL`.
    pub fn update_local_usn(conn: &mut SqliteConnection, delta: i32) -> QueryResult<i32> {
        let result = diesel::update(agentRegistry::table.find("localUpdateCount"))
            .set(agentRegistry::int_1.eq(agentRegistry::int_1 + delta))
            .returning(agentRegistry::int_1)
            .get_result(conn)?;
        if let Some(value) = result {
            Ok(value)
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }

    /// Increments the local update sequence number (USN) and returns the new value.
    ///
    /// # Arguments
    /// * `n` - An integer value representing the amount to increment the local USN.
    ///
    /// Returns a [`diesel::result::Error::NotFound`] error if the `localUpdateCount` entry is not
    /// found or the `int_1` column is `NULL`.
    #[inline]
    pub fn increment_local_usn_by(conn: &mut SqliteConnection, n: usize) -> QueryResult<i32> {
        Self::update_local_usn(conn, n as i32)
    }

    /// Increments the local update sequence number (USN) by one and returns the new value.
    ///
    /// See [`AgentRegistry::increment_local_usn_by`] for more details.
    #[inline]
    pub fn increment_local_usn(conn: &mut SqliteConnection) -> QueryResult<i32> {
        Self::update_local_usn(conn, 1)
    }
}

/// Represents the `cloudAgentRegistry` table in the Rekordbox database.
///
/// This struct maps to the `cloudAgentRegistry` table in the SQLite database used by Rekordbox.
/// It contains similar fields as the [`AgentRegistry`], but is specifically used for cloud-related
/// registry entries.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = cloudAgentRegistry)]
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

impl Model for CloudAgentRegistry {
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
