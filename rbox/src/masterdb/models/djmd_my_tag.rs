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
use super::schema::{djmdContent, djmdMyTag, djmdSongMyTag};
use super::{format_datetime, Date, DateString, RandomIdGenerator};
use crate::model_traits::{
    Model, ModelDelete, ModelInsert, ModelList, ModelTree, ModelUpdate, TreeSeq,
};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdMyTag` table in the Rekordbox database.
///
/// This struct maps to the `djmdMyTag` table in the SQLite database used by Rekordbox.
/// It stores information about custom tags (MyTags) that can be associated with tracks,
/// including metadata such as sequence, name, attributes, and parent relationships.
///
/// # Referenced by
/// * [`DjmdMyTag`] via `parent_id` foreign key.
/// * [`DjmdSongMyTag`] via `my_tag_id` foreign key.
///
/// # References
/// * [`DjmdMyTag`] via `parent_id` foreign key.
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

impl Model for DjmdMyTag {
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

impl ModelUpdate for DjmdMyTag {
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
        diesel::update(djmdMyTag::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for DjmdMyTag {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        // Vec of all deleted ids
        let mut deleted_ids = vec![id.to_string()];

        // Delete the record
        let parent_id: String = diesel::delete(djmdMyTag::table.find(id))
            .returning(djmdMyTag::parent_id)
            .get_result(conn)?;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove all child djmdMyTag records recursively
        let mut parent_ids = VecDeque::from(vec![id.to_string()]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted: Vec<String> =
                diesel::delete(djmdMyTag::table.filter(djmdMyTag::parent_id.eq(parent_id)))
                    .returning(djmdMyTag::id)
                    .get_results(conn)?;
            deleted_ids.extend(deleted.clone());
            // Add children to the queue
            for deleted_id in deleted {
                parent_ids.push_back(deleted_id);
            }
        }
        DjmdSongMyTag::delete_by_tag_ids(conn, &deleted_ids)?;

        AgentRegistry::increment_local_usn_by(conn, deleted_ids.len())?;

        Ok(deleted_ids.len())
    }
}

impl ModelTree for DjmdMyTag {
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
        diesel::update(djmdMyTag::table.find(id))
            .set((
                djmdMyTag::seq.eq(seq),
                djmdMyTag::parent_id.eq(parent_id),
                djmdMyTag::updated_at.eq(&format_datetime(&now)),
                djmdMyTag::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdMyTag {
    /// Queries a record from the `djmdMyTag` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdMyTag::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Queries the records from the `djmdMyTag` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdMyTag::parent_id.eq(parent_id))
            .order(djmdMyTag::seq)
            .load(conn)
    }

    /// Queries all records from the `djmdMyTag` table their associated `content_id` via `djmdSongMyTag`
    pub fn by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Vec<Self>> {
        djmdMyTag::table
            .inner_join(djmdSongMyTag::table.on(djmdMyTag::id.eq(djmdSongMyTag::my_tag_id)))
            .filter(djmdSongMyTag::content_id.eq(cid))
            .select(Self::as_select())
            .load(conn)
    }

    /// Queries the records from the `djmdContent` table associated with the given `my_tag_id`.
    pub fn get_contents(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<DjmdContent>> {
        djmdContent::table
            .inner_join(djmdSongMyTag::table.on(djmdContent::id.eq(djmdSongMyTag::content_id)))
            .filter(djmdSongMyTag::my_tag_id.eq(&id))
            .order(djmdSongMyTag::track_no)
            .select(DjmdContent::as_select())
            .load(conn)
    }

    /// Returns the playlist type (`attribute`) of a record in the `djmdMyTag` table or `None` if not found.
    pub fn get_attribute(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<i32>> {
        if id == "root" {
            return Ok(Some(1));
        }
        djmdMyTag::table
            .find(id)
            .select(djmdMyTag::attribute)
            .get_result(conn)
            .optional()
    }

    /// Returns `true` if the record in the `djmdMyTag` table exists and is a my-tag, `false` otherwise.
    pub fn is_my_tag(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 0), // 0: playlist
            None => Ok(false),
        }
    }

    /// Returns `true` if the record in the `djmdMyTag` table exists and is a section, `false` otherwise.
    pub fn is_section(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 1), // 1: folder
            None => Ok(false),
        }
    }

    /// Checks if a record with the given `name` exists in the `djmdMyTag` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdMyTag::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Set the name of a record in the `djmdMyTag` table.
    pub fn rename(conn: &mut SqliteConnection, id: &str, name: &str) -> QueryResult<Self> {
        let datestr = format_datetime(&Utc::now());
        let usn = AgentRegistry::increment_local_usn(conn)?;
        Ok(diesel::update(djmdMyTag::table.find(id))
            .set((
                djmdMyTag::name.eq(name),
                djmdMyTag::updated_at.eq(&datestr),
                djmdMyTag::rb_local_usn.eq(usn),
            ))
            .get_result(conn)?)
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

impl TreeSeq for DjmdMyTag {
    type ParentId = str;
    const START_SEQ: i32 = 1;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        Self::is_section(conn, parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = djmdMyTag::table
            .filter(djmdMyTag::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        djmdMyTag::table
            .filter(djmdMyTag::parent_id.eq(parent_id))
            .order(djmdMyTag::seq)
            .select(djmdMyTag::seq)
            .load(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                    SELECT ID, ROW_NUMBER() OVER (ORDER BY Seq) + (? - 1) AS new_seq
                    FROM djmdMyTag WHERE ParentID =?
                ) UPDATE djmdMyTag
                SET Seq = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdMyTag.ID)
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
            djmdMyTag::table
                .filter(djmdMyTag::parent_id.eq(parent_id))
                .filter(djmdMyTag::seq.ge(seq)),
        )
        .set((
            djmdMyTag::seq.eq(djmdMyTag::seq + 1),
            djmdMyTag::updated_at.eq(&format_datetime(&now)),
            djmdMyTag::rb_local_usn.eq(usn),
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
            djmdMyTag::table
                .filter(djmdMyTag::parent_id.eq(parent_id))
                .filter(djmdMyTag::seq.gt(seq)),
        )
        .set((
            djmdMyTag::seq.eq(djmdMyTag::seq - 1),
            djmdMyTag::updated_at.eq(&format_datetime(&now)),
            djmdMyTag::rb_local_usn.eq(usn),
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
            djmdMyTag::table
                .filter(djmdMyTag::parent_id.eq(parent_id))
                .filter(djmdMyTag::seq.ge(start))
                .filter(djmdMyTag::seq.lt(end)),
        )
        .set((
            djmdMyTag::seq.eq(djmdMyTag::seq + 1),
            djmdMyTag::updated_at.eq(&format_datetime(&now)),
            djmdMyTag::rb_local_usn.eq(usn),
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
            djmdMyTag::table
                .filter(djmdMyTag::parent_id.eq(parent_id))
                .filter(djmdMyTag::seq.gt(start))
                .filter(djmdMyTag::seq.le(end)),
        )
        .set((
            djmdMyTag::seq.eq(djmdMyTag::seq - 1),
            djmdMyTag::updated_at.eq(&format_datetime(&now)),
            djmdMyTag::rb_local_usn.eq(usn),
        ))
        .execute(conn)
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
/// let new = NewDjmdMyTag::new("Name", 0).seq(1);
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

impl ModelInsert for NewDjmdMyTag {
    type Model = DjmdMyTag;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let id = Self::Model::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        let parent_id = self.parent_id.unwrap_or("root".to_string());
        let count = DjmdMyTag::count_children(conn, &parent_id)?;
        let item = Self::Model {
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
        diesel::insert_into(djmdMyTag::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdMyTag {
    /// Creates a new `NewDjmdMyTag` with the required `name` field.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
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
    pub fn parent_id<S: Into<String>>(mut self, parent_id: S) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }
}

/// Represents the `djmdSongMyTag` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongMyTag` table in the SQLite database used by Rekordbox.
/// It links the `djmdMyTag` records to the associated `djmdContent` records.
///
/// # References
/// * [`DjmdMyTag`] via `my_tag_id` foreign key.
/// * [`DjmdContent`] via `content_id` foreign key.
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

impl Model for DjmdSongMyTag {
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

impl ModelDelete for DjmdSongMyTag {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let query = djmdSongMyTag::table.find(id);
        let my_tag_id: String = query.select(djmdSongMyTag::my_tag_id).first(conn)?;
        let result = diesel::delete(query).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Reorder the track_no numbers
        Self::reset_seq(conn, &my_tag_id)?;
        Ok(result)
    }
}

impl ModelList for DjmdSongMyTag {
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize> {
        let res = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };

        let old_seq = res.track_no;
        // *Note*: Moving other records increments USN by 1 for all changes
        let res = Self::update_seq_before_move_in(conn, &res.my_tag_id, old_seq, seq)?;
        let (seq, _n) = match res {
            Some((s, n)) => (s, n),
            None => return Ok(0),
        };

        // Update seq of actual record
        let now = Utc::now();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        diesel::update(djmdSongMyTag::table.find(id))
            .set((
                djmdSongMyTag::track_no.eq(seq),
                djmdSongMyTag::updated_at.eq(&format_datetime(&now)),
                djmdSongMyTag::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdSongMyTag {
    /// Queries all records from the `djmdSongMyTag` table by its `my_tag_id`.
    pub fn by_tag_id(conn: &mut SqliteConnection, tag_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongMyTag::my_tag_id.eq(tag_id))
            .order(djmdSongMyTag::track_no)
            .get_results(conn)
    }

    /// Queries a record from the `djmdSongMyTag` table by its `content_id`.
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSongMyTag::content_id.eq(cid))
            .first(conn)
            .optional()
    }

    /// Creates a filter for records by `my_tag_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_tag_id(id: &str) -> _ {
        djmdSongMyTag::table.filter(djmdSongMyTag::my_tag_id.eq(id))
    }

    /// Creates a filter for records by `my_tag_ids`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_tag_ids(ids: &[String]) -> _ {
        djmdSongMyTag::table.filter(djmdSongMyTag::my_tag_id.eq_any(ids))
    }

    /// Counts the number of records in the `djmdSongMyTag` table for a given `my_tag_id`.
    pub fn count(conn: &mut SqliteConnection, my_tag_id: &str) -> QueryResult<i32> {
        Self::count_children(conn, my_tag_id)
    }

    /// Deletes a record from the `djmdSongMyTag` table based on its `my_tag_id`.
    pub fn delete_by_tag_id(conn: &mut SqliteConnection, my_tag_id: &str) -> QueryResult<usize> {
        diesel::delete(Self::filter_by_tag_id(my_tag_id)).execute(conn)
    }

    /// Deletes multiple records from the `djmdSongMyTag` table based on their `my_tag_id`.
    pub fn delete_by_tag_ids(
        conn: &mut SqliteConnection,
        my_tag_ids: &[String],
    ) -> QueryResult<usize> {
        diesel::delete(Self::filter_by_tag_ids(my_tag_ids)).execute(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdSongMyTag` table.
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

impl TreeSeq for DjmdSongMyTag {
    type ParentId = str;
    const START_SEQ: i32 = 1;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        DjmdMyTag::is_my_tag(conn, parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = Self::filter_by_tag_id(parent_id).count().get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        Self::filter_by_tag_id(parent_id)
            .order(djmdSongMyTag::track_no)
            .select(djmdSongMyTag::track_no)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                    SELECT ID, ROW_NUMBER() OVER (ORDER BY Seq) + (? - 1) AS new_seq
                    FROM djmdSongMyTag WHERE MyTagID =?
                ) UPDATE djmdSongMyTag
                SET Seq = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSongMyTag.ID)
                WHERE MyTagID =?;"#,
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
            djmdSongMyTag::table
                .filter(djmdSongMyTag::my_tag_id.eq(parent_id))
                .filter(djmdSongMyTag::track_no.ge(seq)),
        )
        .set((
            djmdSongMyTag::track_no.eq(djmdSongMyTag::track_no + 1),
            djmdSongMyTag::updated_at.eq(&format_datetime(&now)),
            djmdSongMyTag::rb_local_usn.eq(usn),
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
            djmdSongMyTag::table
                .filter(djmdSongMyTag::my_tag_id.eq(parent_id))
                .filter(djmdSongMyTag::track_no.gt(seq)),
        )
        .set((
            djmdSongMyTag::track_no.eq(djmdSongMyTag::track_no - 1),
            djmdSongMyTag::updated_at.eq(&format_datetime(&now)),
            djmdSongMyTag::rb_local_usn.eq(usn),
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
            djmdSongMyTag::table
                .filter(djmdSongMyTag::my_tag_id.eq(parent_id))
                .filter(djmdSongMyTag::track_no.ge(start))
                .filter(djmdSongMyTag::track_no.lt(end)),
        )
        .set((
            djmdSongMyTag::track_no.eq(djmdSongMyTag::track_no + 1),
            djmdSongMyTag::updated_at.eq(&format_datetime(&now)),
            djmdSongMyTag::rb_local_usn.eq(usn),
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
            djmdSongMyTag::table
                .filter(djmdSongMyTag::my_tag_id.eq(parent_id))
                .filter(djmdSongMyTag::track_no.gt(start))
                .filter(djmdSongMyTag::track_no.le(end)),
        )
        .set((
            djmdSongMyTag::track_no.eq(djmdSongMyTag::track_no - 1),
            djmdSongMyTag::updated_at.eq(&format_datetime(&now)),
            djmdSongMyTag::rb_local_usn.eq(usn),
        ))
        .execute(conn)
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
/// let new = NewDjmdMyTag::new("my_tag_id", "content_id").track_no(1);
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

impl ModelInsert for NewDjmdSongMyTag {
    type Model = DjmdSongMyTag;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        // Check content
        if !DjmdContent::id_exists(conn, &self.content_id)? {
            return Err(Error::NotFound);
        }

        let parent_id = &self.my_tag_id;
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

        // Get next USN: We increment by 2 (1 for creating, 1 for renaming from 'New Tag')
        let usn = AgentRegistry::increment_local_usn_by(conn, 2)?;
        let item = Self::Model {
            id,
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            my_tag_id: self.my_tag_id,
            content_id: self.content_id,
            track_no: seq,
            ..Default::default()
        };
        diesel::insert_into(djmdSongMyTag::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdSongMyTag {
    pub fn new<S1: Into<String>, S2: Into<String>>(my_tag_id: S1, content_id: S2) -> Self {
        Self {
            my_tag_id: my_tag_id.into(),
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
