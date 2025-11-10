// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::content::Content;
use super::schema::{content, history, history_content};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `history` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `history` table in the Device Library Plus export database.
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
    /// The ID of the parent [`History`] or `None` for top-level record.
    pub parent_id: Option<i32>,
}

impl History {
    /// Queries all records from the `history` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `history` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries the records from the `history` table by their `parent_id`.
    pub fn by_parent_id(
        conn: &mut SqliteConnection,
        parent_id: Option<i32>,
    ) -> QueryResult<Vec<Self>> {
        if let Some(parent_id) = parent_id {
            Self::query()
                .filter(history::parent_id.eq(parent_id))
                .load(conn)
        } else {
            Self::query()
                .filter(history::parent_id.is_null())
                .load(conn)
        }
    }

    /// Queries the records from the `content` table associated with the given `history`.
    pub fn get_contents(conn: &mut SqliteConnection, id: i32) -> QueryResult<Vec<Content>> {
        content::table
            .inner_join(history_content::table.on(content::id.eq(history_content::content_id)))
            .filter(history_content::history_id.eq(id))
            .select(Content::as_select())
            .load(conn)
    }
}

/// Represents the `history_content` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `history_content` table in the Device Library Plus export database.
/// It stores information about the relationship between songs and history entries, including
/// metadata such as track number and associated history or content IDs.
///
/// # References
/// * [`History`] via `history_id` foreign key.
/// * [`Content`] via `content_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, Associations)]
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

impl HistoryContent {
    /// Queries all records from the `history_content` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
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

    /// Deletes all record from the `history_content` table referencing a `content` record.
    pub fn delete_by_content_id(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        let result =
            diesel::delete(history_content::table.filter(history_content::content_id.eq(id)))
                .execute(conn)?;
        // TODO: Update sequence numbers
        Ok(result)
    }

    /// Deletes all record from the `history_content` table referencing multiple `content` records.
    pub fn delete_by_content_ids(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        let result =
            diesel::delete(history_content::table.filter(history_content::content_id.eq_any(ids)))
                .execute(conn)?;
        // TODO: Update sequence numbers
        Ok(result)
    }
}
