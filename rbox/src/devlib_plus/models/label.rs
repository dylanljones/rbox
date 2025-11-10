// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::schema::{content, label};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `label` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `label` table in the Device Library Plus export database.
/// It stores label-related data, allowing multiple tracks to be associated with the same label.
///
/// # Referenced by
/// * [`Content`] via `label_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = label)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Label {
    /// The unique identifier of the label.
    pub id: i32,
    /// The name of the label.
    pub name: String,
}

impl Label {
    /// Queries all records from the `label` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `label` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries a record from the `label` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(label::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `label` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(label::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Updates an existing record in the `label` table.
    pub fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(label::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }

    /// Deletes a record from the `label` table and removes any references to it.
    pub fn delete(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        let result = diesel::delete(label::table.find(id)).execute(conn)?;
        // Remove any references to the label in the content table
        diesel::update(content::table.filter(content::label_id.eq(id)))
            .set(content::label_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }

    /// Deletes multiple records from the `label` table and removes any references to them.
    pub fn delete_all(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        let result = diesel::delete(label::table.filter(label::id.eq_any(ids))).execute(conn)?;
        // Remove any references to the label in the content table
        diesel::update(content::table.filter(content::label_id.eq_any(ids)))
            .set(content::label_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }
}

/// Represents a new record insertale to the `label` table.
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = label)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewLabel {
    /// The name of the label.
    pub name: String,
}

impl NewLabel {
    /// Creates a new label record with the required fields.
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Inserts the new label into the database.
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Label> {
        diesel::insert_into(label::table)
            .values(self)
            .get_result(conn)
    }

    /// Inserts multiple new labels into the database.
    pub fn insert_all(conn: &mut SqliteConnection, items: &[NewLabel]) -> QueryResult<Vec<Label>> {
        diesel::insert_into(label::table)
            .values(items)
            .get_results(conn)
    }
}
