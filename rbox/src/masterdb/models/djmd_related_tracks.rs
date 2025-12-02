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
use super::schema::{djmdContent, djmdRelatedTracks, djmdSongRelatedTracks};
use super::{format_datetime, Date, DateString, RandomIdGenerator};
use crate::model_traits::{
    Model, ModelDelete, ModelInsert, ModelList, ModelTree, ModelUpdate, TreeSeq,
};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdRelatedTracks` table in the Rekordbox database.
///
/// This struct maps to the `djmdRelatedTracks` table in the SQLite database used by Rekordbox.
/// It stores information about related tracks, including metadata such as sequence, name,
/// attributes, and parent relationships.
///
/// # Referenced by
/// * [`DjmdRelatedTracks`] via `parent_id` foreign key.
/// * [`DjmdSongRelatedTracks`] via `related_tracks_id` foreign key.
///
/// # References
/// * [`DjmdRelatedTracks`] via `parent_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdRelatedTracks)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdRelatedTracks, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdRelatedTracks {
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
    /// The name of the related track record.
    pub name: String,
    /// The attribute of the related track, either list (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The ID of the parent [`DjmdRelatedTracks`], `'root'` for top-level records.
    pub parent_id: String,
    /// A JSON string containing the criteria for the related tracks.
    pub criteria: String,
}

impl Model for DjmdRelatedTracks {
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

impl ModelUpdate for DjmdRelatedTracks {
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
        if self.criteria != existing.criteria {
            changes += 1;
        }
        if changes == 0 {
            return Ok(existing);
        }
        self.updated_at = Utc::now();
        self.rb_local_usn = Some(AgentRegistry::increment_local_usn_by(conn, changes)?);
        diesel::update(djmdRelatedTracks::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for DjmdRelatedTracks {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        // Vec of all deleted ids
        let mut deleted_ids = vec![id.to_string()];

        // Delete the record
        let parent_id: String = diesel::delete(djmdRelatedTracks::table.find(id))
            .returning(djmdRelatedTracks::parent_id)
            .get_result(conn)?;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove all child djmdRelatedTracks records recursively
        let mut parent_ids = VecDeque::from(vec![id.to_string()]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted: Vec<String> = diesel::delete(
                djmdRelatedTracks::table.filter(djmdRelatedTracks::parent_id.eq(parent_id)),
            )
            .returning(djmdRelatedTracks::id)
            .get_results(conn)?;
            deleted_ids.extend(deleted.clone());
            // Add children to the queue
            for deleted_id in deleted {
                parent_ids.push_back(deleted_id);
            }
        }

        DjmdSongRelatedTracks::delete_by_related_tracks_ids(conn, &deleted_ids)?;

        AgentRegistry::increment_local_usn_by(conn, deleted_ids.len())?;

        Ok(deleted_ids.len())
    }
}

impl ModelTree for DjmdRelatedTracks {
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
        diesel::update(djmdRelatedTracks::table.find(id))
            .set((
                djmdRelatedTracks::seq.eq(seq),
                djmdRelatedTracks::parent_id.eq(parent_id),
                djmdRelatedTracks::updated_at.eq(&format_datetime(&now)),
                djmdRelatedTracks::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdRelatedTracks {
    /// Queries a record from the `djmdRelatedTracks` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdRelatedTracks::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Queries the records from the `djmdRelatedTracks` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdRelatedTracks::parent_id.eq(parent_id))
            .order(djmdRelatedTracks::seq)
            .load(conn)
    }

    /// Queries a record from the `djmdRelatedTracks` table by its associated `content_id` via `djmdSongRelatedTracks`
    pub fn by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Vec<Self>> {
        djmdRelatedTracks::table
            .inner_join(
                djmdSongRelatedTracks::table
                    .on(djmdRelatedTracks::id.eq(djmdSongRelatedTracks::related_tracks_id)),
            )
            .filter(djmdSongRelatedTracks::content_id.eq(cid))
            .select(Self::as_select())
            .load(conn)
    }

    /// Queries the records from the `djmdContent` table associated with the given `djmdRelatedTracks`.
    pub fn get_contents(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<DjmdContent>> {
        djmdContent::table
            .inner_join(
                djmdSongRelatedTracks::table
                    .on(djmdContent::id.eq(djmdSongRelatedTracks::content_id)),
            )
            .filter(djmdSongRelatedTracks::related_tracks_id.eq(&id))
            .order(djmdSongRelatedTracks::track_no)
            .select(DjmdContent::as_select())
            .load(conn)
    }

    /// Returns the type (`attribute`) of a record in the `djmdRelatedTracks` table or `None` if not found.
    pub fn get_attribute(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<i32>> {
        if id == "root" {
            return Ok(Some(1));
        }
        djmdRelatedTracks::table
            .find(id)
            .select(djmdRelatedTracks::attribute)
            .get_result(conn)
            .optional()
    }

    /// Returns `true` if the record in the `djmdRelatedTracks` table exists and is a list, `false` otherwise.
    pub fn is_related_tracks(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 0), // 0: playlist
            None => Ok(false),
        }
    }

    /// Returns `true` if the record in the `djmdRelatedTracks` table exists and is a section, `false` otherwise.
    pub fn is_folder(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 1), // 1: folder
            None => Ok(false),
        }
    }

    /// Checks if a record with the given `name` exists in the `djmdRelatedTracks` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdRelatedTracks::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Set the name of a record in the `djmdRelatedTracks` table.
    pub fn rename(conn: &mut SqliteConnection, id: &str, name: &str) -> QueryResult<Self> {
        let datestr = format_datetime(&Utc::now());
        let usn = AgentRegistry::increment_local_usn(conn)?;
        Ok(diesel::update(djmdRelatedTracks::table.find(id))
            .set((
                djmdRelatedTracks::name.eq(name),
                djmdRelatedTracks::updated_at.eq(&datestr),
                djmdRelatedTracks::rb_local_usn.eq(usn),
            ))
            .get_result(conn)?)
    }

    /// Generates a new unique identifier for a record in the `djmdRelatedTracks` table.
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

impl TreeSeq for DjmdRelatedTracks {
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
    fn count_children(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<i32> {
        let count: i64 = djmdRelatedTracks::table
            .filter(djmdRelatedTracks::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        djmdRelatedTracks::table
            .filter(djmdRelatedTracks::parent_id.eq(parent_id))
            .order(djmdRelatedTracks::seq)
            .select(djmdRelatedTracks::seq)
            .load(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                    SELECT ID, ROW_NUMBER() OVER (ORDER BY Seq) + (? - 1)  AS new_seq
                    FROM djmdRelatedTracks WHERE ParentID =?
                ) UPDATE djmdRelatedTracks
                SET Seq = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdRelatedTracks.ID)
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
            djmdRelatedTracks::table
                .filter(djmdRelatedTracks::parent_id.eq(parent_id))
                .filter(djmdRelatedTracks::seq.ge(seq)),
        )
        .set((
            djmdRelatedTracks::seq.eq(djmdRelatedTracks::seq + 1),
            djmdRelatedTracks::updated_at.eq(&format_datetime(&now)),
            djmdRelatedTracks::rb_local_usn.eq(usn),
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
            djmdRelatedTracks::table
                .filter(djmdRelatedTracks::parent_id.eq(parent_id))
                .filter(djmdRelatedTracks::seq.gt(seq)),
        )
        .set((
            djmdRelatedTracks::seq.eq(djmdRelatedTracks::seq - 1),
            djmdRelatedTracks::updated_at.eq(&format_datetime(&now)),
            djmdRelatedTracks::rb_local_usn.eq(usn),
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
            djmdRelatedTracks::table
                .filter(djmdRelatedTracks::parent_id.eq(parent_id))
                .filter(djmdRelatedTracks::seq.ge(start))
                .filter(djmdRelatedTracks::seq.lt(end)),
        )
        .set((
            djmdRelatedTracks::seq.eq(djmdRelatedTracks::seq + 1),
            djmdRelatedTracks::updated_at.eq(&format_datetime(&now)),
            djmdRelatedTracks::rb_local_usn.eq(usn),
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
            djmdRelatedTracks::table
                .filter(djmdRelatedTracks::parent_id.eq(parent_id))
                .filter(djmdRelatedTracks::seq.gt(start))
                .filter(djmdRelatedTracks::seq.le(end)),
        )
        .set((
            djmdRelatedTracks::seq.eq(djmdRelatedTracks::seq - 1),
            djmdRelatedTracks::updated_at.eq(&format_datetime(&now)),
            djmdRelatedTracks::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `djmdRelatedTracks` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdRelatedTracks;
///
/// let new = NewDjmdRelatedTracks::new("Name", 0).seq(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdRelatedTracks {
    /// The name of the related track.
    pub name: String,
    /// The attribute of the related track, either list (`0`) or a folder (`1`)
    pub attribute: i32,
    /// The sequence/order number of the record (1-based index)
    pub seq: Option<i32>,
    /// The ID of the parent [`DjmdRelatedTracks`], `'root'` for top-level records.
    pub parent_id: Option<String>,
    /// A JSON string containing the criteria for the related tracks.
    pub criteria: Option<String>,
}

impl ModelInsert for NewDjmdRelatedTracks {
    type Model = DjmdRelatedTracks;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let parent_id = self.parent_id.unwrap_or("root".to_string());

        // Handle seq and USN of moved records (also checkks parent)
        let (seq, n) = DjmdRelatedTracks::update_seq_before_insert(conn, &parent_id, self.seq)?;
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
            criteria: self.criteria.unwrap_or_default(),
            ..Default::default()
        };
        diesel::insert_into(djmdRelatedTracks::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdRelatedTracks {
    /// Creates a new `NewDjmdRelatedTracks` with the required `name` and `attribute` field.
    pub fn new<S: Into<String>>(name: S, attribute: i32) -> Self {
        Self {
            name: name.into(),
            attribute,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdRelatedTracks` as a related tracks list.
    pub fn related_tracks<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: 0,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdRelatedTracks` as a folder.
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

    /// Builder for `criteria`.
    pub fn criteria<S: Into<String>>(mut self, criteria: S) -> Self {
        self.criteria = Some(criteria.into());
        self
    }
}

/// Represents the `djmdSongRelatedTracks` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongRelatedTracks` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and related tracks, including
/// metadata such as update sequence numbers, timestamps, and associated content or related tracks IDs.
///
/// # References
/// * [`DjmdRelatedTracks`] via `related_tracks_id` foreign key.
/// * [`DjmdContent`] via `content_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdSongRelatedTracks)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdSongRelatedTracks, foreign_key = related_tracks_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdSongRelatedTracks {
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

    /// The ID of the associated related tracks entry in [`DjmdRelatedTracks`].
    pub related_tracks_id: String,
    /// The ID of the associated content entry in [`DjmdContent`].
    pub content_id: String,
    /// The track number in the related tracks entry.
    pub track_no: i32,
}

impl Model for DjmdSongRelatedTracks {
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

impl ModelDelete for DjmdSongRelatedTracks {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let query = djmdSongRelatedTracks::table.find(id);
        let my_tag_id: String = query
            .select(djmdSongRelatedTracks::related_tracks_id)
            .first(conn)?;
        let result = diesel::delete(query).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Reorder the track_no numbers
        Self::reset_seq(conn, &my_tag_id)?;
        Ok(result)
    }
}

impl ModelList for DjmdSongRelatedTracks {
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize> {
        let res = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };

        let old_seq = res.track_no;
        // *Note*: Moving other records increments USN by 1 for all changes
        let res = Self::update_seq_before_move_in(conn, &res.related_tracks_id, old_seq, seq)?;
        let (seq, _n) = match res {
            Some((s, n)) => (s, n),
            None => return Ok(0),
        };

        // Update seq of actual record
        let now = Utc::now();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        diesel::update(djmdSongRelatedTracks::table.find(id))
            .set((
                djmdSongRelatedTracks::track_no.eq(seq),
                djmdSongRelatedTracks::updated_at.eq(&format_datetime(&now)),
                djmdSongRelatedTracks::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdSongRelatedTracks {
    /// Queries all records from the `djmdSongRelatedTracks` table by its `sampler_id`.
    pub fn by_related_tracks_id(
        conn: &mut SqliteConnection,
        related_tracks_id: &str,
    ) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongRelatedTracks::related_tracks_id.eq(related_tracks_id))
            .order(djmdSongRelatedTracks::track_no)
            .get_results(conn)
    }

    /// Queries a record from the `djmdSongRelatedTracks` table by its `content_id`.
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSongRelatedTracks::content_id.eq(cid))
            .first(conn)
            .optional()
    }

    /// Creates a filter for records by `related_tracks_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_related_id(id: &str) -> _ {
        djmdSongRelatedTracks::table.filter(djmdSongRelatedTracks::related_tracks_id.eq(id))
    }

    /// Creates a filter for records by `related_tracks_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_related_ids(ids: &[String]) -> _ {
        djmdSongRelatedTracks::table.filter(djmdSongRelatedTracks::related_tracks_id.eq_any(ids))
    }

    /// Counts the number of records in the `djmdSongRelatedTracks` table for a given `related_tracks_id`.
    pub fn count(conn: &mut SqliteConnection, related_tracks_id: &str) -> QueryResult<i32> {
        Self::count_children(conn, related_tracks_id)
    }

    /// Deletes a record from the `djmdSongRelatedTracks` table based on its `related_tracks_id`.
    pub fn delete_by_related_tracks_id(
        conn: &mut SqliteConnection,
        related_tracks_id: &str,
    ) -> QueryResult<usize> {
        diesel::delete(Self::filter_by_related_id(related_tracks_id)).execute(conn)
    }

    /// Deletes multiple records from the `djmdSongRelatedTracks` table based on their `related_tracks_id`.
    pub fn delete_by_related_tracks_ids(
        conn: &mut SqliteConnection,
        related_tracks_ids: &[String],
    ) -> QueryResult<usize> {
        diesel::delete(Self::filter_by_related_ids(related_tracks_ids)).execute(conn)
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

impl TreeSeq for DjmdSongRelatedTracks {
    type ParentId = str;
    const START_SEQ: i32 = 1;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        DjmdRelatedTracks::is_related_tracks(conn, parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = Self::filter_by_related_id(parent_id)
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        Self::filter_by_related_id(parent_id)
            .order(djmdSongRelatedTracks::track_no)
            .select(djmdSongRelatedTracks::track_no)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
                r#"WITH ordered AS (
                    SELECT ID, ROW_NUMBER() OVER (ORDER BY TrackNo) + (? - 1) AS new_seq
                    FROM djmdSongRelatedTracks WHERE RelatedTracksID =?
                ) UPDATE djmdSongRelatedTracks
                SET TrackNo = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSongRelatedTracks.ID)
                WHERE RelatedTracksID =?;"#,
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
            djmdSongRelatedTracks::table
                .filter(djmdSongRelatedTracks::related_tracks_id.eq(parent_id))
                .filter(djmdSongRelatedTracks::track_no.ge(seq)),
        )
        .set((
            djmdSongRelatedTracks::track_no.eq(djmdSongRelatedTracks::track_no + 1),
            djmdSongRelatedTracks::updated_at.eq(&format_datetime(&now)),
            djmdSongRelatedTracks::rb_local_usn.eq(usn),
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
            djmdSongRelatedTracks::table
                .filter(djmdSongRelatedTracks::related_tracks_id.eq(parent_id))
                .filter(djmdSongRelatedTracks::track_no.gt(seq)),
        )
        .set((
            djmdSongRelatedTracks::track_no.eq(djmdSongRelatedTracks::track_no - 1),
            djmdSongRelatedTracks::updated_at.eq(&format_datetime(&now)),
            djmdSongRelatedTracks::rb_local_usn.eq(usn),
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
            djmdSongRelatedTracks::table
                .filter(djmdSongRelatedTracks::related_tracks_id.eq(parent_id))
                .filter(djmdSongRelatedTracks::track_no.ge(start))
                .filter(djmdSongRelatedTracks::track_no.lt(end)),
        )
        .set((
            djmdSongRelatedTracks::track_no.eq(djmdSongRelatedTracks::track_no + 1),
            djmdSongRelatedTracks::updated_at.eq(&format_datetime(&now)),
            djmdSongRelatedTracks::rb_local_usn.eq(usn),
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
            djmdSongRelatedTracks::table
                .filter(djmdSongRelatedTracks::related_tracks_id.eq(parent_id))
                .filter(djmdSongRelatedTracks::track_no.gt(start))
                .filter(djmdSongRelatedTracks::track_no.le(end)),
        )
        .set((
            djmdSongRelatedTracks::track_no.eq(djmdSongRelatedTracks::track_no - 1),
            djmdSongRelatedTracks::updated_at.eq(&format_datetime(&now)),
            djmdSongRelatedTracks::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `djmdSongRelatedTracks` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdSongRelatedTracks;
///
/// let new = NewDjmdSongRelatedTracks::new("relatedId", "contentId").track_no(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdSongRelatedTracks {
    /// The ID of the associated playlist in [`DjmdPlaylist`].
    pub related_tracks_id: String,
    /// The ID of the associated content in [`DjmdContent`].
    pub content_id: String,
    /// The track number in the playlist.
    pub track_no: Option<i32>,
}

impl ModelInsert for NewDjmdSongRelatedTracks {
    type Model = DjmdSongRelatedTracks;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        // Check content
        if !DjmdContent::id_exists(conn, &self.content_id)? {
            return Err(Error::NotFound);
        }
        let parent_id = &self.related_tracks_id;
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
        // Get next USN: We increment by 2 (1 for creating, 1 for renaming from 'New Playlist')
        let usn = AgentRegistry::increment_local_usn_by(conn, 2)?;
        let item = Self::Model {
            id,
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            related_tracks_id: self.related_tracks_id,
            content_id: self.content_id,
            track_no: seq,
            ..Default::default()
        };
        diesel::insert_into(djmdSongRelatedTracks::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdSongRelatedTracks {
    /// Creates a new `NewDjmdSongRelatedTracks` with the required `related_tracks_id` and `content_id` field.
    pub fn new<S1: Into<String>, S2: Into<String>>(related_tracks_id: S1, content_id: S2) -> Self {
        Self {
            related_tracks_id: related_tracks_id.into(),
            content_id: content_id.into(),
            ..Default::default()
        }
    }

    /// Inserts the new record into the database
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<DjmdSongRelatedTracks> {
        // Check content
        if !DjmdContent::id_exists(conn, &self.content_id)? {
            return Err(Error::NotFound);
        }

        let parent_id = &self.related_tracks_id;
        // Handle seq and USN of moved records (also checkks parent)
        let (seq, n) =
            DjmdSongRelatedTracks::update_seq_before_insert(conn, &parent_id, self.track_no)?;
        if n > 0 {
            // Apply USN of moved records
            AgentRegistry::increment_local_usn(conn)?;
        }

        // Generate meta
        let id = DjmdSongRelatedTracks::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Get next USN: We increment by 2 (1 for creating, 1 for renaming from 'New Playlist')
        let usn = AgentRegistry::increment_local_usn_by(conn, 2)?;
        let item = DjmdSongRelatedTracks {
            id,
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            related_tracks_id: self.related_tracks_id,
            content_id: self.content_id,
            track_no: seq,
            ..Default::default()
        };
        let result = diesel::insert_into(djmdSongRelatedTracks::table)
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
