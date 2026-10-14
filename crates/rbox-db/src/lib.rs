// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared database model operations, ordering, and relationship loading.

pub mod connection;
pub mod error;
pub mod model;
pub mod ordering;
pub mod relations;

pub use connection::{
    create_connection_pool, establish_connection, ConnectionOptions, DbConn, DbPool, JournalMode,
    OpenMode, Synchronous,
};
pub use model::{Model, ModelDelete, ModelInsert, ModelUpdate};
pub use ordering::{
    build_tree_list, delete_tree_batch, ordering_error, tree_delete_roots, validate_seq_numbers,
    ModelList, ModelTree, NodeRef, OrderedModel, TreeNode, TreeSeq,
};
pub use relations::{load_related, ModelRelations};

pub mod enums;
