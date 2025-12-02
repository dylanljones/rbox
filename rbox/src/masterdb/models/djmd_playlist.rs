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
use std::collections::{HashSet, VecDeque};
use uuid::Uuid;

use super::agent_registry::AgentRegistry;
use super::djmd_content::DjmdContent;
use super::schema::{djmdContent, djmdPlaylist, djmdSongPlaylist};
use super::{format_datetime, Date, DateString, RandomIdGenerator};
use crate::model_traits::{
    build_tree_list, Model, ModelDelete, ModelInsert, ModelList, ModelTree, ModelUpdate, NodeRef,
    TreeNode, TreeSeq,
};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdPlaylist` table in the Rekordbox database.
///
/// This struct maps to the `djmdPlaylist` table in the SQLite database used by Rekordbox.
/// It stores information about playlists, including metadata such as sequence, name, image path,
/// attributes, and parent relationships.
///
/// **Note**: This model does not implement `ModelDelete`, but provides slightly different
/// `delete` and `delete_all` methods that return the deleted playlist Ids instead of the number
/// of affected rows. This is required for the playlist XML handling.
///
/// # Referenced by
/// * [`DjmdPlaylist`] via `parent_id` foreign key.
/// * [`DjmdSongPlaylist`] via `playlist_id` foreign key.
///
/// # References
/// * [`DjmdPlaylist`] via `parent_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdPlaylist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdPlaylist, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdPlaylist {
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
    /// The name of the playlist.
    pub name: String,
    /// An optional path to the image associated with the playlist.
    pub image_path: Option<String>,
    /// The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`).
    pub attribute: i32,
    /// The ID of the parent [`DjmdPlaylist`], `'root'` for top-level records.
    pub parent_id: String,
    /// An optional smart list configuration for the playlist (None if not a smart playlist)
    pub smart_list: Option<String>,
}

impl Model for DjmdPlaylist {
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

impl ModelUpdate for DjmdPlaylist {
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
        if self.image_path != existing.image_path {
            changes += 1;
        }
        if self.smart_list != existing.smart_list {
            changes += 1;
        }
        if changes == 0 {
            return Ok(existing);
        }
        self.updated_at = Utc::now();
        self.rb_local_usn = Some(AgentRegistry::increment_local_usn_by(conn, changes)?);
        diesel::update(djmdPlaylist::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for DjmdPlaylist {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        // Vec of all deleted playlist ids
        let mut deleted_ids = vec![id.to_string()];

        // Delete the record
        let parent_id: String = diesel::delete(djmdPlaylist::table.find(id))
            .returning(djmdPlaylist::parent_id)
            .get_result(conn)?;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove all child records recursively
        let mut parent_ids = VecDeque::from(vec![id.to_string()]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted: Vec<String> =
                diesel::delete(djmdPlaylist::table.filter(djmdPlaylist::parent_id.eq(parent_id)))
                    .returning(djmdPlaylist::id)
                    .get_results(conn)?;
            deleted_ids.extend(deleted.clone());
            // Add children to the queue
            for deleted_id in deleted {
                parent_ids.push_back(deleted_id);
            }
        }

        // Remove any djmdSongPlaylist records that are associated with the deleted djmdPlaylists
        DjmdSongPlaylist::delete_by_playlist_ids(conn, &deleted_ids)?;

        let n = deleted_ids.len();
        AgentRegistry::increment_local_usn_by(conn, n)?;

        Ok(n)
    }
}

impl ModelTree for DjmdPlaylist {
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
        diesel::update(djmdPlaylist::table.find(id))
            .set((
                djmdPlaylist::seq.eq(seq),
                djmdPlaylist::parent_id.eq(parent_id),
                djmdPlaylist::updated_at.eq(&format_datetime(&now)),
                djmdPlaylist::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdPlaylist {
    /// Queries records from the `djmdPlaylist` table by `ids`.
    pub fn find_all(conn: &mut SqliteConnection, ids: &[&str]) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdPlaylist::id.eq_any(ids))
            .load(conn)
    }

    /// Queries a record from the `djmdPlaylist` table by its `name`.
    pub fn by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdPlaylist::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Tries to query a record from the `djmdPlaylist` table by its node path in the tree.
    ///
    /// Returns `None` if no unique node can be found for the given `path`.
    pub fn find_by_path(conn: &mut SqliteConnection, path: Vec<&str>) -> QueryResult<Option<Self>> {
        let mut parent_ids: HashSet<String> = HashSet::new();
        parent_ids.insert("root".to_string());
        for name in path {
            let mut new_parent_ids: HashSet<String> = HashSet::new();
            for parent_id in parent_ids.iter() {
                let result: Option<String> = djmdPlaylist::table
                    .filter(djmdPlaylist::parent_id.eq(parent_id))
                    .filter(djmdPlaylist::name.eq(name))
                    .select(djmdPlaylist::id)
                    .first(conn)
                    .optional()?;
                if let Some(id) = result {
                    new_parent_ids.insert(id);
                }
            }
            if new_parent_ids.len() == 0 {
                return Ok(None);
            }
            parent_ids = new_parent_ids;
        }

        if parent_ids.len() == 1 {
            let id = parent_ids.iter().next().unwrap();
            Self::find(conn, &id)
        } else {
            Ok(None)
        }
    }

    /// Queries the records from the `djmdPlaylist` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdPlaylist::parent_id.eq(parent_id))
            .order(djmdPlaylist::seq)
            .load(conn)
    }

    /// Queries all records from the `djmdPlaylist` table by their associated `content_id` via `djmdSongPlaylist`
    pub fn by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Vec<Self>> {
        djmdPlaylist::table
            .inner_join(
                djmdSongPlaylist::table.on(djmdPlaylist::id.eq(djmdSongPlaylist::playlist_id)),
            )
            .filter(djmdSongPlaylist::content_id.eq(cid))
            .select(Self::as_select())
            .load(conn)
    }

    /// Queries the records from the `djmdContent` table associated with the given `djmdPlaylist`.
    pub fn get_contents(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<DjmdContent>> {
        djmdContent::table
            .inner_join(
                djmdSongPlaylist::table.on(djmdContent::id.eq(djmdSongPlaylist::content_id)),
            )
            .filter(djmdSongPlaylist::playlist_id.eq(&id))
            .order(djmdSongPlaylist::track_no)
            .select(DjmdContent::as_select())
            .load(conn)
    }

    /// Returns the playlist type (`attribute`) of a record in the `djmdPlaylist` table or `None` if not found.
    pub fn get_attribute(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<i32>> {
        if id == "root" {
            return Ok(Some(1));
        }
        djmdPlaylist::table
            .find(id)
            .select(djmdPlaylist::attribute)
            .get_result(conn)
            .optional()
    }

    /// Returns `true` if the record in the `djmdPlaylist` table exists and is a playlist, `false` otherwise.
    pub fn is_playlist(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 0), // 0: playlist
            None => Ok(false),
        }
    }

    /// Returns `true` if the record in the `djmdPlaylist` table exists and is a folder, `false` otherwise.
    pub fn is_folder(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 1), // 1: folder
            None => Ok(false),
        }
    }

    /// Set the name of a record in the `djmdPlaylist` table.
    pub fn rename(conn: &mut SqliteConnection, id: &str, name: &str) -> QueryResult<Self> {
        let datestr = format_datetime(&Utc::now());
        let usn = AgentRegistry::increment_local_usn(conn)?;
        diesel::update(djmdPlaylist::table.find(id))
            .set((
                djmdPlaylist::name.eq(name),
                djmdPlaylist::updated_at.eq(&datestr),
                djmdPlaylist::rb_local_usn.eq(usn),
            ))
            .get_result(conn)
    }

    /// Deletes a record from the database table.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `id`: A reference to the ID of the record to delete.
    ///
    /// # Returns
    /// A `QueryResult` containing the primary keys of the affected rows
    pub fn delete(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<String>> {
        // Vec of all deleted playlist ids
        let mut deleted_ids = vec![id.to_string()];

        // Delete the record
        let parent_id: String = diesel::delete(djmdPlaylist::table.find(id))
            .returning(djmdPlaylist::parent_id)
            .get_result(conn)?;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove all child records recursively
        let mut parent_ids = VecDeque::from(vec![id.to_string()]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted: Vec<String> =
                diesel::delete(djmdPlaylist::table.filter(djmdPlaylist::parent_id.eq(parent_id)))
                    .returning(djmdPlaylist::id)
                    .get_results(conn)?;
            deleted_ids.extend(deleted.clone());
            // Add children to the queue
            for deleted_id in deleted {
                parent_ids.push_back(deleted_id);
            }
        }

        // Remove any djmdSongPlaylist records that are associated with the deleted djmdPlaylists
        DjmdSongPlaylist::delete_by_playlist_ids(conn, &deleted_ids)?;

        AgentRegistry::increment_local_usn_by(conn, deleted_ids.len())?;

        Ok(deleted_ids)
    }

    /// Deletes multiple records from the database table.
    ///
    /// This method uses a naive loop to delete each record individually.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `ids`: A vector of references to the IDs of the records to delete.
    ///
    /// # Returns
    /// A `QueryResult` containing the primary keys of the affected rows
    pub fn delete_all(conn: &mut SqliteConnection, ids: Vec<&str>) -> QueryResult<Vec<String>> {
        let mut results = Vec::new();
        for id in ids {
            let deleted_ids = Self::delete(conn, id)?;
            results.extend(deleted_ids);
        }
        Ok(results)
    }

    /// Queries all records from the `djmdPlaylist` table and returns a tree structure.
    pub fn tree(conn: &mut SqliteConnection) -> QueryResult<Vec<NodeRef<DjmdPlaylistTreeNode>>> {
        let playlists = Self::all(conn)?;
        let roots = build_tree_list(playlists, "root".to_string());
        Ok(roots)
    }

    /// Generates a new unique identifier for a record in the `djmdPlaylist` table.
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

impl TreeSeq for DjmdPlaylist {
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
        let count: i64 = djmdPlaylist::table
            .filter(djmdPlaylist::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        djmdPlaylist::table
            .filter(djmdPlaylist::parent_id.eq(parent_id))
            .order(djmdPlaylist::seq)
            .select(djmdPlaylist::seq)
            .load(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY Seq) + (? - 1) AS new_seq
                FROM djmdPlaylist WHERE ParentID =?
            ) UPDATE djmdPlaylist
            SET Seq = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdPlaylist.ID)
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
            djmdPlaylist::table
                .filter(djmdPlaylist::parent_id.eq(parent_id))
                .filter(djmdPlaylist::seq.ge(seq)),
        )
        .set((
            djmdPlaylist::seq.eq(djmdPlaylist::seq + 1),
            djmdPlaylist::updated_at.eq(format_datetime(&now)),
            djmdPlaylist::rb_local_usn.eq(usn),
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
            djmdPlaylist::table
                .filter(djmdPlaylist::parent_id.eq(parent_id))
                .filter(djmdPlaylist::seq.gt(seq)),
        )
        .set((
            djmdPlaylist::seq.eq(djmdPlaylist::seq - 1),
            djmdPlaylist::updated_at.eq(format_datetime(&now)),
            djmdPlaylist::rb_local_usn.eq(usn),
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
            djmdPlaylist::table
                .filter(djmdPlaylist::parent_id.eq(parent_id))
                .filter(djmdPlaylist::seq.ge(start))
                .filter(djmdPlaylist::seq.lt(end)),
        )
        .set((
            djmdPlaylist::seq.eq(djmdPlaylist::seq + 1),
            djmdPlaylist::updated_at.eq(&format_datetime(&now)),
            djmdPlaylist::rb_local_usn.eq(usn),
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
            djmdPlaylist::table
                .filter(djmdPlaylist::parent_id.eq(parent_id))
                .filter(djmdPlaylist::seq.gt(start))
                .filter(djmdPlaylist::seq.le(end)),
        )
        .set((
            djmdPlaylist::seq.eq(djmdPlaylist::seq - 1),
            djmdPlaylist::updated_at.eq(&format_datetime(&now)),
            djmdPlaylist::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `djmdPlaylist` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdPlaylist;
///
/// let new = NewDjmdPlaylist::playlist("Name").seq(2);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdPlaylist {
    /// The name of the playlist.
    pub name: String,
    /// The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`).
    pub attribute: i32,
    /// The sequence/order number of the record (1-based index)
    pub seq: Option<i32>,
    /// The ID of the parent [`DjmdPlaylist`], `'root'` for top-level records (default).
    pub parent_id: Option<String>,
    /// An optional path to the image associated with the playlist.
    pub image_path: Option<String>,
    /// An optional smart list configuration for the playlist (None if not a smart playlist)
    pub smart_list: Option<String>,
}

impl ModelInsert for NewDjmdPlaylist {
    type Model = DjmdPlaylist;

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
            image_path: self.image_path,
            smart_list: self.smart_list,
            ..Default::default()
        };
        diesel::insert_into(djmdPlaylist::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdPlaylist {
    /// Creates a new `NewDjmdPlaylist` with the required `name` and `attribute` field.
    pub fn new<S: Into<String>>(name: S, attribute: i32) -> Self {
        Self {
            name: name.into(),
            attribute,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdPlaylist` as a playlist.
    pub fn playlist<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: 0,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdPlaylist` as a folder.
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

    /// Builder for `image_path`.
    pub fn image_path<S: Into<String>>(mut self, image_path: S) -> Self {
        self.image_path = Some(image_path.into());
        self
    }

    /// Builder for `smart_list`.
    pub fn smart_list<S: Into<String>>(mut self, smart_list: S) -> Self {
        self.smart_list = Some(smart_list.into());
        self
    }
}

/// Represents the `djmdSongPlaylist` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongPlaylist` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and playlists, including
/// metadata such as track number and associated playlist or content IDs.
///
/// # References
/// * [`DjmdPlaylist`] via `playlist_id` foreign key.
/// * [`DjmdContent`] via `content_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdSongPlaylist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdPlaylist, foreign_key = playlist_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdSongPlaylist {
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

    /// The ID of the associated playlist in [`DjmdPlaylist`].
    pub playlist_id: String,
    /// The ID of the associated content in [`DjmdContent`].
    pub content_id: String,
    /// The track number in the playlist.
    pub track_no: i32,
}

impl Model for DjmdSongPlaylist {
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

impl ModelDelete for DjmdSongPlaylist {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let query = djmdSongPlaylist::table.find(id);
        let playlist_id: String = query.select(djmdSongPlaylist::playlist_id).first(conn)?;
        let result = diesel::delete(query).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Reorder the track_no numbers
        Self::reset_seq(conn, &playlist_id)?;
        Ok(result)
    }
}

impl ModelList for DjmdSongPlaylist {
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize> {
        let res = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };

        let old_seq = res.track_no;
        // *Note*: Moving other records increments USN by 1 for all changes
        let res = Self::update_seq_before_move_in(conn, &res.playlist_id, old_seq, seq)?;
        let (seq, _n) = match res {
            Some((s, n)) => (s, n),
            None => return Ok(0),
        };

        // Update seq of actual record
        let now = Utc::now();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        diesel::update(djmdSongPlaylist::table.find(id))
            .set((
                djmdSongPlaylist::track_no.eq(seq),
                djmdSongPlaylist::updated_at.eq(&format_datetime(&now)),
                djmdSongPlaylist::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }
}

impl DjmdSongPlaylist {
    /// Queries records from the `djmdPlaylist` table by `ids`.
    pub fn find_all(conn: &mut SqliteConnection, ids: &[&str]) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongPlaylist::id.eq_any(ids))
            .load(conn)
    }

    /// Queries all records from the `djmdSongPlaylist` table by their `playlist_id`.
    pub fn by_playlist_id(
        conn: &mut SqliteConnection,
        playlist_id: &str,
    ) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(djmdSongPlaylist::playlist_id.eq(playlist_id))
            .order(djmdSongPlaylist::track_no)
            .get_results(conn)
    }

    /// Queries a record from the `djmdSongPlaylist` table by its `content_id`.
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdSongPlaylist::content_id.eq(cid))
            .first(conn)
            .optional()
    }

    /// Creates a filter for records by `playlist_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_playlist_id(id: &str) -> _ {
        djmdSongPlaylist::table.filter(djmdSongPlaylist::playlist_id.eq(id))
    }

    /// Creates a filter for records by `playlist_ids`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_playlist_ids(ids: &[String]) -> _ {
        djmdSongPlaylist::table.filter(djmdSongPlaylist::playlist_id.eq_any(ids))
    }

    /// Counts the number of records in the `djmdSongPlaylist` table for a given `playlist_id`.
    pub fn count(conn: &mut SqliteConnection, playlist_id: &str) -> QueryResult<i32> {
        Self::count_children(conn, playlist_id)
    }

    /// Deletes all records from the `djmdSongPlaylist` table acssociated with a `playlist_id`.
    pub fn delete_by_playlist_id(
        conn: &mut SqliteConnection,
        playlist_id: &str,
    ) -> QueryResult<usize> {
        diesel::delete(Self::filter_by_playlist_id(playlist_id)).execute(conn)
    }

    /// Deletes all records from the `djmdSongPlaylist` table acssociated with one of `playlist_ids`.
    pub fn delete_by_playlist_ids(
        conn: &mut SqliteConnection,
        playlist_ids: &[String],
    ) -> QueryResult<usize> {
        diesel::delete(Self::filter_by_playlist_ids(playlist_ids)).execute(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdSongPlaylist` table.
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

impl TreeSeq for DjmdSongPlaylist {
    type ParentId = str;
    const START_SEQ: i32 = 1;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        DjmdPlaylist::is_playlist(conn, parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = Self::filter_by_playlist_id(parent_id)
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        Self::filter_by_playlist_id(parent_id)
            .order(djmdSongPlaylist::track_no)
            .select(djmdSongPlaylist::track_no)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY TrackNo) + (? - 1) AS new_seq
                FROM djmdSongPlaylist WHERE PlaylistID =?
            ) UPDATE djmdSongPlaylist
            SET TrackNo = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSongPlaylist.ID)
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
            djmdSongPlaylist::table
                .filter(djmdSongPlaylist::playlist_id.eq(parent_id))
                .filter(djmdSongPlaylist::track_no.ge(seq)),
        )
        .set((
            djmdSongPlaylist::track_no.eq(djmdSongPlaylist::track_no + 1),
            djmdSongPlaylist::updated_at.eq(format_datetime(&now)),
            djmdSongPlaylist::rb_local_usn.eq(usn),
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
            djmdSongPlaylist::table
                .filter(djmdSongPlaylist::playlist_id.eq(parent_id))
                .filter(djmdSongPlaylist::track_no.gt(seq)),
        )
        .set((
            djmdSongPlaylist::track_no.eq(djmdSongPlaylist::track_no - 1),
            djmdSongPlaylist::updated_at.eq(format_datetime(&now)),
            djmdSongPlaylist::rb_local_usn.eq(usn),
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
            djmdSongPlaylist::table
                .filter(djmdSongPlaylist::playlist_id.eq(parent_id))
                .filter(djmdSongPlaylist::track_no.ge(start))
                .filter(djmdSongPlaylist::track_no.lt(end)),
        )
        .set((
            djmdSongPlaylist::track_no.eq(djmdSongPlaylist::track_no + 1),
            djmdSongPlaylist::updated_at.eq(&format_datetime(&now)),
            djmdSongPlaylist::rb_local_usn.eq(usn),
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
            djmdSongPlaylist::table
                .filter(djmdSongPlaylist::playlist_id.eq(parent_id))
                .filter(djmdSongPlaylist::track_no.gt(start))
                .filter(djmdSongPlaylist::track_no.le(end)),
        )
        .set((
            djmdSongPlaylist::track_no.eq(djmdSongPlaylist::track_no - 1),
            djmdSongPlaylist::updated_at.eq(&format_datetime(&now)),
            djmdSongPlaylist::rb_local_usn.eq(usn),
        ))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `djmdSongPlaylist` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdSongPlaylist;
///
/// let new = NewSongDjmdPlaylist::new("playlistId", "contentId").track_no(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdSongPlaylist {
    /// The ID of the associated playlist in [`DjmdPlaylist`].
    pub playlist_id: String,
    /// The ID of the associated content in [`DjmdContent`].
    pub content_id: String,
    /// The track number in the playlist.
    pub track_no: Option<i32>,
}

impl ModelInsert for NewDjmdSongPlaylist {
    type Model = DjmdSongPlaylist;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        // Check content
        if !DjmdContent::id_exists(conn, &self.content_id)? {
            return Err(Error::NotFound);
        }

        let parent_id = &self.playlist_id;
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
            playlist_id: self.playlist_id,
            content_id: self.content_id,
            track_no: seq,
            ..Default::default()
        };
        diesel::insert_into(djmdSongPlaylist::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdSongPlaylist {
    /// Creates a new `NewDjmdSongPlaylist` with the required `playlist_id` and `content_id` field.
    pub fn new<S1: Into<String>, S2: Into<String>>(playlist_id: S1, content_id: S2) -> Self {
        Self {
            playlist_id: playlist_id.into(),
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

/// Represents the `djmdPlaylist` table in the Rekordbox database in the form of a tree.
///
/// See [`DjmdPlaylist`] for the flat representation.
#[derive(Debug, Clone, PartialEq)]
pub struct DjmdPlaylistTreeNode {
    pub id: String,
    pub uuid: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Date,
    pub updated_at: Date,

    pub seq: i32,
    pub name: String,
    pub attribute: i32,
    pub parent_id: String,
    pub image_path: Option<String>,
    pub smart_list: Option<String>,
    pub children: Vec<NodeRef<Self>>,
}

impl TreeNode for DjmdPlaylistTreeNode {
    type Model = DjmdPlaylist;
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn sequence(&self) -> i32 {
        self.seq
    }

    fn parent_id(&self) -> Self::Id {
        self.parent_id.clone()
    }

    fn children(&self) -> &Vec<NodeRef<Self>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<NodeRef<Self>> {
        &mut self.children
    }
}

impl From<DjmdPlaylist> for DjmdPlaylistTreeNode {
    fn from(playlist: DjmdPlaylist) -> Self {
        Self {
            id: playlist.id,
            uuid: playlist.uuid,
            rb_data_status: playlist.rb_data_status,
            rb_local_data_status: playlist.rb_local_data_status,
            rb_local_deleted: playlist.rb_local_deleted,
            rb_local_synced: playlist.rb_local_synced,
            usn: playlist.usn,
            rb_local_usn: playlist.rb_local_usn,
            created_at: playlist.created_at,
            updated_at: playlist.updated_at,
            seq: playlist.seq,
            name: playlist.name,
            attribute: playlist.attribute,
            parent_id: playlist.parent_id,
            image_path: playlist.image_path,
            smart_list: playlist.smart_list,
            children: Vec::new(),
        }
    }
}
