// Author: Dylan Jones
// Date:   2025-11-10

use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};

/// A trait for models that implement a tree-like structure with reordering via a sequence number.
///
/// The sequence number defines the order of the model in a list or collection.
/// It is one-based (i.e., starts from 1).
pub trait SequenceTree {
    type Id;
    /// The start sequence number (usually 1).
    const START_SEQ: i32 = 1;

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
        for (i, seq) in seq_numbers.iter().enumerate() {
            if (i as i32) + Self::START_SEQ != *seq {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Reset the sequence numbers of all models inside a parent.
    ///
    /// The sequence numbers are reset to the *current* order of the models inside their parent
    /// starting at 1:
    ///
    /// ```
    /// Old: 1, 2, 4, 5, 7, 8
    /// New: 1, 2, 3, 4, 5, 6
    /// ```
    ///
    /// This is helpful when deleting records or for fixing the sequence after an invalid state.
    /// When adding or moving records, the sequence numbers should be set explicitly.
    fn reset_seq(conn: &mut SqliteConnection, parent_id: &Self::Id) -> QueryResult<usize>;

    /// Increment the sequence numbers of models inside a parent after an insert operation.
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
    /// This operation is used when inserting a new record at a specific position.
    fn increment_seq_before_insert(
        conn: &mut SqliteConnection,
        parent_id: &Self::Id,
        seq: i32,
    ) -> QueryResult<usize>;

    /// Update the sequence number for an insert operation.
    ///
    /// The sequence numbers of records inside a parent with a sequence number greater equal
    /// the new sequence number are incremented by one *if* the new sequence number is valid
    /// and is not the last element.
    ///
    /// Returns the updated new sequence number.
    fn update_seq_before_insert(
        conn: &mut SqliteConnection,
        parent_id: &Self::Id,
        seq: Option<i32>,
    ) -> QueryResult<i32> {
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
                Self::increment_seq_before_insert(conn, parent_id, seq)?;
            } // Otherwise seq is valid and last
            seq
        } else {
            // Add as last child in parent (seq = count + 1)
            count + Self::START_SEQ
        };
        Ok(seq)
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
