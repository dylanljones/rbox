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
use uuid::Uuid;

use super::agent_registry::AgentRegistry;
use super::djmd_content::DjmdContent;
use super::schema::{djmdContent, djmdMyTag, djmdSongMyTag};
use super::util::{format_datetime, Date, DateString};
use super::RandomIdGenerator;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdMyTag` table in the Rekordbox database.
///
/// This struct maps to the `djmdMyTag` table in the SQLite database used by Rekordbox.
/// It stores information about custom tags (MyTags) that can be associated with tracks,
/// including metadata such as sequence, name, attributes, and parent relationships.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdMyTag)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdMyTag, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdMyTag {
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

    /// The sequence/order number of the tag (1-based index)
    pub seq: i32,
    /// The name of the tag
    pub name: String,
    /// The attribute of the tag, either my-tag (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The ID of the parent [`DjmdMyTag`], `'root'` for top-level my-tag records.
    pub parent_id: String,
}

impl DjmdMyTag {
    /// Queries all records from the `djmdMyTag` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdMyTag` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries a record from the `djmdMyTag` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdMyTag::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Queries a record from the `djmdMyTag` table by its associated `content_id` via `djmdSongMyTag`
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        djmdMyTag::table
            .inner_join(djmdSongMyTag::table.on(djmdMyTag::id.eq(djmdSongMyTag::my_tag_id)))
            .filter(djmdSongMyTag::content_id.eq(cid))
            .select(DjmdMyTag::as_select())
            .first(conn)
            .optional()
    }

    /// Queries the records from the `djmdMyTag` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdMyTag::parent_id.eq(parent_id))
            .load(conn)
    }

    /// Queries the records from the `djmdContent` table associated with the given `my_tag_id`.
    pub fn get_contents(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<DjmdContent>> {
        djmdContent::table
            .inner_join(djmdSongMyTag::table.on(djmdContent::id.eq(djmdSongMyTag::content_id)))
            .filter(djmdSongMyTag::my_tag_id.eq(&id))
            .select(DjmdContent::as_select())
            .load(conn)
    }

    /// Counts the number of child records in the `djmdMyTag` table for a given `parent_id`.
    pub fn count_children(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<i32> {
        let count: i64 = djmdMyTag::table
            .filter(djmdMyTag::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    /// Counts the number of child records in the `djmdMyTag` table for a record inside the same parent.
    pub fn count_parent_children(conn: &mut SqliteConnection, id: &str) -> QueryResult<i32> {
        let parent_id: String = djmdMyTag::table
            .find(id)
            .select(djmdMyTag::parent_id)
            .get_result(conn)?;
        Self::count_children(conn, &parent_id)
    }

    /// Checks if a record with the given `id` exists in the `djmdMyTag` table.
    pub fn id_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }

    /// Checks if a record with the given `name` exists in the `djmdMyTag` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdMyTag::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Updates an existing record in the `djmdMyTag` table.
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
        Ok(diesel::update(djmdMyTag::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)?)
    }

    /// Reset the `seq` numbers of all records from the `djmdMyTag` table for a given `parent_id`.
    ///
    /// The `seq` numbers are reset to the *current* order of the records in the `djmdMyTag` table,
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
                FROM djmdMyTag WHERE ParentID =?
            ) UPDATE djmdMyTag
            SET Seq = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdMyTag.ID);"#,
        )
        .bind::<diesel::sql_types::Text, _>(parent_id)
        .execute(conn)
    }

    /// Move a record in the `djmdMyTag` table inside the same parent.
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
                djmdMyTag::table
                    .filter(djmdMyTag::parent_id.eq(rec.parent_id))
                    .filter(djmdMyTag::seq.gt(old_seq))
                    .filter(djmdMyTag::seq.le(seq)),
            )
            .set((
                djmdMyTag::seq.eq(djmdMyTag::seq - 1),
                djmdMyTag::updated_at.eq(&datestr),
                djmdMyTag::rb_local_usn.eq(usn),
            ))
            .execute(conn)?;
        } else if seq < old_seq {
            diesel::update(
                djmdMyTag::table
                    .filter(djmdMyTag::parent_id.eq(rec.parent_id))
                    .filter(djmdMyTag::seq.lt(old_seq))
                    .filter(djmdMyTag::seq.ge(seq)),
            )
            .set((
                djmdMyTag::seq.eq(djmdMyTag::seq + 1),
                djmdMyTag::updated_at.eq(&datestr),
                djmdMyTag::rb_local_usn.eq(usn),
            ))
            .execute(conn)?;
        } else {
            return Ok(0);
        }

        // Update sequence number of record
        diesel::update(djmdMyTag::table.find(id))
            .set((
                djmdMyTag::seq.eq(seq),
                djmdMyTag::updated_at.eq(&datestr),
                djmdMyTag::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }

    /// Move a record in the `djmdMyTag` table to a new parent.
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
            let my_tag_attr: i32 = djmdMyTag::table
                .find(&parent_id)
                .select(djmdMyTag::attribute)
                .get_result(conn)?;
            if my_tag_attr != 1 {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new("Cannot add a DjmdMyTag child to a DjmdMyTag my-tag".to_string()),
                ));
            }

            // Update seq of records in the old parent:
            // Decrease seq of records with seq higher than old seq
            diesel::update(
                djmdMyTag::table
                    .filter(djmdMyTag::parent_id.eq(old_parent_id))
                    .filter(djmdMyTag::seq.gt(old_seq)),
            )
            .set((
                djmdMyTag::seq.eq(djmdMyTag::seq - 1),
                djmdMyTag::updated_at.eq(&datestr),
                djmdMyTag::rb_local_usn.eq(usn),
            ))
            .execute(conn)?;

            if seq < count {
                // Move to center in the new parent
                // Increase seq of records with seq higher than old seq
                diesel::update(
                    djmdMyTag::table
                        .filter(djmdMyTag::parent_id.eq(parent_id))
                        .filter(djmdMyTag::seq.ge(old_seq)),
                )
                .set((
                    djmdMyTag::seq.eq(djmdMyTag::seq + 1),
                    djmdMyTag::updated_at.eq(&datestr),
                    djmdMyTag::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
            }

            // Update Seq and parent of actual record
            diesel::update(djmdMyTag::table.find(id))
                .set((
                    djmdMyTag::seq.eq(seq),
                    djmdMyTag::parent_id.eq(parent_id),
                    djmdMyTag::updated_at.eq(&datestr),
                    djmdMyTag::rb_local_usn.eq(usn),
                ))
                .execute(conn)
        } else {
            if seq > old_seq {
                // Seq is increased (move down in parent):
                // Decrease seq of all other records with seq between old seq and new seq
                diesel::update(
                    djmdMyTag::table
                        .filter(djmdMyTag::parent_id.eq(rec.parent_id))
                        .filter(djmdMyTag::seq.gt(old_seq))
                        .filter(djmdMyTag::seq.le(seq)),
                )
                .set((
                    djmdMyTag::seq.eq(djmdMyTag::seq - 1),
                    djmdMyTag::updated_at.eq(&datestr),
                    djmdMyTag::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
            } else if seq < old_seq {
                // Seq is decreased (moved down in parent):
                // Increase seq of all records with seq between old seq and new seq
                diesel::update(
                    djmdMyTag::table
                        .filter(djmdMyTag::parent_id.eq(rec.parent_id))
                        .filter(djmdMyTag::seq.lt(old_seq))
                        .filter(djmdMyTag::seq.ge(seq)),
                )
                .set((
                    djmdMyTag::seq.eq(djmdMyTag::seq + 1),
                    djmdMyTag::updated_at.eq(&datestr),
                    djmdMyTag::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
            } else {
                return Ok(0);
            }
            // Update sequence number of record
            diesel::update(djmdMyTag::table.find(id))
                .set((
                    djmdMyTag::seq.eq(seq),
                    djmdMyTag::updated_at.eq(&datestr),
                    djmdMyTag::rb_local_usn.eq(usn),
                ))
                .execute(conn)
        }
    }

    /// Deletes a record from the `djmdMyTag` table and removes any references to it.
    pub fn delete(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
        // Delete the record
        let parent_id: String = diesel::delete(djmdMyTag::table.find(id))
            .returning(djmdMyTag::parent_id)
            .get_result(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        let mut result = 1;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove any djmdSongMyTag records that are associated with this djmdMyTag
        DjmdSongMyTag::delete_by_tag_id(conn, id)?;

        // Remove all child djmdMyTag records recursively
        let mut parent_ids = VecDeque::from(vec![id.to_string()]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted_ids: Vec<String> =
                diesel::delete(djmdMyTag::table.filter(djmdMyTag::parent_id.eq(parent_id)))
                    .returning(djmdMyTag::id)
                    .get_results(conn)?;
            AgentRegistry::increment_local_usn_by(conn, deleted_ids.len())?;
            result += deleted_ids.len();

            // Delete any songs referencing the deleted tags
            DjmdSongMyTag::delete_by_tag_ids(conn, &deleted_ids)?;
            // Add children to the queue
            for deleted_id in deleted_ids {
                parent_ids.push_back(deleted_id);
            }
        }
        Ok(result)
    }

    /// Generates a new unique identifier for a record in the `djmdMyTag` table.
    fn generate_id(conn: &mut SqliteConnection) -> QueryResult<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                if !Self::id_exists(conn, &tmp_id)? {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }
}

/// Represents a new record insertale to the `djmdMyTag` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdMyTag;
///
/// let new = NewDjmdMyTag::new("Name".into(), 0).seq(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdMyTag {
    /// The name of the tag
    pub name: String,
    /// The sequence/order number of the tag (1-based index)
    pub seq: Option<i32>,
    /// The attribute of the tag, either my-tag (`0`) or a section (`1`)
    pub attribute: Option<i32>,
    /// The ID of the parent [`DjmdMyTag`].
    pub parent_id: Option<String>,
}

impl NewDjmdMyTag {
    /// Creates a new `NewDjmdMyTag` with the required `name` field.
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// Inserts the new record into the database
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<DjmdMyTag> {
        let id = DjmdMyTag::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();
        let usn = AgentRegistry::increment_local_usn(conn)?;

        let parent_id = self.parent_id.unwrap_or("root".to_string());
        let count = DjmdMyTag::count_parent_children(conn, &parent_id)?;

        let item = DjmdMyTag {
            id,
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            name: self.name,
            seq: self.seq.unwrap_or(count + 1),
            attribute: self.attribute.unwrap_or(0),
            parent_id,
            ..Default::default()
        };

        if let Some(seq) = self.seq {
            // Update sequence numbers above new item
            diesel::update(
                djmdMyTag::table
                    .filter(djmdMyTag::parent_id.eq(&item.parent_id))
                    .filter(djmdMyTag::seq.ge(seq)),
            )
            .set(djmdMyTag::seq.eq(djmdMyTag::seq + 1))
            .execute(conn)?;
        }

        let result = diesel::insert_into(djmdMyTag::table)
            .values(item)
            .get_result(conn)?;
        Ok(result)
    }

    /// Inserts the new record or returns an existing record if a tag with the same name already exists.
    pub fn insert_if_not_exists(self, conn: &mut SqliteConnection) -> QueryResult<DjmdMyTag> {
        match DjmdMyTag::find_by_name(conn, &self.name)? {
            Some(e) => Ok(e),
            None => self.insert(conn),
        }
    }

    /// Builder for `seq`.
    pub fn seq(mut self, seq: i32) -> Self {
        self.seq = Some(seq);
        self
    }

    /// Builder for `parent_id`.
    pub fn parent_id(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

/// Represents the `djmdSongMyTag` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongMyTag` table in the SQLite database used by Rekordbox.
/// It links the `djmdMyTag` records to the associated `djmdContent` records.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdSongMyTag)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdMyTag, foreign_key = my_tag_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdSongMyTag {
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

    /// The ID of the associated [`DjmdMyTag`].
    pub my_tag_id: String,
    /// The ID of the associated [`DjmdContent`].
    pub content_id: String,
    /// The track number in the tag.
    pub track_no: i32,
}

impl DjmdSongMyTag {
    /// Queries all records from the `djmdSongMyTag` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdSongMyTag` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries all records from the `djmdSongMyTag` table by its `my_tag_id`.
    pub fn by_tag_id(conn: &mut SqliteConnection, tag_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongMyTag::my_tag_id.eq(tag_id))
            .get_results(conn)
    }

    /// Queries a record from the `djmdSongMyTag` table by its `content_id`.
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSongMyTag::content_id.eq(cid))
            .first(conn)
            .optional()
    }

    /// Counts the number of records in the `djmdSongMyTag` table for a given `my_tag_id`.
    pub fn count(conn: &mut SqliteConnection, my_tag_id: &str) -> QueryResult<i32> {
        let count: i64 = djmdSongMyTag::table
            .filter(djmdSongMyTag::my_tag_id.eq(my_tag_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    /// Reset the `track_no` numbers of all records from the `djmdSongMyTag` table for a given `my_tag_id`.
    ///
    /// The `track_no` numbers are reset to the *current* order of the records in the
    /// `djmdSongMyTag` table, starting at 1:
    ///
    /// ```
    /// Old: 1, 2, 3, 5, 7, 8, 9
    /// New: 1, 2, 3, 4, 5, 6, 7
    /// ```
    ///
    /// This is helpful when deleting records or for fixing the sequence after an invalid state.
    /// When adding or moving records, the `track_no` numbers should be set explicitly.
    pub fn reset_seq(conn: &mut SqliteConnection, my_tag_id: &str) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY TrackNo) AS new_seq
                FROM djmdSongMyTag WHERE MyTagID =?
            ) UPDATE djmdSongMyTag
            SET TrackNo = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSongMyTag.ID);"#,
        )
        .bind::<diesel::sql_types::Text, _>(my_tag_id)
        .execute(conn)
    }

    /// Deletes a record from the `djmdSongMyTag` table
    pub fn delete(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
        let query = djmdSongMyTag::table.find(id);
        let my_tag_id: String = query.select(djmdSongMyTag::my_tag_id).first(conn)?;

        let result = diesel::delete(query).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Reorder the track_no numbers
        Self::reset_seq(conn, &my_tag_id)?;

        Ok(result)
    }

    /// Deletes a record from the `djmdSongMyTag` table based on its `my_tag_id`.
    pub fn delete_by_tag_id(conn: &mut SqliteConnection, my_tag_id: &str) -> QueryResult<usize> {
        let result =
            diesel::delete(djmdSongMyTag::table.filter(djmdSongMyTag::my_tag_id.eq(my_tag_id)))
                .execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, result)?;
        Ok(result)
    }

    /// Deletes multiple records from the `djmdSongMyTag` table based on their `my_tag_id`.
    pub fn delete_by_tag_ids(
        conn: &mut SqliteConnection,
        my_tag_ids: &[String],
    ) -> QueryResult<usize> {
        let result = diesel::delete(
            djmdSongMyTag::table.filter(djmdSongMyTag::my_tag_id.eq_any(my_tag_ids)),
        )
        .execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, result)?;
        Ok(result)
    }
}

/// Represents a new record insertale to the `djmdSongMyTag` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdSongMyTag;
///
/// let new = NewDjmdMyTag::new("Name".into(), 0).seq(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdSongMyTag {
    /// The ID of the associated [`DjmdMyTag`].
    pub my_tag_id: String,
    /// The ID of the associated [`DjmdContent`].
    pub content_id: String,
    /// The track number in the tag.
    pub track_no: Option<i32>,
}

impl NewDjmdSongMyTag {
    pub fn new(my_tag_id: String, content_id: String) -> Self {
        Self {
            my_tag_id,
            content_id,
            ..Default::default()
        }
    }

    /// Inserts the new record into the database
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<DjmdSongMyTag> {
        let id = DjmdMyTag::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        let my_tag_attr: i32 = djmdMyTag::table
            .find(&self.my_tag_id)
            .select(djmdMyTag::attribute)
            .get_result(conn)?;
        if my_tag_attr != 0 {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new("Cannot add a DjmdSongMyTag to a DjmdMyTag folder".to_string()),
            ));
        }

        let count = DjmdSongMyTag::count(conn, &self.my_tag_id)?;

        let item = DjmdSongMyTag {
            id,
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            my_tag_id: self.my_tag_id,
            content_id: self.content_id,
            track_no: self.track_no.unwrap_or(count),
            ..Default::default()
        };

        if let Some(track_no) = self.track_no {
            // Update sequence numbers above new item
            diesel::update(
                djmdSongMyTag::table
                    .filter(djmdSongMyTag::my_tag_id.eq(&item.my_tag_id))
                    .filter(djmdSongMyTag::track_no.ge(track_no)),
            )
            .set(djmdSongMyTag::track_no.eq(djmdSongMyTag::track_no + 1))
            .execute(conn)?;
        }

        let result = diesel::insert_into(djmdSongMyTag::table)
            .values(item)
            .get_result(conn)?;
        Ok(result)
    }

    /// Builder for `track_no`.
    pub fn track_no(mut self, track_no: i32) -> Self {
        self.track_no = Some(track_no);
        self
    }
}
