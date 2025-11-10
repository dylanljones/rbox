// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::menu_item::MenuItem;
use super::schema::{category, menuItem};
use crate::model_traits::Sequence;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

type JoinTuple = (Category, Option<MenuItem>);

/// Represents the `category` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `category` table in the Device Library Plus export database.
///
/// # References
/// * [`MenuItem`] via `menu_item_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = category)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(MenuItem, foreign_key = menu_item_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Category {
    /// The unique identifier of the category.
    pub id: i32,
    /// A foreign key referencing the [`MenuItem`].
    pub menu_item_id: i32,
    /// The sequence/order of the category.
    pub seq: i32,
    /// A flag if the category is visible.
    pub is_visible: i32,
}

impl Category {
    /// Queries all records from the `category` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `category` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Updates an existing record in the `category` table.
    pub fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(category::table.find(self.id))
            .set(self)
            .get_result(conn)
    }

    /// Deletes a record from the `category` table and removes any references to it.
    pub fn delete(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        let result = diesel::delete(category::table.find(id)).execute(conn)?;
        // TODO: Update seq
        Ok(result)
    }
}

impl Sequence for Category {
    #[inline]
    fn seq(&self) -> i32 {
        self.seq
    }
}

/// Represents a complete category with its associated menu item.
#[derive(Debug, Clone, PartialEq)]
pub struct CategoryJoin {
    /// The unique identifier of the category.
    pub id: i32,
    /// A foreign key referencing the [`MenuItem`].
    pub menu_item_id: i32,
    /// The sequence/order of the category.
    pub seq: i32,
    /// A flag if the category is visible.
    pub is_visible: i32,

    /// The associated [`MenuItem`]
    pub menu_item: Option<MenuItem>,
}

impl CategoryJoin {
    fn from_join(tuple: JoinTuple) -> Self {
        let mut result = Self::from(tuple.0);
        result.menu_item = tuple.1;
        result
    }

    /// Queries all records from the `category` table with joined associations.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        let results: Vec<JoinTuple> = category::table
            .left_outer_join(menuItem::table)
            .load(conn)?;
        Ok(results.into_iter().map(Self::from_join).collect())
    }

    /// Queries a record from the `category` table with joined associations by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        let result: Option<JoinTuple> = category::table
            .find(id)
            .left_outer_join(menuItem::table)
            .first(conn)
            .optional()?;
        Ok(result.map(Self::from_join))
    }
}

impl Into<Category> for CategoryJoin {
    fn into(self) -> Category {
        Category {
            id: self.id,
            menu_item_id: self.menu_item_id,
            seq: self.seq,
            is_visible: self.is_visible,
        }
    }
}

impl From<Category> for CategoryJoin {
    fn from(raw: Category) -> Self {
        CategoryJoin {
            id: raw.id,
            menu_item_id: raw.menu_item_id,
            seq: raw.seq,
            is_visible: raw.is_visible,
            menu_item: None,
        }
    }
}

/// Represents a new record insertale to the `category` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::devlib_plus::models::NewCategory;
///
/// let new = NewCategory::new("Name".into()).is_visible(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = category)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewCategory {
    /// A foreign key referencing the [`MenuItem`].
    pub menu_item_id: i32,
    /// The sequence/order of the category.
    pub seq: i32,
    /// A flag if the category is visible.
    pub is_visible: i32,
}

impl NewCategory {
    /// Creates a new category record with the required fields.
    pub fn new(menu_item_id: i32) -> Self {
        Self {
            menu_item_id,
            ..Default::default()
        }
    }

    /// Inserts the new category into the database.
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Category> {
        // TODO: Update seq
        diesel::insert_into(category::table)
            .values(self)
            .get_result(conn)
    }

    /// Builder for `seq`.
    pub fn image_id(mut self, seq: i32) -> Self {
        self.seq = seq;
        self
    }

    /// Builder for `is_visible`.
    pub fn visible(mut self, is_visible: i32) -> Self {
        self.is_visible = is_visible;
        self
    }
}
