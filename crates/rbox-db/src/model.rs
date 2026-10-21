// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Basic query, insertion, update, and deletion traits shared by database models.

use diesel::prelude::*;
use std::fmt::Debug;

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
