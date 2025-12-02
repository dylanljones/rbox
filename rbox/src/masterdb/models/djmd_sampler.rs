// Author: Dylan Jones
// Date:   2025-09-02

use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;
use std::collections::VecDeque;
use uuid::Uuid;

use super::agent_registry::AgentRegistry;
use super::djmd_content::DjmdContent;
use super::schema::{djmdContent, djmdSampler, djmdSongSampler};
use super::{format_datetime, Date, DateString, RandomIdGenerator};
use crate::model_traits::{
    Model, ModelDelete, ModelInsert, ModelList, ModelTree, ModelUpdate, TreeSeq,
};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdSampler` table in the Rekordbox database.
///
/// This struct maps to the `djmdSampler` table in the SQLite database used by Rekordbox.
/// It stores information about samplers, including metadata such as sequence, name,
/// attributes, and parent relationships.
///
/// # Referenced by
/// * [`DjmdSampler`] via `parent_id` foreign key.
/// * [`DjmdSongSampler`] via `sampler_id` foreign key.
///
/// # References
/// * [`DjmdSampler`] via `parent_id` foreign key.
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

impl Model for DjmdSampler {
    type Id = str;

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

impl ModelUpdate for DjmdSampler {
    fn update(mut self, conn: &mut SqliteConnection) -> QueryResult<Self> {
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
        diesel::update(djmdSampler::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for DjmdSampler {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        // Vec of all deleted sampler ids
        let mut deleted_ids = vec![id.to_string()];
        // Delete the record
        let parent_id: String = diesel::delete(djmdSampler::table.find(id))
            .returning(djmdSampler::parent_id)
            .get_result(conn)?;
        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;
        // Remove all child records recursively
        let mut parent_ids = VecDeque::from(vec![id.to_string()]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted: Vec<String> =
                diesel::delete(djmdSampler::table.filter(djmdSampler::parent_id.eq(parent_id)))
                    .returning(djmdSampler::id)
                    .get_results(conn)?;
            deleted_ids.extend(deleted.clone());
            // Add children to the queue
            for deleted_id in deleted {
                parent_ids.push_back(deleted_id);
            }
        }
        // Remove any records that are associated with the deleted samplers
        DjmdSongSampler::delete_by_sampler_ids(conn, &deleted_ids)?;

        let n = deleted_ids.len();
        AgentRegistry::increment_local_usn_by(conn, n)?;

        Ok(n)
    }
}

impl ModelTree for DjmdSampler {
    fn move_to(
        conn: &mut SqliteConnection,
        id: &Self::Id,
        parent_id: Option<&Self::Id>,
        seq: Option<i32>,
    ) -> QueryResult<usize> {
        let res = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };
        let old_seq = res.seq;
        let old_parent_id = res.parent_id.clone();
        let parent_id = parent_id.unwrap_or(&old_parent_id);

        // *Note*: Moving other records increments USN by 1 for all changes
        let res = Self::update_seq_before_move(conn, &old_parent_id, parent_id, old_seq, seq)?;
        let (seq, _n) = match res {
            Some((s, n)) => (s, n),
            None => return Ok(0),
        };

        // Update Seq and parent of actual record
        let now = Utc::now();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        diesel::update(djmdSampler::table.find(id))
            .set((
                djmdSampler::seq.eq(seq),
                djmdSampler::parent_id.eq(parent_id),
                djmdSampler::updated_at.eq(&format_datetime(&now)),
                djmdSampler::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdSampler {
    /// Queries a record from the `djmdSampler` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSampler::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Queries the records from the `djmdSampler` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSampler::parent_id.eq(parent_id))
            .order(djmdSampler::seq)
            .load(conn)
    }

    /// Queries all records from the `djmdSampler` table by their associated `content_id` via `djmdSongSampler`
    pub fn by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Vec<Self>> {
        djmdSampler::table
            .inner_join(djmdSongSampler::table.on(djmdSampler::id.eq(djmdSongSampler::sampler_id)))
            .filter(djmdSongSampler::content_id.eq(cid))
            .select(Self::as_select())
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

    /// Returns the playlist type (`attribute`) of a record in the `djmdSampler` table or `None` if not found.
    pub fn get_attribute(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<i32>> {
        if id == "root" {
            return Ok(Some(1));
        }
        djmdSampler::table
            .find(id)
            .select(djmdSampler::attribute)
            .get_result(conn)
            .optional()
    }

    /// Returns `true` if the record in the `djmdSampler` table exists and is a sampler, `false` otherwise.
    pub fn is_sampler(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 0), // 0: playlist
            None => Ok(false),
        }
    }

    /// Returns `true` if the record in the `djmdSampler` table exists and is a folder, `false` otherwise.
    pub fn is_folder(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 1), // 1: folder
            None => Ok(false),
        }
    }

    /// Set the name of a record in the `djmdSampler` table.
    pub fn rename(conn: &mut SqliteConnection, id: &str, name: &str) -> QueryResult<Self> {
        let datestr = format_datetime(&Utc::now());
        let usn = AgentRegistry::increment_local_usn(conn)?;
        diesel::update(djmdSampler::table.find(id))
            .set((
                djmdSampler::name.eq(name),
                djmdSampler::updated_at.eq(&datestr),
                djmdSampler::rb_local_usn.eq(usn),
            ))
            .get_result(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdSampler` table.
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

impl TreeSeq for DjmdSampler {
    type ParentId = str;
    const START_SEQ: i32 = 1;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        Self::is_folder(conn, parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = djmdSampler::table
            .filter(djmdSampler::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        djmdSampler::table
            .filter(djmdSampler::parent_id.eq(parent_id))
            .order(djmdSampler::seq)
            .select(djmdSampler::seq)
            .load(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY Seq) + (? - 1) AS new_seq
                FROM djmdSampler WHERE ParentID =?
            ) UPDATE djmdSampler
            SET Seq = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSampler.ID);
            WHERE ParentID =?;"#,
        )
        .bind::<diesel::sql_types::Integer, _>(Self::START_SEQ)
        .bind::<diesel::sql_types::Text, _>(parent_id)
        .bind::<diesel::sql_types::Text, _>(parent_id)
        .execute(conn)
    }

    #[inline]
    fn increment_seq_gte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        let now = Utc::now();
        let usn = AgentRegistry::local_usn(conn)? + 1;
        diesel::update(
            djmdSampler::table
                .filter(djmdSampler::parent_id.eq(parent_id))
                .filter(djmdSampler::seq.ge(seq)),
        )
        .set((
            djmdSampler::seq.eq(djmdSampler::seq + 1),
            djmdSampler::updated_at.eq(format_datetime(&now)),
            djmdSampler::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        let now = Utc::now();
        let usn = AgentRegistry::local_usn(conn)? + 1;
        diesel::update(
            djmdSampler::table
                .filter(djmdSampler::parent_id.eq(parent_id))
                .filter(djmdSampler::seq.gt(seq)),
        )
        .set((
            djmdSampler::seq.eq(djmdSampler::seq - 1),
            djmdSampler::updated_at.eq(format_datetime(&now)),
            djmdSampler::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }

    #[inline]
    fn increment_seq_gte_lt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        let now = Utc::now();
        let usn = AgentRegistry::local_usn(conn)? + 1;
        diesel::update(
            djmdSampler::table
                .filter(djmdSampler::parent_id.eq(parent_id))
                .filter(djmdSampler::seq.ge(start))
                .filter(djmdSampler::seq.lt(end)),
        )
        .set((
            djmdSampler::seq.eq(djmdSampler::seq + 1),
            djmdSampler::updated_at.eq(&format_datetime(&now)),
            djmdSampler::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt_lte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        let now = Utc::now();
        let usn = AgentRegistry::local_usn(conn)? + 1;
        diesel::update(
            djmdSampler::table
                .filter(djmdSampler::parent_id.eq(parent_id))
                .filter(djmdSampler::seq.gt(start))
                .filter(djmdSampler::seq.le(end)),
        )
        .set((
            djmdSampler::seq.eq(djmdSampler::seq - 1),
            djmdSampler::updated_at.eq(&format_datetime(&now)),
            djmdSampler::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `djmdSampler` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdSampler;
///
/// let new = NewDjmdPlaylist::sampler("Name").seq(2);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdSampler {
    /// The name of the sampler.
    pub name: String,
    /// The attribute of the sampler, either list (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The sequence/order number of the record (1-based index)
    pub seq: Option<i32>,
    /// The ID of the parent [`DjmdSampler`], `'root'` for top-level records.
    pub parent_id: Option<String>,
}

impl ModelInsert for NewDjmdSampler {
    type Model = DjmdSampler;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let parent_id = self.parent_id.unwrap_or("root".to_string());
        // Handle seq and USN of moved records (also checks parent)
        let (seq, n) = Self::Model::update_seq_before_insert(conn, &parent_id, self.seq)?;
        if n > 0 {
            // Apply USN of moved records
            AgentRegistry::increment_local_usn(conn)?;
        }
        // Generate meta
        let id = Self::Model::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();
        // Get next USN: We increment by 2 (1 for creating, 1 for renaming from 'New Playlist')
        let usn = AgentRegistry::increment_local_usn_by(conn, 2)?;
        let item = Self::Model {
            id,
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            name: self.name,
            seq,
            attribute: self.attribute,
            parent_id,
            ..Default::default()
        };
        diesel::insert_into(djmdSampler::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdSampler {
    /// Creates a new `NewDjmdSampler` with the required `name` and `attribute` field.
    pub fn new<S: Into<String>>(name: S, attribute: i32) -> Self {
        Self {
            name: name.into(),
            attribute,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdSampler` as a sampler.
    pub fn sampler<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: 0,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdSampler` as a folder.
    pub fn folder<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: 1,
            ..Default::default()
        }
    }

    /// Builder for `seq`.
    pub fn seq(mut self, seq: i32) -> Self {
        self.seq = Some(seq);
        self
    }

    /// Builder for `parent_id`.
    pub fn parent_id<S: Into<String>>(mut self, parent_id: S) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }
}

/// Represents the `djmdSongSampler` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongSampler` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and samplers, including
/// metadata such as update sequence numbers, timestamps, and associated sampler or content IDs.
///
/// # References
/// * [`DjmdSampler`] via `sampler_id` foreign key.
/// * [`DjmdContent`] via `content_id` foreign key.
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

impl Model for DjmdSongSampler {
    type Id = str;

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

impl ModelDelete for DjmdSongSampler {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let query = djmdSongSampler::table.find(id);
        let sampler_id: String = query.select(djmdSongSampler::sampler_id).first(conn)?;
        let result = diesel::delete(query).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Reorder the track_no numbers
        Self::reset_seq(conn, &sampler_id)?;
        Ok(result)
    }
}

impl ModelList for DjmdSongSampler {
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize> {
        let res = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };
        let old_seq = res.track_no;
        // *Note*: Moving other records increments USN by 1 for all changes
        let res = Self::update_seq_before_move_in(conn, &res.sampler_id, old_seq, seq)?;
        let (seq, _n) = match res {
            Some((s, n)) => (s, n),
            None => return Ok(0),
        };
        // Update seq of actual record
        let now = Utc::now();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        diesel::update(djmdSongSampler::table.find(id))
            .set((
                djmdSongSampler::track_no.eq(seq),
                djmdSongSampler::updated_at.eq(&format_datetime(&now)),
                djmdSongSampler::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdSongSampler {
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

    /// Creates a filter for records by `sampler_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_sampler_id(id: &str) -> _ {
        djmdSongSampler::table.filter(djmdSongSampler::sampler_id.eq(id))
    }

    /// Creates a filter for records by `sampler_ids`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_sampler_ids(ids: &[String]) -> _ {
        djmdSongSampler::table.filter(djmdSongSampler::sampler_id.eq_any(ids))
    }

    /// Counts the number of records in the `djmdSongSampler` table for a given `sampler_id`.
    pub fn count(conn: &mut SqliteConnection, sampler_id: &str) -> QueryResult<i32> {
        Self::count_children(conn, sampler_id)
    }

    /// Deletes a record from the `djmdSongSampler` table based on its `sampler_id`.
    pub fn delete_by_sampler_id(
        conn: &mut SqliteConnection,
        sampler_id: &str,
    ) -> QueryResult<usize> {
        diesel::delete(Self::filter_by_sampler_id(sampler_id)).execute(conn)
    }

    /// Deletes multiple records from the `djmdSongSampler` table based on their `sampler_id`.
    pub fn delete_by_sampler_ids(
        conn: &mut SqliteConnection,
        sampler_ids: &[String],
    ) -> QueryResult<usize> {
        diesel::delete(Self::filter_by_sampler_ids(sampler_ids)).execute(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdSongSampler` table.
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

impl TreeSeq for DjmdSongSampler {
    type ParentId = str;
    const START_SEQ: i32 = 1;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        DjmdSampler::is_sampler(conn, parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = Self::filter_by_sampler_id(parent_id)
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        Self::filter_by_sampler_id(parent_id)
            .order(djmdSongSampler::track_no)
            .select(djmdSongSampler::track_no)
            .get_results(conn)
    }
    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY TrackNo) + (? - 1) AS new_seq
                FROM djmdSongSampler WHERE SamplerID =?
            ) UPDATE djmdSongSampler
            SET TrackNo = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSongSampler.ID)
            WHERE PlaylistID =?;"#,
        )
        .bind::<diesel::sql_types::Integer, _>(Self::START_SEQ)
        .bind::<diesel::sql_types::Text, _>(parent_id)
        .bind::<diesel::sql_types::Text, _>(parent_id)
        .execute(conn)
    }

    #[inline]
    fn increment_seq_gte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        let now = Utc::now();
        let usn = AgentRegistry::local_usn(conn)? + 1;
        diesel::update(
            djmdSongSampler::table
                .filter(djmdSongSampler::sampler_id.eq(parent_id))
                .filter(djmdSongSampler::track_no.ge(seq)),
        )
        .set((
            djmdSongSampler::track_no.eq(djmdSongSampler::track_no + 1),
            djmdSongSampler::updated_at.eq(format_datetime(&now)),
            djmdSongSampler::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        let now = Utc::now();
        let usn = AgentRegistry::local_usn(conn)? + 1;
        diesel::update(
            djmdSongSampler::table
                .filter(djmdSongSampler::sampler_id.eq(parent_id))
                .filter(djmdSongSampler::track_no.gt(seq)),
        )
        .set((
            djmdSongSampler::track_no.eq(djmdSongSampler::track_no - 1),
            djmdSongSampler::updated_at.eq(format_datetime(&now)),
            djmdSongSampler::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }

    #[inline]
    fn increment_seq_gte_lt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        let now = Utc::now();
        let usn = AgentRegistry::local_usn(conn)? + 1;
        diesel::update(
            djmdSongSampler::table
                .filter(djmdSongSampler::sampler_id.eq(parent_id))
                .filter(djmdSongSampler::track_no.ge(start))
                .filter(djmdSongSampler::track_no.lt(end)),
        )
        .set((
            djmdSongSampler::track_no.eq(djmdSongSampler::track_no + 1),
            djmdSongSampler::updated_at.eq(&format_datetime(&now)),
            djmdSongSampler::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt_lte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        let now = Utc::now();
        let usn = AgentRegistry::local_usn(conn)? + 1;
        diesel::update(
            djmdSongSampler::table
                .filter(djmdSongSampler::sampler_id.eq(parent_id))
                .filter(djmdSongSampler::track_no.gt(start))
                .filter(djmdSongSampler::track_no.le(end)),
        )
        .set((
            djmdSongSampler::track_no.eq(djmdSongSampler::track_no - 1),
            djmdSongSampler::updated_at.eq(&format_datetime(&now)),
            djmdSongSampler::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `djmdSongSampler` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdSongSampler;
///
/// let new = NewDjmdSongSampler::new("samplerId", "contentId").track_no(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdSongSampler {
    /// The ID of the associated sampler in [`DjmdSampler`].
    pub sampler_id: String,
    /// The ID of the associated content in [`DjmdContent`].
    pub content_id: String,
    /// The sequence/order number of the record (1-based index)
    pub track_no: Option<i32>,
}

impl ModelInsert for NewDjmdSongSampler {
    type Model = DjmdSongSampler;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        // Check content
        if !DjmdContent::id_exists(conn, &self.content_id)? {
            return Err(Error::NotFound);
        }
        let parent_id = &self.sampler_id;
        // Handle seq and USN of moved records (also checkks parent)
        let (seq, n) = Self::Model::update_seq_before_insert(conn, &parent_id, self.track_no)?;
        if n > 0 {
            // Apply USN of moved records
            AgentRegistry::increment_local_usn(conn)?;
        }
        // Generate meta
        let id = Self::Model::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();
        // Get next USN: We increment by 2 (1 for creating, 1 for renaming from 'New Sampler')
        let usn = AgentRegistry::increment_local_usn_by(conn, 2)?;
        let item = Self::Model {
            id,
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            sampler_id: self.sampler_id,
            content_id: self.content_id,
            track_no: seq,
            ..Default::default()
        };
        diesel::insert_into(djmdSongSampler::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdSongSampler {
    /// Creates a new `NewDjmdSongSampler` with the required `sampler_id` and `content_id` field.
    pub fn new<S1: Into<String>, S2: Into<String>>(sampler_id: S1, content_id: S2) -> Self {
        Self {
            sampler_id: sampler_id.into(),
            content_id: content_id.into(),
            ..Default::default()
        }
    }

    /// Builder for `track_no`.
    pub fn track_no(mut self, track_no: i32) -> Self {
        self.track_no = Some(track_no);
        self
    }
}
