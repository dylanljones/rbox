// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::schema::{content, key};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `key` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `key` table in the Device Library Plus export database.
/// It stores key-related data, allowing multiple tracks to be associated with the same key.
///
/// # Referenced by
/// * [`Content`] via `key_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = key)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Key {
    /// The unique identifier of the key.
    pub id: i32,
    /// The name of the key.
    pub name: String,
}

impl Key {
    /// Queries all records from the `key` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `key` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries a record from the `key` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(key::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `key` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(key::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Updates an existing record in the `key` table.
    pub fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(key::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }

    /// Deletes a record from the `key` table and removes any references to it.
    pub fn delete(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        let result = diesel::delete(key::table.find(id)).execute(conn)?;
        // Remove any references to the key in the content table
        diesel::update(content::table.filter(content::key_id.eq(id)))
            .set(content::key_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }

    /// Deletes multiple records from the `key` table and removes any references to them.
    pub fn delete_all(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        let result = diesel::delete(key::table.filter(key::id.eq_any(ids))).execute(conn)?;
        // Remove any references to the key in the content table
        diesel::update(content::table.filter(content::key_id.eq_any(ids)))
            .set(content::key_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }
}

/// Represents a new record insertale to the `key` table.
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = key)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewKey {
    /// The name of the key.
    pub name: String,
}

impl NewKey {
    /// Creates a new key record with the required fields.
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Inserts the new key into the database.
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Key> {
        diesel::insert_into(key::table)
            .values(self)
            .get_result(conn)
    }

    /// Inserts multiple new keys into the database.
    pub fn insert_all(conn: &mut SqliteConnection, items: &[NewKey]) -> QueryResult<Vec<Key>> {
        diesel::insert_into(key::table)
            .values(items)
            .get_results(conn)
    }
}
