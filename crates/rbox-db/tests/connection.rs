// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Integration coverage for connection configuration, opening, and pool behavior.

use diesel::connection::SimpleConnection;
use diesel::r2d2::ConnectionManager;
use diesel::{Connection, QueryableByName, RunQueryDsl};
use std::time::Duration;

use rbox_db::connection::{
    create_connection_pool, establish_connection, setup_connection, ConnectionOptions, DbConn,
    DbPool, JournalMode, OpenMode, Synchronous,
};

// Isolate process-global SQLCipher logging and environment variables from other tests.
#[cfg(any(feature = "bundled", feature = "bundled-vendored-openssl"))]
#[test]
fn sqlcipher_logging_environment() {
    const CHILD: &str = "RBOX_TEST_CIPHER_LOG_CHILD";
    if let Ok(expected) = std::env::var(CHILD) {
        #[derive(QueryableByName)]
        struct LogLevel {
            #[diesel(sql_type = diesel::sql_types::Text)]
            cipher_log_level: String,
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("encrypted.db");
        let options = ConnectionOptions {
            key: Some("test passphrase".into()),
            ..ConnectionOptions::for_export()
        };
        if expected == "invalid" {
            let error = establish_connection(&path, &options).err().unwrap();
            assert!(error.to_string().contains("RBOX_SQLCIPHER_LOG_LEVEL"));
            return;
        }
        let mut direct = establish_connection(&path, &options).unwrap();
        direct
            .batch_execute("CREATE TABLE sample(id INTEGER PRIMARY KEY)")
            .unwrap();
        let pool = create_connection_pool(&path, &options, 2).unwrap();
        let mut first = pool.get().unwrap();
        let mut second = pool.get().unwrap();
        for conn in [&mut direct, &mut *first, &mut *second] {
            let actual = diesel::sql_query("PRAGMA cipher_log_level")
                .get_result::<LogLevel>(conn)
                .unwrap();
            assert_eq!(actual.cipher_log_level, expected);
            assert_eq!(
                diesel::sql_query("SELECT count(*) AS value FROM sample")
                    .get_result::<Number>(conn)
                    .unwrap()
                    .value,
                0
            );
        }
        return;
    }
    for (value, expected) in [
        (None, "WARN"),
        (Some("error"), "ERROR"),
        (Some("NONE"), "NONE"),
        (Some("ERROR; SELECT 1"), "invalid"),
    ] {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child
            .args(["--exact", "sqlcipher_logging_environment", "--nocapture"])
            .env(CHILD, expected)
            .env_remove("RBOX_SQLCIPHER_LOG_LEVEL");
        if let Some(value) = value {
            child.env("RBOX_SQLCIPHER_LOG_LEVEL", value);
        }
        let output = child.output().unwrap();
        assert!(
            output.status.success(),
            "{expected}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn read_only() -> ConnectionOptions {
    ConnectionOptions {
        open_mode: OpenMode::ReadOnly,
        ..ConnectionOptions::default()
    }
}

#[derive(QueryableByName)]
struct Number {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    value: i32,
}

fn pragma(conn: &mut DbConn, name: &str) -> i32 {
    let column = if name == "busy_timeout" {
        "timeout"
    } else {
        name
    };
    diesel::sql_query(format!("SELECT {column} AS value FROM pragma_{name}"))
        .get_result::<Number>(conn)
        .unwrap()
        .value
}

#[derive(QueryableByName)]
struct Journal {
    #[diesel(sql_type = diesel::sql_types::Text)]
    journal_mode: String,
}

fn journal(conn: &mut DbConn) -> String {
    diesel::sql_query("PRAGMA journal_mode")
        .get_result::<Journal>(conn)
        .unwrap()
        .journal_mode
}

#[test]
fn direct_connection_applies_defaults_and_enforces_foreign_keys() {
    let dir = tempfile::tempdir().unwrap();
    let mut conn = DbConn::establish(dir.path().join("db.sqlite").to_str().unwrap()).unwrap();
    setup_connection(&mut conn, &ConnectionOptions::for_export()).unwrap();
    assert_eq!(journal(&mut conn), "delete");
    assert_eq!(pragma(&mut conn, "busy_timeout"), 5000);
    assert_eq!(pragma(&mut conn, "foreign_keys"), 1);
    assert_eq!(pragma(&mut conn, "synchronous"), 3);
    assert_eq!(pragma(&mut conn, "query_only"), 0);
    assert_eq!(pragma(&mut conn, "read_uncommitted"), 0);
    assert_eq!(
        pragma(&mut conn, "fullfsync"),
        i32::from(cfg!(target_os = "macos"))
    );
    conn.batch_execute(
        "CREATE TABLE parent(id INTEGER PRIMARY KEY);
        CREATE TABLE child(parent_id INTEGER REFERENCES parent(id));",
    )
    .unwrap();
    assert!(conn.batch_execute("INSERT INTO child VALUES (42)").is_err());
}

#[test]
fn pool_configures_multiple_physical_connections() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db.sqlite");
    let options = ConnectionOptions {
        busy_timeout: Duration::from_millis(1234),
        foreign_keys: false,
        synchronous: Synchronous::Full,
        fullfsync: true,
        read_uncommitted: true,
        ..ConnectionOptions::for_export()
    };
    let pool = DbPool::builder()
        .max_size(2)
        .connection_timeout(Duration::from_secs(5))
        .connection_customizer(Box::new(options))
        .build(ConnectionManager::<DbConn>::new(path.to_str().unwrap()))
        .unwrap();
    let mut first = pool.get().unwrap();
    let mut second = pool.get().unwrap();
    for conn in [&mut *first, &mut *second] {
        assert_eq!(pragma(conn, "busy_timeout"), 1234);
        assert_eq!(pragma(conn, "foreign_keys"), 0);
        assert_eq!(pragma(conn, "synchronous"), 2);
        assert_eq!(pragma(conn, "fullfsync"), 1);
        assert_eq!(pragma(conn, "read_uncommitted"), 1);
    }
}

#[test]
fn journal_modes_are_applied_and_read_only_preserves_wal() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db.sqlite");
    let mut conn = DbConn::establish(path.to_str().unwrap()).unwrap();
    for (mode, expected) in [
        (JournalMode::Persist, "persist"),
        (JournalMode::Truncate, "truncate"),
        (JournalMode::Delete, "delete"),
        (JournalMode::Wal, "wal"),
    ] {
        setup_connection(
            &mut conn,
            &ConnectionOptions {
                journal_mode: mode,
                ..ConnectionOptions::for_export()
            },
        )
        .unwrap();
        assert_eq!(journal(&mut conn), expected);
    }
    conn.batch_execute("CREATE TABLE data(id INTEGER)").unwrap();
    drop(conn);
    let mut conn = establish_connection(&path, &read_only()).unwrap();
    assert_eq!(journal(&mut conn), "wal");
    assert_eq!(pragma(&mut conn, "query_only"), 1);
    assert!(conn.batch_execute("INSERT INTO data VALUES (1)").is_err());
}

#[test]
fn encrypted_connections_validate_and_escape_the_key_in_direct_and_pool_setup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("encrypted.sqlite");
    let options = ConnectionOptions {
        key: Some("a'; PRAGMA foreign_keys = OFF; --".into()),
        ..ConnectionOptions::for_export()
    };
    {
        let mut conn = DbConn::establish(path.to_str().unwrap()).unwrap();
        setup_connection(&mut conn, &options).unwrap();
        conn.batch_execute("CREATE TABLE data(id INTEGER); INSERT INTO data VALUES (42)")
            .unwrap();
        assert_eq!(pragma(&mut conn, "foreign_keys"), 1);
    }
    let mut wrong = DbConn::establish(path.to_str().unwrap()).unwrap();
    assert!(setup_connection(
        &mut wrong,
        &ConnectionOptions {
            key: Some("wrong passphrase".into()),
            ..options.clone()
        }
    )
    .is_err());
    drop(wrong);
    let pool = DbPool::builder()
        .max_size(1)
        .connection_timeout(Duration::from_secs(5))
        .connection_customizer(Box::new(options))
        .build(ConnectionManager::<DbConn>::new(path.to_str().unwrap()))
        .unwrap();
    let row = diesel::sql_query("SELECT id AS value FROM data")
        .get_result::<Number>(&mut *pool.get().unwrap())
        .unwrap();
    assert_eq!(row.value, 42);
}

#[test]
fn invalid_options_and_unavailable_journal_modes_fail() {
    for options in [
        ConnectionOptions {
            busy_timeout: Duration::MAX,
            ..ConnectionOptions::default()
        },
        ConnectionOptions {
            journal_mode: JournalMode::Wal,
            ..read_only()
        },
        ConnectionOptions {
            key: Some("bad\0key".into()),
            ..ConnectionOptions::default()
        },
        ConnectionOptions {
            key: Some(String::new()),
            ..ConnectionOptions::default()
        },
        ConnectionOptions {
            open_mode: OpenMode::ReadWrite,
            ..ConnectionOptions::default()
        },
        ConnectionOptions {
            journal_mode: JournalMode::Wal,
            ..ConnectionOptions::for_export()
        },
    ] {
        let mut conn = DbConn::establish(":memory:").unwrap();
        assert!(setup_connection(&mut conn, &options).is_err());
    }
}

#[test]
fn setup_in_a_transaction_fails_instead_of_ignoring_foreign_keys() {
    let mut conn = DbConn::establish(":memory:").unwrap();
    conn.batch_execute("PRAGMA foreign_keys = OFF; BEGIN")
        .unwrap();
    assert!(setup_connection(&mut conn, &ConnectionOptions::default()).is_err());
    assert_eq!(pragma(&mut conn, "foreign_keys"), 0);
    conn.batch_execute("ROLLBACK").unwrap();
}

#[test]
fn open_helpers_enforce_modes_and_preserve_literal_paths() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library ü # %23.sqlite");
    for open_mode in [OpenMode::ReadOnly, OpenMode::ReadWrite] {
        let options = ConnectionOptions {
            open_mode,
            ..ConnectionOptions::default()
        };
        assert!(establish_connection(&path, &options).is_err());
        assert!(create_connection_pool(&path, &options, 2).is_err());
        assert!(!path.exists());
    }
    {
        let mut conn = establish_connection(&path, &ConnectionOptions::for_export()).unwrap();
        conn.batch_execute("CREATE TABLE data(id INTEGER)").unwrap();
    }
    assert!(path.is_file());
    let options = ConnectionOptions {
        open_mode: OpenMode::ReadWrite,
        ..ConnectionOptions::default()
    };
    {
        let pool = create_connection_pool(&path, &options, 2).unwrap();
        pool.get()
            .unwrap()
            .batch_execute("INSERT INTO data VALUES (1)")
            .unwrap();
    }
    let options = read_only();
    let mut conn = establish_connection(&path, &options).unwrap();
    // query_only is additional protection, not the implementation of the open mode.
    conn.batch_execute("PRAGMA query_only = OFF").unwrap();
    assert!(conn.batch_execute("INSERT INTO data VALUES (2)").is_err());
    let pool = create_connection_pool(&path, &options, 2).unwrap();
    let mut first = pool.get().unwrap();
    let mut second = pool.get().unwrap();
    for conn in [&mut *first, &mut *second] {
        assert_eq!(pragma(conn, "foreign_keys"), 1);
        conn.batch_execute("PRAGMA query_only = OFF").unwrap();
        assert!(conn.batch_execute("INSERT INTO data VALUES (2)").is_err());
    }
}

#[test]
fn pool_helper_initializes_encrypted_wal_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("encrypted.sqlite");
    let options = ConnectionOptions {
        key: Some("test passphrase".into()),
        journal_mode: JournalMode::Wal,
        ..ConnectionOptions::for_export()
    };
    {
        let pool = create_connection_pool(&path, &options, 2).unwrap();
        let mut first = pool.get().unwrap();
        let mut second = pool.get().unwrap();
        assert_eq!(journal(&mut first), "wal");
        assert_eq!(journal(&mut second), "wal");
        first
            .batch_execute("CREATE TABLE data(id INTEGER); INSERT INTO data VALUES (42)")
            .unwrap();
        assert_eq!(
            diesel::sql_query("SELECT id AS value FROM data")
                .get_result::<Number>(&mut *second)
                .unwrap()
                .value,
            42
        );
    }
    let mut conn = establish_connection(&path, &options).unwrap();
    assert_eq!(journal(&mut conn), "wal");
    let wrong = ConnectionOptions {
        key: Some("wrong".into()),
        ..options
    };
    assert!(matches!(
        create_connection_pool(&path, &wrong, 2),
        Err(rbox_db::error::Error::Query(_))
    ));
}

#[test]
fn helpers_reject_invalid_options_before_creating_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.sqlite");
    assert!(matches!(
        create_connection_pool(&path, &ConnectionOptions::for_export(), 0),
        Err(rbox_db::error::Error::InvalidOptions(_))
    ));
    let invalid = ConnectionOptions {
        busy_timeout: Duration::MAX,
        ..ConnectionOptions::for_export()
    };
    assert!(establish_connection(&path, &invalid).is_err());
    assert!(create_connection_pool(&path, &invalid, 2).is_err());
    assert!(!path.exists());
    for path in ["", ":memory:", "bad\0path"] {
        assert!(establish_connection(path, &ConnectionOptions::for_export()).is_err());
        assert!(create_connection_pool(path, &ConnectionOptions::for_export(), 1).is_err());
    }
}

#[test]
fn default_options_open_existing_files_for_writing_without_creating_them() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("default.sqlite");
    let options = ConnectionOptions::new();
    assert!(matches!(
        establish_connection(&path, &options),
        Err(rbox_db::error::Error::Connection(_))
    ));
    assert!(!path.exists());
    drop(establish_connection(&path, &ConnectionOptions::for_export()).unwrap());
    let mut conn = establish_connection(&path, &options).unwrap();
    conn.batch_execute("CREATE TABLE data(id INTEGER)").unwrap();
    let pool = create_connection_pool(&path, &options, 1).unwrap();
    pool.get()
        .unwrap()
        .batch_execute("INSERT INTO data VALUES (1)")
        .unwrap();
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS value FROM data")
            .get_result::<Number>(&mut conn)
            .unwrap()
            .value,
        1
    );
}

#[test]
fn option_setters_apply_together_and_debug_redacts_the_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("builder.sqlite");
    let secret = "private test passphrase";
    let mut options = ConnectionOptions::new();
    options
        .open_mode(OpenMode::ReadWriteCreate)
        .key(secret.into())
        .busy_timeout(Duration::from_millis(27))
        .foreign_keys(false)
        .journal_mode(JournalMode::Delete)
        .synchronous(Synchronous::Normal)
        .fullfsync(false)
        .read_uncommitted(true);
    let debug = format!("{options:?}");
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains(secret));
    let mut conn = establish_connection(&path, &options).unwrap();
    for (name, expected) in [
        ("busy_timeout", 27),
        ("foreign_keys", 0),
        ("synchronous", 1),
        ("fullfsync", 0),
        ("read_uncommitted", 1),
    ] {
        assert_eq!(pragma(&mut conn, name), expected, "{name}");
    }
    assert_eq!(journal(&mut conn), "delete");
}

#[test]
fn all_journal_and_synchronous_modes_apply_to_direct_and_pooled_connections() {
    for (journal_mode, expected_journal) in [
        (JournalMode::Delete, "delete"),
        (JournalMode::Persist, "persist"),
        (JournalMode::Truncate, "truncate"),
        (JournalMode::Wal, "wal"),
    ] {
        for (synchronous, expected_sync) in [
            (Synchronous::Normal, 1),
            (Synchronous::Full, 2),
            (Synchronous::Extra, 3),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("modes.sqlite");
            let options = ConnectionOptions {
                journal_mode,
                synchronous,
                ..ConnectionOptions::for_export()
            };
            let mut conn = establish_connection(&path, &options).unwrap();
            assert_eq!(journal(&mut conn), expected_journal);
            assert_eq!(pragma(&mut conn, "synchronous"), expected_sync);
            drop(conn);
            let pool = create_connection_pool(&path, &options, 2).unwrap();
            let mut first = pool.get().unwrap();
            let mut second = pool.get().unwrap();
            for conn in [&mut *first, &mut *second] {
                assert_eq!(journal(conn), expected_journal);
                assert_eq!(pragma(conn, "synchronous"), expected_sync);
            }
        }
    }
}

#[test]
fn invalid_options_return_typed_errors_without_creating_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invalid.sqlite");
    let mut cases = vec![
        ConnectionOptions {
            busy_timeout: Duration::from_millis(i32::MAX as u64 + 1),
            ..ConnectionOptions::for_export()
        },
        ConnectionOptions {
            key: Some(String::new()),
            ..ConnectionOptions::for_export()
        },
        ConnectionOptions {
            key: Some("secret\0suffix".into()),
            ..ConnectionOptions::for_export()
        },
    ];
    for journal_mode in [
        JournalMode::Delete,
        JournalMode::Truncate,
        JournalMode::Persist,
        JournalMode::Wal,
    ] {
        cases.push(ConnectionOptions {
            journal_mode,
            ..read_only()
        });
    }
    for options in cases {
        assert!(
            matches!(
                establish_connection(&path, &options),
                Err(rbox_db::error::Error::InvalidOptions(_))
            ),
            "{options:?}"
        );
        assert!(
            matches!(
                create_connection_pool(&path, &options, 1),
                Err(rbox_db::error::Error::InvalidOptions(_))
            ),
            "{options:?}"
        );
        assert!(!path.exists());
    }
}

#[test]
fn timeout_boundaries_are_represented_in_milliseconds() {
    for (busy_timeout, expected) in [
        (Duration::ZERO, 0),
        (Duration::from_nanos(999_999), 0),
        (Duration::from_micros(1_999), 1),
        (Duration::from_millis(i32::MAX as u64), i32::MAX),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("timeout.sqlite");
        let options = ConnectionOptions {
            busy_timeout,
            ..ConnectionOptions::for_export()
        };
        let mut conn = establish_connection(&path, &options).unwrap();
        assert_eq!(pragma(&mut conn, "busy_timeout"), expected);
        let pool = create_connection_pool(&path, &options, 1).unwrap();
        assert_eq!(pragma(&mut pool.get().unwrap(), "busy_timeout"), expected);
    }
}

#[test]
fn write_lock_errors_leave_connection_usable_after_release() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("locked.sqlite");
    let options = ConnectionOptions {
        busy_timeout: Duration::ZERO,
        ..ConnectionOptions::for_export()
    };
    let mut writer = establish_connection(&path, &options).unwrap();
    writer
        .batch_execute("CREATE TABLE data(id INTEGER)")
        .unwrap();
    let pool = create_connection_pool(&path, &options, 1).unwrap();
    let mut contender = pool.get().unwrap();
    writer
        .batch_execute("BEGIN IMMEDIATE; INSERT INTO data VALUES (1)")
        .unwrap();
    assert!(contender
        .batch_execute("INSERT INTO data VALUES (2)")
        .is_err());
    writer.batch_execute("COMMIT").unwrap();
    contender
        .batch_execute("INSERT INTO data VALUES (2)")
        .unwrap();
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS value FROM data")
            .get_result::<Number>(&mut *contender)
            .unwrap()
            .value,
        2
    );
}

#[test]
fn pool_limits_checkouts_and_reuses_connections_with_their_settings() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pool.sqlite");
    let options = ConnectionOptions {
        busy_timeout: Duration::from_millis(37),
        ..ConnectionOptions::for_export()
    };
    let pool = create_connection_pool(&path, &options, 1).unwrap();
    let mut conn = pool.get().unwrap();
    conn.batch_execute("CREATE TEMP TABLE marker(id INTEGER); INSERT INTO marker VALUES (42)")
        .unwrap();
    assert!(pool.try_get().is_none());
    drop(conn);
    let mut conn = pool.get().unwrap();
    assert_eq!(pragma(&mut conn, "busy_timeout"), 37);
    assert_eq!(
        diesel::sql_query("SELECT id AS value FROM marker")
            .get_result::<Number>(&mut *conn)
            .unwrap()
            .value,
        42
    );
}

#[test]
fn corrupt_database_errors_do_not_modify_the_original_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("corrupt.sqlite");
    let bytes = b"This is not a SQLite database.";
    std::fs::write(&path, bytes).unwrap();
    let options = ConnectionOptions::default();
    assert!(matches!(
        establish_connection(&path, &options),
        Err(rbox_db::error::Error::Query(_))
    ));
    assert!(matches!(
        create_connection_pool(&path, &options, 1),
        Err(rbox_db::error::Error::Query(_))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn directory_and_missing_parent_return_connection_errors() {
    let dir = tempfile::tempdir().unwrap();
    for path in [dir.path().to_owned(), dir.path().join("missing/db.sqlite")] {
        assert!(matches!(
            establish_connection(&path, &ConnectionOptions::for_export()),
            Err(rbox_db::error::Error::Connection(_))
        ));
        assert!(matches!(
            create_connection_pool(&path, &ConnectionOptions::for_export(), 1),
            Err(rbox_db::error::Error::Connection(_))
        ));
    }
    assert!(!dir.path().join("missing").exists());
}

#[test]
fn relative_paths_work_without_changing_the_process_working_directory() {
    let cwd = std::env::current_dir().unwrap();
    let dir = tempfile::tempdir_in(&cwd).unwrap();
    let relative = dir
        .path()
        .strip_prefix(&cwd)
        .unwrap()
        .join("relative.sqlite");
    let options = ConnectionOptions::for_export();
    let mut conn = establish_connection(&relative, &options).unwrap();
    conn.batch_execute("CREATE TABLE data(id INTEGER)").unwrap();
    let pool = create_connection_pool(&relative, &options, 1).unwrap();
    pool.get()
        .unwrap()
        .batch_execute("INSERT INTO data VALUES (1)")
        .unwrap();
    assert!(dir.path().join("relative.sqlite").is_file());
}

#[cfg(unix)]
#[test]
fn uri_parameters_in_filenames_cannot_override_read_only_mode() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("literal?mode=rwc&cache=shared#fragment.sqlite");
    let mut conn = establish_connection(&path, &ConnectionOptions::for_export()).unwrap();
    conn.batch_execute("CREATE TABLE data(id INTEGER)").unwrap();
    drop(conn);
    let mut conn = establish_connection(&path, &read_only()).unwrap();
    conn.batch_execute("PRAGMA query_only = OFF").unwrap();
    assert!(conn.batch_execute("INSERT INTO data VALUES (1)").is_err());
    let pool = create_connection_pool(&path, &read_only(), 1).unwrap();
    assert!(pool
        .get()
        .unwrap()
        .batch_execute("INSERT INTO data VALUES (1)")
        .is_err());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[cfg(windows)]
#[test]
fn windows_verbatim_paths_open_the_same_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("verbatim.sqlite");
    let mut conn = establish_connection(&path, &ConnectionOptions::for_export()).unwrap();
    conn.batch_execute("CREATE TABLE data(id INTEGER); INSERT INTO data VALUES (42)")
        .unwrap();
    drop(conn);
    let verbatim = std::fs::canonicalize(&path).unwrap();
    let pool = create_connection_pool(&verbatim, &read_only(), 1).unwrap();
    assert_eq!(
        diesel::sql_query("SELECT id AS value FROM data")
            .get_result::<Number>(&mut *pool.get().unwrap())
            .unwrap()
            .value,
        42
    );
}
