// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::menu_item::MenuItem;
use super::schema::{category, menuItem};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate, SequenceVisible};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

type JoinTuple = (Category, Option<MenuItem>);

/// Represents the `category` table in the Rekordbox One Library database.
///
/// This struct maps to the `category` table in the One Library export database.
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

impl Model for Category {
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

impl ModelUpdate for Category {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(category::table.find(self.id))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Category {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let (is_visible, seq) = diesel::delete(category::table.find(id))
            .returning((category::is_visible, category::seq))
            .get_result::<(i32, i32)>(conn)?;
        if is_visible == 1 {
            Self::decrement_seq_after_delete(conn, seq)?;
        }
        Ok(1)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let is_visible: Vec<i32> = diesel::delete(category::table.filter(category::id.eq_any(ids)))
            .returning(category::is_visible)
            .get_results(conn)?;
        if is_visible.iter().any(|&v| v == 1) {
            Self::reset_seq(conn)?;
        }
        Ok(is_visible.len())
    }
}

impl Category {
    /// Creates a filter for records by `is_visible`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_is_visible(is_visible: i32) -> _ {
        category::table.filter(category::is_visible.eq(is_visible))
    }
}

impl SequenceVisible for Category {
    #[inline]
    fn count_visible(conn: &mut SqliteConnection) -> QueryResult<i32> {
        let count: i64 = Self::filter_is_visible(1).count().get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(conn: &mut SqliteConnection) -> QueryResult<Vec<i32>> {
        Self::filter_is_visible(1)
            .order(category::seq)
            .select(category::seq)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT category_id, ROW_NUMBER() OVER (ORDER BY sequenceNo) AS new_seq
                FROM category WHERE isVisible = 1
            ) UPDATE category
            SET sequenceNo = (SELECT new_seq FROM ordered WHERE ordered.category_id = category.category_id)
            WHERE isVisible = 1;"#,
        )
            .execute(conn)
    }

    #[inline]
    fn increment_seq_before_insert(conn: &mut SqliteConnection, seq: i32) -> QueryResult<usize> {
        diesel::update(
            category::table
                .filter(category::is_visible.eq(1))
                .filter(category::seq.ge(seq)),
        )
        .set(category::seq.eq(category::seq + 1))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_after_delete(conn: &mut SqliteConnection, seq: i32) -> QueryResult<usize> {
        diesel::update(
            category::table
                .filter(category::is_visible.eq(1))
                .filter(category::seq.gt(seq)),
        )
        .set(category::seq.eq(category::seq - 1))
        .execute(conn)
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

impl Model for CategoryJoin {
    type Id = i32;

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        let results: Vec<JoinTuple> = category::table
            .left_outer_join(menuItem::table)
            .load(conn)?;
        Ok(results.into_iter().map(Self::from_join).collect())
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        let result: Option<JoinTuple> = category::table
            .find(id)
            .left_outer_join(menuItem::table)
            .first(conn)
            .optional()?;
        Ok(result.map(Self::from_join))
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        Category::id_exists(conn, id)
    }
}

impl CategoryJoin {
    fn from_join(tuple: JoinTuple) -> Self {
        let mut result = Self::from(tuple.0);
        result.menu_item = tuple.1;
        result
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
/// use rbox::one_library::models::NewCategory;
///
/// let new = NewCategory::new(0).is_visible(1);
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

impl ModelInsert for NewCategory {
    type Model = Category;

    fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        self.seq = if self.is_visible == 1 {
            Category::update_seq_before_insert(conn, self.seq)?
        } else {
            0 // Make sure seq is 0 for hidden items
        };
        diesel::insert_into(category::table)
            .values(self)
            .get_result(conn)
    }
}

impl NewCategory {
    /// Creates a new category record with the required fields.
    pub fn new(menu_item_id: i32) -> Self {
        Self {
            menu_item_id,
            ..Default::default()
        }
    }

    /// Builder for `seq`.
    pub fn seq(mut self, seq: i32) -> Self {
        self.seq = seq;
        self
    }

    /// Builder for `is_visible`.
    pub fn visible(mut self, is_visible: i32) -> Self {
        self.is_visible = is_visible;
        self
    }
}
