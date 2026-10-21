// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! SQLCipher connections and connection pools for the Rekordbox master database.
//!
//! Supplies the master database key when the caller has not provided one, then
//! delegates connection setup to `rbox-db`. Caller options are cloned so adding
//! the default key does not modify the original settings.

use std::path::Path;

use rbox_db::connection;
pub use rbox_db::connection::{ConnectionOptions, DbConn, DbPool};
use rbox_db::error::Result;

const MAGIC: &str = "513ge593d49928d46ggb9ggc9d8e:4254c85:f8e426eg8b92843b2gg547195:8";

fn with_key(options: &ConnectionOptions) -> ConnectionOptions {
    let mut options = options.clone();
    let key = String::from_utf8(MAGIC.as_bytes().iter().map(|&b| b - 1).collect()).unwrap();
    options.key.get_or_insert(key);
    options
}

pub fn establish_connection(path: impl AsRef<Path>, options: &ConnectionOptions) -> Result<DbConn> {
    connection::establish_connection(path, &with_key(options))
}

pub fn create_connection_pool(
    path: impl AsRef<Path>,
    options: &ConnectionOptions,
    max_size: u32,
) -> Result<DbPool> {
    connection::create_connection_pool(path, &with_key(options), max_size)
}
