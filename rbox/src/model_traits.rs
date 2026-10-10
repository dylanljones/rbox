// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Database model operations and ordered collections shared by both database formats.
//!
//! [`SequenceStore`] maintains contiguous positions within a collection. [`OrderedModel`]
//! supplies record identity and persistence for batch plans; [`ModelTree`] adds ancestry
//! validation and [`ModelList`] restricts moves to one collection. [`SequenceVisible`] is
//! the compatibility facade for globally ordered visible rows.
//!
//! Insertion and move positions use the format's [`SequenceStore::START_SEQ`]. `None`
//! means append. A batch's destination is its first final position after removing the
//! selected members, and caller order is preserved. Duplicate move IDs are rejected.
//! Batch operations and concrete model mutations run in transactions; low-level shift
//! helpers belong inside those transactions. No-op moves preserve update metadata.
//!
//! [`TreeNode`] represents an in-memory snapshot. Sorting its child vectors does not
//! persist database changes; use the model operations to edit stored ordering.

use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

/// A trait representing a generic model with basic database operations.
///
/// This trait defines the structure for models that can be queried from a database.
/// It includes methods for retrieving all records, finding a specific record by its ID,
/// and checking if a record with a given ID exists.
///
/// # Type Parameters
/// * `Id`: The type of the ID of the model. Must implement `PartialEq` and `Debug`.
///
/// # Methods
/// * `all`: Queries all records in the database table.
/// * `find`: Queries a record by its primary key.
/// * `id_exists`: Checks if a record with the given primary key exists in the database table.
pub trait Model: Sized + Clone {
    /// The type of the ID of the model.
    type Id: ?Sized + PartialEq + Debug + ToOwned;

    /// Queries all records in the database table.
    ///
    /// # Arguments
    /// * `conn`: A mutable reference to the database connection.
    ///
    /// # Returns
    /// A `QueryResult` containing a vector of all records.
    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>>;

    /// Queries a record by its primary key.
    ///
    /// # Arguments
    /// * `conn`: A mutable reference to the database connection.
    /// * `id`: A reference to the ID of the record to find.
    ///
    /// # Returns
    /// A `QueryResult` containing an `Option` with the record if found.
    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>>;

    /// Checks if a record with the given primary key exists.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `id`: A reference to the ID of the record to check.
    ///
    /// # Returns
    /// A `QueryResult` containing a boolean indicating whether the record exists.
    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool>;
}

/// A trait for models that support update operations.
///
/// This trait extends the `Model` trait and provides methods for updating
/// individual and multiple records.
///
/// # Methods
/// * `update`: Updates an existing record in the database table.
/// * `update_all`: Updates multiple records in the database table.
pub trait ModelUpdate: Model {
    /// Updates an existing record in the database table.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    ///
    /// # Returns
    /// A `QueryResult` containing the updated record.
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self>;

    /// Updates multiple records in the database table.
    ///
    /// Apply updates in input order within one transaction. A failure rolls back the entire batch.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `items`: A vector of records to update.
    ///
    /// # Returns
    /// A `QueryResult` containing a vector of updated records.
    fn update_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self>> {
        conn.transaction(|conn| {
            let mut results = Vec::with_capacity(items.len());
            for item in items {
                results.push(item.update(conn)?);
            }
            Ok(results)
        })
    }
}

/// A trait for models that support deletion operations.
///
/// This trait extends the `Model` trait and provides methods for deleting
/// individual and multiple records.
///
/// # Methods
/// * `delete`: Deletes a record from the database table.
/// * `delete_all`: Deletes multiple records from the database table.
pub trait ModelDelete: Model {
    /// Deletes a record from the database table.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `id`: A reference to the ID of the record to delete.
    ///
    /// # Returns
    /// A `QueryResult` containing the number of rows affected.
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize>;

    /// Deletes multiple records from the database table.
    ///
    /// Delete in input order within one transaction. A failure rolls back the entire batch.
    /// Tree models override this to handle overlapping subtrees without deleting twice.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `ids`: A vector of references to the IDs of the records to delete.
    ///
    /// # Returns
    /// A `QueryResult` containing the total number of rows affected.
    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let mut result: usize = 0;
            for id in ids {
                result += Self::delete(conn, id)?;
            }
            Ok(result)
        })
    }
}

/// A trait for models that support insertion operations.
///
/// This trait provides methods for inserting individual and multiple records into the database.
///
/// # Type Parameters
/// * `Model`: The associated model type that implements the `Model` trait.
///
/// # Methods
/// * `insert`: Inserts a new record into the database table.
/// * `insert_all`: Inserts multiple records into the database table.
pub trait ModelInsert: Sized {
    /// The associated model type that implements the `Model` trait.
    type Model: Model;

    /// Inserts a new record into the database table.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    ///
    /// # Returns
    /// A `QueryResult` containing the inserted `Model` record.
    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model>;

    /// Inserts multiple records into the database table.
    ///
    /// Insert in input order within one transaction. A failure rolls back the entire batch.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `items`: A vector of records to insert.
    ///
    /// # Returns
    /// A `QueryResult` containing a vector of the inserted `Model` records.
    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        conn.transaction(|conn| {
            let mut results = Vec::with_capacity(items.len());
            for item in items {
                results.push(item.insert(conn)?);
            }
            Ok(results)
        })
    }
}

/// A database-backed ordered collection. A scope is a parent ID, playlist ID, or
/// `()` for a global filtered collection. Positions are contiguous from `START_SEQ`.
///
/// Mutating primitives must be called inside the transaction that changes the item.
/// Membership filters must be identical for counting, reading, shifting and renumbering.
pub trait SequenceStore {
    /// Collection identifier; retained as `ParentId` for compatibility with `TreeSeq`.
    type ParentId: ?Sized + PartialEq + Debug;

    /// First stored position, dictated by the database format.
    ///
    /// The main Rekordbox `master.db` database uses 1-based indexing for positions,
    /// while the One Library database uses 0-based indexing.
    const START_SEQ: i32 = 1;

    /// Check collection existence and whether it accepts this kind of item.
    fn is_valid_parent(conn: &mut SqliteConnection, parent: &Self::ParentId) -> QueryResult<bool>;

    /// Number of members of the collection.
    fn count_children(conn: &mut SqliteConnection, parent: &Self::ParentId) -> QueryResult<i32>;

    /// Positions in ascending order.
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent: &Self::ParentId,
    ) -> QueryResult<Vec<i32>>;

    /// Renumber by current position and stable item identity; only changed rows are written.
    fn reset_seq(conn: &mut SqliteConnection, parent: &Self::ParentId) -> QueryResult<usize>;

    /// Shift members in a half-open range by `delta`, including backend update metadata.
    fn shift_seq(
        conn: &mut SqliteConnection,
        parent: &Self::ParentId,
        range: std::ops::Range<i32>,
        delta: i32,
    ) -> QueryResult<usize>;

    /// Check for gaps, duplicates and incorrect starting positions.
    fn validate_seq_numbers(
        conn: &mut SqliteConnection,
        parent: &Self::ParentId,
    ) -> QueryResult<bool> {
        Ok(validate_seq_numbers(
            &Self::get_seq_numbers(conn, parent)?,
            Self::START_SEQ,
        ))
    }

    /// Open a slot before an insertion.
    fn increment_seq_gte(
        conn: &mut SqliteConnection,
        parent: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        let end = sequence_end(Self::START_SEQ, Self::count_children(conn, parent)?)?;
        Self::shift_seq(conn, parent, seq..end, 1)
    }
    /// Close a slot after a deletion or before moving an item to another collection.
    fn decrement_seq_gt(
        conn: &mut SqliteConnection,
        parent: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize> {
        let start = seq
            .checked_add(1)
            .ok_or_else(|| ordering_error("Position overflow"))?;
        Self::shift_seq(conn, parent, start..i32::MAX, -1)
    }

    /// Shift positions `start <= position < end` upwards.
    fn increment_seq_gte_lt(
        conn: &mut SqliteConnection,
        parent: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        Self::shift_seq(conn, parent, start..end, 1)
    }

    /// Shift positions `start < position <= end` downwards.
    fn decrement_seq_gt_lte(
        conn: &mut SqliteConnection,
        parent: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize> {
        let start = start
            .checked_add(1)
            .ok_or_else(|| ordering_error("Position overflow"))?;
        let end = end
            .checked_add(1)
            .ok_or_else(|| ordering_error("Position overflow"))?;
        Self::shift_seq(conn, parent, start..end, -1)
    }

    /// Validate an insertion and open its slot. `None` appends.
    fn update_seq_before_insert(
        conn: &mut SqliteConnection,
        parent: &Self::ParentId,
        seq: Option<i32>,
    ) -> QueryResult<(i32, usize)> {
        if !Self::is_valid_parent(conn, parent)? {
            return Err(ordering_error("Invalid collection"));
        }
        let end = sequence_end(Self::START_SEQ, Self::count_children(conn, parent)?)?;
        let seq = checked_position(seq, Self::START_SEQ, end)?;
        let n = if seq < end {
            Self::shift_seq(conn, parent, seq..end, 1)?
        } else {
            0
        };
        Ok((seq, n))
    }

    /// Prepare a move between collections. The caller must update the item in the same transaction.
    fn update_seq_before_move_to(
        conn: &mut SqliteConnection,
        old_parent: &Self::ParentId,
        parent: &Self::ParentId,
        old_seq: i32,
        seq: Option<i32>,
    ) -> QueryResult<Option<(i32, usize)>> {
        if old_parent == parent {
            return Ok(None);
        }
        if !Self::is_valid_parent(conn, parent)? {
            return Err(ordering_error("Invalid collection"));
        }
        let end = sequence_end(Self::START_SEQ, Self::count_children(conn, parent)?)?;
        let seq = checked_position(seq, Self::START_SEQ, end)?;
        let mut n = Self::decrement_seq_gt(conn, old_parent, old_seq)?;
        if seq < end {
            n += Self::shift_seq(conn, parent, seq..end, 1)?;
        }
        Ok(Some((seq, n)))
    }

    /// Prepare a move within a collection, interpreting the destination as its final position.
    fn update_seq_before_move_in(
        conn: &mut SqliteConnection,
        parent: &Self::ParentId,
        old_seq: i32,
        seq: Option<i32>,
    ) -> QueryResult<Option<(i32, usize)>> {
        let end = sequence_end(Self::START_SEQ, Self::count_children(conn, parent)?)?;
        checked_position(Some(old_seq), Self::START_SEQ, end - 1)?;
        let seq = checked_position(seq, Self::START_SEQ, end - 1)?;
        if seq == old_seq {
            return Ok(None);
        }
        let n = if seq > old_seq {
            Self::decrement_seq_gt_lte(conn, parent, old_seq, seq)?
        } else {
            Self::increment_seq_gte_lt(conn, parent, seq, old_seq)?
        };
        Ok(Some((seq, n)))
    }

    /// Prepare a move within or between collections.
    fn update_seq_before_move(
        conn: &mut SqliteConnection,
        old_parent: &Self::ParentId,
        parent: &Self::ParentId,
        old_seq: i32,
        seq: Option<i32>,
    ) -> QueryResult<Option<(i32, usize)>> {
        if old_parent == parent {
            Self::update_seq_before_move_in(conn, parent, old_seq, seq)
        } else {
            Self::update_seq_before_move_to(conn, old_parent, parent, old_seq, seq)
        }
    }
}

/// Compatibility name for the collection ordering interface.
pub use SequenceStore as TreeSeq;

/// Validate an ordered slice without overflowing on large positions.
pub fn validate_seq_numbers(seqs: &[i32], start: i32) -> bool {
    seqs.iter()
        .enumerate()
        .all(|(i, &seq)| i64::from(seq) == i64::from(start) + i as i64)
}

pub(crate) fn ordering_error(message: &str) -> Error {
    Error::DatabaseError(
        DatabaseErrorKind::RestrictViolation,
        Box::new(message.to_owned()),
    )
}

fn sequence_end(start: i32, count: i32) -> QueryResult<i32> {
    if count < 0 {
        return Err(ordering_error("Invalid collection size"));
    }
    start
        .checked_add(count)
        .ok_or_else(|| ordering_error("Position overflow"))
}

fn checked_position(seq: Option<i32>, start: i32, end: i32) -> QueryResult<i32> {
    let seq = seq.unwrap_or(end);
    if seq < start || seq > end {
        return Err(ordering_error("Invalid sequence number"));
    }
    Ok(seq)
}

/// Model hooks used to plan a whole reorder before writing any rows.
pub trait OrderedModel: Model + SequenceStore {
    /// Owned identity of this entry, including every column of a composite key.
    fn order_id(&self) -> <Self::Id as ToOwned>::Owned;
    /// Collection containing this entry.
    fn order_parent(&self) -> &Self::ParentId;
    /// Stored position.
    fn order_position(&self) -> i32;
    /// Read one collection in position order, breaking ties by the full primary key.
    fn ordered(conn: &mut SqliteConnection, parent: &Self::ParentId) -> QueryResult<Vec<Self>>;
    /// Apply a validated plan. Called inside a transaction; update metadata belongs here.
    fn write_order(
        conn: &mut SqliteConnection,
        changes: &[(&Self, &Self::ParentId, i32)],
    ) -> QueryResult<usize>;

    /// Repair a collection in stable position/key order, preserving metadata for unchanged rows.
    fn renumber(conn: &mut SqliteConnection, parent: &Self::ParentId) -> QueryResult<usize> {
        conn.transaction(|conn| {
            let rows = Self::ordered(conn, parent)?;
            let mut changes = Vec::new();
            for (index, row) in rows.iter().enumerate() {
                let index =
                    i32::try_from(index).map_err(|_| ordering_error("Position overflow"))?;
                let seq = sequence_end(Self::START_SEQ, index)?;
                if row.order_position() != seq {
                    changes.push((row, parent, seq));
                }
            }
            Self::write_order(conn, &changes)
        })
    }
}

/// Delete selected subtrees once, even when both an ancestor and its descendant are selected.
/// Duplicate IDs and missing records reject the whole batch before any deletion.
pub(crate) fn delete_tree_batch<M: ModelTree + ModelDelete>(
    conn: &mut SqliteConnection,
    ids: Vec<&M::Id>,
) -> QueryResult<usize> {
    conn.transaction(|conn| {
        let roots = tree_delete_roots::<M>(conn, &ids)?;
        let mut count = 0;
        for root in roots {
            count += M::delete(conn, root)?;
        }
        Ok(count)
    })
}

/// Validate a subtree deletion and remove selections already covered by an ancestor.
pub(crate) fn tree_delete_roots<'a, M: ModelTree>(
    conn: &mut SqliteConnection,
    ids: &[&'a M::Id],
) -> QueryResult<Vec<&'a M::Id>> {
    use std::borrow::Borrow;
    let mut roots = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        if ids[..i].contains(id) {
            return Err(ordering_error("Duplicate delete ID"));
        }
        let row = M::find(conn, id)?.ok_or(Error::NotFound)?;
        let mut parent = row.order_parent().to_owned();
        let mut seen: Vec<<M::Id as ToOwned>::Owned> = vec![(*id).to_owned()];
        let mut covered = false;
        loop {
            let key: &M::Id = parent.borrow();
            if seen.iter().any(|p| p.borrow() == key) {
                return Err(ordering_error("Cyclic ancestry"));
            }
            if ids.contains(&key) {
                covered = true;
            }
            let ancestor = M::find(conn, key)?;
            seen.push(parent);
            match ancestor {
                Some(ancestor) => parent = ancestor.order_parent().to_owned(),
                None => break,
            }
        }
        if !covered {
            roots.push(*id);
        }
    }

    Ok(roots)
}

/// Tree operations add ancestry validation to collection ordering.
pub trait ModelTree: OrderedModel<ParentId = <Self as Model>::Id> {
    /// Reject invalid folders, self-parenting, descendant targets and existing ancestry cycles.
    fn validate_destination(
        conn: &mut SqliteConnection,
        id: &Self::Id,
        parent: &Self::Id,
    ) -> QueryResult<()> {
        use std::borrow::Borrow;
        if !Self::is_valid_parent(conn, parent)? {
            return Err(ordering_error("Invalid parent"));
        }
        let mut current = parent.to_owned();
        let mut seen: Vec<<Self::Id as ToOwned>::Owned> = Vec::new();
        loop {
            let key: &Self::Id = current.borrow();
            if key == id || seen.iter().any(|p| p.borrow() == key) {
                return Err(ordering_error(
                    "A tree move would create or encounter a cycle",
                ));
            }
            let node = Self::find(conn, key)?;
            seen.push(current);
            match node {
                Some(node) => current = node.order_parent().to_owned(),
                None => return Ok(()), // The format's root sentinel is not a stored row.
            }
        }
    }

    /// Move one item atomically; `None` keeps the parent or appends within the destination.
    fn move_to(
        conn: &mut SqliteConnection,
        id: &Self::Id,
        parent_id: Option<&Self::Id>,
        seq: Option<i32>,
    ) -> QueryResult<usize>;

    /// Move distinct items as a block, preserving caller order. The start position is measured
    /// after removing selected destination members. Empty input is a no-op. The whole batch rolls
    /// back on failure, and unchanged rows retain their update metadata.
    fn move_all_to(
        conn: &mut SqliteConnection,
        ids: &[&Self::Id],
        parent_id: &Self::Id,
        start_seq: Option<i32>,
    ) -> QueryResult<usize> {
        conn.transaction(|conn| {
            if ids.is_empty() {
                return Ok(0);
            }
            for id in ids {
                Self::validate_destination(conn, id, parent_id)?;
            }
            reorder_batch::<Self>(conn, ids, Some(parent_id), start_seq)
        })
    }
}

/// Reordering of entries within a single collection.
pub trait ModelList: OrderedModel {
    /// Move one item atomically. `None` appends.
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize>;
    /// Move distinct entries as a block. All entries must belong to the same collection.
    /// Positions refer to the destination after removing the selected entries.
    fn move_all_to(
        conn: &mut SqliteConnection,
        ids: &[&Self::Id],
        start_seq: Option<i32>,
    ) -> QueryResult<usize> {
        conn.transaction(|conn| reorder_batch::<Self>(conn, ids, None, start_seq))
    }
}

fn reorder_batch<M: OrderedModel>(
    conn: &mut SqliteConnection,
    ids: &[&M::Id],
    target: Option<&M::ParentId>,
    seq: Option<i32>,
) -> QueryResult<usize> {
    use std::borrow::Borrow;
    if ids.is_empty() {
        return Ok(0);
    }
    let mut selected = Vec::with_capacity(ids.len());
    for (i, id) in ids.iter().enumerate() {
        if ids[..i].contains(id) {
            return Err(ordering_error("Duplicate move ID"));
        }
        selected.push(M::find(conn, id)?.ok_or(Error::NotFound)?);
    }
    let parent = target.unwrap_or_else(|| selected[0].order_parent());
    if target.is_none() && selected.iter().any(|row| row.order_parent() != parent) {
        return Err(ordering_error("List moves require a single collection"));
    }
    let mut scopes = vec![parent];
    for row in &selected {
        if !scopes.contains(&row.order_parent()) {
            scopes.push(row.order_parent());
        }
    }
    let mut groups = Vec::new();
    for scope in scopes {
        let mut members = M::ordered(conn, scope)?;
        members.retain(|row| !ids.contains(&row.order_id().borrow()));
        if scope == parent {
            let count =
                i32::try_from(members.len()).map_err(|_| ordering_error("Collection too large"))?;
            let end = sequence_end(M::START_SEQ, count)?;
            let position = checked_position(seq, M::START_SEQ, end)?;
            let index = (position - M::START_SEQ) as usize;
            members.splice(index..index, selected.iter().cloned());
        }
        groups.push((scope, members));
    }
    let mut changes = Vec::new();
    let mut moved = 0;
    for (scope, members) in &groups {
        for (i, row) in members.iter().enumerate() {
            let offset = i32::try_from(i).map_err(|_| ordering_error("Collection too large"))?;
            let position = sequence_end(M::START_SEQ, offset)?;
            if row.order_parent() != *scope || row.order_position() != position {
                if ids.contains(&row.order_id().borrow()) {
                    moved += 1;
                }
                changes.push((row, *scope, position));
            }
        }
    }
    if !changes.is_empty() {
        M::write_order(conn, &changes)?;
    }
    Ok(moved)
}

/// Compatibility facade for globally ordered visible rows.
pub trait SequenceVisible: SequenceStore<ParentId = ()> {
    /// Count visible members.
    fn count_visible(conn: &mut SqliteConnection) -> QueryResult<i32> {
        Self::count_children(conn, &())
    }
    /// Read visible positions in order.
    fn get_seq_numbers(conn: &mut SqliteConnection) -> QueryResult<Vec<i32>> {
        <Self as SequenceStore>::get_seq_numbers(conn, &())
    }
    /// Renumber visible members only.
    fn reset_seq(conn: &mut SqliteConnection) -> QueryResult<usize> {
        <Self as SequenceStore>::reset_seq(conn, &())
    }
    /// Validate visible positions.
    fn validate_seq_numbers(conn: &mut SqliteConnection) -> QueryResult<bool> {
        <Self as SequenceStore>::validate_seq_numbers(conn, &())
    }
    /// Open a position in the visible list.
    fn increment_seq_before_insert(conn: &mut SqliteConnection, seq: i32) -> QueryResult<usize> {
        Self::increment_seq_gte(conn, &(), seq)
    }
    /// Prepare an insertion. Zero retains the legacy append convention for this one-based list.
    fn update_seq_before_insert(conn: &mut SqliteConnection, seq: i32) -> QueryResult<i32> {
        Ok(<Self as SequenceStore>::update_seq_before_insert(
            conn,
            &(),
            if seq == 0 { None } else { Some(seq) },
        )?
        .0)
    }
    /// Close the position of a removed or hidden member.
    fn decrement_seq_after_delete(conn: &mut SqliteConnection, seq: i32) -> QueryResult<usize> {
        Self::decrement_seq_gt(conn, &(), seq)
    }
}

pub type NodeRef<T> = Rc<RefCell<T>>;

/// A trait for models that can be used as tree nodes.
pub trait TreeNode: From<Self::Model> {
    /// The reference model.
    type Model: Model;
    /// The unique identifier of the node.
    type Id: ?Sized + Eq + Ord + Debug + Clone + Hash;

    /// Returns the unique identifier of the node.
    fn id(&self) -> Self::Id;
    /// Returns the sequence number (ordering) of the node.
    fn sequence(&self) -> i32;
    /// Returns the parent identifier of the node.
    fn parent_id(&self) -> Self::Id;
    /// Returns a reference to the vector containing the child references.
    fn children(&self) -> &Vec<NodeRef<Self>>;
    /// Returns a mutable reference to the vector containing the child references.
    fn children_mut(&mut self) -> &mut Vec<NodeRef<Self>>;

    /// Sort children by position, breaking ties by their stable identifiers.
    fn sort_children(&mut self) {
        self.children_mut().sort_by(|a, b| {
            let a_seq = a.borrow().sequence();
            let b_seq = b.borrow().sequence();
            a_seq
                .cmp(&b_seq)
                .then_with(|| a.borrow().id().cmp(&b.borrow().id()))
        });
    }
}

/// Sort the children of a tree node recursively.
fn sort_tree_ref<N: TreeNode>(item: &mut NodeRef<N>) {
    let mut item_borrow = item.borrow_mut();
    item_borrow.sort_children();
    for child in item_borrow.children_mut().iter_mut() {
        sort_tree_ref(child);
    }
}

/// Sorts a list of tree nodes recursively by their sequence number.
fn sort_tree_list<N: TreeNode>(items: &mut Vec<NodeRef<N>>) {
    items.sort_by(|a, b| {
        let a_seq = a.borrow().sequence();
        let b_seq = b.borrow().sequence();
        a_seq
            .cmp(&b_seq)
            .then_with(|| a.borrow().id().cmp(&b.borrow().id()))
    });
    for item in items.iter_mut() {
        sort_tree_ref(item);
    }
}

/// Constructs a tree of nodes from a list of models.
pub(crate) fn build_tree_list<M, N>(models: Vec<M>, root_id: N::Id) -> Vec<NodeRef<N>>
where
    N: TreeNode<Model = M>,
    M: Model,
{
    let nodes: Vec<N> = models.into_iter().map(N::from).collect();

    let mut map: HashMap<N::Id, NodeRef<N>> = HashMap::new();
    let mut roots = Vec::new();

    // Populate the map with nodes
    for node in nodes.into_iter() {
        let id = node.id();
        let parent_id = node.parent_id();

        let item: NodeRef<N> = Rc::new(RefCell::new(node));
        if parent_id == root_id {
            roots.push(item.clone());
        }
        map.insert(id.clone(), item);
    }

    // Build the tree structure
    for node in map.values() {
        let parent_id = node.borrow().parent_id();
        if let Some(parent_node) = map.get(&parent_id) {
            parent_node.borrow_mut().children_mut().push(node.clone());
        }
    }

    // Sort the entire tree
    sort_tree_list(&mut roots);

    roots
}
