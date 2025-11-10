// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::schema::menuItem;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `menuItem` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `menuItem` table in the Device Library Plus export database.
/// It stores genre-related data, allowing multiple tracks to be associated with the same genre.
///
/// # Referenced by
/// * [`Category`] via `menu_item_id` foreign key.
/// * [`Sort`] via `menu_item_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = menuItem)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct MenuItem {
    /// The unique identifier of the record.
    pub id: i32,
    /// The kind of menu item (similar to `class` in the main DB).
    pub kind: i32,
    /// The name of the menu item.
    pub name: String,
}

impl MenuItem {
    /// Queries all records from the `menuItem` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `menuItem` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Updates an existing record in the `menuItem` table.
    pub fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(menuItem::table.find(self.id))
            .set(self)
            .get_result(conn)
    }

    /// Deletes a record from the `menuItem` table and removes any references to it.
    pub fn delete(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        let result = diesel::delete(menuItem::table.find(id)).execute(conn)?;
        // TODO: handle references to the menu item
        Ok(result)
    }
}

/// Represents a new record insertale to the `menuItem` table.
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = menuItem)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewMenuItem {
    /// The kind of menu item (similar to `class` in the main DB).
    pub kind: i32,
    /// The name of the menu item.
    pub name: String,
}

impl NewMenuItem {
    /// Creates a new menu item with the required fields.
    pub fn new(name: String, kind: i32) -> Self {
        Self { name, kind }
    }

    /// Inserts the new menu item into the database.
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<MenuItem> {
        diesel::insert_into(menuItem::table)
            .values(self)
            .get_result(conn)
    }

    /// Inserts multiple new menu items into the database.
    pub fn insert_all(
        conn: &mut SqliteConnection,
        items: &[NewMenuItem],
    ) -> QueryResult<Vec<MenuItem>> {
        diesel::insert_into(menuItem::table)
            .values(items)
            .get_results(conn)
    }
}
