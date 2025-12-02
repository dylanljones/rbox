// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::schema::{color, content};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `color` table in the Rekordbox One Library database.
///
///
/// # Referenced by
/// * [`Content`] via `color_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = color)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Color {
    /// The unique identifier of the color.
    pub id: i32,
    /// The name of the color
    pub name: String,
}

impl Model for Color {
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

impl ModelUpdate for Color {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(color::table.find(self.id))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Color {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(color::table.find(id)).execute(conn)?;
        // Remove any references to the color in the content table (0 by default)
        diesel::update(content::table.filter(content::color_id.eq(id)))
            .set(content::color_id.eq(0))
            .execute(conn)?;
        Ok(result)
    }
}

impl Color {
    /// Queries a record from the `color` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(color::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `color` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(color::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }
}

/// Represents a new record insertale to the `color` table.
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = color)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewColor {
    /// The name of the color
    pub name: String,
}

impl ModelInsert for NewColor {
    type Model = Color;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        diesel::insert_into(color::table)
            .values(self)
            .get_result(conn)
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        diesel::insert_into(color::table)
            .values(items)
            .get_results(conn)
    }
}

impl NewColor {
    /// Creates a new color record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}
