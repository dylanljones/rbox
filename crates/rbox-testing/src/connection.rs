// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Connections that keep their temporary database fixtures alive until closed.

use diesel::{RunQueryDsl, SqliteConnection};
use rbox_db::connection::ConnectionOptions;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Owns the connection before the file so the connection is dropped first.
pub struct ConnectionFixture {
    conn: SqliteConnection,
    file: NamedTempFile,
}

impl ConnectionFixture {
    /// Open an isolated One Library fixture using the caller's connection factory.
    pub fn one_library(
        open: impl FnOnce(PathBuf, &ConnectionOptions) -> rbox_db::error::Result<SqliteConnection>,
    ) -> Self {
        let file = crate::data::setup_one_library_path();
        let mut conn = open(file.path().to_path_buf(), &ConnectionOptions::default()).unwrap();
        // Exported tag IDs exceed the signed model range. Start ordered collections empty.
        for table in [
            "myTag_content",
            "myTag",
            "playlist_content",
            "playlist",
            "history_content",
            "history",
        ] {
            diesel::sql_query(format!("DELETE FROM {table}"))
                .execute(&mut conn)
                .unwrap();
        }
        Self { conn, file }
    }

    /// Path to the temporary database, for tests of pooled handlers.
    pub fn path(&self) -> &Path {
        self.file.path()
    }

    /// Create six track identities for One Library association contracts.
    pub fn seed_content_ids(&mut self) -> Vec<i32> {
        (1_000_000..1_000_006)
            .map(|id| {
                diesel::sql_query(format!("INSERT INTO content(content_id) VALUES ({id})"))
                    .execute(&mut self.conn)
                    .unwrap();
                id
            })
            .collect()
    }
}

impl std::ops::Deref for ConnectionFixture {
    type Target = SqliteConnection;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl std::ops::DerefMut for ConnectionFixture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}
