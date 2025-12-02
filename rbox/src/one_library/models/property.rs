// Author: Dylan Jones
// Date:   2025-09-02

use chrono::Utc;
use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::schema::property;
use super::{Date, DateString};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `property` table in the Rekordbox One Library database.
///
/// This struct maps to the `property` table in the One Library export database.
/// It stores information about the device(s) and the export library.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset, Insertable)]
#[diesel(table_name = property)]
#[diesel(primary_key(device_name))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Property {
    /// The name of the device ('' by default)
    pub device_name: String,
    /// The DB version (1000 by default)
    pub db_version: i32,
    /// The number of ['Content'] in the device
    pub number_of_contents: i32,
    /// The date of creation for the device
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_date: Date,
    /// The background color type (0 by default)
    pub back_ground_color_type: i32,
    /// The master.db database id of the tag data
    pub my_tag_master_dbid: i32,
}

impl Default for Property {
    fn default() -> Self {
        Self {
            device_name: "".to_string(),
            db_version: 1000,
            number_of_contents: 0,
            created_date: Utc::now(),
            back_ground_color_type: 0,
            my_tag_master_dbid: 0,
        }
    }
}

impl Model for Property {
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

impl ModelUpdate for Property {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(property::table.find(self.device_name.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Property {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        diesel::delete(property::table.find(id)).execute(conn)
    }
}

impl ModelInsert for Property {
    type Model = Self;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        diesel::insert_into(property::table)
            .values(self)
            .get_result(conn)
    }
}

impl Property {
    /// Creates a new property record with the required fields.
    pub fn new<S: Into<String>>(device_name: S, my_tag_master_dbid: i32) -> Self {
        Self {
            device_name: device_name.into(),
            created_date: Utc::now(),
            my_tag_master_dbid,
            ..Default::default()
        }
    }

    /// Queries the first record from the `property` table.
    pub fn first(conn: &mut SqliteConnection) -> QueryResult<Option<Self>> {
        Self::query().first(conn).optional()
    }

    /// Set the `number_of_contents` field of a record in the `property` table.
    pub fn set_number_of_contents(
        conn: &mut SqliteConnection,
        device_name: &str,
        number_of_contents: i32,
    ) -> QueryResult<usize> {
        diesel::update(property::table.find(device_name))
            .set(property::number_of_contents.eq(number_of_contents))
            .execute(conn)
    }

    /// Set the `number_of_contents` field of the first record in the `property` table.
    pub fn set_number_of_contents_default(
        conn: &mut SqliteConnection,
        number_of_contents: i32,
    ) -> QueryResult<usize> {
        diesel::sql_query(
            r#"UPDATE property SET number_of_contents =?
            WHERE device_name = (SELECT device_name FROM property LIMIT 1)"#,
        )
        .bind::<diesel::sql_types::Integer, _>(number_of_contents)
        .execute(conn)
    }

    /// Set the `number_of_contents` field of all records in the `property` table.
    pub fn set_number_of_contents_all(
        conn: &mut SqliteConnection,
        number_of_contents: i32,
    ) -> QueryResult<usize> {
        diesel::update(property::table)
            .set(property::number_of_contents.eq(number_of_contents))
            .execute(conn)
    }
}
