// Author: Dylan Jones
// Date:   2025-11-07

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, CustomizeConnection, Pool};

use crate::Result;

pub type DbConn = SqliteConnection;
pub type DbPool = Pool<ConnectionManager<DbConn>>;
const MAGIC: &str = "513ge593d49928d46ggb9ggc9d8e:4254c85:f8e426eg8b92843b2gg547195:8";

/// Execute Sqlite pragmas on a connection.
fn setup_connection(conn: &mut DbConn) -> QueryResult<()> {
    let db_key = String::from_utf8(MAGIC.as_bytes().iter().map(|&b| b - 1).collect()).unwrap();
    diesel::sql_query(format!("PRAGMA key = '{db_key}';").as_str()).execute(conn)?;
    diesel::sql_query("PRAGMA foreign_keys = ON;").execute(conn)?;
    diesel::sql_query("PRAGMA journal_mode = WAL").execute(conn)?;
    diesel::sql_query("PRAGMA busy_timeout = 5000;").execute(conn)?;
    diesel::sql_query("PRAGMA synchronous = NORMAL;").execute(conn)?;
    Ok(())
}

/// Opens a connection to an SQLite database and configures it with specific settings.
///
/// # Arguments
/// * `database_url` - A string slice that holds the path to the SQLite database file.
///
/// # Errors
/// * Returns an error if the database connection cannot be established or if any of the
///   configuration commands fail.
///
/// # Example
/// ```no_run
/// let connection = open_connection(":memory:");
/// match connection {
///     Ok(conn) => println!("Connection established successfully!"),
///     Err(e) => println!("Failed to open connection: {}", e),
/// }
/// ```
pub fn establish_connection(database_url: &str) -> Result<DbConn> {
    let mut conn = SqliteConnection::establish(database_url)?;
    setup_connection(&mut conn)?;
    Ok(conn)
}

#[derive(Clone, Debug)]
struct SqlitePragmas;

impl CustomizeConnection<DbConn, r2d2::Error> for SqlitePragmas {
    fn on_acquire(&self, conn: &mut DbConn) -> std::result::Result<(), diesel::r2d2::Error> {
        setup_connection(conn)?;
        Ok(())
    }
}

/// Opens a connection pool to an SQLite database and configures it with specific settings.
///
/// # Arguments
/// * `database_url` - A string slice that holds the path to the SQLite database file.
///
/// # Errors
/// * Returns an error if the database connection cannot be established or if any of the
///   configuration commands fail.
///
/// # Example
/// ```no_run
/// let pool = establish_pool(":memory:", 8);
/// match pool {
///     Ok(pool) => {
///         let conn = pool.get().expect("Failed to get connection.");
///         println!("Connection established successfully!")
///     },
///     Err(e) => println!("Failed to create pool: {}", e),
/// }
/// ```
pub fn create_connection_pool(database_url: &str, max_size: u32) -> Result<DbPool> {
    let manager = ConnectionManager::<DbConn>::new(database_url);
    let pool = Pool::builder()
        .max_size(max_size)
        .connection_customizer(Box::new(SqlitePragmas))
        .build(manager)?;
    Ok(pool)
}
