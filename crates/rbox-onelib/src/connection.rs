// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Connections, pools, and schema initialization for One Library databases.
//!
//! Supplies the One Library SQLCipher key unless the caller overrides it, and
//! delegates connection configuration to `rbox-db`. Schema initialization
//! executes the embedded SQL in a transaction to create tables in a new database;
//! it does not migrate existing schemas.

use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};
use std::path::Path;

use rbox_db::connection;
pub use rbox_db::connection::{ConnectionOptions, DbConn, DbPool};
use rbox_db::error::Result;

const MAGIC: &str = "s9heeos5l958941bs7dr{cll1fm7rzunc4usccy916kn85wf{75j6p9gosrszrmt";

const SCHEMA: &str = include_str!("schema.sql");

fn with_key(options: &ConnectionOptions) -> ConnectionOptions {
    let mut options = options.clone();
    let key = String::from_utf8(MAGIC.as_bytes().iter().map(|&b| b - 1).collect()).unwrap();
    options.key.get_or_insert(key);
    options
}

pub fn initialize_schema(conn: &mut SqliteConnection) -> Result<()> {
    conn.transaction::<_, diesel::result::Error, _>(|conn| conn.batch_execute(SCHEMA))
        .map_err(|err| err.into())
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
