// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::schema::sort;
use crate::model_traits::Sequence;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `sort` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `sort` table in the Device Library Plus export database.
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

impl Sort {
    /// Queries all records from the `sort` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `sort` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries all records from the `sort` table that are visible.
    pub fn all_visible(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().filter(sort::is_visible.eq(1)).load(conn)
    }

    /// Queries all records from the `sort` table that are not visible.
    pub fn all_hidden(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().filter(sort::is_visible.eq(0)).load(conn)
    }

    /// Counts the number of records in the `djmdPlaylist` table that are visible.
    pub fn count_visible(conn: &mut SqliteConnection) -> QueryResult<i32> {
        let count: i64 = sort::table
            .filter(sort::is_visible.eq(1))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }
}

impl Sequence for Sort {
    #[inline]
    fn seq(&self) -> i32 {
        self.seq
    }
}

/// Represents a new record insertale to the `sort` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::devlib_plus::models::NewSort;
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

impl NewSort {
    /// Creates a new `sort` record with the required fields.
    pub fn new(menu_item_id: i32) -> Self {
        Self {
            menu_item_id,
            ..Default::default()
        }
    }

    /// Inserts the new sort into the database.
    pub fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Sort> {
        if self.is_visible == 1 {
            // Visible, handle seq
            if self.seq == 0 {
                // If seq is 0 (default), insert as the last item (seq = count + 1)
                let count = Sort::count_visible(conn)?;
                self.seq = count + 1;
            } else {
                // Check if valid seq and shift other items up
                let count = Sort::count_visible(conn)?;
                if self.seq < 1 || self.seq > count {
                    return Err(Error::DatabaseError(
                        DatabaseErrorKind::RestrictViolation,
                        Box::new("Invalid sequence number".to_string()),
                    ));
                }
                diesel::update(
                    sort::table
                        .filter(sort::is_visible.eq(1))
                        .filter(sort::seq.ge(self.seq)),
                )
                .set(sort::seq.eq(sort::seq + 1))
                .execute(conn)?;
            }
        } else {
            // Make sure seq is 0 for hidden items
            self.seq = 0;
        }
        diesel::insert_into(sort::table)
            .values(self)
            .get_result(conn)
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
