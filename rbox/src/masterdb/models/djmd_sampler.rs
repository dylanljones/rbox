// Author: Dylan Jones
// Date:   2025-09-02

use chrono::Utc;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use std::collections::VecDeque;

use super::agent_registry::AgentRegistry;
use super::djmd_content::DjmdContent;
use super::schema::{djmdContent, djmdSampler, djmdSongSampler};
use super::util::{format_datetime, Date, DateString};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdSampler` table in the Rekordbox database.
///
/// This struct maps to the `djmdSampler` table in the SQLite database used by Rekordbox.
/// It stores information about samplers, including metadata such as sequence, name,
/// attributes, and parent relationships.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdSampler)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdSampler, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdSampler {
    /// A unique identifier for the entry.
    pub id: String,
    /// A unique universal identifier for the entry.
    pub uuid: String,
    /// An integer representing the data status in Rekordbox.
    pub rb_data_status: i32,
    /// An integer representing the local data status in Rekordbox.
    pub rb_local_data_status: i32,
    /// An integer indicating whether the entry is locally deleted.
    pub rb_local_deleted: i32,
    /// An integer indicating whether the entry is locally synced.
    pub rb_local_synced: i32,
    /// An optional integer representing the update sequence number.
    pub usn: Option<i32>,
    /// An optional integer representing the local update sequence number.
    pub rb_local_usn: Option<i32>,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    /// The sequence/order number of the record (1-based index)
    pub seq: i32,
    /// The name of the sampler.
    pub name: String,
    /// The attribute of the sampler, either list (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The ID of the parent [`DjmdSampler`], `'root'` for top-level records.
    pub parent_id: String,
}

impl DjmdSampler {
    /// Queries all records from the `djmdSampler` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdSampler` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries a record from the `djmdSampler` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSampler::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Queries a record from the `djmdSampler` table by its associated `content_id` via `djmdSongSampler`
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        djmdSampler::table
            .inner_join(djmdSongSampler::table.on(djmdSampler::id.eq(djmdSongSampler::sampler_id)))
            .filter(djmdSongSampler::content_id.eq(cid))
            .select(Self::as_select())
            .first(conn)
            .optional()
    }

    /// Queries the records from the `djmdSampler` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSampler::parent_id.eq(parent_id))
            .load(conn)
    }

    /// Queries the records from the `djmdContent` table associated with the given `djmdSampler`.
    pub fn get_contents(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<DjmdContent>> {
        djmdContent::table
            .inner_join(djmdSongSampler::table.on(djmdContent::id.eq(djmdSongSampler::content_id)))
            .filter(djmdSongSampler::sampler_id.eq(&id))
            .select(DjmdContent::as_select())
            .load(conn)
    }

    /// Counts the number of child records in the `djmdSampler` table for a given `parent_id`.
    pub fn count_children(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<i32> {
        let count: i64 = djmdSampler::table
            .filter(djmdSampler::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    /// Counts the number of child records in the `djmdSampler` table for a record inside the same parent.
    pub fn count_parent_children(conn: &mut SqliteConnection, id: &str) -> QueryResult<i32> {
        let parent_id: String = djmdSampler::table
            .find(id)
            .select(djmdSampler::parent_id)
            .get_result(conn)?;
        Self::count_children(conn, &parent_id)
    }

    /// Checks if a record with the given `id` exists in the `djmdSampler` table.
    pub fn id_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }

    /// Checks if a record with the given `name` exists in the `djmdSampler` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdSampler::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Updates an existing record in the `djmdSampler` table.
    pub fn update(mut self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        let existing = match Self::find(conn, &self.id)? {
            Some(e) => e,
            None => return Err(Error::NotFound),
        };
        let mut changes = 0;
        if self.seq != existing.seq {
            changes += 1;
        }
        if self.name != existing.name {
            changes += 1;
        }
        if self.attribute != existing.attribute {
            changes += 1;
        }
        if self.parent_id != existing.parent_id {
            changes += 1;
        }
        if changes == 0 {
            return Ok(existing);
        }
        self.updated_at = Utc::now();
        self.rb_local_usn = Some(AgentRegistry::increment_local_usn_by(conn, changes)?);
        Ok(diesel::update(djmdSampler::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)?)
    }

    /// Reset the `seq` numbers of all records from the `djmdSampler` table for a given `parent_id`.
    ///
    /// The `seq` numbers are reset to the *current* order of the records in the `djmdSampler` table,
    /// starting at 1:
    ///
    /// ```
    /// Old: 1, 2, 3, 5, 7, 8, 9
    /// New: 1, 2, 3, 4, 5, 6, 7
    /// ```
    ///
    /// This is helpful when deleting records or for fixing the sequence after an invalid state.
    /// When adding or moving records, the `seq` numbers should be set explicitly.
    pub fn reset_seq(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY Seq) AS new_seq
                FROM djmdSampler WHERE ParentID =?
            ) UPDATE djmdSampler
            SET Seq = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSampler.ID);"#,
        )
        .bind::<diesel::sql_types::Text, _>(parent_id)
        .execute(conn)
    }

    /// Move a record in the `djmdSampler` table inside the same parent.
    pub fn move_in(conn: &mut SqliteConnection, id: &str, seq: i32) -> QueryResult<usize> {
        if seq <= 0 {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new("Invalid sequence number".to_string()),
            ));
        }
        let rec = match Self::find(conn, &id)? {
            Some(r) => r,
            None => return Ok(0),
        };
        let count = Self::count_children(conn, &rec.parent_id)?;
        if seq > count {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new("Invalid sequence number".to_string()),
            ));
        }

        let old_seq = rec.seq;
        let datestr = format_datetime(&Utc::now());
        let usn = AgentRegistry::increment_local_usn(conn)?;

        // Update sequence number of other children

        if seq > old_seq {
            diesel::update(
                djmdSampler::table
                    .filter(djmdSampler::parent_id.eq(rec.parent_id))
                    .filter(djmdSampler::seq.gt(old_seq))
                    .filter(djmdSampler::seq.le(seq)),
            )
            .set((
                djmdSampler::seq.eq(djmdSampler::seq - 1),
                djmdSampler::updated_at.eq(&datestr),
                djmdSampler::rb_local_usn.eq(usn),
            ))
            .execute(conn)?;
        } else if seq < old_seq {
            diesel::update(
                djmdSampler::table
                    .filter(djmdSampler::parent_id.eq(rec.parent_id))
                    .filter(djmdSampler::seq.lt(old_seq))
                    .filter(djmdSampler::seq.ge(seq)),
            )
            .set((
                djmdSampler::seq.eq(djmdSampler::seq + 1),
                djmdSampler::updated_at.eq(&datestr),
                djmdSampler::rb_local_usn.eq(usn),
            ))
            .execute(conn)?;
        } else {
            return Ok(0);
        }

        // Update sequence number of record
        diesel::update(djmdSampler::table.find(id))
            .set((
                djmdSampler::seq.eq(seq),
                djmdSampler::updated_at.eq(&datestr),
                djmdSampler::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }

    /// Move a record in the `djmdSampler` table to a new parent.
    pub fn move_to(
        conn: &mut SqliteConnection,
        id: &str,
        parent_id: &str,
        seq: Option<i32>,
    ) -> QueryResult<usize> {
        let rec = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Ok(0),
        };
        let old_seq = rec.seq;
        let old_parent_id = rec.parent_id.clone();
        let count = Self::count_children(conn, &parent_id)?;
        // Insert at end by default
        let seq = seq.unwrap_or(count);

        if seq <= 0 {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new("Invalid sequence number".to_string()),
            ));
        }
        if seq > count {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new("Invalid sequence number".to_string()),
            ));
        }

        let datestr = format_datetime(&Utc::now());
        let usn = AgentRegistry::increment_local_usn(conn)?;

        if parent_id == old_parent_id {
            // parent didn't change, just move
            return Self::move_in(conn, id, seq);
        }

        if parent_id != old_parent_id {
            // parent changed, move to new parent

            let my_tag_attr: i32 = djmdSampler::table
                .find(&parent_id)
                .select(djmdSampler::attribute)
                .get_result(conn)?;
            if my_tag_attr != 1 {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new("Cannot add a DjmdSampler child to a DjmdMyTag list".to_string()),
                ));
            }

            // Update seq of records in the old parent:
            // Decrease seq of records with seq higher than old seq
            diesel::update(
                djmdSampler::table
                    .filter(djmdSampler::parent_id.eq(old_parent_id))
                    .filter(djmdSampler::seq.gt(old_seq)),
            )
            .set((
                djmdSampler::seq.eq(djmdSampler::seq - 1),
                djmdSampler::updated_at.eq(&datestr),
                djmdSampler::rb_local_usn.eq(usn),
            ))
            .execute(conn)?;

            if seq < count {
                // Move to center in the new parent
                // Increase seq of records with seq higher than old seq
                diesel::update(
                    djmdSampler::table
                        .filter(djmdSampler::parent_id.eq(parent_id))
                        .filter(djmdSampler::seq.ge(old_seq)),
                )
                .set((
                    djmdSampler::seq.eq(djmdSampler::seq + 1),
                    djmdSampler::updated_at.eq(&datestr),
                    djmdSampler::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
            }

            // Update Seq and parent of actual record
            diesel::update(djmdSampler::table.find(id))
                .set((
                    djmdSampler::seq.eq(seq),
                    djmdSampler::parent_id.eq(parent_id),
                    djmdSampler::updated_at.eq(&datestr),
                    djmdSampler::rb_local_usn.eq(usn),
                ))
                .execute(conn)
        } else {
            if seq > old_seq {
                // Seq is increased (move down in parent):
                // Decrease seq of all other records with seq between old seq and new seq
                diesel::update(
                    djmdSampler::table
                        .filter(djmdSampler::parent_id.eq(rec.parent_id))
                        .filter(djmdSampler::seq.gt(old_seq))
                        .filter(djmdSampler::seq.le(seq)),
                )
                .set((
                    djmdSampler::seq.eq(djmdSampler::seq - 1),
                    djmdSampler::updated_at.eq(&datestr),
                    djmdSampler::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
            } else if seq < old_seq {
                // Seq is decreased (moved down in parent):
                // Increase seq of all records with seq between old seq and new seq
                diesel::update(
                    djmdSampler::table
                        .filter(djmdSampler::parent_id.eq(rec.parent_id))
                        .filter(djmdSampler::seq.lt(old_seq))
                        .filter(djmdSampler::seq.ge(seq)),
                )
                .set((
                    djmdSampler::seq.eq(djmdSampler::seq + 1),
                    djmdSampler::updated_at.eq(&datestr),
                    djmdSampler::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
            } else {
                return Ok(0);
            }
            // Update sequence number of record
            diesel::update(djmdSampler::table.find(id))
                .set((
                    djmdSampler::seq.eq(seq),
                    djmdSampler::updated_at.eq(&datestr),
                    djmdSampler::rb_local_usn.eq(usn),
                ))
                .execute(conn)
        }
    }

    /// Deletes a record from the `djmdSampler` table and removes any references to it.
    pub fn delete(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
        // Delete the record
        let parent_id: String = diesel::delete(djmdSampler::table.find(id))
            .returning(djmdSampler::parent_id)
            .get_result(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        let mut result = 1;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove any djmdSongSampler records that are associated with this djmdSampler
        DjmdSongSampler::delete_by_sampler_id(conn, id)?;

        // Remove all child djmdSampler records recursively
        let mut parent_ids = VecDeque::from(vec![id.to_string()]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted_ids: Vec<String> =
                diesel::delete(djmdSampler::table.filter(djmdSampler::parent_id.eq(parent_id)))
                    .returning(djmdSampler::id)
                    .get_results(conn)?;
            AgentRegistry::increment_local_usn_by(conn, deleted_ids.len())?;
            result += deleted_ids.len();

            // Delete any songs referencing the deleted tags
            DjmdSongSampler::delete_by_sampler_ids(conn, &deleted_ids)?;
            // Add children to the queue
            for deleted_id in deleted_ids {
                parent_ids.push_back(deleted_id);
            }
        }
        Ok(result)
    }
}

/// Represents the `djmdSongSampler` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongSampler` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and samplers, including
/// metadata such as update sequence numbers, timestamps, and associated sampler or content IDs.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdSongSampler)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdSampler, foreign_key = sampler_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdSongSampler {
    /// A unique identifier for the entry.
    pub id: String,
    /// A unique universal identifier for the entry.
    pub uuid: String,
    /// An integer representing the data status in Rekordbox.
    pub rb_data_status: i32,
    /// An integer representing the local data status in Rekordbox.
    pub rb_local_data_status: i32,
    /// An integer indicating whether the entry is locally deleted.
    pub rb_local_deleted: i32,
    /// An integer indicating whether the entry is locally synced.
    pub rb_local_synced: i32,
    /// An optional integer representing the update sequence number.
    pub usn: Option<i32>,
    /// An optional integer representing the local update sequence number.
    pub rb_local_usn: Option<i32>,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    /// The ID of the associated sampler in [`DjmdSampler`].
    pub sampler_id: String,
    /// The ID of the associated content in [`DjmdContent`].
    pub content_id: String,
    /// The sequence/order number of the record (1-based index)
    pub track_no: i32,
}

impl DjmdSongSampler {
    /// Queries all records from the `djmdSongSampler` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdSongSampler` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries all records from the `djmdSongSampler` table by its `sampler_id`.
    pub fn by_sampler_id(conn: &mut SqliteConnection, sampler_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongSampler::sampler_id.eq(sampler_id))
            .get_results(conn)
    }

    /// Queries a record from the `djmdSongSampler` table by its `content_id`.
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSongSampler::content_id.eq(cid))
            .first(conn)
            .optional()
    }

    /// Counts the number of records in the `djmdSongSampler` table for a given `sampler_id`.
    pub fn count(conn: &mut SqliteConnection, sampler_id: &str) -> QueryResult<i32> {
        let count: i64 = djmdSongSampler::table
            .filter(djmdSongSampler::sampler_id.eq(sampler_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    /// Reset the `track_no` numbers of all records from the `djmdSongSampler` table for a given `sampler_id`.
    ///
    /// The `track_no` numbers are reset to the *current* order of the records in the
    /// `djmdSongSampler` table, starting at 1:
    ///
    /// ```
    /// Old: 1, 2, 3, 5, 7, 8, 9
    /// New: 1, 2, 3, 4, 5, 6, 7
    /// ```
    ///
    /// This is helpful when deleting records or for fixing the sequence after an invalid state.
    /// When adding or moving records, the `track_no` numbers should be set explicitly.
    pub fn reset_seq(conn: &mut SqliteConnection, sampler_id: &str) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY TrackNo) AS new_seq
                FROM djmdSongSampler WHERE SamplerID =?
            ) UPDATE djmdSongSampler
            SET TrackNo = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSongSampler.ID);"#,
        )
        .bind::<diesel::sql_types::Text, _>(sampler_id)
        .execute(conn)
    }

    /// Deletes a record from the `djmdSongSampler` table
    pub fn delete(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
        let query = djmdSongSampler::table.find(id);
        let my_tag_id: String = query.select(djmdSongSampler::sampler_id).first(conn)?;

        let result = diesel::delete(query).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Reorder the track_no numbers
        Self::reset_seq(conn, &my_tag_id)?;

        Ok(result)
    }

    /// Deletes a record from the `djmdSongSampler` table based on its `sampler_id`.
    pub fn delete_by_sampler_id(
        conn: &mut SqliteConnection,
        sampler_id: &str,
    ) -> QueryResult<usize> {
        let result = diesel::delete(
            djmdSongSampler::table.filter(djmdSongSampler::sampler_id.eq(sampler_id)),
        )
        .execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, result)?;
        Ok(result)
    }

    /// Deletes multiple records from the `djmdSongSampler` table based on their `sampler_id`.
    pub fn delete_by_sampler_ids(
        conn: &mut SqliteConnection,
        sampler_ids: &[String],
    ) -> QueryResult<usize> {
        let result = diesel::delete(
            djmdSongSampler::table.filter(djmdSongSampler::sampler_id.eq_any(sampler_ids)),
        )
        .execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, result)?;
        Ok(result)
    }
}
