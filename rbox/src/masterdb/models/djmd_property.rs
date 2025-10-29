// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::djmd_device::DjmdDevice;
use super::schema;
use super::util::{Date, DateString};
#[cfg(feature = "pyo3")]
use crate::py_mapping::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `DjmdProperty` table in the Rekordbox database.
///
/// This struct maps to the `DjmdProperty` table in the SQLite database used by Rekordbox.
/// It stores metadata and properties related to the database, including versioning,
/// drive information, and associated device details.
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
#[diesel(table_name = schema::djmdProperty)]
#[diesel(primary_key(db_id))]
#[diesel(belongs_to(DjmdDevice, foreign_key = device_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdProperty {
    /// A unique identifier for the database entry.
    pub db_id: String,
    /// An optional string representing the version of the database.
    pub db_version: Option<String>,
    /// An optional string representing the base drive of the database.
    pub base_db_drive: Option<String>,
    /// An optional string representing the current drive of the database.
    pub current_db_drive: Option<String>,
    /// An optional string representing the ID of the associated device in [`DjmdDevice`].
    pub device_id: Option<String>,
    /// An optional string for reserved data.
    pub reserved1: Option<String>,
    /// An optional string for reserved data.
    pub reserved2: Option<String>,
    /// An optional string for reserved data.
    pub reserved3: Option<String>,
    /// An optional string for reserved data.
    pub reserved4: Option<String>,
    /// An optional string for reserved data.
    pub reserved5: Option<String>,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,
}

/// Represents the `DjmdCloudProperty` table in the Rekordbox database.
///
/// This struct maps to the `DjmdCloudProperty` table in the SQLite database used by Rekordbox.
/// It stores metadata and properties related to cloud-based entries, including status,
/// synchronization details, and reserved fields for future use.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = schema::djmdCloudProperty)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdCloudProperty {
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

    /// An optional string for reserved data.
    pub reserved1: Option<String>,
    /// An optional string for reserved data.
    pub reserved2: Option<String>,
    /// An optional string for reserved data.
    pub reserved3: Option<String>,
    /// An optional string for reserved data.
    pub reserved4: Option<String>,
    /// An optional string for reserved data.
    pub reserved5: Option<String>,
}
