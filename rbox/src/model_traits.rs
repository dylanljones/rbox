// Author: Dylan Jones
// Date:   2025-11-10

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
    type Id: ?Sized + PartialEq + Debug;

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
    /// By default, this method uses a naive loop to update each record individually.
    /// It can be overridden for better performance.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `items`: A vector of records to update.
    ///
    /// # Returns
    /// A `QueryResult` containing a vector of updated records.
    fn update_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self>> {
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            results.push(item.update(conn)?);
        }
        Ok(results)
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
    /// By default, this method uses a naive loop to delete each record individually.
    /// It can be overridden for better performance.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `ids`: A vector of references to the IDs of the records to delete.
    ///
    /// # Returns
    /// A `QueryResult` containing the total number of rows affected.
    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let mut result: usize = 0;
        for id in ids {
            result += Self::delete(conn, id)?;
        }
        Ok(result)
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
    /// By default, this method uses a naive loop to insert each record individually.
    /// It can be overridden for better performance.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `items`: A vector of records to insert.
    ///
    /// # Returns
    /// A `QueryResult` containing a vector of the inserted `Model` records.
    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            results.push(item.insert(conn)?);
        }
        Ok(results)
    }
}

/// A trait for models that support tree operations.
///
/// This trait extends the `Model` trait and provides methods for moving records between parents.
pub trait ModelTree: Model + TreeSeq {
    /// Moves a record to a new sequence number and/or parent.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `id`: A reference to the ID of the record to move.
    /// - `parent_id`: A reference to the ID of the new parent. If `None`, the record is moved to
    ///    the given sequence number inside the same parent.
    /// - `seq`: The new sequence number. If `None`, the record is moved to the end of the parent.
    ///
    /// # Returns
    /// A `QueryResult` containing the number of rows affected.
    fn move_to(
        conn: &mut SqliteConnection,
        id: &Self::Id,
        parent_id: Option<&Self::Id>,
        seq: Option<i32>,
    ) -> QueryResult<usize>;

    /// Moves multiple records to a new sequence number and/or parent.
    ///
    /// By default, this method uses a naive loop to move each record individually.
    /// It can be overridden for better performance.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `id`: A reference to the ID of the record to move.
    /// - `parent_id`: A reference to the ID of the new parent. If `None`, the records are moved to
    ///    the given start sequence number inside the same parent.
    /// - `start_seq`: The new starting sequence number to insert the records.
    ///    If `None`, the records are moved to the end of the parent.
    ///
    /// # Returns
    /// A `QueryResult` containing the number of rows affected.
    fn move_all_to(
        conn: &mut SqliteConnection,
        ids: &[&Self::Id],
        parent_id: &Self::Id,
        start_seq: Option<i32>,
    ) -> QueryResult<usize> {
        let mut result: usize = 0;
        let mut seq: Option<i32> = start_seq;
        for id in ids {
            result += Self::move_to(conn, id, Some(parent_id), seq)?;
            if seq.is_some() {
                // Increment seq for the next move if it is not none
                seq = Some(seq.unwrap() + 1);
            }
        }
        Ok(result)
    }
}

/// A trait for models that support list operations.
///
/// This trait extends the `Model` trait and provides methods for moving records inside lists
pub trait ModelList: Model + TreeSeq {
    /// Moves a record to a new sequence number.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `id`: A reference to the ID of the record to move.
    /// - `seq`: The new sequence number. If `None`, the record is moved to the end of the parent.
    ///
    /// # Returns
    /// A `QueryResult` containing the number of rows affected.
    fn move_to(conn: &mut SqliteConnection, id: &Self::Id, seq: Option<i32>) -> QueryResult<usize>;

    /// Moves multiple records to a new sequence number.
    ///
    /// By default, this method uses a naive loop to move each record individually.
    /// It can be overridden for better performance.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `id`: A reference to the ID of the record to move.
    /// - `start_seq`: The new starting sequence number to insert the records.
    ///    If `None`, the records are moved to the end of the parent.
    ///
    /// # Returns
    /// A `QueryResult` containing the number of rows affected.
    fn move_all_to(
        conn: &mut SqliteConnection,
        ids: &[&Self::Id],
        start_seq: Option<i32>,
    ) -> QueryResult<usize> {
        let mut result: usize = 0;
        let mut seq: Option<i32> = start_seq;
        for id in ids {
            result += Self::move_to(conn, id, seq)?;
            if seq.is_some() {
                // Increment seq for the next move if it is not none
                seq = Some(seq.unwrap() + 1);
            }
        }
        Ok(result)
    }
}

/// Checks if the given vector of numbers is a valid sequence order
///
/// A valid order is when the numbers start from `start` and are incremental:
/// ```
/// start, start + 1, start + 2, ...
/// ```
pub fn validate_seq_numbers(seqs: &[i32], start: i32) -> bool {
    for (i, seq) in seqs.iter().enumerate() {
        let expected = i as i32 + start;
        if expected != *seq {
            return false;
        }
    }
    true
}

/// A trait for models that implement a tree-like structure with reordering via sequence numbers.
///
/// The sequence number defines the order of the model in a list or collection.
///
/// This trait provides methods to manage and validate the sequence numbers of child models
/// within a parent model. It includes operations for resetting, incrementing, decrementing,
/// and validating sequence numbers, as well as handling insert and move operations.
///
/// # Usage
/// Implement this trait for models that require tree-like structures with sequence-based ordering.
/// The methods provided can be used to manage and maintain the integrity of sequence numbers
/// during operations like insertion, deletion, and reordering.
pub trait TreeSeq {
    /// The type of the ID of the model.
    type ParentId: ?Sized + PartialEq + Debug;
    /// The start sequence number (usually 1).
    const START_SEQ: i32 = 1;

    /// Checks if the given id is a valid parent node.
    fn is_valid_parent(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool>;

    /// Returns the number of child models inside the parent.
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<i32>;

    /// Returns the sequence numbers of all child models inside the parent.
    ///
    /// This is mostly used for validating the sequence numbers.
    fn get_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<Vec<i32>>;

    /// Validate the sequence numbers of all child models inside the parent.
    ///
    /// Checks if the sequence numbers start from `START_SEQ` and are incremental
    fn validate_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
    ) -> QueryResult<bool> {
        let seq_numbers = Self::get_seq_numbers(conn, parent_id)?;
        Ok(validate_seq_numbers(&seq_numbers, Self::START_SEQ))
    }

    /// Reset the sequence numbers of all models inside a parent.
    ///
    /// The sequence numbers are reset to the *current* order of the models inside their parent
    /// starting at 1:
    ///
    /// # Returns
    /// The number of affected rows.
    ///
    /// ```
    /// Old: 1, 2, 4, 5, 7, 8
    /// New: 1, 2, 3, 4, 5, 6
    /// ```
    ///
    /// This is helpful when deleting records or for fixing the sequence after an invalid state.
    /// When adding or moving records, the sequence numbers should be set explicitly.
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::ParentId) -> QueryResult<usize>;

    /// Increment the sequence numbers of models inside a parent greater equal the given position.
    ///
    /// `s -> s + 1 | s >= seq`
    ///
    /// This operation generally is used *before* inserting a new record with the given sequence
    /// number.
    ///
    /// # Returns
    /// The number of affected rows.
    ///
    /// # Example
    /// ```
    /// seq: 3
    /// Old: 1, 2, 3, 4
    /// New: 1, 2, 4, 5
    /// ```
    fn increment_seq_gte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize>;

    /// Decrement the sequence numbers of models inside a parent greater than the given position.
    ///
    /// `s -> s - 1 | s > seq`
    ///
    /// This operation generally is used *after* deleting a record with the given sequence number.
    ///
    /// # Returns
    /// The number of affected rows.
    ///
    /// # Example
    /// ```
    /// seq: 3
    /// Old: 1, 2, 4
    /// New: 1, 2, 3
    /// ```
    fn decrement_seq_gt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: i32,
    ) -> QueryResult<usize>;

    /// Increments the sequence numbers of models inside a parent between start and end.
    ///
    /// `s -> s + 1 | start <= s < end`
    ///
    /// This operation generally is used *before* moving a record *up* in the same parent.
    ///
    /// # Returns
    /// The number of affected rows.
    ///
    /// # Example
    /// ```
    /// start: 3, end: 6
    /// Old: 1, 2, 3, 4, 5, 6, 7
    /// New: 1, 2, 4, 5, 6, 6, 7
    /// ```
    fn increment_seq_gte_lt(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize>;

    /// Decrement the sequence numbers of models inside a parent between start and end.
    ///
    /// `s -> s - 1 | start < s <= end`
    ///
    /// This operation generally is used *before* moving a record *down* in the same parent.
    ///
    /// # Returns
    /// The number of affected rows.
    ///
    /// # Example
    /// ```
    /// start: 3, end: 6
    /// Old: 1, 2, 3, 4, 5, 6, 7
    /// New: 1, 2, 3, 3, 4, 5, 7
    /// ```
    fn decrement_seq_gt_lte(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        start: i32,
        end: i32,
    ) -> QueryResult<usize>;

    /// Update the sequence number for an insert operation.
    ///
    /// The sequence numbers of records inside a parent with a sequence number greater equal
    /// the new sequence number are incremented by one *if* the new sequence number is valid
    /// and is not the last element:
    ///
    /// Returns the updated new sequence number and the number of rows updated by `increment_seq_gte`.
    fn update_seq_before_insert(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        seq: Option<i32>,
    ) -> QueryResult<(i32, usize)> {
        let mut n: usize = 0;
        // Check parent
        if !Self::is_valid_parent(conn, &parent_id)? {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new(format!("Invalid parent ID: {:?}", parent_id)),
            ));
        }
        let count = Self::count_children(conn, parent_id)?;
        let seq = if let Some(seq) = seq {
            if seq < Self::START_SEQ || seq > Self::START_SEQ + count {
                // Invalid seq, return error
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new("Invalid sequence number".to_string()),
                ));
            } else if seq < Self::START_SEQ + count {
                // Shift other items up
                n = Self::increment_seq_gte(conn, parent_id, seq)?;
            } // Otherwise seq is valid and last
            seq
        } else {
            // Add as last child in parent (seq = count + 1)
            count + Self::START_SEQ
        };
        Ok((seq, n))
    }

    /// Update the sequence number for a move operation to a different parent.
    ///
    /// The sequence numbers of records inside the old and new parent are updated accordingly.
    ///
    /// Returns the updated new sequence number and the number of rows affected by
    /// the sequence number shifts.
    fn update_seq_before_move_to(
        conn: &mut SqliteConnection,
        old_parent_id: &Self::ParentId,
        parent_id: &Self::ParentId,
        old_seq: i32,
        seq: Option<i32>,
    ) -> QueryResult<Option<(i32, usize)>> {
        // *Note*: Moving other records increments USN by 1 for all changes
        if parent_id == old_parent_id {
            return Ok(None);
        }
        // Check parent
        if !Self::is_valid_parent(conn, parent_id)? {
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new(format!("Invalid parent ID: {:?}", parent_id)),
            ));
        }
        // check sequence
        let count = Self::count_children(conn, parent_id)?;
        let max_seq = Self::START_SEQ + count;
        let seq = if let Some(seq) = seq {
            if seq < Self::START_SEQ || seq > max_seq {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new(format!(
                        "Invalid sequence number: {} (count: {})",
                        seq, count
                    )),
                ));
            }
            seq
        } else {
            max_seq // Insert at the end by default
        };
        // Update seq of records in the old parent:
        // Decrease seq of records above old seq: s > old_seq
        let mut n = Self::decrement_seq_gt(conn, &old_parent_id, old_seq)?;
        if seq < max_seq {
            // Move to body in the new parent
            // Increase seq of records below seq: s >= seq
            n += Self::increment_seq_gte(conn, &parent_id, seq)?;
        };

        Ok(Some((seq, n)))
    }

    /// Update the sequence number for a move operation inside the same parent.
    ///
    /// The sequence numbers of records inside the parent are updated accordingly.
    ///
    /// Returns the updated new sequence number and the number of rows affected by
    /// the sequence number shifts.
    fn update_seq_before_move_in(
        conn: &mut SqliteConnection,
        parent_id: &Self::ParentId,
        old_seq: i32,
        seq: Option<i32>,
    ) -> QueryResult<Option<(i32, usize)>> {
        // check sequence
        let count = Self::count_children(conn, parent_id)?;
        let max_seq = Self::START_SEQ + count - 1;
        let seq = if let Some(seq) = seq {
            if seq < Self::START_SEQ || seq > max_seq {
                return Err(Error::DatabaseError(
                    DatabaseErrorKind::RestrictViolation,
                    Box::new(format!(
                        "Invalid sequence number: {} (count: {})",
                        seq, count
                    )),
                ));
            }
            seq
        } else {
            max_seq // Insert at the end by default
        };
        if seq == old_seq {
            return Ok(None);
        }
        // Handle sequence numbers
        let mut n: usize = 0;
        if seq > old_seq {
            // Seq is increased (move down in parent):
            // Decrease seq of all other records between old seq and seq: old_seq < x <= seq
            n += Self::decrement_seq_gt_lte(conn, &parent_id, old_seq, seq)?;
        } else if seq < old_seq {
            // Seq is decreased (moved up in parent):
            // Increase seq of all records between old seq and seq: seq <= x < old_seq
            n += Self::increment_seq_gte_lt(conn, &parent_id, seq, old_seq)?;
        };
        Ok(Some((seq, n)))
    }

    /// Update the sequence number for a general move operation.
    ///
    /// The sequence numbers of records inside all involved parents are updated accordingly.
    ///
    /// Returns the updated new sequence number and the number of rows affected by
    /// the sequence number shifts.
    ///
    /// # See Also
    /// * [Self::update_seq_before_move_to]: Handles move operations to a different parent.
    /// * [Self::update_seq_before_move_in]: Handles move operations inside the same parent.
    fn update_seq_before_move(
        conn: &mut SqliteConnection,
        old_parent_id: &Self::ParentId,
        parent_id: &Self::ParentId,
        old_seq: i32,
        seq: Option<i32>,
    ) -> QueryResult<Option<(i32, usize)>> {
        if old_parent_id == parent_id {
            Self::update_seq_before_move_in(conn, parent_id, old_seq, seq)
        } else {
            Self::update_seq_before_move_to(conn, old_parent_id, parent_id, old_seq, seq)
        }
    }
}

/// A trait for models that implement reordering via a sequence number for *visible* models.
///
/// The sequence number defines the order of the model in a list or collection.
/// It is one-based (i.e., starts from 1).
pub trait SequenceVisible {
    /// The start sequence number (usually 1).
    const START_SEQ: i32 = 1;

    /// Returns the number visible models.
    fn count_visible(conn: &mut SqliteConnection) -> QueryResult<i32>;

    /// Returns the sequence numbers of all visible models.
    ///
    /// This is mostly used for validating the sequence numbers.
    fn get_seq_numbers(conn: &mut SqliteConnection) -> QueryResult<Vec<i32>>;

    /// Reset the sequence numbers of all visible models.
    ///
    /// The sequence numbers of visible models are reset to the *current* order starting at 1:
    ///
    /// ```
    /// Old: 1, 2, 4, 5, 7, 8
    /// New: 1, 2, 3, 4, 5, 6
    /// ```
    ///
    /// This is helpful when deleting records or for fixing the sequence after an invalid state.
    /// When adding or moving records, the sequence numbers should be set explicitly.
    fn reset_seq(conn: &mut SqliteConnection) -> QueryResult<usize>;

    /// Increment the sequence numbers of visible models *before* an insert operation.
    ///
    /// The sequence numbers of records inside a parent with a sequence number greater equal
    /// the new sequence number are incremented by one:
    ///
    /// ```
    /// seq: 3
    /// Old: 1, 2, 3, 4
    /// New: 1, 2, 4, 5
    /// ```
    ///
    /// This operation is used *before* inserting a new record at a specific position.
    fn increment_seq_before_insert(conn: &mut SqliteConnection, seq: i32) -> QueryResult<usize>;

    /// Validate the sequence numbers of all visible models.
    ///
    /// Checks if the sequence numbers start from `START_SEQ` and are incremental
    fn validate_seq_numbers(conn: &mut SqliteConnection) -> QueryResult<bool> {
        let seq_numbers = Self::get_seq_numbers(conn)?;
        for (i, seq) in seq_numbers.iter().enumerate() {
            if (i as i32) + Self::START_SEQ != *seq {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Update the sequence number *before* an insert operation.
    ///
    /// The sequence numbers of visible records with a sequence number greater equal
    /// the new sequence number are incremented by one *if* the new sequence number is valid
    /// and is not the last element.
    ///
    /// Returns the updated new sequence number.
    fn update_seq_before_insert(conn: &mut SqliteConnection, seq: i32) -> QueryResult<i32> {
        let mut seq = seq;

        let count = Self::count_visible(conn)?;
        if seq == 0 {
            // Add as last child in parent (seq = count + 1)
            seq = count + 1;
        } else if 0 < seq && seq <= count + 1 {
            // Shift other items up if seq is valid and not last
            if seq < count + 1 {
                Self::increment_seq_before_insert(conn, seq)?;
            }
        } else {
            // Invalid seq, return error
            return Err(Error::DatabaseError(
                DatabaseErrorKind::RestrictViolation,
                Box::new("Invalid sequence number".to_string()),
            ));
        }
        Ok(seq)
    }

    /// Decrement the sequence numbers of visible models *after* a delete operation.
    ///
    /// The sequence numbers of visible records with a sequence number greater than
    /// the old sequence number are decremented by one:
    ///
    /// ```
    /// seq: 3
    /// Old: 1, 2, 4
    /// New: 1, 2, 3
    /// ```
    ///
    /// This operation is used when deleting a record at a specific position.
    fn decrement_seq_after_delete(conn: &mut SqliteConnection, seq: i32) -> QueryResult<usize>;
}

pub type NodeRef<T> = Rc<RefCell<T>>;

/// A trait for models that can be used as tree nodes.
pub trait TreeNode: From<Self::Model> {
    /// The reference model.
    type Model: Model;
    /// The unique identifier of the node.
    type Id: ?Sized + Eq + PartialEq + Debug + Clone + Hash;

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

    /// Sorts the children of the node by their sequence number.
    fn sort_children(&mut self) {
        self.children_mut().sort_by(|a, b| {
            let a_seq = a.borrow().sequence();
            let b_seq = b.borrow().sequence();
            a_seq.cmp(&b_seq)
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
        a_seq.cmp(&b_seq)
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
