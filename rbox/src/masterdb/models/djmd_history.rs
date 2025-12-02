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
use super::schema::{djmdContent, djmdHistory, djmdSongHistory};
use super::{format_datetime, Date, DateString, RandomIdGenerator};
use crate::model_traits::{
    Model, ModelDelete, ModelInsert, ModelList, ModelTree, ModelUpdate, TreeSeq,
};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdHistory` table in the Rekordbox database.
///
/// This struct maps to the `djmdHistory` table in the SQLite database used by Rekordbox.
/// It stores information about history entries, including metadata such as sequence, name,
/// attributes, and parent relationships.
///
/// # Referenced by
/// * [`DjmdHistory`] via `parent_id` foreign key.
/// * [`DjmdSongHistory`] via `history_id` foreign key.
///
/// # References
/// * [`DjmdHistory`] via `parent_id` foreign key.
#[derive(Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = djmdHistory)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdHistory, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdHistory {
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

    /// The sequence/order number of the history (1-based index)
    pub seq: i32,
    /// The name of the history.
    pub name: String,
    /// The attribute of the tag, either history (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The ID of the parent [`DjmdHistory`], `'root'` for top-level my-tag records.
    pub parent_id: String,
    /// The creation date of the history.
    pub date_created: String,
}

impl Model for DjmdHistory {
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

impl ModelUpdate for DjmdHistory {
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
        if self.date_created != existing.date_created {
            changes += 1;
        }
        if changes == 0 {
            return Ok(existing);
        }
        self.updated_at = Utc::now();
        self.rb_local_usn = Some(AgentRegistry::increment_local_usn_by(conn, changes)?);
        diesel::update(djmdHistory::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for DjmdHistory {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        // Vec of all deleted history ids
        let mut deleted_ids = vec![id.to_string()];

        // Delete the record
        let parent_id: String = diesel::delete(djmdHistory::table.find(id))
            .returning(djmdHistory::parent_id)
            .get_result(conn)?;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove all child djmdHistory records recursively
        let mut parent_ids = VecDeque::from(vec![id.to_string()]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted: Vec<String> =
                diesel::delete(djmdHistory::table.filter(djmdHistory::parent_id.eq(parent_id)))
                    .returning(djmdHistory::id)
                    .get_results(conn)?;
            deleted_ids.extend(deleted.clone());
            // Add children to the queue
            for deleted_id in deleted {
                parent_ids.push_back(deleted_id);
            }
        }

        // Remove any djmdSongHistory records that are associated with the deleted djmdHistories
        DjmdSongHistory::delete_by_history_ids(conn, &deleted_ids)?;

        AgentRegistry::increment_local_usn_by(conn, deleted_ids.len())?;

        Ok(deleted_ids.len())
    }
}

impl ModelTree for DjmdHistory {
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
        diesel::update(djmdHistory::table.find(id))
            .set((
                djmdHistory::seq.eq(seq),
                djmdHistory::parent_id.eq(parent_id),
                djmdHistory::updated_at.eq(&format_datetime(&now)),
                djmdHistory::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdHistory {
    /// Queries the records from the `djmdHistory` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdHistory::parent_id.eq(parent_id))
            .order(djmdHistory::seq)
            .load(conn)
    }

    /// Queries all record from the `djmdHistory` table by their associated `content_id` via `djmdSongHistory`
    pub fn by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Vec<Self>> {
        djmdHistory::table
            .inner_join(djmdSongHistory::table.on(djmdHistory::id.eq(djmdSongHistory::history_id)))
            .filter(djmdSongHistory::content_id.eq(cid))
            .select(Self::as_select())
            .load(conn)
    }

    /// Queries the records from the `djmdContent` table associated with the given `djmdHistory`.
    pub fn get_contents(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<DjmdContent>> {
        djmdContent::table
            .inner_join(djmdSongHistory::table.on(djmdContent::id.eq(djmdSongHistory::content_id)))
            .filter(djmdSongHistory::history_id.eq(&id))
            .order(djmdSongHistory::track_no)
            .select(DjmdContent::as_select())
            .load(conn)
    }

    /// Returns the playlist type (`attribute`) of a record in the `djmdHistory` table or `None` if not found.
    pub fn get_attribute(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<i32>> {
        if id == "root" {
            return Ok(Some(1));
        }
        djmdHistory::table
            .find(id)
            .select(djmdHistory::attribute)
            .get_result(conn)
            .optional()
    }

    /// Returns `true` if the record in the `djmdHistory` table exists and is a history, `false` otherwise.
    pub fn is_history(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 0), // 0: playlist
            None => Ok(false),
        }
    }

    /// Returns `true` if the record in the `djmdHistory` table exists and is a folder, `false` otherwise.
    pub fn is_folder(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 1), // 1: folder
            None => Ok(false),
        }
    }

    /// Set the name of a record in the `djmdHistory` table.
    pub fn rename(conn: &mut SqliteConnection, id: &str, name: &str) -> QueryResult<Self> {
        let datestr = format_datetime(&Utc::now());
        let usn = AgentRegistry::increment_local_usn(conn)?;
        Ok(diesel::update(djmdHistory::table.find(id))
            .set((
                djmdHistory::name.eq(name),
                djmdHistory::updated_at.eq(&datestr),
                djmdHistory::rb_local_usn.eq(usn),
            ))
            .get_result(conn)?)
    }

    /// Generates a new unique identifier for a record in the `djmdHistory` table.
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

impl TreeSeq for DjmdHistory {
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
        let count: i64 = djmdHistory::table
            .filter(djmdHistory::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        djmdHistory::table
            .filter(djmdHistory::parent_id.eq(parent_id))
            .order(djmdHistory::seq)
            .select(djmdHistory::seq)
            .load(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY Seq) + (? - 1) AS new_seq
                FROM djmdHistory WHERE ParentID =?
            ) UPDATE djmdHistory
            SET Seq = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdHistory.ID)
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
            djmdHistory::table
                .filter(djmdHistory::parent_id.eq(parent_id))
                .filter(djmdHistory::seq.ge(seq)),
        )
        .set((
            djmdHistory::seq.eq(djmdHistory::seq + 1),
            djmdHistory::updated_at.eq(format_datetime(&now)),
            djmdHistory::rb_local_usn.eq(usn),
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
            djmdHistory::table
                .filter(djmdHistory::parent_id.eq(parent_id))
                .filter(djmdHistory::seq.gt(seq)),
        )
        .set((
            djmdHistory::seq.eq(djmdHistory::seq - 1),
            djmdHistory::updated_at.eq(format_datetime(&now)),
            djmdHistory::rb_local_usn.eq(usn),
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
            djmdHistory::table
                .filter(djmdHistory::parent_id.eq(parent_id))
                .filter(djmdHistory::seq.ge(start))
                .filter(djmdHistory::seq.lt(end)),
        )
        .set((
            djmdHistory::seq.eq(djmdHistory::seq + 1),
            djmdHistory::updated_at.eq(&format_datetime(&now)),
            djmdHistory::rb_local_usn.eq(usn),
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
            djmdHistory::table
                .filter(djmdHistory::parent_id.eq(parent_id))
                .filter(djmdHistory::seq.gt(start))
                .filter(djmdHistory::seq.le(end)),
        )
        .set((
            djmdHistory::seq.eq(djmdHistory::seq - 1),
            djmdHistory::updated_at.eq(&format_datetime(&now)),
            djmdHistory::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `djmdHistory` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdHistory;
///
/// let new = NewDjmdHistory::history("Name").seq(2);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdHistory {
    /// The name of the history.
    pub name: String,
    /// The attribute of the tag, either history (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The sequence/order number of the history (1-based index)
    pub seq: Option<i32>,
    /// The ID of the parent [`DjmdHistory`], `'root'` for top-level my-tag records.
    pub parent_id: Option<String>,
    /// The creation date of the history.
    pub date_created: Option<String>,
}

impl ModelInsert for NewDjmdHistory {
    type Model = DjmdHistory;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let parent_id = self.parent_id.unwrap_or("root".to_string());
        // Handle seq and USN of moved records (also checkks parent)
        let (seq, n) = Self::Model::update_seq_before_insert(conn, &parent_id, self.seq)?;
        if n > 0 {
            // Apply USN of moved records
            AgentRegistry::increment_local_usn(conn)?;
        }
        // Generate meta
        let id = Self::Model::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();
        // Use today as date_created if not specified
        let date_created = self
            .date_created
            .unwrap_or_else(|| now.format("%Y-%m-%d").to_string());
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
            date_created,
            ..Default::default()
        };
        diesel::insert_into(djmdHistory::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdHistory {
    /// Creates a new `NewDjmdHistory` with the required `name` and `attribute` field.
    pub fn new<S: Into<String>>(name: S, attribute: i32) -> Self {
        Self {
            name: name.into(),
            attribute,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdHistory` as a history.
    pub fn history<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: 0,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdHistory` as a folder.
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

    /// Builder for `date_created`.
    pub fn date_created<S: Into<String>>(mut self, date_created: S) -> Self {
        self.parent_id = Some(date_created.into());
        self
    }
}

/// Represents the `djmdSongHistory` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongHistory` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and history entries, including
/// metadata such as track number and associated history or content IDs.
///
/// # References
/// * [`DjmdHistory`] via `history_id` foreign key.
/// * [`DjmdContent`] via `content_id` foreign key.
#[derive(Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = djmdSongHistory)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdHistory, foreign_key = history_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdSongHistory {
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

    /// The ID of the associated history entry in [`DjmdHistory`].
    pub history_id: String,
    /// The ID of the associated content entry in [`DjmdContent`].
    pub content_id: String,
    /// The track number in the history entry.
    pub track_no: i32,
}

impl Model for DjmdSongHistory {
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

impl ModelDelete for DjmdSongHistory {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let query = djmdSongHistory::table.find(id);
        let history_id: String = query.select(djmdSongHistory::history_id).first(conn)?;

        let result = diesel::delete(query).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Reorder the track_no numbers
        Self::reset_seq(conn, &history_id)?;

        Ok(result)
    }
}

impl ModelList for DjmdSongHistory {
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize> {
        let res = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };

        let old_seq = res.track_no;
        // *Note*: Moving other records increments USN by 1 for all changes
        let res = Self::update_seq_before_move_in(conn, &res.history_id, old_seq, seq)?;
        let (seq, _n) = match res {
            Some((s, n)) => (s, n),
            None => return Ok(0),
        };

        // Update seq of actual record
        let now = Utc::now();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        diesel::update(djmdSongHistory::table.find(id))
            .set((
                djmdSongHistory::track_no.eq(seq),
                djmdSongHistory::updated_at.eq(&format_datetime(&now)),
                djmdSongHistory::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdSongHistory {
    /// Queries all records from the `djmdSongHistory` table by its `history_id`
    pub fn by_history_id(conn: &mut SqliteConnection, history_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongHistory::history_id.eq(history_id))
            .load(conn)
    }

    /// Queries a record from the `djmdSongHistory` table by its `content_id`.
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSongHistory::content_id.eq(cid))
            .first(conn)
            .optional()
    }

    /// Creates a filter for records by `history_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_history_id(id: &str) -> _ {
        djmdSongHistory::table.filter(djmdSongHistory::history_id.eq(id))
    }

    /// Creates a filter for records by `history_ids`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_history_ids(ids: &[String]) -> _ {
        djmdSongHistory::table.filter(djmdSongHistory::history_id.eq_any(ids))
    }

    /// Checks if a record with the given `id` exists in the `djmdSongHistory` table.
    pub fn id_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }

    /// Deletes all records from the `djmdSongHistory` table acssociated with a `history_id`.
    pub fn delete_by_history_id(
        conn: &mut SqliteConnection,
        history_id: &str,
    ) -> QueryResult<usize> {
        let result = diesel::delete(Self::filter_by_history_id(history_id)).execute(conn)?;
        // AgentRegistry::increment_local_usn_by(conn, result)?;
        Ok(result)
    }

    /// Deletes all records from the `djmdSongHistory` table acssociated with one of `history_ids`.
    pub fn delete_by_history_ids(
        conn: &mut SqliteConnection,
        history_ids: &[String],
    ) -> QueryResult<usize> {
        let result = diesel::delete(Self::filter_by_history_ids(history_ids)).execute(conn)?;
        // AgentRegistry::increment_local_usn_by(conn, result)?;
        Ok(result)
    }

    /// Generates a new unique identifier for a record in the `djmdSongHistory` table.
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

impl TreeSeq for DjmdSongHistory {
    type ParentId = str;
    const START_SEQ: i32 = 1;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        DjmdHistory::is_history(conn, parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = Self::filter_by_history_id(parent_id)
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        Self::filter_by_history_id(parent_id)
            .select(djmdSongHistory::track_no)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY TrackNo) + (? - 1) AS new_seq
                FROM djmdSongHistory WHERE HistoryID =?
            ) UPDATE djmdSongHistory
            SET TrackNo = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSongHistory.ID)
            WHERE HistoryID =?;"#,
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
            djmdSongHistory::table
                .filter(djmdSongHistory::history_id.eq(parent_id))
                .filter(djmdSongHistory::track_no.ge(seq)),
        )
        .set((
            djmdSongHistory::track_no.eq(djmdSongHistory::track_no + 1),
            djmdSongHistory::updated_at.eq(format_datetime(&now)),
            djmdSongHistory::rb_local_usn.eq(usn),
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
            djmdSongHistory::table
                .filter(djmdSongHistory::history_id.eq(parent_id))
                .filter(djmdSongHistory::track_no.gt(seq)),
        )
        .set((
            djmdSongHistory::track_no.eq(djmdSongHistory::track_no - 1),
            djmdSongHistory::updated_at.eq(format_datetime(&now)),
            djmdSongHistory::rb_local_usn.eq(usn),
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
            djmdSongHistory::table
                .filter(djmdSongHistory::history_id.eq(parent_id))
                .filter(djmdSongHistory::track_no.ge(start))
                .filter(djmdSongHistory::track_no.lt(end)),
        )
        .set((
            djmdSongHistory::track_no.eq(djmdSongHistory::track_no + 1),
            djmdSongHistory::updated_at.eq(&format_datetime(&now)),
            djmdSongHistory::rb_local_usn.eq(usn),
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
            djmdSongHistory::table
                .filter(djmdSongHistory::history_id.eq(parent_id))
                .filter(djmdSongHistory::track_no.gt(start))
                .filter(djmdSongHistory::track_no.le(end)),
        )
        .set((
            djmdSongHistory::track_no.eq(djmdSongHistory::track_no - 1),
            djmdSongHistory::updated_at.eq(&format_datetime(&now)),
            djmdSongHistory::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `djmdSongHistory` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdSongHistory;
///
/// let new = NewSongDjmdPlaylist::new("historyId".into(), "contentId".into()).track_no(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdSongHistory {
    /// The ID of the associated history entry in [`DjmdHistory`].
    pub history_id: String,
    /// The ID of the associated content entry in [`DjmdContent`].
    pub content_id: String,
    /// The track number in the history entry.
    pub track_no: Option<i32>,
}

impl NewDjmdSongHistory {
    /// Creates a new `NewDjmdSongHistory` with the required `history_id` and `content_id` field.
    pub fn new(history_id: String, content_id: String) -> Self {
        Self {
            history_id,
            content_id,
            ..Default::default()
        }
    }

    /// Inserts the new record into the database
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<DjmdSongHistory> {
        // Check content
        if !DjmdContent::id_exists(conn, &self.content_id)? {
            return Err(Error::NotFound);
        }

        let parent_id = &self.history_id;
        // Handle seq and USN of moved records (also checkks parent)
        let (seq, n) = DjmdSongHistory::update_seq_before_insert(conn, &parent_id, self.track_no)?;
        if n > 0 {
            // Apply USN of moved records
            AgentRegistry::increment_local_usn(conn)?;
        }

        // Generate meta
        let id = DjmdSongHistory::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Get next USN: We increment by 2 (1 for creating, 1 for renaming from 'New History')
        let usn = AgentRegistry::increment_local_usn_by(conn, 2)?;
        let item = DjmdSongHistory {
            id,
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            history_id: self.history_id,
            content_id: self.content_id,
            track_no: seq,
            ..Default::default()
        };
        let result = diesel::insert_into(djmdSongHistory::table)
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
