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
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

use super::content::Content;
use super::schema::{content, playlist, playlist_content};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelList, ModelTree, TreeSeq};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

pub type PlaylistTreeNodeRef = Rc<RefCell<PlaylistTreeNode>>;

/// Represents the `playlist` table in the Rekordbox One Library database.
///
/// This struct maps to the `playlist` table in the One Library export database.
/// It stores information about playlists, including metadata such as sequence, name, image path,
/// attributes, and parent relationships.
///
/// # Referenced by
/// * [`Playlist`] via `parent_id` foreign key.
/// * [`PlaylistContent`] via `playlist_id` foreign key.
///
/// # References
/// * [`Playlist`] via `parent_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = playlist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(Playlist, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Playlist {
    /// The unique identifier of the playlist.
    pub id: i32,
    /// The sequence/order number of the playlist (1-based index).
    pub seq: i32,
    /// The name of the playlist.
    pub name: String,
    /// Optional foreign key referencing the artwork [`Image`] (`0` or `None` if no image).
    pub image_id: Option<i32>,
    /// The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`).
    pub attribute: i32,
    /// The ID of the parent [`Playlist`], `0` for top-level records.
    pub parent_id: i32,
}

impl Model for Playlist {
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

impl ModelDelete for Playlist {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        // Vec of all deleted playlist ids
        let mut deleted_ids = vec![*id];

        // Delete the record
        let parent_id: i32 = diesel::delete(playlist::table.find(id))
            .returning(playlist::parent_id)
            .get_result(conn)?;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove all child playlist records recursively
        let mut parent_ids = VecDeque::from(vec![*id]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted: Vec<i32> =
                diesel::delete(playlist::table.filter(playlist::parent_id.eq(parent_id)))
                    .returning(playlist::id)
                    .get_results(conn)?;
            deleted_ids.extend(deleted.clone());
            // Add children to the queue
            for deleted_id in deleted {
                parent_ids.push_back(deleted_id);
            }
        }

        // Remove any playlist_content records that are associated with the deleted playlists
        PlaylistContent::delete_by_playlist_ids(conn, &deleted_ids)?;

        Ok(deleted_ids.len())
    }
}

impl ModelTree for Playlist {
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
        diesel::update(playlist::table.find(id))
            .set((playlist::seq.eq(seq), playlist::parent_id.eq(parent_id)))
            .execute(conn)
    }
}

impl Playlist {
    /// Queries all records from the `playlist` table and returns a tree structure.
    pub fn tree(conn: &mut SqliteConnection) -> QueryResult<Vec<PlaylistTreeNodeRef>> {
        let playlists: Vec<Self> = Self::query().load(conn)?;
        Ok(build_tree(&playlists))
    }

    /// Queries the records from the `playlist` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(playlist::parent_id.eq(parent_id))
            .order(playlist::seq)
            .load(conn)
    }

    /// Queries the records from the `content` table associated with the given `playlist`.
    pub fn get_contents(conn: &mut SqliteConnection, id: i32) -> QueryResult<Vec<Content>> {
        content::table
            .inner_join(playlist_content::table.on(content::id.eq(playlist_content::content_id)))
            .filter(playlist_content::playlist_id.eq(id))
            .order(playlist_content::seq)
            .select(Content::as_select())
            .load(conn)
    }

    /// Tries to query a record from the `playlist` table by its node path in the tree.
    ///
    /// Returns `None` if no unique node can be found for the given `path`.
    pub fn find_by_path(conn: &mut SqliteConnection, path: Vec<&str>) -> QueryResult<Option<Self>> {
        let mut parent_ids: HashSet<i32> = HashSet::new();
        parent_ids.insert(0);
        for name in path {
            let mut new_parent_ids: HashSet<i32> = HashSet::new();
            for parent_id in parent_ids.iter() {
                let result: Option<i32> = playlist::table
                    .filter(playlist::parent_id.eq(parent_id))
                    .filter(playlist::name.eq(name))
                    .select(playlist::id)
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
            Self::find(conn, id)
        } else {
            Ok(None)
        }
    }

    /// Returns the playlist type (`attribute`) of a record in the `playlist` table or `None` if not found.
    pub fn get_attribute(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<i32>> {
        if id == 0 {
            return Ok(Some(1));
        }
        playlist::table
            .find(id)
            .select(playlist::attribute)
            .get_result(conn)
            .optional()
    }

    /// Returns `true` if the record in the `playlist` table exists and is a playlist, `false` otherwise.
    pub fn is_playlist(conn: &mut SqliteConnection, id: i32) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 0), // 0: playlist
            None => Ok(false),
        }
    }

    /// Returns `true` if the record in the `playlist` table exists and is a folder, `false` otherwise.
    pub fn is_folder(conn: &mut SqliteConnection, id: i32) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == 1), // 1: folder
            None => Ok(false),
        }
    }

    /// Set the name of a record in the `playlist` table.
    pub fn rename(conn: &mut SqliteConnection, id: i32, name: &str) -> QueryResult<usize> {
        diesel::update(playlist::table.find(id))
            .set(playlist::name.eq(name))
            .execute(conn)
    }
}

impl TreeSeq for Playlist {
    type ParentId = i32;
    const START_SEQ: i32 = 0;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        Self::is_folder(conn, *parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = playlist::table
            .filter(playlist::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        playlist::table
            .filter(playlist::parent_id.eq(parent_id))
            .order(playlist::seq)
            .select(playlist::seq)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT playlist_id, ROW_NUMBER() OVER (ORDER BY sequenceNo) + (? - 1) AS new_seq
                FROM playlist WHERE playlist_id_parent =?
            ) UPDATE playlist
            SET sequenceNo = (SELECT new_seq FROM ordered WHERE ordered.playlist_id = playlist.playlist_id)
            WHERE playlist_id_parent = ?;"#,
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
            playlist::table
                .filter(playlist::seq.ge(seq))
                .filter(playlist::parent_id.eq(parent_id)),
        )
        .set(playlist::seq.eq(playlist::seq + 1))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            playlist::table
                .filter(playlist::seq.gt(seq))
                .filter(playlist::parent_id.eq(parent_id)),
        )
        .set(playlist::seq.eq(playlist::seq - 1))
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
            playlist::table
                .filter(playlist::seq.ge(start))
                .filter(playlist::seq.lt(end))
                .filter(playlist::parent_id.eq(parent_id)),
        )
        .set(playlist::seq.eq(playlist::seq + 1))
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
            playlist::table
                .filter(playlist::seq.gt(start))
                .filter(playlist::seq.le(end))
                .filter(playlist::parent_id.eq(parent_id)),
        )
        .set(playlist::seq.eq(playlist::seq - 1))
        .execute(conn)
    }
}

/// Represents a new record insertale to the `playlist` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::one_library::models::NewPlaylist;
///
/// let new = NewPlaylist::playlist("Name").seq(2);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = playlist)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewPlaylist {
    /// The sequence/order number of the playlist (1-based index).
    pub seq: i32,
    /// The name of the playlist.
    pub name: String,
    /// Optional foreign key referencing the artwork [`Image`] (`0` or `None` if no image).
    pub image_id: Option<i32>,
    /// The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`).
    pub attribute: i32,
    /// The ID of the parent [`Playlist`], `0` for top-level records.
    pub parent_id: i32,
}

impl ModelInsert for NewPlaylist {
    type Model = Playlist;

    fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let seq = if self.seq == 0 { None } else { Some(self.seq) };
        let (seq, _n) = Playlist::update_seq_before_insert(conn, &self.parent_id, seq)?;
        self.seq = seq;
        diesel::insert_into(playlist::table)
            .values(&self)
            .get_result(conn)
    }
}

impl NewPlaylist {
    /// Creates a new playlist record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Creates a new playlist record as a playlist (`attribute=0)`.
    pub fn playlist<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: 0,
            ..Default::default()
        }
    }

    /// Creates a new playlist record as a folder (`attribute=1)`.
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

    /// Builder for `image_id`.
    pub fn image_id(mut self, id: i32) -> Self {
        self.image_id = Some(id);
        self
    }
}

/// Represents the `playlist_content` table in the Rekordbox One Library database.
///
/// This struct maps to the `playlist_content` table in the One Library export database.
/// It stores information about the relationship between songs and playlists, including
/// metadata such as track number and associated playlist or content IDs.
///
/// This struct also is insertable into the `playlist_content` table.
///
/// # References
/// * [`Playlist`] via `playlist_id` foreign key.
/// * [`Content`] via `content_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable, Associations)]
#[diesel(table_name = playlist_content)]
#[diesel(primary_key(playlist_id, content_id))]
#[diesel(belongs_to(Playlist, foreign_key = playlist_id))]
#[diesel(belongs_to(Content, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct PlaylistContent {
    /// The ID of the associated [`Playlist`].
    pub playlist_id: i32,
    /// The ID of the associated [`Content`].
    pub content_id: i32,
    /// The sequence/order number of the history content (1-based index)
    pub seq: i32,
}

impl Model for PlaylistContent {
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

impl ModelDelete for PlaylistContent {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let playlist_id = id.0;
        let result = diesel::delete(playlist_content::table.find(*id)).execute(conn)?;
        Self::reset_seq(conn, &playlist_id)?;
        Ok(result)
    }
}

impl ModelInsert for PlaylistContent {
    type Model = Self;

    fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        if !Content::id_exists(conn, &self.content_id)? {
            return Err(Error::NotFound);
        }

        let seq = if self.seq == 0 { None } else { Some(self.seq) };
        let (seq, _n) = PlaylistContent::update_seq_before_insert(conn, &self.playlist_id, seq)?;
        self.seq = seq;

        diesel::insert_into(playlist_content::table)
            .values(self)
            .get_result(conn)
    }
}

impl ModelList for PlaylistContent {
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize> {
        let res = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };

        let old_seq = res.seq;
        let res = Self::update_seq_before_move_in(conn, &res.playlist_id, old_seq, seq)?;
        let (seq, _n) = match res {
            Some((s, n)) => (s, n),
            None => return Ok(0),
        };

        diesel::update(playlist_content::table.find(*id))
            .set(playlist_content::seq.eq(seq))
            .execute(conn)
    }
}

impl PlaylistContent {
    /// Creates a new `playlist_content` record with the required fields.
    pub fn new(playlist_id: i32, content_id: i32) -> Self {
        Self {
            playlist_id,
            content_id,
            seq: 0,
        }
    }

    /// Builder for `seq`.
    pub fn seq(mut self, seq: i32) -> Self {
        self.seq = seq;
        self
    }

    /// Queries all records from the `playlist_content` table by its `playlist_id`
    pub fn by_playlist_id(conn: &mut SqliteConnection, playlist_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(playlist_content::playlist_id.eq(playlist_id))
            .load(conn)
    }

    /// Queries all records from the `playlist_content` table by its `content_id`
    pub fn by_content_id(conn: &mut SqliteConnection, content_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(playlist_content::content_id.eq(content_id))
            .load(conn)
    }

    /// Creates a filter for records by `playlist_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_playlist_id(playlist_id: i32) -> _ {
        playlist_content::table.filter(playlist_content::playlist_id.eq(playlist_id))
    }

    /// Creates a filter for records by multiple `playlist_id`s.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_playlist_ids(playlist_ids: &[i32]) -> _ {
        playlist_content::table.filter(playlist_content::playlist_id.eq_any(playlist_ids))
    }

    /// Creates a filter for records by `content_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_content_id(content_id: i32) -> _ {
        playlist_content::table.filter(playlist_content::content_id.eq(content_id))
    }

    /// Creates a filter for records by multiple `content_id`s.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_content_ids(content_ids: &[i32]) -> _ {
        playlist_content::table.filter(playlist_content::content_id.eq_any(content_ids))
    }

    /// Counts the number of records in the `playlist_content` table for a given `playlist_id`.
    pub fn count(conn: &mut SqliteConnection, playlist_id: i32) -> QueryResult<i32> {
        let count: i64 = Self::filter_by_playlist_id(playlist_id)
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    /// Deletes all record from the `playlist_content` table referencing a `playlist` record.
    pub fn delete_by_playlist_id(
        conn: &mut SqliteConnection,
        playlist_id: i32,
    ) -> QueryResult<usize> {
        // We don't need to handle seq, since we're deleting all records referencing the playlist
        diesel::delete(Self::filter_by_playlist_id(playlist_id)).execute(conn)
    }

    /// Deletes all record from the `playlist_content` table referencing multiple `playlist` records.
    pub fn delete_by_playlist_ids(
        conn: &mut SqliteConnection,
        playlist_ids: &[i32],
    ) -> QueryResult<usize> {
        // We don't need to handle seq, since we're deleting all records referencing the playlists
        diesel::delete(Self::filter_by_playlist_ids(playlist_ids)).execute(conn)
    }

    /// Deletes all record from the `playlist_content` table referencing a `content` record.
    pub fn delete_by_content_id(
        conn: &mut SqliteConnection,
        content_id: i32,
    ) -> QueryResult<usize> {
        let playlist_id: Option<i32> = diesel::delete(Self::filter_by_content_id(content_id))
            .returning(playlist_content::playlist_id)
            .get_result(conn)
            .optional()?;
        if let Some(playlist_id) = playlist_id {
            Self::reset_seq(conn, &playlist_id)?;
            Ok(1)
        } else {
            Ok(0)
        }
    }

    /// Deletes all record from the `playlist_content` table referencing a `content` record.
    pub fn delete_by_content_ids(
        conn: &mut SqliteConnection,
        content_ids: &[i32],
    ) -> QueryResult<usize> {
        let playlist_ids: Vec<i32> = diesel::delete(Self::filter_by_content_ids(content_ids))
            .returning(playlist_content::playlist_id)
            .get_results(conn)?;
        for playlist_id in &playlist_ids {
            Self::reset_seq(conn, playlist_id)?;
        }
        Ok(playlist_ids.len())
    }
}

impl TreeSeq for PlaylistContent {
    type ParentId = i32;
    const START_SEQ: i32 = 0;

    #[inline]
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        Playlist::is_playlist(conn, *parent_id)
    }

    #[inline]
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = Self::filter_by_playlist_id(*parent_id)
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    #[inline]
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        Self::filter_by_playlist_id(*parent_id)
            .select(playlist_content::seq)
            .get_results(conn)
    }

    #[inline]
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT playlist_id, ROW_NUMBER() OVER (ORDER BY sequenceNo) + (? - 1) AS new_seq
                FROM playlist_content WHERE playlist_id =?
            ) UPDATE playlist_content
            SET sequenceNo = (SELECT new_seq FROM ordered WHERE ordered.playlist_id = playlist_content.playlist_id)
            WHERE playlist_id =?;"#,
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
            playlist_content::table
                .filter(playlist_content::seq.ge(seq))
                .filter(playlist_content::playlist_id.eq(parent_id)),
        )
        .set(playlist_content::seq.eq(playlist_content::seq + 1))
        .execute(conn)
    }

    #[inline]
    fn decrement_seq_gt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            playlist_content::table
                .filter(playlist_content::seq.gt(seq))
                .filter(playlist_content::playlist_id.eq(parent_id)),
        )
        .set(playlist_content::seq.eq(playlist_content::seq - 1))
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
            playlist_content::table
                .filter(playlist_content::seq.ge(start))
                .filter(playlist_content::seq.lt(end))
                .filter(playlist_content::playlist_id.eq(parent_id)),
        )
        .set(playlist_content::seq.eq(playlist_content::seq + 1))
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
            playlist_content::table
                .filter(playlist_content::seq.gt(start))
                .filter(playlist_content::seq.le(end))
                .filter(playlist_content::playlist_id.eq(parent_id)),
        )
        .set(playlist_content::seq.eq(playlist_content::seq - 1))
        .execute(conn)
    }
}

/// Represents the `playlist` table in the Rekordbox database in the form of a tree.
///
/// See [`Playlist`] for the flat representation.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistTreeNode {
    pub id: i32,
    pub seq: i32,
    pub name: String,
    pub image_id: Option<i32>,
    pub attribute: i32,
    pub parent_id: i32,
    pub children: Vec<PlaylistTreeNodeRef>,
}

impl From<Playlist> for PlaylistTreeNode {
    fn from(playlist: Playlist) -> Self {
        Self {
            id: playlist.id,
            seq: playlist.seq,
            name: playlist.name,
            image_id: playlist.image_id,
            attribute: playlist.attribute,
            parent_id: playlist.parent_id,
            children: vec![],
        }
    }
}

impl Into<Playlist> for PlaylistTreeNode {
    fn into(self) -> Playlist {
        Playlist {
            id: self.id,
            seq: self.seq,
            name: self.name,
            image_id: self.image_id,
            attribute: self.attribute,
            parent_id: self.parent_id,
        }
    }
}

/// Recursively sort a playlist tree item and its children
fn sort_tree(item: &mut PlaylistTreeNodeRef) {
    // Sort the current node's children by a chosen criterion (e.g., ID or name)
    item.borrow_mut().children.sort_by(|a, b| {
        let a_seq = a.borrow().seq;
        let b_seq = b.borrow().seq;
        a_seq.cmp(&b_seq)
    });
    // Recursively sort each child
    for child in &mut item.borrow_mut().children {
        sort_tree(child);
    }
}

/// Sort a list of playlist tree items and their children
fn sort_tree_list(tree: &mut Vec<PlaylistTreeNodeRef>) {
    // Sort the root nodes
    tree.sort_by(|a, b| {
        let a_seq = a.borrow().seq;
        let b_seq = b.borrow().seq;
        a_seq.cmp(&b_seq)
    });
    // Sort each tree item recursively
    for item in tree {
        sort_tree(item);
    }
}

/// Build a playlist tree from a flat list of playlists
fn build_tree(playlists: &[Playlist]) -> Vec<PlaylistTreeNodeRef> {
    let nodes: Vec<PlaylistTreeNode> = playlists
        .iter()
        .map(|p| PlaylistTreeNode::from(p.clone()))
        .collect();
    let mut map = HashMap::<i32, PlaylistTreeNodeRef>::new();
    let mut roots = Vec::new();

    // Populate the map with nodes
    for pl in nodes.iter() {
        let item = Rc::new(RefCell::new(pl.clone()));
        map.insert(pl.id.clone(), item.clone());
        if pl.parent_id == 0 {
            roots.push(item.clone());
        }
    }

    // Build the tree structure
    for id in map.keys() {
        let node = map.get(id).unwrap();
        let parent_id = node.borrow().parent_id.clone();
        if let Some(parent_node) = map.get(&parent_id) {
            parent_node.borrow_mut().children.push(node.clone());
        }
    }
    sort_tree_list(&mut roots);

    roots
}
