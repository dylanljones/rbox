// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Explicit relationship loading into detached model snapshots.
//!
//! [`ModelRelations`] defines resolution for individual models and batches.
//! [`load_related`] deduplicates foreign keys and loads them in bounded batches;
//! concrete models define their relationship views and queries.

use diesel::prelude::*;
use diesel::result::Error;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::model::Model;

/// Explicit loading of a model's direct relationships into a detached snapshot.
///
/// Resolution never writes to the database. Implementations preserve input order and
/// duplicates, batch foreign-key lookups, and read related rows in one transaction.
/// Supplied models retain their existing values; they are not reloaded from the database.
pub trait ModelRelations: Model {
    /// Concrete view containing the original model and its resolved relationships.
    type Resolved;

    /// Resolve all supported relationships for a batch. Empty input performs no queries.
    /// Missing relationship targets are represented by `None`; query errors propagate.
    fn resolve_many(
        conn: &mut SqliteConnection,
        models: Vec<Self>,
    ) -> QueryResult<Vec<Self::Resolved>>;

    /// Resolve all supported relationships for this snapshot.
    fn resolve(self, conn: &mut SqliteConnection) -> QueryResult<Self::Resolved> {
        Self::resolve_many(conn, vec![self])?
            .into_iter()
            .next()
            .ok_or(Error::NotFound)
    }
}

/// Load distinct foreign keys in bounded batches, skipping empty sets.
pub fn load_related<K, V>(
    conn: &mut SqliteConnection,
    ids: impl IntoIterator<Item = K>,
    mut load: impl FnMut(&mut SqliteConnection, &[K]) -> QueryResult<Vec<V>>,
    key: impl Fn(&V) -> K,
) -> QueryResult<HashMap<K, V>>
where
    K: Eq + Hash,
{
    let ids: Vec<K> = ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let mut rows = HashMap::new();
    for chunk in ids.chunks(500) {
        for row in load(conn, chunk)? {
            rows.insert(key(&row), row);
        }
    }
    Ok(rows)
}
