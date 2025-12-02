// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::schema::{content, genre};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `genre` table in the Rekordbox One Library database.
///
/// This struct maps to the `genre` table in the One Library export database.
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

impl Model for Genre {
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

impl ModelUpdate for Genre {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(genre::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Genre {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(genre::table.find(id)).execute(conn)?;
        // Remove any references to the genre in the content table
        diesel::update(content::table.filter(content::genre_id.eq(id)))
            .set(content::genre_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let result = diesel::delete(genre::table.filter(genre::id.eq_any(&ids))).execute(conn)?;
        // Remove any references to the genre in the content table
        diesel::update(content::table.filter(content::genre_id.eq_any(&ids)))
            .set(content::genre_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }
}

impl Genre {
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

impl ModelInsert for NewGenre {
    type Model = Genre;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        diesel::insert_into(genre::table)
            .values(self)
            .get_result(conn)
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        diesel::insert_into(genre::table)
            .values(items)
            .get_results(conn)
    }
}

impl NewGenre {
    /// Creates a new genre record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[cfg(feature = "master-db")]
impl Into<NewGenre> for crate::masterdb::models::DjmdGenre {
    fn into(self) -> NewGenre {
        NewGenre { name: self.name }
    }
}
