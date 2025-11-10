// Author: Dylan Jones
// Date:   2025-09-02

use chrono::Utc;
use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::schema::recommendedLike;
use super::{Date, DateString};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `recommendedLike` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `recommendedLike` table in the Device Library Plus export database.
/// It stores information about recommended track relationships, including metadata such as
/// a rating, creation timestamps, and associated content IDs.
///
/// This struct also is insertable into the `recommendedLike` table.
///
/// # References
/// * [`Content`] via `content_id_1` foreign key.
/// * [`Content`] via `content_id_2` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable)]
#[diesel(table_name = recommendedLike)]
#[diesel(primary_key(content_id_1, content_id_2))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct RecommendedLike {
    /// The ID of the first associated [`Content`].
    pub content_id_1: i32,
    /// The ID of the second associated [`Content`].
    pub content_id_2: i32,
    /// The rating of the relationship, between 1-5.
    pub rating: i32,
    /// The timestamp when the entry was created, serialized/deserialized as `Datetime<Utc>`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_date: Date,
}

impl RecommendedLike {
    /// Creates a new `recommendedLike` record with the required fields.
    pub fn new(content_id_1: i32, content_id_2: i32, rating: i32) -> Self {
        let created_date = Utc::now();
        Self {
            content_id_1,
            content_id_2,
            rating,
            created_date,
        }
    }

    /// Queries all records from the `recommendedLike` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries all records from the `recommendedLike` table that are linked to the given `content`.
    pub fn by_content_id(conn: &mut SqliteConnection, content_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(
                recommendedLike::content_id_1
                    .eq(content_id)
                    .or(recommendedLike::content_id_2.eq(content_id)),
            )
            .load(conn)
    }

    /// Get related `content` ids for a given `content`.
    ///
    /// Returns the other content ID for a specific ID. The order of content IDs does not matter.
    pub fn related_content_ids(
        conn: &mut SqliteConnection,
        content_id: i32,
    ) -> QueryResult<Vec<i32>> {
        let items = Self::query()
            .filter(
                recommendedLike::content_id_1
                    .eq(content_id)
                    .or(recommendedLike::content_id_2.eq(content_id)),
            )
            .load(conn);
        let mut ids = Vec::new();
        for item in items? {
            if item.content_id_1 == content_id {
                ids.push(item.content_id_2);
            } else {
                ids.push(item.content_id_1);
            }
        }
        Ok(ids)
    }

    /// Deletes a record from the `recommendedLike` table.
    pub fn delete(
        conn: &mut SqliteConnection,
        content_id_1: i32,
        content_id_2: i32,
    ) -> QueryResult<usize> {
        diesel::delete(recommendedLike::table.find((content_id_1, content_id_2))).execute(conn)
    }

    /// Deletes all record from the `recommendedLike` table referencing a `content` record.
    pub fn delete_by_content_id(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(
            recommendedLike::table.filter(
                recommendedLike::content_id_1
                    .eq(id)
                    .or(recommendedLike::content_id_2.eq(id)),
            ),
        )
        .execute(conn)
    }

    /// Deletes all record from the `history_content` table referencing multiple `content` records.
    pub fn delete_by_content_ids(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        diesel::delete(
            recommendedLike::table.filter(
                recommendedLike::content_id_1
                    .eq_any(ids)
                    .or(recommendedLike::content_id_2.eq_any(ids)),
            ),
        )
        .execute(conn)
    }

    /// Inserts the new item into the database.
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<RecommendedLike> {
        diesel::insert_into(recommendedLike::table)
            .values(self)
            .get_result(conn)
    }
}
