// Author: Dylan Jones
// Date:   2025-09-02

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use uuid::Uuid;

use super::agent_registry::AgentRegistry;
use super::djmd_content::DjmdContent;
use super::schema::{djmdContent, djmdPlaylist, djmdSongPlaylist};
use super::{format_datetime, Date, DateString, RandomIdGenerator};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdPlaylist` table in the Rekordbox database.
///
/// This struct maps to the `djmdPlaylist` table in the SQLite database used by Rekordbox.
/// It stores information about playlists, including metadata such as sequence, name, image path,
/// attributes, and parent relationships.
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

impl DjmdPlaylist {
    /// Queries all records from the `djmdPlaylist` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdPlaylist` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    /// Queries a record from the `djmdPlaylist` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdPlaylist::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Queries a record from the `djmdPlaylist` table by its associated `content_id` via `djmdSongPlaylist`
    pub fn find_by_content_id(conn: &mut SqliteConnection, cid: &str) -> QueryResult<Option<Self>> {
        djmdPlaylist::table
            .inner_join(
                djmdSongPlaylist::table.on(djmdPlaylist::id.eq(djmdSongPlaylist::playlist_id)),
            )
            .filter(djmdSongPlaylist::content_id.eq(cid))
            .select(Self::as_select())
            .first(conn)
            .optional()
    }

    /// Creates a filter for records by `parent_id` and `name`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_children_by_name<'a>(parent_id: &'a str, name: &'a str) -> _ {
        djmdPlaylist::table
            .filter(djmdPlaylist::parent_id.eq(parent_id))
            .filter(djmdPlaylist::name.eq(name))
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
                let result: Option<String> = Self::filter_children_by_name(&parent_id, name)
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

    /// Returns the playlist type (`attribute`) of a record in the `djmdPlaylist` table.
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

    /// Counts the number of child records in the `djmdPlaylist` table for a given `parent_id`.
    pub fn count_children(conn: &mut SqliteConnection, parent_id: &str) -> QueryResult<i32> {
        let count: i64 = djmdPlaylist::table
            .filter(djmdPlaylist::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    /// Counts the number of child records in the `djmdPlaylist` table for a record inside the same parent.
    pub fn count_parent_children(conn: &mut SqliteConnection, id: &str) -> QueryResult<i32> {
        let parent_id: String = djmdPlaylist::table
            .find(id)
            .select(djmdPlaylist::parent_id)
            .get_result(conn)?;
        Self::count_children(conn, &parent_id)
    }

    /// Checks if a record with the given `id` exists in the `djmdPlaylist` table.
    pub fn id_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }

    /// Updates an existing record in the `djmdPlaylist` table.
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
        Ok(diesel::update(djmdPlaylist::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)?)
    }

    /// Set the name of a record in the `djmdPlaylist` table.
    pub fn rename(conn: &mut SqliteConnection, id: &str, name: &str) -> QueryResult<Self> {
        let datestr = format_datetime(&Utc::now());
        let usn = AgentRegistry::increment_local_usn(conn)?;
        Ok(diesel::update(djmdPlaylist::table.find(id))
            .set((
                djmdPlaylist::name.eq(name),
                djmdPlaylist::updated_at.eq(&datestr),
                djmdPlaylist::rb_local_usn.eq(usn),
            ))
            .get_result(conn)?)
    }

    /// Reset the `seq` numbers of all records from the `djmdPlaylist` table for a given `parent_id`.
    ///
    /// The `seq` numbers are reset to the *current* order of the records in the `djmdPlaylist` table,
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
                FROM djmdPlaylist WHERE ParentID =?
            ) UPDATE djmdPlaylist
            SET Seq = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdPlaylist.ID)
            WHERE ParentID = ?;"#,
        )
        .bind::<diesel::sql_types::Text, _>(parent_id)
        .bind::<diesel::sql_types::Text, _>(parent_id)
        .execute(conn)
    }

    /// Filters child records with `seq` greater than `start`:
    ///
    /// `start < seq`
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_children_gt(parent_id: &str, start: i32) -> _ {
        djmdPlaylist::table
            .filter(djmdPlaylist::parent_id.eq(parent_id))
            .filter(djmdPlaylist::seq.gt(start))
    }

    /// Filters child records with `seq` greater equal than `start`.
    ///
    /// `start <= start`
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_children_gte(parent_id: &str, start: i32) -> _ {
        djmdPlaylist::table
            .filter(djmdPlaylist::parent_id.eq(parent_id))
            .filter(djmdPlaylist::seq.ge(start))
    }

    /// Filters child records with `seq` greater than `start` and less equal `stop`.
    ///
    /// `start < seq <= stop`
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_children_gt_lte(parent_id: &str, start: i32, stop: i32) -> _ {
        djmdPlaylist::table
            .filter(djmdPlaylist::parent_id.eq(parent_id))
            .filter(djmdPlaylist::seq.gt(start))
            .filter(djmdPlaylist::seq.le(stop))
    }

    /// Filters child records with `seq` greater than `start` and less equal `stop`.
    ///
    /// `start <= seq < stop`
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_children_gte_lt(parent_id: &str, start: i32, stop: i32) -> _ {
        djmdPlaylist::table
            .filter(djmdPlaylist::parent_id.eq(parent_id))
            .filter(djmdPlaylist::seq.ge(start))
            .filter(djmdPlaylist::seq.lt(stop))
    }

    /// Move a record in the `djmdPlaylist` table inside the same parent.
    pub fn move_in(conn: &mut SqliteConnection, id: &str, seq: i32) -> QueryResult<usize> {
        let rec = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Ok(0),
        };
        let count = Self::count_children(conn, &rec.parent_id)?;
        let old_seq = rec.seq;
        if seq <= 0 || seq > count {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new("Invalid sequence number".to_string()),
            ));
        }
        if seq == old_seq {
            return Ok(0);
        }

        // Moving increments USN by 1 for all changes
        let usn = AgentRegistry::increment_local_usn(conn)?;
        let datestr = format_datetime(&Utc::now());

        if seq > old_seq {
            // Seq is increased (move down in parent):
            // Decrease seq of all other records with seq between old seq and new seq
            // old_seq < x <= seq
            diesel::update(Self::filter_children_gt_lte(&rec.parent_id, old_seq, seq))
                .set((
                    djmdPlaylist::seq.eq(djmdPlaylist::seq - 1),
                    djmdPlaylist::updated_at.eq(&datestr),
                    djmdPlaylist::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
        } else if seq < old_seq {
            // Seq is decreased (moved up in parent):
            // Increase seq of all records with seq between old seq and new seq
            // seq <= x < old_seq
            diesel::update(Self::filter_children_gte_lt(&rec.parent_id, seq, old_seq))
                .set((
                    djmdPlaylist::seq.eq(djmdPlaylist::seq + 1),
                    djmdPlaylist::updated_at.eq(&datestr),
                    djmdPlaylist::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
        }
        // Update sequence number of record
        diesel::update(djmdPlaylist::table.find(id))
            .set((
                djmdPlaylist::seq.eq(seq),
                djmdPlaylist::updated_at.eq(&datestr),
                djmdPlaylist::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }

    /// Move a record in the `djmdPlaylist` table to a new parent.
    pub fn move_to(
        conn: &mut SqliteConnection,
        id: &str,
        parent_id: &str,
        seq: Option<i32>,
    ) -> QueryResult<usize> {
        let rec = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };
        let old_seq = rec.seq;
        let old_parent_id = rec.parent_id.clone();
        let count = Self::count_children(conn, &parent_id)?;

        let datestr = format_datetime(&Utc::now());

        if parent_id != old_parent_id {
            if !DjmdPlaylist::id_exists(conn, &parent_id)? {
                return Err(Error::NotFound);
            };

            let seq = seq.unwrap_or(count + 1); // Insert at the end by default
                                                // parent changed, move to new parent
            let attr = Self::get_attribute(conn, parent_id)?.unwrap_or(1);
            if attr != 1 {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new("Cannot add a child to a DjmdPlaylist list".to_string()),
                ));
            }
            if seq <= 0 || seq > count + 1 {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new("Invalid sequence number".to_string()),
                ));
            }

            // Moving increments USN by 1 for all changes
            let usn = AgentRegistry::increment_local_usn(conn)?;

            // Update seq of records in the old parent:
            // Decrease seq of records with seq higher than old seq: old_seq < x
            diesel::update(Self::filter_children_gt(&old_parent_id, old_seq))
                .set((
                    djmdPlaylist::seq.eq(djmdPlaylist::seq - 1),
                    djmdPlaylist::updated_at.eq(&datestr),
                    djmdPlaylist::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;

            let new_seq = if seq <= count {
                // Move to center in the new parent
                // Increase seq of records with seq higher than new: seq <= x
                diesel::update(Self::filter_children_gte(&parent_id, seq))
                    .set((
                        djmdPlaylist::seq.eq(djmdPlaylist::seq + 1),
                        djmdPlaylist::updated_at.eq(&datestr),
                        djmdPlaylist::rb_local_usn.eq(usn),
                    ))
                    .execute(conn)?;
                seq
            } else {
                count + 1
            };
            // Update Seq and parent of actual record
            diesel::update(djmdPlaylist::table.find(id))
                .set((
                    djmdPlaylist::seq.eq(new_seq),
                    djmdPlaylist::parent_id.eq(parent_id),
                    djmdPlaylist::updated_at.eq(&datestr),
                    djmdPlaylist::rb_local_usn.eq(usn),
                ))
                .execute(conn)
        } else {
            let seq = seq.unwrap_or(old_seq); // Insert at the end by default
            if seq == old_seq {
                return Ok(0);
            }

            // Move to seq in the same parent
            if seq <= 0 || seq > count {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new("Invalid sequence number".to_string()),
                ));
            }

            // Moving increments USN by 1 for all changes
            let usn = AgentRegistry::increment_local_usn(conn)?;
            if seq > old_seq {
                // Seq is increased (move down in parent):
                // Decrease seq of all other records with seq between old seq and new seq
                // old_seq < x <= seq
                diesel::update(Self::filter_children_gt_lte(&rec.parent_id, old_seq, seq))
                    .set((
                        djmdPlaylist::seq.eq(djmdPlaylist::seq - 1),
                        djmdPlaylist::updated_at.eq(&datestr),
                        djmdPlaylist::rb_local_usn.eq(usn),
                    ))
                    .execute(conn)?;
            } else if seq < old_seq {
                // Seq is decreased (moved up in parent):
                // Increase seq of all records with seq between old seq and new seq
                // seq <= x < old_seq
                diesel::update(Self::filter_children_gte_lt(&rec.parent_id, seq, old_seq))
                    .set((
                        djmdPlaylist::seq.eq(djmdPlaylist::seq + 1),
                        djmdPlaylist::updated_at.eq(&datestr),
                        djmdPlaylist::rb_local_usn.eq(usn),
                    ))
                    .execute(conn)?;
            }
            // Update sequence number of record
            diesel::update(djmdPlaylist::table.find(id))
                .set((
                    djmdPlaylist::seq.eq(seq),
                    djmdPlaylist::updated_at.eq(&datestr),
                    djmdPlaylist::rb_local_usn.eq(usn),
                ))
                .execute(conn)
        }
    }

    /// Deletes a record from the `djmdPlaylist` table and removes any references to it.
    pub fn delete(conn: &mut SqliteConnection, id: &str) -> QueryResult<Vec<String>> {
        // TODO: Increment USN only by one!!!

        let mut deleted_ids = Vec::new();
        deleted_ids.push(id.to_string());

        // Delete the record
        let parent_id: String = diesel::delete(djmdPlaylist::table.find(id))
            .returning(djmdPlaylist::parent_id)
            .get_result(conn)?;

        AgentRegistry::increment_local_usn(conn)?;
        // let mut result = 1;

        // Reorder the seq numbers of tags left in the parent
        Self::reset_seq(conn, &parent_id)?;

        // Remove any djmdSongSampler records that are associated with this djmdPlaylist
        DjmdSongPlaylist::delete_by_playlist_id(conn, id)?;

        // Remove all child djmdPlaylist records recursively
        let mut parent_ids = VecDeque::from(vec![id.to_string()]);
        while let Some(parent_id) = parent_ids.pop_front() {
            // Delete children
            let deleted: Vec<String> =
                diesel::delete(djmdPlaylist::table.filter(djmdPlaylist::parent_id.eq(parent_id)))
                    .returning(djmdPlaylist::id)
                    .get_results(conn)?;
            deleted_ids.extend(deleted.clone());
            AgentRegistry::increment_local_usn_by(conn, deleted.len())?;
            // result += deleted.len();

            // Delete any songs referencing the deleted tags
            DjmdSongPlaylist::delete_by_playlist_ids(conn, &deleted)?;
            // Add children to the queue
            for deleted_id in deleted {
                parent_ids.push_back(deleted_id);
            }
        }
        Ok(deleted_ids)
    }

    /// Queries all records from the `djmdPlaylist` table and returns a tree structure.
    pub fn tree(
        conn: &mut SqliteConnection,
    ) -> QueryResult<Vec<Rc<RefCell<DjmdPlaylistTreeItem>>>> {
        let playlists = DjmdPlaylist::all(conn)?;
        let nodes: Vec<DjmdPlaylistTreeItem> = playlists
            .iter()
            .map(|p| DjmdPlaylistTreeItem::from(p.clone()))
            .collect();

        let mut map = HashMap::<String, Rc<RefCell<DjmdPlaylistTreeItem>>>::new();
        let mut roots = Vec::new();

        // Populate the map with nodes
        for pl in nodes.iter() {
            let item = Rc::new(RefCell::new(pl.clone()));
            map.insert(pl.id.clone(), item.clone());
            if pl.parent_id == "root".to_string() {
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

/// Represents a new record insertale to the `djmdPlaylist` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdPlaylist;
///
/// let new = NewDjmdPlaylist::new("Name".into()).seq(2);
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

impl NewDjmdPlaylist {
    /// Creates a new `NewDjmdPlaylist` with the required `name` and `attribute` field.
    pub fn new(name: String, attribute: i32) -> Self {
        Self {
            name,
            attribute,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdPlaylist` as a playlist.
    pub fn playlist(name: String) -> Self {
        Self {
            name,
            attribute: 0,
            ..Default::default()
        }
    }

    /// Creates a new `NewDjmdPlaylist` as a folder.
    pub fn folder(name: String) -> Self {
        Self {
            name,
            attribute: 1,
            ..Default::default()
        }
    }

    /// Inserts the new record into the database
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<DjmdPlaylist> {
        let id = DjmdPlaylist::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();
        // let usn = AgentRegistry::increment_local_usn(conn)?;
        let parent_id = self.parent_id.unwrap_or("root".to_string());
        let count = DjmdPlaylist::count_children(conn, &parent_id)?;

        if let Some(attr) = DjmdPlaylist::get_attribute(conn, &parent_id)? {
            if attr != 1 {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new("Cannot add a child to a DjmdPlaylist list".to_string()),
                ));
            }
        } else {
            return Err(Error::NotFound);
        }

        let seq = if let Some(seq) = self.seq {
            if seq <= 0 || seq > count + 1 {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new("Invalid sequence number".to_string()),
                ));
            }
            if seq < count + 1 {
                // Shift playlists with seq higher than the new seq number by one
                // Shifting the other playlists increments the USN by 1 for *all* moved playlists
                let move_usn = AgentRegistry::increment_local_usn(conn)?;
                diesel::update(DjmdPlaylist::filter_children_gte(&parent_id, seq))
                    .set((
                        djmdPlaylist::seq.eq(djmdPlaylist::seq + 1),
                        djmdPlaylist::updated_at.eq(format_datetime(&now)),
                        djmdPlaylist::rb_local_usn.eq(move_usn),
                    ))
                    .execute(conn)?;
            };
            seq
        } else {
            count + 1
        };

        // Get next USN: We increment by 2 (1 for creating, 1 for renaming from 'New Playlist')
        let usn = AgentRegistry::increment_local_usn_by(conn, 2)?;
        let item = DjmdPlaylist {
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
        let result = diesel::insert_into(djmdPlaylist::table)
            .values(item)
            .get_result(conn)?;
        Ok(result)
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

    /// Builder for `image_path`.
    pub fn image_path(mut self, image_path: String) -> Self {
        self.image_path = Some(image_path);
        self
    }

    /// Builder for `smart_list`.
    pub fn smart_list(mut self, smart_list: String) -> Self {
        self.smart_list = Some(smart_list);
        self
    }
}

/// Represents the `djmdSongPlaylist` table in the Rekordbox database.
///
/// This struct maps to the `djmdSongPlaylist` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and playlists, including
/// metadata such as track number and associated playlist or content IDs.
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

impl DjmdSongPlaylist {
    /// Queries all records from the `djmdSongPlaylist` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `djmdSongPlaylist` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
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
        let count: i64 = Self::filter_by_playlist_id(playlist_id)
            .count()
            .get_result(conn)?;
        Ok(count as i32)
    }

    /// Checks if a record with the given `id` exists in the `djmdSongPlaylist` table.
    pub fn id_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }

    /// Reset the `track_no` numbers of all records from the `djmdSongPlaylist` table for a given `playlist_id`.
    ///
    /// The `track_no` numbers are reset to the *current* order of the records in the
    /// `djmdSongPlaylist` table, starting at 1:
    ///
    /// ```
    /// Old: 1, 2, 3, 5, 7, 8, 9
    /// New: 1, 2, 3, 4, 5, 6, 7
    /// ```
    ///
    /// This is helpful when deleting records or for fixing the sequence after an invalid state.
    /// When adding or moving records, the `track_no` numbers should be set explicitly.
    pub fn reset_seq(conn: &mut SqliteConnection, playlist_id: &str) -> QueryResult<usize> {
        diesel::sql_query(
            r#"WITH ordered AS (
                SELECT ID, ROW_NUMBER() OVER (ORDER BY TrackNo) AS new_seq
                FROM djmdSongPlaylist WHERE PlaylistID =?
            ) UPDATE djmdSongPlaylist
            SET TrackNo = (SELECT new_seq FROM ordered WHERE ordered.ID = djmdSongPlaylist.ID)
            WHERE PlaylistID =?;"#,
        )
        .bind::<diesel::sql_types::Text, _>(playlist_id)
        .bind::<diesel::sql_types::Text, _>(playlist_id)
        .execute(conn)
    }

    /// Filters child records with `seq` greater equal than `start`.
    ///
    /// `start <= start`
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_children_gte(playlist_id: &str, start: i32) -> _ {
        djmdSongPlaylist::table
            .filter(djmdSongPlaylist::playlist_id.eq(playlist_id))
            .filter(djmdSongPlaylist::track_no.ge(start))
    }

    /// Filters child records with `seq` greater than `start` and less equal `stop`.
    ///
    /// `start < seq <= stop`
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_children_gt_lte(playlist_id: &str, start: i32, stop: i32) -> _ {
        djmdSongPlaylist::table
            .filter(djmdSongPlaylist::playlist_id.eq(playlist_id))
            .filter(djmdSongPlaylist::track_no.gt(start))
            .filter(djmdSongPlaylist::track_no.le(stop))
    }

    /// Filters child records with `seq` greater than `start` and less equal `stop`.
    ///
    /// `start <= seq < stop`
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_children_gte_lt(playlist_id: &str, start: i32, stop: i32) -> _ {
        djmdSongPlaylist::table
            .filter(djmdSongPlaylist::playlist_id.eq(playlist_id))
            .filter(djmdSongPlaylist::track_no.ge(start))
            .filter(djmdSongPlaylist::track_no.lt(stop))
    }

    /// Move a record in the `djmdSongPlaylist` table.
    pub fn move_to(conn: &mut SqliteConnection, id: &str, seq: i32) -> QueryResult<usize> {
        let res = match Self::find(conn, id)? {
            Some(r) => r,
            None => return Err(Error::NotFound),
        };
        let old_seq = res.track_no;
        if seq == old_seq {
            return Ok(0);
        }

        let count = Self::count(conn, &res.playlist_id)?;
        if seq <= 0 || seq > count {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new("Invalid sequence number".to_string()),
            ));
        }
        let usn = AgentRegistry::increment_local_usn(conn)?;
        let utcnow: DateTime<Utc> = Utc::now();
        let datestr: String = format_datetime(&utcnow);
        if seq > old_seq {
            // Seq is increased (move down in parent):
            // Decrease seq of all other records with seq between old seq and new seq
            // old_seq < x <= seq
            diesel::update(Self::filter_children_gt_lte(&res.playlist_id, old_seq, seq))
                .set((
                    djmdSongPlaylist::track_no.eq(djmdSongPlaylist::track_no - 1),
                    djmdSongPlaylist::updated_at.eq(&datestr),
                    djmdSongPlaylist::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
        } else if seq < old_seq {
            // Seq is decreased (moved up in parent):
            // Increase seq of all records with seq between old seq and new seq
            // seq <= x < old_seq
            diesel::update(Self::filter_children_gte_lt(&res.playlist_id, seq, old_seq))
                .set((
                    djmdSongPlaylist::track_no.eq(djmdSongPlaylist::track_no + 1),
                    djmdSongPlaylist::updated_at.eq(&datestr),
                    djmdSongPlaylist::rb_local_usn.eq(usn),
                ))
                .execute(conn)?;
        }
        // Update sequence number of record
        diesel::update(djmdSongPlaylist::table.find(id))
            .set((
                djmdSongPlaylist::track_no.eq(seq),
                djmdSongPlaylist::updated_at.eq(&datestr),
                djmdSongPlaylist::rb_local_usn.eq(usn),
            ))
            .execute(conn)
    }

    /// Deletes a record from the `djmdSongPlaylist` table
    pub fn delete(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
        let query = djmdSongPlaylist::table.find(id);
        let my_tag_id: String = query.select(djmdSongPlaylist::playlist_id).first(conn)?;

        let result = diesel::delete(query).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Reorder the track_no numbers
        Self::reset_seq(conn, &my_tag_id)?;

        Ok(result)
    }

    /// Deletes all records from the `djmdSongPlaylist` table acssociated with a `playlist_id`.
    pub fn delete_by_playlist_id(
        conn: &mut SqliteConnection,
        playlist_id: &str,
    ) -> QueryResult<usize> {
        let result = diesel::delete(Self::filter_by_playlist_id(playlist_id)).execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, result)?;
        Ok(result)
    }

    /// Deletes all records from the `djmdSongPlaylist` table acssociated with one of `playlist_ids`.
    pub fn delete_by_playlist_ids(
        conn: &mut SqliteConnection,
        playlist_ids: &[String],
    ) -> QueryResult<usize> {
        let result = diesel::delete(Self::filter_by_playlist_ids(playlist_ids)).execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, result)?;
        Ok(result)
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

impl DjmdSongPlaylist {
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        playlist_id: String,
        content_id: String,
        seq: i32,
    ) -> Self {
        Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            playlist_id: playlist_id.clone(),
            content_id: content_id.clone(),
            track_no: seq,
        }
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
/// let new = NewSongDjmdPlaylist::new("Name".into());
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

impl NewDjmdSongPlaylist {
    /// Creates a new `NewDjmdSongPlaylist` with the required `playlist_id` and `content_id` field.
    pub fn new(playlist_id: String, content_id: String) -> Self {
        Self {
            playlist_id,
            content_id,
            ..Default::default()
        }
    }

    /// Builder for `track_no`.
    pub fn track_no(mut self, track_no: i32) -> Self {
        self.track_no = Some(track_no);
        self
    }

    /// Inserts the new record into the database
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<DjmdSongPlaylist> {
        let playlist = match DjmdPlaylist::find(conn, &self.playlist_id)? {
            Some(p) => p,
            None => return Err(Error::NotFound),
        };
        if playlist.attribute != 0 {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new("Cannot add a song to a DjmdPlaylist folder or smart list".to_string()),
            ));
        }
        if !DjmdContent::id_exists(conn, &self.content_id)? {
            return Err(Error::NotFound);
        }

        let id = DjmdSongPlaylist::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let now = Utc::now();
        let count = DjmdSongPlaylist::count(conn, &self.playlist_id)?;
        // Get next USN: We increment by 1 *before* moving others
        let usn: i32 = AgentRegistry::increment_local_usn(conn)?;

        let seq = if let Some(seq) = self.track_no {
            if seq <= 0 || seq > count + 1 {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new("Invalid sequence number".to_string()),
                ));
            }
            if seq < count + 1 {
                // Shift songs with seq higher than the new seq number by one
                // Shifting the other songs increments the USN by 1 for *all* moved songs
                let move_usn = AgentRegistry::increment_local_usn(conn)?;
                diesel::update(DjmdSongPlaylist::filter_children_gte(
                    &self.playlist_id,
                    seq,
                ))
                .set((
                    djmdSongPlaylist::track_no.eq(djmdSongPlaylist::track_no + 1),
                    djmdSongPlaylist::updated_at.eq(format_datetime(&now)),
                    djmdSongPlaylist::rb_local_usn.eq(move_usn),
                ))
                .execute(conn)?;
            };
            seq
        } else {
            count + 1
        };

        let item = DjmdSongPlaylist {
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
        let result = diesel::insert_into(djmdSongPlaylist::table)
            .values(item)
            .get_result(conn)?;
        Ok(result)
    }
}

/// Represents the `djmdPlaylist` table in the Rekordbox database in the form of a tree.
///
/// See [`DjmdPlaylist`] for the flat representation.
#[derive(Debug, Clone, PartialEq)]
pub struct DjmdPlaylistTreeItem {
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
    pub children: Vec<Rc<RefCell<Self>>>,
}

impl From<DjmdPlaylist> for DjmdPlaylistTreeItem {
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

impl Into<DjmdPlaylist> for DjmdPlaylistTreeItem {
    fn into(self) -> DjmdPlaylist {
        DjmdPlaylist {
            id: self.id,
            uuid: self.uuid,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at,
            updated_at: self.updated_at,
            seq: self.seq,
            name: self.name,
            attribute: self.attribute,
            parent_id: self.parent_id,
            image_path: self.image_path,
            smart_list: self.smart_list,
        }
    }
}
/// Recursively sort a playlist tree item and its children
fn sort_tree(item: &mut Rc<RefCell<DjmdPlaylistTreeItem>>) {
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
fn sort_tree_list(tree: &mut Vec<Rc<RefCell<DjmdPlaylistTreeItem>>>) {
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
