// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Connection types and configuration shared by the Rekordbox database formats.
//!
//! [`ConnectionOptions`] defaults to opening an existing database read-write and
//! preserving its journal mode. [`ConnectionOptions::for_export`] selects creation
//! and DELETE journaling for a new device export. Database-specific callers supply
//! the encryption key. These options describe configuration; constructing them
//! does not open a connection or execute any PRAGMAs.
//!
//! Set `RBOX_SQLCIPHER_LOG_LEVEL=ERROR` before starting tests to suppress SQLCipher
//! warnings while retaining errors. Supported levels are `NONE`, `ERROR`, `WARN`,
//! `INFO`, `DEBUG`, and `TRACE` (case-insensitive). Each connection applies the override
//! before key setup; unset leaves logging unchanged. This is process-wide,
//! does not disable memory locking or fix quota failures, and is ignored by plain
//! SQLite or SQLCipher versions without `cipher_log_level`.

use diesel::r2d2::{self, ConnectionManager, CustomizeConnection, Pool};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{Connection, QueryResult, RunQueryDsl, SqliteConnection};
use std::fmt;
use std::path::Path;
use std::time::Duration;

use crate::error::{Error, Result};

/// SQLite connection used by the database model APIs.
pub type DbConn = SqliteConnection;

/// Pool of SQLite connections sharing the same database and configuration.
pub type DbPool = Pool<ConnectionManager<DbConn>>;

/// Whether opening a database permits writes or creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OpenMode {
    /// Open an existing database for reading only.
    ReadOnly,
    /// Open an existing database for reading and writing; do not create it.
    #[default]
    ReadWrite,
    /// Open a database for reading and writing, creating it if missing.
    ReadWriteCreate,
}

/// Journal policy applied when opening or creating a database.
///
/// Initialize persistent mode changes before building a pool to avoid competing
/// changes. Connection-local rollback modes still need setting on each handle.
/// Read-only connections must use [`JournalMode::Preserve`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JournalMode {
    /// Leave the existing journal mode unchanged.
    #[default]
    Preserve,
    /// Delete the rollback journal after committing; suitable for new exports.
    Delete,
    /// Truncate the rollback journal after committing.
    Truncate,
    /// Retain the rollback journal and invalidate its header after committing.
    Persist,
    /// Use write-ahead logging, which can keep committed data in a sidecar file.
    Wal,
}

/// Synchronization policy for writable connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Synchronous {
    /// Reduce synchronization overhead. In WAL mode, power loss can undo commits.
    Normal,
    /// Synchronize each commit in WAL mode.
    Full,
    /// Add directory synchronization for DELETE journals; equivalent to FULL in WAL.
    #[default]
    Extra,
}

/// Configuration to apply consistently to direct and pooled connections.
///
/// Defaults preserve the journal mode, enable foreign keys, wait up to five seconds
/// for locks, and disable dirty reads. Writable connections use EXTRA synchronization
/// and, on macOS, full filesystem synchronization. Cache sizes, page sizes, temporary
/// storage, and cipher compatibility settings are left to the backend or format.
///
/// The key must be applied before reading the database, and the busy timeout before
/// operations that may need locks. A schema read is required to validate decryption.
/// Connection-local settings must be applied to every new pooled connection.
///
/// ```
/// use rbox_db::connection::{ConnectionOptions, OpenMode};
///
/// let options = ConnectionOptions {
///     open_mode: OpenMode::ReadWrite,
///     key: Some("database-specific passphrase".into()),
///     ..ConnectionOptions::default()
/// };
/// ```
#[derive(Clone)]
pub struct ConnectionOptions {
    /// Access mode. Defaults to read-write; creation requires an explicit opt-in.
    ///
    /// The opener must select the matching SQLite URI mode (`ro`, `rw`, or `rwc`).
    /// [`setup_connection`] can restrict queries but cannot change file-open flags.
    pub open_mode: OpenMode,
    /// SQLCipher passphrase supplied by the database format; `None` means no key.
    ///
    /// Redacted from debug output. This value is a passphrase, not a raw-key SQL literal.
    pub key: Option<String>,
    /// Time to wait for database locks. Defaults to five seconds.
    pub busy_timeout: Duration,
    /// Enforce foreign-key constraints on this connection. Defaults to `true`.
    pub foreign_keys: bool,
    /// Journal policy. Defaults to preserving the current mode.
    pub journal_mode: JournalMode,
    /// Synchronization policy for writes. Defaults to [`Synchronous::Extra`].
    pub synchronous: Synchronous,
    /// Request full filesystem synchronization where supported (macOS).
    ///
    /// Defaults to `true` on macOS and `false` elsewhere. When enabled, it also
    /// covers checkpoints, so no separate checkpoint synchronization option is needed.
    pub fullfsync: bool,
    /// Permit dirty reads where SQLite supports them. Defaults to `false`.
    pub read_uncommitted: bool,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            open_mode: Default::default(),
            key: None,
            busy_timeout: Duration::from_secs(5),
            foreign_keys: true,
            journal_mode: Default::default(),
            synchronous: Default::default(),
            fullfsync: cfg!(target_os = "macos"),
            read_uncommitted: false,
        }
    }
}

impl ConnectionOptions {
    /// Create a new instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Options for creating a device export with DELETE journaling and durable writes.
    ///
    /// Supply the export format's key before connecting. To open an existing export
    /// while preserving its journal mode, use [`Self::default`] and select the desired
    /// [`OpenMode`] instead.
    pub fn for_export() -> Self {
        Self {
            open_mode: OpenMode::ReadWriteCreate,
            journal_mode: JournalMode::Delete,
            ..Self::default()
        }
    }

    /// Set the access mode. Defaults to read-write.
    ///
    /// Creation requires an explicit opt-in.
    /// Read-only connections must preserve the journal mode.
    pub fn open_mode(&mut self, open_mode: OpenMode) -> &mut Self {
        self.open_mode = open_mode;
        self
    }

    /// Set the SQLCipher passphrase. Defaults to `None`.
    pub fn key(&mut self, key: String) -> &mut Self {
        self.key = Some(key);
        self
    }

    /// Set the busy timeout. Defaults to five seconds.
    pub fn busy_timeout(&mut self, busy_timeout: Duration) -> &mut Self {
        self.busy_timeout = busy_timeout;
        self
    }

    /// Enable or disable foreign-key enforcement. Defaults to `true`.
    pub fn foreign_keys(&mut self, foreign_keys: bool) -> &mut Self {
        self.foreign_keys = foreign_keys;
        self
    }

    /// Set the journal mode. Defaults to preserving the current mode.
    pub fn journal_mode(&mut self, journal_mode: JournalMode) -> &mut Self {
        self.journal_mode = journal_mode;
        self
    }

    /// Set the synchronization policy for writes. Defaults to [`Synchronous::Extra`].
    pub fn synchronous(&mut self, synchronous: Synchronous) -> &mut Self {
        self.synchronous = synchronous;
        self
    }

    /// Enable or disable full filesystem synchronization where supported (macOS). Defaults to `true` on macOS and `false` elsewhere.
    pub fn fullfsync(&mut self, fullfsync: bool) -> &mut Self {
        self.fullfsync = fullfsync;
        self
    }

    /// Enable or disable dirty reads where SQLite supports them. Defaults to `false`.
    pub fn read_uncommitted(&mut self, read_uncommitted: bool) -> &mut Self {
        self.read_uncommitted = read_uncommitted;
        self
    }

    /// Validate the options for correctness and compatibility.
    fn validate(&self) -> Result<()> {
        let message = if self.busy_timeout.as_millis() > i32::MAX as u128 {
            Some("Busy timeout exceeds SQLite's maximum of i32::MAX milliseconds")
        } else if self.open_mode == OpenMode::ReadOnly && self.journal_mode != JournalMode::Preserve
        {
            Some("Read-only connections must preserve journal mode")
        } else if self
            .key
            .as_ref()
            .is_some_and(|key| key.is_empty() || key.contains('\0'))
        {
            Some("The SQLCipher passphrase must be nonempty and contain no NUL bytes")
        } else {
            None
        };
        match message {
            Some(message) => Err(Error::InvalidOptions(message.into())),
            None => Ok(()),
        }
    }
}

impl fmt::Debug for ConnectionOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionOptions")
            .field("open_mode", &self.open_mode)
            .field("key", &self.key.as_ref().map(|_| "[REDACTED]"))
            .field("busy_timeout", &self.busy_timeout)
            .field("foreign_keys", &self.foreign_keys)
            .field("journal_mode", &self.journal_mode)
            .field("synchronous", &self.synchronous)
            .field("fullfsync", &self.fullfsync)
            .field("read_uncommitted", &self.read_uncommitted)
            .finish()
    }
}

/// Configure a freshly opened connection before querying or starting transactions.
///
/// Applies the key and busy timeout, validates decryption with a schema read, and
/// applies the remaining options. Journal changes are checked against SQLite's
/// returned mode. Writable connections reject OFF and MEMORY journaling when
/// preserving the existing mode; read-only connections require Preserve.
///
/// The opener must select SQLite URI mode `ro`, `rw`, or `rwc` to match `open_mode`.
/// This function cannot change OS-level access flags or undo file creation. It also
/// sets `query_only` for read-only connections. On failure, discard the connection:
/// setup is not transactional and some settings may already have been applied.
///
/// ```no_run
/// use diesel::{Connection, SqliteConnection};
/// use rbox_db::connection::{setup_connection, ConnectionOptions, OpenMode};
///
/// let mut conn = SqliteConnection::establish("file:library.db?mode=ro")?;
/// let options = ConnectionOptions { open_mode: OpenMode::ReadOnly, ..Default::default() };
/// setup_connection(&mut conn, &options)?;
/// # Ok::<(), rbox_db::error::Error>(())
/// ```
pub fn setup_connection(conn: &mut DbConn, options: &ConnectionOptions) -> QueryResult<()> {
    let invalid = |message: &str| {
        DieselError::DatabaseError(
            DatabaseErrorKind::UnableToSendCommand,
            Box::new(message.to_owned()),
        )
    };
    options
        .validate()
        .map_err(|error| invalid(&error.to_string()))?;
    let timeout = options.busy_timeout.as_millis();
    let read_only = options.open_mode == OpenMode::ReadOnly;
    // SQLCipher has no runtime environment setting for this PRAGMA. Validate the
    // value before interpolation, and apply it before key setup can log warnings.
    let log_level = match std::env::var("RBOX_SQLCIPHER_LOG_LEVEL") {
        Ok(value) => {
            let value = value.trim().to_ascii_uppercase();
            if !matches!(
                value.as_str(),
                "NONE" | "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE"
            ) {
                return Err(invalid(
                    "RBOX_SQLCIPHER_LOG_LEVEL must be NONE, ERROR, WARN, INFO, DEBUG, or TRACE",
                ));
            }
            Some(value)
        }
        Err(std::env::VarError::NotPresent) => None,
        Err(_) => {
            return Err(invalid(
                "RBOX_SQLCIPHER_LOG_LEVEL must contain valid Unicode",
            ))
        }
    };
    if let Some(level) = &log_level {
        // Activation replaces NONE with the default level. Keep warnings filtered
        // during activation, then apply the requested level after setting the key.
        let initial = if level == "NONE" { "ERROR" } else { level };
        diesel::sql_query(format!("PRAGMA cipher_log_level = {initial}")).execute(conn)?;
    }

    if let Some(key) = &options.key {
        // PRAGMA does not support bound parameters. Escape the SQL string literal.
        let key = key.replace('\'', "''");
        diesel::sql_query(format!("PRAGMA key = '{key}'")).execute(conn)?;
    }

    if let Some(level) = &log_level {
        diesel::sql_query(format!("PRAGMA cipher_log_level = {level}")).execute(conn)?;
    }
    diesel::sql_query(format!("PRAGMA busy_timeout = {timeout}")).execute(conn)?;

    #[derive(diesel::QueryableByName)]
    struct SchemaCount {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        #[diesel(column_name = count)]
        _count: i64,
    }
    diesel::sql_query("SELECT count(*) AS count FROM sqlite_master")
        .get_result::<SchemaCount>(conn)?;

    // This fails inside transactions, before foreign_keys can silently ignore changes.
    let synchronous = match options.synchronous {
        Synchronous::Normal => "NORMAL",
        Synchronous::Full => "FULL",
        Synchronous::Extra => "EXTRA",
    };
    diesel::sql_query(format!("PRAGMA synchronous = {synchronous}")).execute(conn)?;
    for (name, enabled) in [
        ("query_only", read_only),
        ("foreign_keys", options.foreign_keys),
        ("fullfsync", options.fullfsync),
        ("read_uncommitted", options.read_uncommitted),
    ] {
        diesel::sql_query(format!("PRAGMA {name} = {}", i32::from(enabled))).execute(conn)?;
    }

    #[derive(diesel::QueryableByName)]
    struct Journal {
        #[diesel(sql_type = diesel::sql_types::Text)]
        journal_mode: String,
    }
    let current = diesel::sql_query("PRAGMA journal_mode").get_result::<Journal>(conn)?;
    let requested = match options.journal_mode {
        JournalMode::Preserve => None,
        JournalMode::Delete => Some("delete"),
        JournalMode::Truncate => Some("truncate"),
        JournalMode::Persist => Some("persist"),
        JournalMode::Wal => Some("wal"),
    };
    if let Some(mode) = requested {
        if !current.journal_mode.eq_ignore_ascii_case(mode) {
            let actual = diesel::sql_query(format!("PRAGMA journal_mode = {mode}"))
                .get_result::<Journal>(conn)?;
            if !actual.journal_mode.eq_ignore_ascii_case(mode) {
                return Err(invalid("SQLite did not apply the requested journal mode"));
            }
        }
    } else if !read_only
        && !matches!(
            current.journal_mode.to_ascii_lowercase().as_str(),
            "delete" | "truncate" | "persist" | "wal"
        )
    {
        return Err(invalid("Writable connections require durable journaling"));
    }
    Ok(())
}

/// Apply settings to each new physical pool connection, before it is handed out.
///
/// Match the manager URL's access mode to `open_mode`. The customizer runs once per
/// connection, not on every checkout; callers must not leave changed PRAGMAs behind.
///
/// ```no_run
/// use diesel::r2d2::ConnectionManager;
/// use rbox_db::connection::{ConnectionOptions, DbConn, DbPool, OpenMode};
///
/// let manager = ConnectionManager::<DbConn>::new("file:library.db?mode=ro");
/// let options = ConnectionOptions { open_mode: OpenMode::ReadOnly, ..Default::default() };
/// let pool = DbPool::builder()
///     .connection_customizer(Box::new(options))
///     .build(manager)?;
/// # Ok::<(), rbox_db::error::Error>(())
/// ```
impl CustomizeConnection<DbConn, r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut DbConn) -> std::result::Result<(), r2d2::Error> {
        setup_connection(conn, self).map_err(r2d2::Error::QueryError)
    }
}

/// Open a filesystem database and apply its connection options.
///
/// Read-only and read-write modes require an existing file. Only
/// [`OpenMode::ReadWriteCreate`] permits creation. The path is treated literally,
/// not as a SQLite URI; reserved characters are escaped before adding the access
/// mode. Relative paths are resolved against the current working directory.
/// In-memory databases and caller-provided URI parameters are not supported here.
///
/// # Errors
/// Returns an error for invalid options or paths, opening failures, incorrect keys,
/// or settings SQLite cannot apply. Invalid options are rejected before opening.
///
/// ```no_run
/// use rbox_db::connection::{establish_connection, ConnectionOptions};
///
/// let conn = establish_connection("library.db", &ConnectionOptions::default())?;
/// # Ok::<(), rbox_db::error::Error>(())
/// ```
pub fn establish_connection(path: impl AsRef<Path>, options: &ConnectionOptions) -> Result<DbConn> {
    options.validate()?;
    let url = connection_url(path.as_ref(), options.open_mode)?;
    let mut conn = DbConn::establish(&url)?;
    setup_connection(&mut conn, options)?;
    Ok(conn)
}

/// Open a pool of configured connections to a filesystem database.
///
/// Uses the same path and access rules as [`establish_connection`]. A temporary
/// connection validates the database and initializes journal mode before pool workers
/// start. It stays open until the pool is ready, retaining WAL state during startup.
/// Each new pool connection then receives the same options through the customizer.
/// Options are cloned so callers can reuse them for another connection or pool.
///
/// # Errors
/// Returns opening/setup errors immediately instead of retrying them until the pool
/// times out. Pool startup errors are returned separately. `max_size` must be nonzero.
///
/// ```no_run
/// use rbox_db::connection::{create_connection_pool, ConnectionOptions};
///
/// let pool = create_connection_pool("library.db", &ConnectionOptions::default(), 4)?;
/// let conn = pool.get()?;
/// # Ok::<(), rbox_db::error::Error>(())
/// ```
pub fn create_connection_pool(
    path: impl AsRef<Path>,
    options: &ConnectionOptions,
    max_size: u32,
) -> Result<DbPool> {
    options.validate()?;
    if max_size == 0 {
        return Err(Error::InvalidOptions(
            "Pool size must be greater than zero".into(),
        ));
    }
    let url = connection_url(path.as_ref(), options.open_mode)?;
    let mut initial = DbConn::establish(&url)?;
    setup_connection(&mut initial, options)?;
    let pool = DbPool::builder()
        .max_size(max_size)
        .connection_customizer(Box::new(options.clone()))
        .build(ConnectionManager::<DbConn>::new(url))?;
    drop(initial);
    Ok(pool)
}

fn connection_url(path: &Path, mode: OpenMode) -> Result<String> {
    let bad_path =
        |message: String| Error::Connection(diesel::ConnectionError::BadConnection(message));
    if path.as_os_str().is_empty() || path == Path::new(":memory:") {
        return Err(bad_path(
            "Expected a nonempty filesystem database path".into(),
        ));
    }
    let absolute = std::path::absolute(path).map_err(|error| bad_path(error.to_string()))?;
    let path = absolute
        .to_str()
        .ok_or_else(|| bad_path("Database path is not UTF-8".into()))?;
    if path.contains('\0') {
        return Err(bad_path("Database path contains a NUL byte".into()));
    }

    #[cfg(windows)]
    let normalized = path
        .strip_prefix(r"\\?\UNC\")
        .map(|rest| format!("//{rest}"))
        .unwrap_or_else(|| path.strip_prefix(r"\\?\").unwrap_or(path).to_owned())
        .replace('\\', "/");
    #[cfg(windows)]
    let path = normalized.as_str();

    // Four slashes keep a Windows UNC server in the path, not the URI authority.
    let mut url = if path.starts_with("//") {
        "file://"
    } else {
        "file:"
    }
    .to_owned();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~:".contains(&byte) {
            url.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(&mut url, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    url.push_str(match mode {
        OpenMode::ReadOnly => "?mode=ro",
        OpenMode::ReadWrite => "?mode=rw",
        OpenMode::ReadWriteCreate => "?mode=rwc",
    });
    Ok(url)
}
