// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! SQLite row models and operations for `Playlist`.

use diesel::prelude::*;
use diesel::result::Error;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use super::content::Content;
use crate::enums::PlaylistType;
use crate::schema::{content, playlist, playlist_content};
use rbox_db::{
    build_tree_list, delete_tree_batch, ordering_error, Model, ModelDelete, ModelInsert, ModelList,
    ModelTree, OrderedModel, TreeNode, TreeSeq,
};

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
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = playlist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(Playlist, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Playlist {
    /// The unique identifier of the playlist.
    pub id: i32,
    /// Position among siblings, starting at zero.
    pub seq: i32,
    /// The name of the playlist.
    pub name: String,
    /// Optional foreign key referencing the artwork [`Image`] (`0` or `None` if no image).
    pub image_id: Option<i32>,
    /// The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`).
    pub attribute: PlaylistType,
    /// The ID of the parent [`Playlist`], `0` for top-level records.
    pub parent_id: i32,
}

impl Model for Playlist {
    type Id = i32;

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query()
            .order((playlist::parent_id, playlist::seq, playlist::id))
            .load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }
}

impl ModelDelete for Playlist {
    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        delete_tree_batch::<Self>(conn, ids)
    }

    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        conn.transaction(|conn| {
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
        })
    }
}

impl ModelTree for Playlist {
    fn move_to(
        conn: &mut SqliteConnection,
        id: &Self::Id,
        parent_id: Option<&Self::Id>,
        seq: Option<i32>,
    ) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let res = match Self::find(conn, &id)? {
                Some(r) => r,
                None => return Err(Error::NotFound),
            };
            let old_seq = res.seq;
            let old_parent_id = res.parent_id.clone();
            let parent_id = parent_id.unwrap_or(&old_parent_id);

            Self::validate_destination(conn, id, parent_id)?;
            let res = Self::update_seq_before_move(conn, &old_parent_id, parent_id, old_seq, seq)?;
            let (seq, _n) = match res {
                Some((s, n)) => (s, n),
                None => return Ok(0),
            };
            diesel::update(playlist::table.find(id))
                .set((playlist::seq.eq(seq), playlist::parent_id.eq(parent_id)))
                .execute(conn)
        })
    }
}

impl Playlist {
    /// Queries all records from the `playlist` table and returns a tree structure.
    pub fn tree(conn: &mut SqliteConnection) -> QueryResult<Vec<PlaylistTreeNodeRef>> {
        let playlists: Vec<Self> = Self::query().load(conn)?;
        Ok(build_tree_list(playlists, 0))
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
    pub fn get_attribute(
        conn: &mut SqliteConnection,
        id: i32,
    ) -> QueryResult<Option<PlaylistType>> {
        if id == 0 {
            return Ok(Some(PlaylistType::Folder));
        }
        let value: Option<i32> = playlist::table
            .find(id)
            .select(playlist::attribute)
            .get_result(conn)
            .optional()?;
        Ok(value.map(|v| PlaylistType::try_from(v).unwrap()))
    }

    /// Returns `true` if the record in the `playlist` table exists and is a playlist, `false` otherwise.
    pub fn is_playlist(conn: &mut SqliteConnection, id: i32) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == PlaylistType::List),
            None => Ok(false),
        }
    }

    /// Returns `true` if the record in the `playlist` table exists and is a folder, `false` otherwise.
    pub fn is_folder(conn: &mut SqliteConnection, id: i32) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == PlaylistType::Folder),
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

    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        Self::is_folder(conn, *parent_id)
    }

    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = playlist::table
            .filter(playlist::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        i32::try_from(count).map_err(|_| ordering_error("Collection too large"))
    }

    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        playlist::table
            .filter(playlist::parent_id.eq(parent_id))
            .order(playlist::seq)
            .select(playlist::seq)
            .load(conn)
    }

    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        <Self as OrderedModel>::renumber(conn, parent_id)
    }

    fn shift_seq(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        range: std::ops::Range<i32>,
        delta: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            playlist::table
                .filter(playlist::parent_id.eq(parent_id))
                .filter(playlist::seq.ge(range.start))
                .filter(playlist::seq.lt(range.end)),
        )
        .set(playlist::seq.eq(playlist::seq + delta))
        .execute(conn)
    }
}

impl OrderedModel for Playlist {
    fn order_id(&self) -> i32 {
        self.id
    }

    fn order_parent(&self) -> &Self::ParentId {
        &self.parent_id
    }

    fn order_position(&self) -> i32 {
        self.seq
    }

    fn ordered(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<Vec<Self>> {
        playlist::table
            .filter(playlist::parent_id.eq(parent_id))
            .order((playlist::seq, playlist::id))
            .select(Self::as_select())
            .load(conn)
    }

    fn write_order(
        conn: &mut SqliteConnection,
        changes: &[(&Self, &Self::ParentId, i32)],
    ) -> QueryResult<usize> {
        if changes.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for (row, parent_id, seq) in changes {
            count += diesel::update(playlist::table.find(row.id))
                .set((playlist::seq.eq(seq), playlist::parent_id.eq(*parent_id)))
                .execute(conn)?;
        }
        Ok(count)
    }
}

/// Represents a new record insertale to the `playlist` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox_onelib::models::NewPlaylist;
///
/// let new = NewPlaylist::playlist("Name").seq(2);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = playlist)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewPlaylist {
    /// Insertion position in the collection; `None` appends.
    pub seq: Option<i32>,
    /// The name of the playlist.
    pub name: String,
    /// Optional foreign key referencing the artwork [`Image`] (`0` or `None` if no image).
    pub image_id: Option<i32>,
    /// The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`).
    pub attribute: PlaylistType,
    /// The ID of the parent [`Playlist`], `0` for top-level records.
    pub parent_id: i32,
}

impl ModelInsert for NewPlaylist {
    type Model = Playlist;

    fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        conn.transaction(|conn| {
            let seq = self.seq;
            let (seq, _n) = Playlist::update_seq_before_insert(conn, &self.parent_id, seq)?;
            self.seq = Some(seq);
            diesel::insert_into(playlist::table)
                .values(&self)
                .get_result(conn)
        })
    }
}

impl NewPlaylist {
    /// Creates a new playlist record with the required fields.
    pub fn new<S: Into<String>>(name: S, attribute: PlaylistType) -> Self {
        if !(attribute == PlaylistType::List || attribute == PlaylistType::Folder) {
            panic!("Invalid playlist type: {:?}", attribute);
        }

        Self {
            name: name.into(),
            attribute,
            ..Default::default()
        }
    }

    /// Creates a new playlist record as a playlist (`attribute=0)`.
    pub fn playlist<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: PlaylistType::List,
            ..Default::default()
        }
    }

    /// Creates a new playlist record as a folder (`attribute=1)`.
    pub fn folder<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: PlaylistType::Folder,
            ..Default::default()
        }
    }

    /// Set the insertion position among siblings; zero inserts first.
    pub fn seq(mut self, seq: i32) -> Self {
        self.seq = Some(seq);
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
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct PlaylistContent {
    /// The ID of the associated [`Playlist`].
    pub playlist_id: i32,
    /// The ID of the associated [`Content`].
    pub content_id: i32,
    /// Position within the playlist, starting at one. Before insertion, `-1` requests append.
    pub seq: i32,
}

impl Model for PlaylistContent {
    type Id = (i32, i32);

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query()
            .order((
                playlist_content::playlist_id,
                playlist_content::seq,
                playlist_content::playlist_id,
                playlist_content::content_id,
            ))
            .load(conn)
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
        conn.transaction(|conn| {
            let playlist_id = id.0;
            let result = diesel::delete(playlist_content::table.find(*id)).execute(conn)?;
            Self::reset_seq(conn, &playlist_id)?;
            Ok(result)
        })
    }
}

impl ModelInsert for PlaylistContent {
    type Model = Self;

    fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        conn.transaction(|conn| {
            if !Content::id_exists(conn, &self.content_id)? {
                return Err(Error::NotFound);
            }

            let seq = if self.seq == -1 { None } else { Some(self.seq) };
            if Self::id_exists(conn, &(self.playlist_id, self.content_id))? {
                return Err(Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    Box::new("Content already belongs to this collection".to_owned()),
                ));
            }
            let (seq, _n) =
                PlaylistContent::update_seq_before_insert(conn, &self.playlist_id, seq)?;
            self.seq = seq;

            diesel::insert_into(playlist_content::table)
                .values(self)
                .get_result(conn)
        })
    }
}

impl ModelList for PlaylistContent {
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize> {
        conn.transaction(|conn| {
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
        })
    }
}

impl PlaylistContent {
    /// Create an entry that will be appended to the playlist on insertion.
    pub fn new(playlist_id: i32, content_id: i32) -> Self {
        Self {
            playlist_id,
            content_id,
            seq: -1,
        }
    }

    /// Set the stored position; one is the first entry and -1 requests append on insertion.
    pub fn seq(mut self, seq: i32) -> Self {
        self.seq = seq;
        self
    }

    /// Queries all records from the `playlist_content` table by its `playlist_id`
    pub fn by_playlist_id(conn: &mut SqliteConnection, playlist_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(playlist_content::playlist_id.eq(playlist_id))
            .order((playlist_content::seq, playlist_content::content_id))
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
        conn.transaction(|conn| {
            // We don't need to handle seq, since we're deleting all records referencing the playlist
            diesel::delete(Self::filter_by_playlist_id(playlist_id)).execute(conn)
        })
    }

    /// Deletes all record from the `playlist_content` table referencing multiple `playlist` records.
    pub fn delete_by_playlist_ids(
        conn: &mut SqliteConnection,
        playlist_ids: &[i32],
    ) -> QueryResult<usize> {
        conn.transaction(|conn| {
            // We don't need to handle seq, since we're deleting all records referencing the playlists
            diesel::delete(Self::filter_by_playlist_ids(playlist_ids)).execute(conn)
        })
    }

    /// Deletes all record from the `playlist_content` table referencing a `content` record.
    pub fn delete_by_content_id(
        conn: &mut SqliteConnection,
        content_id: i32,
    ) -> QueryResult<usize> {
        conn.transaction(|conn| Self::delete_by_content_ids(conn, &[content_id]))
    }

    /// Deletes all record from the `playlist_content` table referencing a `content` record.
    pub fn delete_by_content_ids(
        conn: &mut SqliteConnection,
        content_ids: &[i32],
    ) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let playlist_ids: Vec<i32> = diesel::delete(Self::filter_by_content_ids(content_ids))
                .returning(playlist_content::playlist_id)
                .get_results(conn)?;
            for playlist_id in &playlist_ids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
            {
                Self::reset_seq(conn, playlist_id)?;
            }
            Ok(playlist_ids.len())
        })
    }
}

impl TreeSeq for PlaylistContent {
    type ParentId = i32;
    const START_SEQ: i32 = 1;

    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        Playlist::is_playlist(conn, *parent_id)
    }
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = playlist_content::table
            .filter(playlist_content::playlist_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        i32::try_from(count).map_err(|_| ordering_error("Collection too large"))
    }
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        playlist_content::table
            .filter(playlist_content::playlist_id.eq(parent_id))
            .order(playlist_content::seq)
            .select(playlist_content::seq)
            .load(conn)
    }
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize> {
        <Self as OrderedModel>::renumber(conn, parent_id)
    }
    fn shift_seq(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        range: std::ops::Range<i32>,
        delta: i32,
    ) -> QueryResult<usize> {
        diesel::update(
            playlist_content::table
                .filter(playlist_content::playlist_id.eq(parent_id))
                .filter(playlist_content::seq.ge(range.start))
                .filter(playlist_content::seq.lt(range.end)),
        )
        .set(playlist_content::seq.eq(playlist_content::seq + delta))
        .execute(conn)
    }
}

impl OrderedModel for PlaylistContent {
    fn order_id(&self) -> (i32, i32) {
        (self.playlist_id, self.content_id)
    }
    fn order_parent(&self) -> &Self::ParentId {
        &self.playlist_id
    }
    fn order_position(&self) -> i32 {
        self.seq
    }
    fn ordered(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<Vec<Self>> {
        playlist_content::table
            .filter(playlist_content::playlist_id.eq(parent_id))
            .order((
                playlist_content::seq,
                (playlist_content::playlist_id, playlist_content::content_id),
            ))
            .select(Self::as_select())
            .load(conn)
    }
    fn write_order(
        conn: &mut SqliteConnection,
        changes: &[(&Self, &Self::ParentId, i32)],
    ) -> QueryResult<usize> {
        if changes.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for (row, _parent_id, seq) in changes {
            count +=
                diesel::update(playlist_content::table.find((row.playlist_id, row.content_id)))
                    .set(playlist_content::seq.eq(seq))
                    .execute(conn)?;
        }
        Ok(count)
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
    pub attribute: PlaylistType,
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

impl TreeNode for PlaylistTreeNode {
    type Model = Playlist;
    type Id = i32;
    fn id(&self) -> i32 {
        self.id
    }
    fn sequence(&self) -> i32 {
        self.seq
    }
    fn parent_id(&self) -> i32 {
        self.parent_id
    }
    fn children(&self) -> &Vec<PlaylistTreeNodeRef> {
        &self.children
    }
    fn children_mut(&mut self) -> &mut Vec<PlaylistTreeNodeRef> {
        &mut self.children
    }
}
