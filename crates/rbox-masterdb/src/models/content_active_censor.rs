// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! SQLite row models and operations for `ContentActiveCensor`.

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use rbox_db::Model;

use super::djmd_content::DjmdContent;
use super::{Date, DateString};
use crate::schema::contentActiveCensor;

/// Represents the `contentActiveCensor` table in the Rekordbox database.
///
/// This struct maps to the `contentActiveCensor` table in the SQLite database used by Rekordbox.
/// It contains the active censor entries [`DjmdActiveCensor`] of a file as stringified JSON data.
///
/// # References
/// * [`DjmdContent`] via `content_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = contentActiveCensor)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct ContentActiveCensor {
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
    /// A string representing the active censors.
    pub active_censors: String,
    /// The count of active censors.
    pub rb_activecensor_count: i32,
}

impl Model for ContentActiveCensor {
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
