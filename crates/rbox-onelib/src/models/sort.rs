// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use crate::schema::sort;
use rbox_db::ordering::{SequenceStore, SequenceVisible};
use rbox_db::{ordering_error, Model, ModelDelete, ModelInsert, ModelUpdate};

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
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
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
        Self::query().order((sort::seq, sort::id)).load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }
}

impl ModelUpdate for Sort {
    fn update(mut self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        conn.transaction(|conn| {
            let old = Self::find(conn, &self.id)?.ok_or(diesel::result::Error::NotFound)?;
            if self.is_visible != 0 && self.is_visible != 1 {
                return Err(ordering_error("Visibility must be zero or one"));
            }
            self.seq = match (old.is_visible == 1, self.is_visible == 1) {
                (false, true) => {
                    <Self as SequenceVisible>::update_seq_before_insert(conn, self.seq)?
                }
                (true, false) => {
                    Self::decrement_seq_gt(conn, &(), old.seq)?;
                    0
                }
                (true, true) => Self::update_seq_before_move_in(
                    conn,
                    &(),
                    old.seq,
                    if self.seq == 0 { None } else { Some(self.seq) },
                )?
                .map_or(old.seq, |(seq, _)| seq),
                (false, false) => 0,
            };
            diesel::update(sort::table.find(self.id))
                .set(self)
                .get_result(conn)
        })
    }
}

impl ModelDelete for Sort {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let (is_visible, seq) = diesel::delete(sort::table.find(id))
                .returning((sort::is_visible, sort::seq))
                .get_result::<(i32, i32)>(conn)?;
            if is_visible == 1 {
                Self::decrement_seq_after_delete(conn, seq)?;
            }
            Ok(1)
        })
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let is_visible: Vec<i32> = diesel::delete(sort::table.filter(sort::id.eq_any(ids)))
                .returning(sort::is_visible)
                .get_results(conn)?;
            if is_visible.iter().any(|&v| v == 1) {
                <Self as SequenceVisible>::reset_seq(conn)?;
            }
            Ok(is_visible.len())
        })
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
        Self::query()
            .filter(sort::is_visible.eq(1))
            .order((sort::seq, sort::id))
            .load(conn)
    }

    /// Queries all records from the `sort` table that are not visible.
    pub fn all_hidden(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().filter(sort::is_visible.eq(0)).load(conn)
    }
}

impl SequenceStore for Sort {
    type ParentId = ();
    fn is_valid_parent(_: &mut SqliteConnection, _: &()) -> QueryResult<bool> {
        Ok(true)
    }
    fn count_children(conn: &mut SqliteConnection, _: &()) -> QueryResult<i32> {
        let count: i64 = Self::filter_is_visible(1).count().get_result(conn)?;
        i32::try_from(count).map_err(|_| ordering_error("Collection too large"))
    }
    fn get_seq_numbers(conn: &mut SqliteConnection, _: &()) -> QueryResult<Vec<i32>> {
        Self::filter_is_visible(1)
            .order(sort::seq)
            .select(sort::seq)
            .load(conn)
    }
    fn reset_seq(conn: &mut SqliteConnection, _: &()) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let rows: Vec<(i32, i32)> = Self::filter_is_visible(1)
                .order((sort::seq, sort::id))
                .select((sort::id, sort::seq))
                .load(conn)?;
            let mut count = 0;
            for (i, (id, old)) in rows.into_iter().enumerate() {
                let seq = i32::try_from(i)
                    .ok()
                    .and_then(|i| Self::START_SEQ.checked_add(i))
                    .ok_or_else(|| ordering_error("Position overflow"))?;
                if seq != old {
                    count += diesel::update(sort::table.find(id))
                        .set(sort::seq.eq(seq))
                        .execute(conn)?;
                }
            }
            Ok(count)
        })
    }
    fn shift_seq(
        conn: &mut SqliteConnection,
        _: &(),
        range: std::ops::Range<i32>,
        delta: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            Self::filter_is_visible(1)
                .filter(sort::seq.ge(range.start))
                .filter(sort::seq.lt(range.end)),
        )
        .set(sort::seq.eq(sort::seq + delta))
        .execute(conn)
    }
}
impl SequenceVisible for Sort {}

/// Represents a new record insertale to the `sort` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox_onelib::models::NewSort;
///
/// let new = NewSort::new(1234).visible(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = sort)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
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
        conn.transaction(|conn| {
            if self.is_visible != 0 && self.is_visible != 1 {
                return Err(ordering_error("Visibility must be zero or one"));
            }
            self.seq = if self.is_visible == 1 {
                <Sort as SequenceVisible>::update_seq_before_insert(conn, self.seq)?
            } else {
                0 // Make sure seq is 0 for hidden items
            };
            diesel::insert_into(sort::table)
                .values(self)
                .get_result(conn)
        })
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
