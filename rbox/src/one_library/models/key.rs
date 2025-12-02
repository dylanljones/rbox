// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::schema::{content, key};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `key` table in the Rekordbox One Library database.
///
/// This struct maps to the `key` table in the One Library export database.
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

impl Model for Key {
    type Id = i32;

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

impl ModelUpdate for Key {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(key::table.find(self.id))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Key {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(key::table.find(id)).execute(conn)?;
        // Remove any references to the key in the content table
        diesel::update(content::table.filter(content::key_id.eq(id)))
            .set(content::key_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let result = diesel::delete(key::table.filter(key::id.eq_any(&ids))).execute(conn)?;
        // Remove any references to the key in the content table
        diesel::update(content::table.filter(content::key_id.eq_any(&ids)))
            .set(content::key_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }
}

impl Key {
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

impl ModelInsert for NewKey {
    type Model = Key;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        diesel::insert_into(key::table)
            .values(self)
            .get_result(conn)
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        diesel::insert_into(key::table)
            .values(items)
            .get_results(conn)
    }
}

impl NewKey {
    /// Creates a new key record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[cfg(feature = "master-db")]
impl Into<NewKey> for crate::masterdb::models::DjmdKey {
    fn into(self) -> NewKey {
        NewKey { name: self.name }
    }
}
