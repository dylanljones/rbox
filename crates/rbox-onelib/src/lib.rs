// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Access to Rekordbox One Library (Device Library Plus) export databases.
//!
//! Provides the database handler, Diesel schema and row models, and SQLCipher
//! connection helpers. Model modules also expose relationship views and ordered
//! collection operations.
//!
//! The optional `master-db` feature enables conversions from master database
//! models for export workflows.

pub mod connection;
pub mod database;
pub mod enums;
pub mod error;
pub mod models;
pub mod schema;

pub use connection::{
    create_connection_pool, establish_connection, initialize_schema, DbConn, DbPool,
};
pub use database::OneLibrary;
pub use error::{Error, Result};
pub use models::*;

pub mod dsl {
    use super::schema;

    pub use schema::album::dsl::album;
    pub use schema::artist::dsl::artist;
    pub use schema::category::dsl::category;
    pub use schema::color::dsl::color;
    pub use schema::content::dsl::content;
    pub use schema::cue::dsl::cue;
    pub use schema::genre::dsl::genre;
    pub use schema::history::dsl::history;
    pub use schema::history_content::dsl::history_content;
    pub use schema::hotCueBankList::dsl::hotCueBankList;
    pub use schema::hotCueBankList_cue::dsl::hotCueBankList_cue;
    pub use schema::image::dsl::image;
    pub use schema::key::dsl::key;
    pub use schema::label::dsl::label;
    pub use schema::menuItem::dsl::menuItem;
    pub use schema::myTag::dsl::myTag;
    pub use schema::myTag_content::dsl::myTag_content;
    pub use schema::playlist::dsl::playlist;
    pub use schema::playlist_content::dsl::playlist_content;
    pub use schema::property::dsl::property;
    pub use schema::recommendedLike::dsl::recommendedLike;
    pub use schema::sort::dsl::sort;
}
