// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use diesel::prelude::*;
use diesel::result::Error;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;
use std::collections::{HashSet, VecDeque};

use super::content::Content;
use crate::enums::MyTagType;
use crate::schema::{content, myTag, myTag_content};
use rbox_db::{
    delete_tree_batch, ordering_error, Model, ModelDelete, ModelInsert, ModelTree, OrderedModel,
    TreeSeq,
};

/// Represents the `myTag` table in the Rekordbox One Library database.
///
/// This struct maps to the `myTag` table in the One Library export database.
/// It stores information about my-tags, including metadata such as sequence, name
/// attributes, and parent relationships.
///
/// # Referenced by
/// * [`MyTag`] via `parent_id` foreign key.
/// * [`MyTagContent`] via `my_tag_id` foreign key.
///
/// # References
/// * [`MyTag`] via `parent_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = myTag)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(MyTag, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct MyTag {
    /// The unique identifier of the tag.
    pub id: i32,
    /// Position among siblings, starting at zero.
    pub seq: i32,
    /// The name of the tag.
    pub name: String,
    /// The tag type, either list (`0`) or a folder (`1`).
    pub attribute: MyTagType,
    /// The ID of the parent [`MyTag`], `0` for top-level records.
    pub parent_id: i32,
}

impl Model for MyTag {
    type Id = i32;

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query()
            .order((myTag::parent_id, myTag::seq, myTag::id))
            .load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }
}

impl ModelDelete for MyTag {
    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        delete_tree_batch::<Self>(conn, ids)
    }

    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        conn.transaction(|conn| {
            // Vec of all deleted myTag ids
            let mut deleted_ids = vec![*id];

            // Delete the record
            let parent_id: i32 = diesel::delete(myTag::table.find(id))
                .returning(myTag::parent_id)
                .get_result(conn)?;

            // Reorder the seq numbers of tags left in the parent
            Self::reset_seq(conn, &parent_id)?;

            // Remove all child myTag records recursively
            let mut parent_ids = VecDeque::from(vec![*id]);
            while let Some(parent_id) = parent_ids.pop_front() {
                // Delete children
                let deleted: Vec<i32> =
                    diesel::delete(myTag::table.filter(myTag::parent_id.eq(parent_id)))
                        .returning(myTag::id)
                        .get_results(conn)?;
                deleted_ids.extend(deleted.clone());
                // Add children to the queue
                for deleted_id in deleted {
                    parent_ids.push_back(deleted_id);
                }
            }

            // Remove any myTag_content records that are associated with the deleted myTags
            MyTagContent::delete_by_my_tag_ids(conn, &deleted_ids)?;

            Ok(deleted_ids.len())
        })
    }
}

impl ModelTree for MyTag {
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
            diesel::update(myTag::table.find(id))
                .set((myTag::seq.eq(seq), myTag::parent_id.eq(parent_id)))
                .execute(conn)
        })
    }
}

impl MyTag {
    /// Queries the records from the `myTag` table by their `parent_id`.
    pub fn by_parent_id(conn: &mut SqliteConnection, parent_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(myTag::parent_id.eq(parent_id))
            .order(myTag::seq)
            .load(conn)
    }

    /// Queries the records from the `content` table associated with the given `myTag`.
    pub fn get_contents(conn: &mut SqliteConnection, id: i32) -> QueryResult<Vec<Content>> {
        content::table
            .inner_join(myTag_content::table.on(content::id.eq(myTag_content::content_id)))
            .filter(myTag_content::my_tag_id.eq(id))
            .select(Content::as_select())
            .load(conn)
    }

    /// Queries all records from the `myTag` table their associated `content_id` via `myTag_content`
    pub fn by_content_id(conn: &mut SqliteConnection, cid: i32) -> QueryResult<Vec<Self>> {
        myTag::table
            .inner_join(myTag_content::table.on(myTag::id.eq(myTag_content::my_tag_id)))
            .filter(myTag_content::content_id.eq(cid))
            .select(Self::as_select())
            .load(conn)
    }

    /// Tries to query a record from the `myTag` table by its node path in the tree.
    ///
    /// Returns `None` if no unique node can be found for the given `path`.
    pub fn find_by_path(conn: &mut SqliteConnection, path: Vec<&str>) -> QueryResult<Option<Self>> {
        let mut parent_ids: HashSet<i32> = HashSet::new();
        parent_ids.insert(0);
        for name in path {
            let mut new_parent_ids: HashSet<i32> = HashSet::new();
            for parent_id in parent_ids.iter() {
                let result: Option<i32> = myTag::table
                    .filter(myTag::parent_id.eq(parent_id))
                    .filter(myTag::name.eq(name))
                    .select(myTag::id)
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

    /// Returns the myTag type (`attribute`) of a record in the `myTag` table or `None` if not found.
    pub fn get_attribute(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<MyTagType>> {
        if id == 0 {
            return Ok(Some(MyTagType::Folder));
        }
        let value: Option<i32> = myTag::table
            .find(id)
            .select(myTag::attribute)
            .get_result(conn)
            .optional()?;
        Ok(value.map(|v| MyTagType::try_from(v).unwrap()))
    }

    /// Returns `true` if the record in the `myTag` table exists and is a myTag, `false` otherwise.
    pub fn is_tag(conn: &mut SqliteConnection, id: i32) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == MyTagType::List),
            None => Ok(false),
        }
    }

    /// Returns `true` if the record in the `myTag` table exists and is a folder, `false` otherwise.
    pub fn is_folder(conn: &mut SqliteConnection, id: i32) -> QueryResult<bool> {
        match Self::get_attribute(conn, id)? {
            Some(attr) => Ok(attr == MyTagType::Folder),
            None => Ok(false),
        }
    }

    /// Set the name of a record in the `myTag` table.
    pub fn rename(conn: &mut SqliteConnection, id: i32, name: &str) -> QueryResult<usize> {
        diesel::update(myTag::table.find(id))
            .set(myTag::name.eq(name))
            .execute(conn)
    }
}

impl TreeSeq for MyTag {
    type ParentId = i32;
    const START_SEQ: i32 = 0;
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        Self::is_folder(conn, *parent_id)
    }
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32> {
        let count: i64 = myTag::table
            .filter(myTag::parent_id.eq(parent_id))
            .count()
            .get_result(conn)?;
        i32::try_from(count).map_err(|_| ordering_error("Collection too large"))
    }
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>> {
        myTag::table
            .filter(myTag::parent_id.eq(parent_id))
            .order(myTag::seq)
            .select(myTag::seq)
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
            myTag::table
                .filter(myTag::parent_id.eq(parent_id))
                .filter(myTag::seq.ge(range.start))
                .filter(myTag::seq.lt(range.end)),
        )
        .set(myTag::seq.eq(myTag::seq + delta))
        .execute(conn)
    }
}

impl OrderedModel for MyTag {
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
        myTag::table
            .filter(myTag::parent_id.eq(parent_id))
            .order((myTag::seq, myTag::id))
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
            count += diesel::update(myTag::table.find(row.id))
                .set((myTag::seq.eq(seq), myTag::parent_id.eq(*parent_id)))
                .execute(conn)?;
        }
        Ok(count)
    }
}

/// Represents a new record insertale to the `myTag` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox_onelib::models::NewMyTag;
///
/// let new = NewMyTag::tag("Name").seq(2);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = myTag)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewMyTag {
    /// Insertion position in the collection; `None` appends.
    pub seq: Option<i32>,
    /// The name of the tag.
    pub name: String,
    /// The tag type, either list (`0`) or a folder (`1`).
    pub attribute: MyTagType,
    /// The ID of the parent [`MyTag`], `0` for top-level records.
    pub parent_id: i32,
}

impl ModelInsert for NewMyTag {
    type Model = MyTag;

    fn insert(mut self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        conn.transaction(|conn| {
            let seq = self.seq;
            let (seq, _n) = MyTag::update_seq_before_insert(conn, &self.parent_id, seq)?;
            self.seq = Some(seq);
            diesel::insert_into(myTag::table)
                .values(&self)
                .get_result(conn)
        })
    }
}

impl NewMyTag {
    /// Creates a new myTag record with the required fields.
    pub fn new<S: Into<String>>(name: S, attribute: MyTagType) -> Self {
        Self {
            name: name.into(),
            attribute,
            ..Default::default()
        }
    }

    /// Creates a new myTag record as a myTag (`attribute=0)`.
    pub fn tag<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: MyTagType::List,
            ..Default::default()
        }
    }

    /// Creates a new myTag record as a folder (`attribute=1)`.
    pub fn folder<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribute: MyTagType::Folder,
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
}

/// Represents the `myTag_content` table in the Rekordbox One Library database.
///
/// This struct maps to the `myTag_content` table in the One Library export database.
/// It stores information about the relationship between songs and myTags.
///
/// This struct also is insertable into the `myTag_content` table.
///
/// # References
/// * [`MyTag`] via `my_tag_id` foreign key.
/// * [`Content`] via `content_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable, Associations)]
#[diesel(table_name = myTag_content)]
#[diesel(primary_key(my_tag_id, content_id))]
#[diesel(belongs_to(MyTag, foreign_key = my_tag_id))]
#[diesel(belongs_to(Content, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct MyTagContent {
    /// The ID of the associated [`MyTag`].
    pub my_tag_id: i32,
    /// The ID of the associated [`Content`].
    pub content_id: i32,
}

impl Model for MyTagContent {
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

impl ModelDelete for MyTagContent {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        conn.transaction(|conn| diesel::delete(myTag_content::table.find(*id)).execute(conn))
    }
}

impl ModelInsert for MyTagContent {
    type Model = MyTagContent;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        conn.transaction(|conn| {
            if !Content::id_exists(conn, &self.content_id)? {
                return Err(Error::NotFound);
            }
            diesel::insert_into(myTag_content::table)
                .values(&self)
                .get_result(conn)
        })
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        conn.transaction(|conn| {
            for item in &items {
                if !Content::id_exists(conn, &item.content_id)? {
                    return Err(Error::NotFound);
                }
            }
            diesel::insert_into(myTag_content::table)
                .values(&items)
                .get_results(conn)
        })
    }
}

impl MyTagContent {
    /// Creates a new `myTag_content` record with the required fields.
    pub fn new(my_tag_id: i32, content_id: i32) -> Self {
        Self {
            my_tag_id,
            content_id,
        }
    }

    /// Queries all records from the `myTag_content` table by its `my_tag_id`
    pub fn by_my_tag_id(conn: &mut SqliteConnection, my_tag_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(myTag_content::my_tag_id.eq(my_tag_id))
            .load(conn)
    }

    /// Queries all records from the `myTag_content` table by its `content_id`
    pub fn by_content_id(conn: &mut SqliteConnection, content_id: i32) -> QueryResult<Vec<Self>> {
        Self::query()
            .filter(myTag_content::content_id.eq(content_id))
            .load(conn)
    }

    /// Creates a filter for records by `my_tag_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_my_tag_id(my_tag_id: i32) -> _ {
        myTag_content::table.filter(myTag_content::my_tag_id.eq(my_tag_id))
    }

    /// Creates a filter for records by multiple `my_tag_id`s.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_my_tag_ids(my_tag_ids: &[i32]) -> _ {
        myTag_content::table.filter(myTag_content::my_tag_id.eq_any(my_tag_ids))
    }

    /// Creates a filter for records by `content_id`.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_content_id(content_id: i32) -> _ {
        myTag_content::table.filter(myTag_content::content_id.eq(content_id))
    }

    /// Creates a filter for records by multiple `content_id`s.
    #[diesel::dsl::auto_type(no_type_alias)]
    fn filter_by_content_ids(content_ids: &[i32]) -> _ {
        myTag_content::table.filter(myTag_content::content_id.eq_any(content_ids))
    }

    /// Deletes all record from the `myTag_content` table referencing a `myTag` record.
    pub fn delete_by_my_tag_id(conn: &mut SqliteConnection, my_tag_id: i32) -> QueryResult<usize> {
        conn.transaction(|conn| diesel::delete(Self::filter_by_my_tag_id(my_tag_id)).execute(conn))
    }

    /// Deletes all record from the `myTag_content` table referencing multiple `myTag` records.
    pub fn delete_by_my_tag_ids(
        conn: &mut SqliteConnection,
        my_tag_ids: &[i32],
    ) -> QueryResult<usize> {
        conn.transaction(|conn| {
            diesel::delete(Self::filter_by_my_tag_ids(my_tag_ids)).execute(conn)
        })
    }

    /// Deletes all record from the `myTag_content` table referencing a `content` record.
    pub fn delete_by_content_id(
        conn: &mut SqliteConnection,
        content_id: i32,
    ) -> QueryResult<usize> {
        conn.transaction(|conn| {
            diesel::delete(Self::filter_by_content_id(content_id)).execute(conn)
        })
    }

    /// Deletes all record from the `myTag_content` table referencing a `content` record.
    pub fn delete_by_content_ids(
        conn: &mut SqliteConnection,
        content_ids: &[i32],
    ) -> QueryResult<usize> {
        conn.transaction(|conn| {
            diesel::delete(Self::filter_by_content_ids(content_ids)).execute(conn)
        })
    }
}
