// Author: Dylan Jones
// Date:   2025-11-10

use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
use std::fmt::Debug;

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
pub trait SequenceTree {
    /// The type of the ID of the model.
    type Id: ?Sized + PartialEq + Debug;
    /// The start sequence number (usually 1).
    const START_SEQ: i32 = 1;

    /// Checks if the given id is a valid parent node.
    fn is_valid_parent(conn: &mut SqliteConnection, parent_id: &Self::Id) -> QueryResult<bool>;

    /// Returns the number of child models inside the parent.
    fn count_children(conn: &mut SqliteConnection, parent_id: &Self::Id) -> QueryResult<i32>;

    /// Returns the sequence numbers of all child models inside the parent.
    ///
    /// This is mostly used for validating the sequence numbers.
    fn get_seq_numbers(conn: &mut SqliteConnection, parent_id: &Self::Id) -> QueryResult<Vec<i32>>;

    /// Validate the sequence numbers of all child models inside the parent.
    ///
    /// Checks if the sequence numbers start from `START_SEQ` and are incremental
    fn validate_seq_numbers(
        conn: &mut SqliteConnection,
        parent_id: &Self::Id,
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
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::Id) -> QueryResult<usize>;

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
        parent_id: &Self::Id,
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
        parent_id: &Self::Id,
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
        parent_id: &Self::Id,
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
        parent_id: &Self::Id,
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
        parent_id: &Self::Id,
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
        old_parent_id: &Self::Id,
        parent_id: &Self::Id,
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
        parent_id: &Self::Id,
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
        old_parent_id: &Self::Id,
        parent_id: &Self::Id,
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
