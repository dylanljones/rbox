// Author: Dylan Jones
// Date:   2025-11-06

use diesel::{connection::SimpleConnection, prelude::*, SqliteConnection};

use super::dsl::*;
use super::enums::*;
use super::models::*;
use super::schema;
use super::util::format_datetime;
use crate::error::{Error, Result};
use crate::pathlib::NormalizePath;

const MAGIC: &str = "s9heeos5l958941bs7dr{cll1fm7rzunc4usccy916kn85wf{75j6p9gosrszrmt";

/// Opens a connection to a SQLite database and configures it with specific settings.
///
/// # Arguments
/// * `path` - A string slice that holds the path to the SQLite database file.
///
/// # Returns
/// * `Result<SqliteConnection>` - Returns a `SqliteConnection` object if successful, or an error.
///
/// # Errors
/// * Returns an error if the database connection cannot be established or if any of the
///   configuration commands fail.
///
/// # Configuration
/// The function applies the following settings to the SQLite connection:
/// - Sets the encryption key using the `PRAGMA key` command.
/// - Enables foreign key constraints using the `PRAGMA foreign_keys = ON` command.
/// - Sets the journal mode to Write-Ahead Logging (WAL) using the `PRAGMA journal_mode = WAL` command.
/// - Sets the synchronous mode to NORMAL using the `PRAGMA synchronous = NORMAL` command.
///
/// # Example
/// ```no_run
/// let connection = open_connection(":memory:");
/// match connection {
///     Ok(conn) => println!("Connection established successfully!"),
///     Err(e) => println!("Failed to open connection: {}", e),
/// }
/// ```
fn open_connection(path: &str) -> Result<SqliteConnection> {
    let db_key = String::from_utf8(MAGIC.as_bytes().iter().map(|&b| b - 1).collect())
        .map_err(|e| e.utf8_error())?;

    let mut conn = SqliteConnection::establish(path)?;
    conn.batch_execute(format!("PRAGMA key = '{db_key}';").as_str())?;
    conn.batch_execute("PRAGMA foreign_keys = ON")?;
    conn.batch_execute("PRAGMA journal_mode = WAL")?;
    conn.batch_execute("PRAGMA synchronous = NORMAL")?;
    Ok(conn)
}
