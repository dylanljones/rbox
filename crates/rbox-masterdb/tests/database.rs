// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Constructor coverage for the handler's shared connection and error APIs.

use diesel::connection::SimpleConnection;
use rbox_core::RekordboxOptions;
use rbox_db::ConnectionOptions;
use rbox_masterdb::{establish_connection, Error, MasterDb};

#[test]
fn new_opens_an_encrypted_database_with_handler_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("master.db");
    {
        let mut conn = establish_connection(&path, &ConnectionOptions::for_export()).unwrap();
        conn.batch_execute("CREATE TABLE constructor_test(id INTEGER)")
            .unwrap();
    }
    let db = MasterDb::new(&path).unwrap();
    db.pool
        .get()
        .unwrap()
        .batch_execute("INSERT INTO constructor_test VALUES (1)")
        .unwrap();
}

#[test]
fn from_options_uses_the_supplied_database_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("custom.db");
    {
        let mut conn = establish_connection(&path, &ConnectionOptions::for_export()).unwrap();
        conn.batch_execute("CREATE TABLE constructor_test(id INTEGER)")
            .unwrap();
    }
    let options = RekordboxOptions {
        db_path: path,
        analysis_root: dir.path().join("analysis"),
        settings_root: dir.path().join("settings"),
    };
    let db = MasterDb::from_options(&options).unwrap();
    db.pool
        .get()
        .unwrap()
        .batch_execute("INSERT INTO constructor_test VALUES (1)")
        .unwrap();
}

#[test]
fn missing_databases_return_typed_errors_without_creating_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.db");
    assert!(matches!(MasterDb::new(&path),
        Err(Error::Core(rbox_core::error::Error::FileNotFound(missing))) if missing == path));
    let options = RekordboxOptions {
        db_path: path.clone(),
        analysis_root: dir.path().join("analysis"),
        settings_root: dir.path().join("settings"),
    };
    assert!(matches!(
        MasterDb::from_options(&options),
        Err(Error::Database(rbox_db::error::Error::Connection(_)))
    ));
    assert!(!path.exists());
}
