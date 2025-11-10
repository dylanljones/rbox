// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::schema::{content, genre};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `genre` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `genre` table in the Device Library Plus export database.
/// It stores genre-related data, allowing multiple tracks to be associated with the same genre.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = genre)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Genre {
    /// The unique identifier of the genre.
    pub id: i32,
    /// The name of the genre.
    pub name: String,
}

impl Genre {
    /// Queries all records from the `genre` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `genre` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries a record from the `genre` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(genre::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `genre` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(genre::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Updates an existing record in the `genre` table.
    pub fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(genre::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }

    /// Deletes a record from the `genre` table and removes any references to it.
    pub fn delete(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        let result = diesel::delete(genre::table.find(id)).execute(conn)?;
        // Remove any references to the genre in the content table
        diesel::update(content::table.filter(content::genre_id.eq(id)))
            .set(content::genre_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }

    /// Deletes multiple records from the `genre` table and removes any references to them.
    pub fn delete_all(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        let result = diesel::delete(genre::table.filter(genre::id.eq_any(ids))).execute(conn)?;
        // Remove any references to the genre in the content table
        diesel::update(content::table.filter(content::genre_id.eq_any(ids)))
            .set(content::genre_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }
}

/// Represents a new record insertale to the `genre` table.
#[derive(Debug, Clone, PartialEq, Insertable)]
#[diesel(table_name = genre)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewGenre {
    /// The name of the genre.
    pub name: String,
}

impl NewGenre {
    /// Creates a new genre record with the required fields.
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Inserts the new genre into the database.
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Genre> {
        diesel::insert_into(genre::table)
            .values(self)
            .get_result(conn)
    }

    /// Inserts multiple new genres into the database.
    pub fn insert_all(conn: &mut SqliteConnection, items: &[NewGenre]) -> QueryResult<Vec<Genre>> {
        diesel::insert_into(genre::table)
            .values(items)
            .get_results(conn)
    }
}
