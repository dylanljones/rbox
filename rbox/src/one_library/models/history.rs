// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
use diesel::result::Error;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;
use std::collections::VecDeque;

use super::content::Content;
use super::schema::{content, history, history_content};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelList, ModelTree, TreeSeq};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `history` table in the Rekordbox One Library database.
///
/// This struct maps to the `history` table in the One Library export database.
/// It stores information about history entries, including metadata such as sequence, name,
/// attributes, and parent relationships.
///
/// # Referenced by
/// * [`History`] via `parent_id` foreign key.
/// * [`HistoryContent`] via `history_id` foreign key.
///
/// # References
/// * [`History`] via `parent_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = history)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(History, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct History {
    /// A unique identifier for the entry.
    pub id: i32,
    /// The sequence/order number of the history (1-based index)
    pub seq: i32,
    /// The name of the history.
    pub name: String,
    /// The attribute of the tag, either history (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The ID of the parent [`History`] or `0` for top-level record.
    pub parent_id: i32,
}

impl Model for History {
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

impl ModelDelete for History {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        // Vec of all deleted ids
        let mut deleted_ids = vec![*id];
        // Delete the record
        let parent_id: i32 = diesel::delete(history::table.find(id))
            .returning(history::parent_id)
            .get_result(conn)?;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove all child records recursively
        let mut parent_ids = VecDeque::from(vec![*id]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted: Vec<i32> =
                diesel::delete(history::table.filter(history::parent_id.eq(parent_id)))
                    .returning(history::id)
                    .get_results(conn)?;
            deleted_ids.extend(deleted.clone());
            // Add children to the queue
            for deleted_id in deleted {
                parent_ids.push_back(deleted_id);
            }
        }

        // Remove any history_content records that are associated with the deleted histories
        HistoryContent::delete_by_history_ids(conn, &deleted_ids)?;

        Ok(deleted_ids.len())
    }
}

impl ModelTree for History {
    fn move_to(
        conn: &mut SqliteConnection,
        id: &Self::Id,
        parent_id: Option<&Self::Id>,
        seq: Option<i32>,
    ) -> QueryResult<usize> {
        let res = match Self::find(conn, &id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };
        let old_seq = res.seq;
        let old_parent_id = res.parent_id.clone();
        let parent_id = parent_id.unwrap_or(&old_parent_id);

        let res = Self::update_seq_before_move(conn, &old_parent_id, parent_id, old_seq, seq)?;
        let (seq, _n) = match res {
            Some((s, n)) => (s, n),
            None => return Ok(0),
        };
        diesel::update(history::table.find(id))
            .set((history::seq.eq(seq), history::parent_id.eq(parent_id)))
            .execute(conn)
    }
}

impl History {
    /// Queries the records from the `history` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(history::parent_id.eq(parent_id))
            .order(history::seq)
            .load(conn)
    }

    /// Queries the records from the `content` table associated with the given `history`.
    pub fn get_contents(conn: &mut SqliteConnection, id: i32) -> QueryResult<Vec<Content>> {
        content::table
            .inner_join(history_content::table.on(content::id.eq(history_content::content_id)))
            .filter(history_content::history_id.eq(id))
            .select(Content::as_select())
            .load(conn)
    }

    /// Returns the history type (`attribute`) of a record in the `history` table or `None` if not found.
    pub fn get_attribute(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<i32>> {
        if id == 0 {
            return Ok(Some(1));
        }
        history::table
            .find(id)
            .select(history::attribute)
            .get_result(conn)
            .optional()
    }

    /// Returns `true` if the record in the `history` table exists and is a history, `false` otherwise.
    pub fn is_history(conn: &mut SqliteConnection, id: i32) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 0), // 0: history
            None => Ok(false),
        }
    }

    /// Returns `true` if the record in the `history` table exists and is a folder, `false` otherwise.
    pub fn is_folder(conn: &mut SqliteConnection, id: i32) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 1), // 1: folder
            None => Ok(false),
        }
    }

    /// Set the name of a record in the `history` table.
    pub fn rename(conn: &mut SqliteConnection, id: i32, name: &str) -> QueryResult<usize> {
        diesel::update(history::table.find(id))
            .set(history::name.eq(name))
            .execute(conn)
    }
}

impl TreeSeq for History {
    type ParentId = i32;
    const START_SEQ: i32 = 1;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        Self::is_folder(conn, *parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = history::table
            .filter(history::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        history::table
            .filter(history::parent_id.eq(parent_id))
            .order(history::seq)
            .select(history::seq)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT history_id, ROW_NUMBER() OVER (ORDER BY sequenceNo) + (? - 1) AS new_seq
                FROM history WHERE history_id_parent =?
            ) UPDATE history
            SET sequenceNo = (SELECT new_seq FROM ordered WHERE ordered.history_id = history.history_id)
            WHERE history_id_parent = ?;"#,
        )
            .bind::<diesel::sql_types::Integer, _>(Self::START_SEQ)
            .bind::<diesel::sql_types::Integer, _>(parent_id)
            .bind::<diesel::sql_types::Integer, _>(parent_id)
            .execute(conn)
    }

    #[inline]
    fn increment_seq_gte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            history::table
                .filter(history::seq.ge(seq))
                .filter(history::parent_id.eq(parent_id)),
        )
        .set(history::seq.eq(history::seq + 1))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            history::table
                .filter(history::seq.gt(seq))
                .filter(history::parent_id.eq(parent_id)),
        )
        .set(history::seq.eq(history::seq - 1))
        .execute(conn)
    }

    #[inline]
    fn increment_seq_gte_lt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            history::table
                .filter(history::seq.ge(start))
                .filter(history::seq.lt(end))
                .filter(history::parent_id.eq(parent_id)),
        )
        .set(history::seq.eq(history::seq + 1))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt_lte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            history::table
                .filter(history::seq.gt(start))
                .filter(history::seq.le(end))
                .filter(history::parent_id.eq(parent_id)),
        )
        .set(history::seq.eq(history::seq - 1))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `history` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::one_library::models::NewHistory;
///
/// let new = NewHistory::history("Name").seq(2);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = history)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewHistory {
    /// The sequence/order number of the history (1-based index)
    pub seq: i32,
    /// The name of the history.
    pub name: String,
    /// The attribute of the tag, either history (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The ID of the parent [`History`] or `0` for top-level record.
    pub parent_id: i32,
}

impl ModelInsert for NewHistory {
    type Model = History;

    fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let seq = if self.seq == 0 { None } else { Some(self.seq) };
        let (seq, _n) = History::update_seq_before_insert(conn, &self.parent_id, seq)?;
        self.seq = seq;
        diesel::insert_into(history::table)
            .values(&self)
            .get_result(conn)
    }
}

impl NewHistory {
    /// Creates a new history record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Creates a new history record as a history (`attribute=0)`.
    pub fn history<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: 0,
            ..Default::default()
        }
    }

    /// Creates a new history record as a folder (`attribute=1)`.
    pub fn folder<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: 1,
            ..Default::default()
        }
    }

    /// Builder for `seq`.
    pub fn seq(mut self, seq: i32) -> Self {
        self.seq = seq;
        self
    }

    /// Builder for `parent_id`.
    pub fn parent_id(mut self, id: i32) -> Self {
        self.parent_id = id;
        self
    }
}

/// Represents the `history_content` table in the Rekordbox One Library database.
///
/// This struct maps to the `history_content` table in the One Library export database.
/// It stores information about the relationship between songs and history entries, including
/// metadata such as track number and associated history or content IDs.
///
/// # References
/// * [`History`] via `history_id` foreign key.
/// * [`Content`] via `content_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable, Associations)]
#[diesel(table_name = history_content)]
#[diesel(primary_key(history_id, content_id))]
#[diesel(belongs_to(History, foreign_key = history_id))]
#[diesel(belongs_to(Content, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct HistoryContent {
    /// The ID of the associated [`History`].
    pub history_id: i32,
    /// The ID of the associated [`Content`].
    pub content_id: i32,
    /// The sequence/order number of the history content (1-based index)
    pub seq: i32,
}

impl Model for HistoryContent {
    type Id = (i32, i32);

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        Self::query().find(*id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(*id))).get_result(conn)
    }
}

impl ModelDelete for HistoryContent {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let history_id = id.0;
        let result = diesel::delete(history_content::table.find(*id)).execute(conn)?;
        Self::reset_seq(conn, &history_id)?;
        Ok(result)
    }
}

impl ModelInsert for HistoryContent {
    type Model = HistoryContent;

    fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        if !Content::id_exists(conn, &self.content_id)? {
            return Err(Error::NotFound);
        }
        let seq = if self.seq == 0 { None } else { Some(self.seq) };
        let (seq, _n) = HistoryContent::update_seq_before_insert(conn, &self.history_id, seq)?;
        self.seq = seq;
        diesel::insert_into(history_content::table)
            .values(&self)
            .get_result(conn)
    }
}

impl ModelList for HistoryContent {
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize> {
        let res = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };

        let old_seq = res.seq;
        let res = Self::update_seq_before_move_in(conn, &res.history_id, old_seq, seq)?;
        let (seq, _n) = match res {
            Some((s, n)) => (s, n),
            None => return Ok(0),
        };

        diesel::update(history_content::table.find(*id))
            .set(history_content::seq.eq(seq))
            .execute(conn)
    }
}

impl HistoryContent {
    /// Creates a new `history_content` record with the required fields.
    pub fn new(history_id: i32, content_id: i32) -> Self {
        Self {
            history_id,
            content_id,
            seq: 0,
        }
    }

    /// Builder for `seq`.
    pub fn seq(mut self, seq: i32) -> Self {
        self.seq = seq;
        self
    }

    /// Queries all records from the `history_content` table by its `history_id`
    pub fn by_history_id(conn: &mut SqliteConnection, history_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(history_content::history_id.eq(history_id))
            .load(conn)
    }

    /// Queries all records from the `history_content` table by its `content_id`
    pub fn by_content_id(conn: &mut SqliteConnection, content_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(history_content::history_id.eq(content_id))
            .load(conn)
    }

    /// Creates a filter for records by `history_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_history_id(history_id: i32) -> _ {
        history_content::table.filter(history_content::history_id.eq(history_id))
    }

    /// Creates a filter for records by multiple `history_id`s.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_history_ids(history_ids: &[i32]) -> _ {
        history_content::table.filter(history_content::history_id.eq_any(history_ids))
    }

    /// Creates a filter for records by `content_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_content_id(content_id: i32) -> _ {
        history_content::table.filter(history_content::content_id.eq(content_id))
    }

    /// Creates a filter for records by multiple `content_id`s.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_content_ids(content_ids: &[i32]) -> _ {
        history_content::table.filter(history_content::content_id.eq_any(content_ids))
    }

    /// Counts the number of records in the `history_content` table for a given `history_id`.
    pub fn count(conn: &mut SqliteConnection, history_id: i32) -> QueryResult<i32> {
        Self::count_children(conn, &history_id)
    }

    /// Deletes all record from the `history_content` table referencing a `history` record.
    pub fn delete_by_history_id(
        conn: &mut SqliteConnection,
        history_id: i32,
    ) -> QueryResult<usize> {
        // We don't need to handle seq, since we're deleting all records referencing the history
        diesel::delete(Self::filter_by_history_id(history_id)).execute(conn)
    }

    /// Deletes all record from the `history_content` table referencing multiple `history` records.
    pub fn delete_by_history_ids(
        conn: &mut SqliteConnection,
        history_ids: &[i32],
    ) -> QueryResult<usize> {
        // We don't need to handle seq, since we're deleting all records referencing the historys
        diesel::delete(Self::filter_by_history_ids(history_ids)).execute(conn)
    }

    /// Deletes all record from the `history_content` table referencing a `content` record.
    pub fn delete_by_content_id(conn: &mut SqliteConnection, cid: i32) -> QueryResult<usize> {
        let history_id: Option<i32> = diesel::delete(Self::filter_by_content_id(cid))
            .returning(history_content::history_id)
            .get_result(conn)
            .optional()?;
        if let Some(history_id) = history_id {
            Self::reset_seq(conn, &history_id)?;
            Ok(1)
        } else {
            Ok(0)
        }
    }

    /// Deletes all record from the `history_content` table referencing multiple `content` records.
    pub fn delete_by_content_ids(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        let history_ids: Vec<i32> = diesel::delete(Self::filter_by_content_ids(ids))
            .returning(history_content::history_id)
            .get_results(conn)?;
        for history in &history_ids {
            Self::reset_seq(conn, history)?;
        }
        Ok(history_ids.len())
    }
}

impl TreeSeq for HistoryContent {
    type ParentId = i32;
    const START_SEQ: i32 = 0;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        History::is_history(conn, *parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = Self::filter_by_history_id(*parent_id)
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        Self::filter_by_history_id(*parent_id)
            .select(history_content::seq)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT history_id, ROW_NUMBER() OVER (ORDER BY sequenceNo) + (? - 1) AS new_seq
                FROM history_content WHERE history_id =?
            ) UPDATE history_content
            SET sequenceNo = (SELECT new_seq FROM ordered WHERE ordered.history_id = history_content.history_id)
            WHERE history_id =?;"#,
        )
            .bind::<diesel::sql_types::Integer, _>(Self::START_SEQ)
            .bind::<diesel::sql_types::Integer, _>(parent_id)
            .bind::<diesel::sql_types::Integer, _>(parent_id)
            .bind::<diesel::sql_types::Integer, _>(parent_id)
            .execute(conn)
    }

    #[inline]
    fn increment_seq_gte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            history_content::table
                .filter(history_content::seq.ge(seq))
                .filter(history_content::history_id.eq(parent_id)),
        )
        .set(history_content::seq.eq(history_content::seq + 1))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            history_content::table
                .filter(history_content::seq.gt(seq))
                .filter(history_content::history_id.eq(parent_id)),
        )
        .set(history_content::seq.eq(history_content::seq - 1))
        .execute(conn)
    }

    #[inline]
    fn increment_seq_gte_lt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            history_content::table
                .filter(history_content::seq.ge(start))
                .filter(history_content::seq.lt(end))
                .filter(history_content::history_id.eq(parent_id)),
        )
        .set(history_content::seq.eq(history_content::seq + 1))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt_lte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            history_content::table
                .filter(history_content::seq.gt(start))
                .filter(history_content::seq.le(end))
                .filter(history_content::history_id.eq(parent_id)),
        )
        .set(history_content::seq.eq(history_content::seq - 1))
        .execute(conn)
    }
}
