// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::schema::sort;
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate, SequenceVisible};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `sort` table in the Rekordbox One Library database.
///
/// This struct maps to the `sort` table in the One Library export database.
///
/// # References
/// * [`MenuItem`] via `menu_item_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = sort)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(MenuItem, foreign_key = menu_item_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Sort {
    /// The unique identifier of the record.
    pub id: i32,
    /// An optional foreign key referencing the [`MenuItem`].
    pub menu_item_id: i32,
    /// The sequence/order number of the *visible* items (1-based index)
    pub seq: i32,
    /// A flag if the item is visible in the menu
    pub is_visible: i32,
    /// A flag if the item is selected as a sub-column
    pub is_selected_as_sub_column: i32,
}

impl Model for Sort {
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

impl ModelUpdate for Sort {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(sort::table.find(self.id))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Sort {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let (is_visible, seq) = diesel::delete(sort::table.find(id))
            .returning((sort::is_visible, sort::seq))
            .get_result::<(i32, i32)>(conn)?;
        if is_visible == 1 {
            Self::decrement_seq_after_delete(conn, seq)?;
        }
        Ok(1)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let is_visible: Vec<i32> = diesel::delete(sort::table.filter(sort::id.eq_any(ids)))
            .returning(sort::is_visible)
            .get_results(conn)?;
        if is_visible.iter().any(|&v| v == 1) {
            Self::reset_seq(conn)?;
        }
        Ok(1)
    }
}

impl Sort {
    /// Creates a filter for records by `is_visible`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_is_visible(is_visible: i32) -> _ {
        sort::table.filter(sort::is_visible.eq(is_visible))
    }

    /// Queries all records from the `sort` table that are visible.
    pub fn all_visible(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().filter(sort::is_visible.eq(1)).load(conn)
    }

    /// Queries all records from the `sort` table that are not visible.
    pub fn all_hidden(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().filter(sort::is_visible.eq(0)).load(conn)
    }
}

impl SequenceVisible for Sort {
    #[inline]
    fn count_visible(conn: &mut SqliteConnection) -> QueryResult<i32> {
        let count: i64 = Self::filter_is_visible(1).count().get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(conn: &mut SqliteConnection) -> QueryResult<Vec<i32>> {
        Self::filter_is_visible(1)
            .order(sort::seq)
            .select(sort::seq)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT sort_id, ROW_NUMBER() OVER (ORDER BY sequenceNo) AS new_seq
                FROM sort WHERE isVisible = 1
            ) UPDATE sort
            SET sequenceNo = (SELECT new_seq FROM ordered WHERE ordered.sort_id = sort.sort_id) + ?
            WHERE isVisible = 1;"#,
        )
        .bind::<diesel::sql_types::Integer, _>(Self::START_SEQ)
        .execute(conn)
    }

    #[inline]
    fn increment_seq_before_insert(conn: &mut SqliteConnection, seq: i32) -> QueryResult<usize> {
        diesel::update(
            sort::table
                .filter(sort::is_visible.eq(1))
                .filter(sort::seq.ge(seq)),
        )
        .set(sort::seq.eq(sort::seq + 1))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_after_delete(conn: &mut SqliteConnection, seq: i32) -> QueryResult<usize> {
        diesel::update(
            sort::table
                .filter(sort::is_visible.eq(1))
                .filter(sort::seq.gt(seq)),
        )
        .set(sort::seq.eq(sort::seq - 1))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `sort` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::one_library::models::NewSort;
///
/// let new = NewSort::new(1234).visible(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = sort)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewSort {
    /// An optional foreign key referencing the [`MenuItem`].
    pub menu_item_id: i32,
    /// The sequence/order number of the item (1-based index)
    pub seq: i32,
    /// A flag if the item is visible in the menu
    pub is_visible: i32,
    /// A flag if the item is selected as a sub-column
    pub is_selected_as_sub_column: i32,
}

impl ModelInsert for NewSort {
    type Model = Sort;

    fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        self.seq = if self.is_visible == 1 {
            Sort::update_seq_before_insert(conn, self.seq)?
        } else {
            0 // Make sure seq is 0 for hidden items
        };
        diesel::insert_into(sort::table)
            .values(self)
            .get_result(conn)
    }
}

impl NewSort {
    /// Creates a new `sort` record with the required fields.
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
    pub fn visible(mut self, value: i32) -> Self {
        self.is_visible = value;
        self
    }

    /// Builder for `is_selected_as_sub_column`.
    pub fn selected_as_sub_column(mut self, value: i32) -> Self {
        self.is_selected_as_sub_column = value;
        self
    }
}
