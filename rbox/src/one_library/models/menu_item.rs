// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::schema::menuItem;
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `menuItem` table in the Rekordbox One Library database.
///
/// This struct maps to the `menuItem` table in the One Library export database.
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

impl Model for MenuItem {
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

impl ModelUpdate for MenuItem {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(menuItem::table.find(self.id))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for MenuItem {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(menuItem::table.find(id)).execute(conn)?;
        // TODO: handle references to the menu item
        Ok(result)
    }
}

impl MenuItem {
    /// Queries a record from the `menuItem` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(menuItem::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `menuItem` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(menuItem::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
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

impl ModelInsert for NewMenuItem {
    type Model = MenuItem;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        diesel::insert_into(menuItem::table)
            .values(self)
            .get_result(conn)
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        diesel::insert_into(menuItem::table)
            .values(items)
            .get_results(conn)
    }
}

impl NewMenuItem {
    /// Creates a new menu item with the required fields.
    pub fn new<S: Into<String>>(name: S, kind: i32) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }
}
