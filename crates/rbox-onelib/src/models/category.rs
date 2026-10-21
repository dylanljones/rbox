// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Database records and explicit relationship loading.

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rbox_db::ordering::{SequenceStore, SequenceVisible};
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::menu_item::MenuItem;
use crate::schema::{category, menuItem};
use rbox_db::{
    load_related, ordering_error, Model, ModelDelete, ModelInsert, ModelRelations, ModelUpdate,
};

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
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
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

/// Category snapshot with all direct metadata relationships resolved.
///
/// Field access performs no queries, and editing this view does not persist changes.
/// Related rows are not expanded recursively. Missing targets remain `None`;
/// the original row retains its foreign keys.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct CategoryView {
    /// Original category row, including its foreign keys.
    pub category: Category,
    /// Resolved menu item.
    pub menu_item: Option<MenuItem>,
}

/// Read row fields directly while keeping relationship resolution explicit.
impl std::ops::Deref for CategoryView {
    type Target = Category;

    fn deref(&self) -> &Self::Target {
        &self.category
    }
}

impl ModelRelations for Category {
    type Resolved = CategoryView;

    fn resolve_many(
        conn: &mut SqliteConnection,
        models: Vec<Self>,
    ) -> QueryResult<Vec<Self::Resolved>> {
        if models.is_empty() {
            return Ok(Vec::new());
        }
        conn.transaction(|conn| {
            let menu_items = load_related(
                conn,
                models.iter().map(|row| row.menu_item_id),
                |conn, ids| {
                    MenuItem::query()
                        .filter(menuItem::id.eq_any(ids))
                        .load(conn)
                },
                |row| row.id,
            )?;
            Ok(models
                .into_iter()
                .map(|category| CategoryView {
                    menu_item: menu_items.get(&category.menu_item_id).cloned(),
                    category,
                })
                .collect())
        })
    }
}

impl Model for Category {
    type Id = i32;

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query()
            .order((category::seq, category::id))
            .load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }
}

impl ModelUpdate for Category {
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
            diesel::update(category::table.find(self.id))
                .set(self)
                .get_result(conn)
        })
    }
}

impl ModelDelete for Category {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let (is_visible, seq) = diesel::delete(category::table.find(id))
                .returning((category::is_visible, category::seq))
                .get_result::<(i32, i32)>(conn)?;
            if is_visible == 1 {
                Self::decrement_seq_after_delete(conn, seq)?;
            }
            Ok(1)
        })
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let is_visible: Vec<i32> =
                diesel::delete(category::table.filter(category::id.eq_any(ids)))
                    .returning(category::is_visible)
                    .get_results(conn)?;
            if is_visible.iter().any(|&v| v == 1) {
                <Self as SequenceVisible>::reset_seq(conn)?;
            }
            Ok(is_visible.len())
        })
    }
}

impl Category {
    /// Load all rows and their direct relationships using batched queries.
    /// Base and related rows are read within the same transaction.
    pub fn all_with_relations(conn: &mut SqliteConnection) -> QueryResult<Vec<CategoryView>> {
        conn.transaction(|conn| {
            let rows = Self::all(conn)?;
            Self::resolve_many(conn, rows)
        })
    }

    /// Load a row and its direct relationships in one read transaction.
    /// Returns a detached snapshot; missing relationship targets remain `None`.
    pub fn find_with_relations(
        conn: &mut SqliteConnection,
        id: i32,
    ) -> QueryResult<Option<CategoryView>> {
        conn.transaction(|conn| {
            Self::find(conn, &id)?
                .map(|row| row.resolve(conn))
                .transpose()
        })
    }

    /// Creates a filter for records by `is_visible`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_is_visible(is_visible: i32) -> _ {
        category::table.filter(category::is_visible.eq(is_visible))
    }
}

impl SequenceStore for Category {
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
            .order(category::seq)
            .select(category::seq)
            .load(conn)
    }
    fn reset_seq(conn: &mut SqliteConnection, _: &()) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let rows: Vec<(i32, i32)> = Self::filter_is_visible(1)
                .order((category::seq, category::id))
                .select((category::id, category::seq))
                .load(conn)?;
            let mut count = 0;
            for (i, (id, old)) in rows.into_iter().enumerate() {
                let seq = i32::try_from(i)
                    .ok()
                    .and_then(|i| Self::START_SEQ.checked_add(i))
                    .ok_or_else(|| ordering_error("Position overflow"))?;
                if seq != old {
                    count += diesel::update(category::table.find(id))
                        .set(category::seq.eq(seq))
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
                .filter(category::seq.ge(range.start))
                .filter(category::seq.lt(range.end)),
        )
        .set(category::seq.eq(category::seq + delta))
        .execute(conn)
    }
}
impl SequenceVisible for Category {}

/// Represents a new record insertale to the `category` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox_onelib::models::NewCategory;
///
/// let new = NewCategory::new(0).visible(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = category)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
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
        conn.transaction(|conn| {
            if self.is_visible != 0 && self.is_visible != 1 {
                return Err(ordering_error("Visibility must be zero or one"));
            }
            self.seq = if self.is_visible == 1 {
                <Category as SequenceVisible>::update_seq_before_insert(conn, self.seq)?
            } else {
                0 // Make sure seq is 0 for hidden items
            };
            diesel::insert_into(category::table)
                .values(self)
                .get_result(conn)
        })
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
