// Author: Dylan Jones
// Date:   2025-05-01

//! # Rekordbox Master Database Handler
//!
//! This module provides an interface for interacting with the Rekordbox `master.db` SQLite database.
//! It enables querying, updating, and managing various tables such as playlists, songs, tags, and settings.
//!
//! ## Structure
//! - The main struct [`MasterDb`] encapsulates the database connection and provides methods for accessing and modifying database entries.
//! - Each table in the database has corresponding query methods for retrieving all entries, fetching by ID, and performing insert/update/delete operations.
//! - Utility functions are included for playlist management, sequence handling, and safe/unsafe write control.
//!
//! ## Basic Usage
//! ```no_run
//! use rbox::MasterDb;
//!
//! // Open the default Rekordbox database
//! let mut db = MasterDb::open().unwrap();
//!
//! // Query all playlists
//! let playlists = db.get_playlists().unwrap();
//!
//! // Insert a new playlist
//! let new_playlist = db.create_playlist("My Playlist".to_string(), None, None, None, None).unwrap();
//! ```
//!
//! ## Safety
//! By default, write operations are restricted if Rekordbox is running. Use `set_unsafe_writes(true)` to override this behavior if necessary.
//!
//! ## Tables Supported
//! - AgentRegistry, CloudAgentRegistry, ContentActiveCensor, DjmdPlaylist, DjmdSongPlaylist, DjmdProperty, DjmdCloudProperty, DjmdRecommendLike, DjmdRelatedTracks, DjmdSampler, DjmdSongTagList, DjmdSort, ImageFile, SettingFile, UuidIDMap, and more.
//!
//! See individual method documentation for details on arguments, return values, and error handling.

// #![allow(unused)]
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::dsl::{exists, select};
use diesel::{connection::SimpleConnection, prelude::*, query_dsl::RunQueryDsl, SqliteConnection};
use dunce;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use uuid::Uuid;

use super::enums::*;
use super::models::*;
use super::playlist_xml::MasterPlaylistXml;
use super::random_id::RandomIdGenerator;
use super::util::{format_datetime, sort_tree_list};
use super::{
    agentRegistry, cloudAgentRegistry, contentActiveCensor, contentCue, contentFile,
    djmdActiveCensor, djmdAlbum, djmdArtist, djmdCategory, djmdCloudProperty, djmdColor,
    djmdContent, djmdCue, djmdDevice, djmdGenre, djmdHistory, djmdHotCueBanklist, djmdKey,
    djmdLabel, djmdMenuItems, djmdMixerParam, djmdMyTag, djmdPlaylist, djmdProperty,
    djmdRecommendLike, djmdRelatedTracks, djmdSampler, djmdSongPlaylist, djmdSongTagList, djmdSort,
    imageFile, schema, settingFile, uuidIDMap,
};
use crate::anlz::{find_anlz_files, Anlz, AnlzFiles, AnlzPaths};
use crate::options::RekordboxOptions;
use crate::pathlib::NormalizePath;
use crate::util::is_rekordbox_running;

const MAGIC: &str = "513ge593d49928d46ggb9ggc9d8e:4254c85:f8e426eg8b92843b2gg547195:8";

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
    let key = String::from_utf8(MAGIC.as_bytes().iter().map(|&b| b - 1).collect())?;

    let mut conn = SqliteConnection::establish(path)?;
    conn.batch_execute(format!("PRAGMA key = '{key}';").as_str())?;
    conn.batch_execute("PRAGMA foreign_keys = ON")?;
    conn.batch_execute("PRAGMA journal_mode = WAL")?;
    conn.batch_execute("PRAGMA synchronous = NORMAL")?;
    // conn.batch_execute("PRAGMA busy_timeout = 100")?;
    // conn.batch_execute("PRAGMA wal_autocheckpoint = 1000")?;
    // conn.batch_execute("PRAGMA wal_checkpoint(TRUNCATE)")?;
    Ok(conn)
}

pub struct MasterDb {
    /// Represents the SQLite database connection used for interacting with the database.
    pub conn: SqliteConnection,
    /// Stores the path to the PIONEER share directory, which contains analysis and other files.
    /// This is optional and may not be set if the directory is not found.
    pub share_dir: Option<PathBuf>,
    /// Stores the path to the `masterPlaylist6.xml` file located in the same directory as the database.
    /// This is optional and may not be set if the file is not found.
    pub plxml_path: Option<PathBuf>,
    /// Indicates whether unsafe writes to the database are allowed while Rekordbox is running.
    /// - `true`: Unsafe writes are enabled, allowing modifications to the database.
    /// - `false`: Unsafe writes are disabled, preventing modifications to the database.
    unsafe_writes: bool,
}

impl MasterDb {
    /// Open a Rekordbox database specified by path.
    ///
    /// The path must be a valid Rekordbox database file. The function will try to locate the
    /// `share` directory and the `masterPlaylist6.xml` file in the same directory as the database
    /// file. If they are not found, the database can still be used, however, some features such as
    /// playlist management and locating analysis files will return errors.
    pub fn new<P: AsRef<OsStr>>(path: P) -> Result<Self> {
        let path_obj = Path::new(&path);
        if !path_obj.exists() {
            return Err(anyhow::anyhow!("Database file does not exist"));
        }
        let parent_dir = path_obj.parent().expect("Failed to get parent directory");
        let share_dir_path = parent_dir.join("share");
        let share_dir_str = if share_dir_path.exists() {
            Some(share_dir_path.normalize())
        } else {
            None
        };
        let pl_xml_path = parent_dir.join("masterPlaylists6.xml");
        let pl_xml_path_str = if pl_xml_path.exists() {
            Some(pl_xml_path.normalize())
        } else {
            None
        };
        let conn = open_connection(path_obj.to_str().unwrap())?;
        Ok(Self {
            conn,
            share_dir: share_dir_str,
            plxml_path: pl_xml_path_str,
            unsafe_writes: false,
        })
    }

    /// Open the Rekordbox database specified by the options [`RekordboxOptions`]
    ///
    /// The options specified by the user must be valid. The `master.db` file, the `share` directory
    /// and the `masterPlaylist6.xml` file will be extracted from the options.
    pub fn from_options(options: &RekordboxOptions) -> Result<Self> {
        let share_dir = options.analysis_root.normalize();
        let plxml_path = options.get_db_dir()?.normalize();
        let conn = open_connection(options.db_path.to_str().unwrap())?;

        Ok(Self {
            conn,
            share_dir: Some(share_dir),
            plxml_path: Some(plxml_path),
            unsafe_writes: false,
        })
    }

    /// Open the default Rekordbox `master.db` database.
    ///
    /// The default location of the `master.db` file is determined by the [`RekordboxOptions`] struct.
    pub fn open() -> Result<Self> {
        let options = RekordboxOptions::open()?;
        Self::from_options(&options)
    }

    /// Sets the unsafe writes flag for the database.
    ///
    /// # Arguments
    /// * `unsafe_writes` - A boolean value indicating whether unsafe writes are allowed.
    ///   - `true`: Unsafe writes are enabled, allowing modifications to the database even if Rekordbox is running.
    ///   - `false`: Unsafe writes are disabled, preventing modifications to the database while Rekordbox is running.
    ///
    /// This method is useful for controlling write operations to the database in scenarios
    /// where Rekordbox may be actively using the database.
    pub fn set_unsafe_writes(&mut self, unsafe_writes: bool) {
        self.unsafe_writes = unsafe_writes;
    }

    // -- AgentRegistry ----------------------------------------------------------------------------

    /// Retrieves all entries from the [`AgentRegistry`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<AgentRegistry>>` - A vector of [`AgentRegistry`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let registries = db.get_agent_registry().unwrap();
    /// for registry in registries {
    ///     println!("{:?}", registry);
    /// }
    /// ```
    pub fn get_agent_registry(&mut self) -> Result<Vec<AgentRegistry>> {
        let results = agentRegistry.load::<AgentRegistry>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves an [`AgentRegistry`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the agent registry entry.
    ///
    /// # Returns
    /// * `Result<Option<AgentRegistry>>` - Returns an `Option` containing the [`AgentRegistry`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let registry = db.get_agent_registry_by_id("some_id").unwrap();
    /// match registry {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_agent_registry_by_id(&mut self, id: &str) -> Result<Option<AgentRegistry>> {
        let result = agentRegistry
            .filter(schema::agentRegistry::registry_id.eq(id))
            .first::<AgentRegistry>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves the local update sequence number (USN) from the [`AgentRegistry`] table.
    ///
    /// # Returns
    /// * `Result<i32>` - Returns the local USN as an integer if found, or an error if the entry
    ///   does not exist.
    ///
    /// # Errors
    /// * Returns an error if the `localUpdateCount` entry is not found in the [`AgentRegistry`] table
    ///   or if the database query fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let local_usn = db.get_local_usn().unwrap();
    /// println!("Local USN: {}", local_usn);
    /// ```
    pub fn get_local_usn(&mut self) -> Result<i32> {
        let result = agentRegistry
            .filter(schema::agentRegistry::registry_id.eq("localUpdateCount"))
            .select(schema::agentRegistry::int_1)
            .first::<Option<i32>>(&mut self.conn)?;
        // Raise error if not found, otherwise return value
        if let Some(value) = result {
            Ok(value)
        } else {
            Err(anyhow::anyhow!("localUpdateCount not found"))
        }
    }

    // fn set_local_usn(&mut self, usn: i32) -> Result<usize> {
    //     let result = diesel::update(
    //         agentRegistry.filter(schema::agentRegistry::registry_id.eq("localUpdateCount")),
    //     )
    //     .set(schema::agentRegistry::int_1.eq(usn))
    //     .execute(&mut self.conn)?;
    //     Ok(result)
    // }

    /// Increments the local update sequence number (USN) in the [`AgentRegistry`] table.
    ///
    /// # Arguments
    /// * `usn` - An integer value representing the amount to increment the local USN.
    ///
    /// # Returns
    /// * `Result<i32>` - Returns the updated USN as an integer if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if the database update operation fails or if the updated USN is not found.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let new_usn = db.increment_local_usn(1).unwrap();
    /// println!("Updated USN: {}", new_usn);
    /// ```
    fn increment_local_usn(&mut self, usn: i32) -> Result<i32> {
        let result = diesel::update(
            agentRegistry.filter(schema::agentRegistry::registry_id.eq("localUpdateCount")),
        )
        .set(schema::agentRegistry::int_1.eq(schema::agentRegistry::int_1 + usn))
        .returning(schema::agentRegistry::int_1)
        .get_result::<Option<i32>>(&mut self.conn)?;
        let new_usn = result.expect("No new USN");
        Ok(new_usn)
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    /// Retrieves all entries from the [`CloudAgentRegistry`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<CloudAgentRegistry>>` - A vector of [`CloudAgentRegistry`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let registries = db.get_cloud_agent_registry().unwrap();
    /// for registry in registries {
    ///     println!("{:?}", registry);
    /// }
    /// ```
    pub fn get_cloud_agent_registry(&mut self) -> Result<Vec<CloudAgentRegistry>> {
        let results = cloudAgentRegistry.load::<CloudAgentRegistry>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`CloudAgentRegistry`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the cloud agent registry entry.
    ///
    /// # Returns
    /// * `Result<Option<CloudAgentRegistry>>` - Returns an `Option` containing the [`CloudAgentRegistry`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let registry = db.get_cloud_agent_registry_by_id("some_id").unwrap();
    /// match registry {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_cloud_agent_registry_by_id(
        &mut self,
        id: &str,
    ) -> Result<Option<CloudAgentRegistry>> {
        let result = cloudAgentRegistry
            .filter(schema::cloudAgentRegistry::ID.eq(id))
            .first::<CloudAgentRegistry>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    /// Retrieves all entries from the [`ContentActiveCensor`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<ContentActiveCensor>>` - A vector of [`ContentActiveCensor`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let active_censors = db.get_content_active_censor().unwrap();
    /// for censor in active_censors {
    ///     println!("{:?}", censor);
    /// }
    /// ```
    pub fn get_content_active_censor(&mut self) -> Result<Vec<ContentActiveCensor>> {
        let results = contentActiveCensor.load::<ContentActiveCensor>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`ContentActiveCensor`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content active censor entry.
    ///
    /// # Returns
    /// * `Result<Option<ContentActiveCensor>>` - Returns an `Option` containing the [`ContentActiveCensor`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let active_censor = db.get_content_active_censor_by_id("some_id").unwrap();
    /// match active_censor {
    ///     Some(censor) => println!("{:?}", censor),
    ///     None => println!("No active censor found for the given ID"),
    /// }
    /// ```
    pub fn get_content_active_censor_by_id(
        &mut self,
        id: &str,
    ) -> Result<Option<ContentActiveCensor>> {
        let result = contentActiveCensor
            .filter(schema::contentActiveCensor::ID.eq(id))
            .first::<ContentActiveCensor>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ContentCue -------------------------------------------------------------------------------

    /// Retrieves all entries from the [`ContentCue`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<ContentCue>>` - A vector of [`ContentCue`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let cues = db.get_content_cue().unwrap();
    /// for cue in cues {
    ///     println!("{:?}", cue);
    /// }
    /// ```
    pub fn get_content_cue(&mut self) -> Result<Vec<ContentCue>> {
        let results = contentCue.load::<ContentCue>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`ContentCue`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content cue entry.
    ///
    /// # Returns
    /// * `Result<Option<ContentCue>>` - Returns an `Option` containing the [`ContentCue`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let cue = db.get_content_cue_by_id("some_id").unwrap();
    /// match cue {
    ///     Some(cue) => println!("{:?}", cue),
    ///     None => println!("No cue found for the given ID"),
    /// }
    /// ```
    pub fn get_content_cue_by_id(&mut self, id: &str) -> Result<Option<ContentCue>> {
        let result = contentCue
            .filter(schema::contentCue::ID.eq(id))
            .first::<ContentCue>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ContentFile ------------------------------------------------------------------------------

    /// Retrieves all entries from the [`contentFile`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<ContentFile>>` - A vector of [`ContentFile`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let files = db.get_content_file().unwrap();
    /// for file in files {
    ///     println!("{:?}", file);
    /// }
    /// ```
    pub fn get_content_file(&mut self) -> Result<Vec<ContentFile>> {
        let results = contentFile.load::<ContentFile>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`ContentFile`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content file entry.
    ///
    /// # Returns
    /// * `Result<Option<ContentFile>>` - Returns an `Option` containing the [`ContentFile`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let file = db.get_content_file_by_id("some_id").unwrap();
    /// match file {
    ///     Some(file) => println!("{:?}", file),
    ///     None => println!("No file found for the given ID"),
    /// }
    /// ```
    pub fn get_content_file_by_id(&mut self, id: &str) -> Result<Option<ContentFile>> {
        let result = contentFile
            .filter(schema::contentFile::ID.eq(id))
            .first::<ContentFile>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdActiveCensor`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdActiveCensor>>` - A vector of [`DjmdActiveCensor`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let active_censors = db.get_active_censor().unwrap();
    /// for censor in active_censors {
    ///     println!("{:?}", censor);
    /// }
    /// ```
    pub fn get_active_censor(&mut self) -> Result<Vec<DjmdActiveCensor>> {
        let results = djmdActiveCensor.load::<DjmdActiveCensor>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdActiveCensor`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the active censor entry.
    ///
    /// # Returns
    /// * `Result<Option<DjmdActiveCensor>>` - Returns an `Option` containing the [`DjmdActiveCensor`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let active_censor = db.get_active_censor_by_id("some_id").unwrap();
    /// match active_censor {
    ///     Some(censor) => println!("{:?}", censor),
    ///     None => println!("No active censor found for the given ID"),
    /// }
    /// ```
    pub fn get_active_censor_by_id(&mut self, id: &str) -> Result<Option<DjmdActiveCensor>> {
        let result = djmdActiveCensor
            .filter(schema::djmdActiveCensor::ID.eq(id))
            .first::<DjmdActiveCensor>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Album ------------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdAlbum`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdAlbum>>` - A vector of [`DjmdAlbum`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let albums = db.get_album().unwrap();
    /// for album in albums {
    ///     println!("{:?}", album);
    /// }
    /// ```
    pub fn get_album(&mut self) -> Result<Vec<DjmdAlbum>> {
        let results: Vec<DjmdAlbum> = djmdAlbum.load(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdAlbum`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the album.
    ///
    /// # Returns
    /// * `Result<Option<DjmdAlbum>>` - Returns an `Option` containing the [`DjmdAlbum`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let album = db.get_album_by_id("some_id").unwrap();
    /// match album {
    ///     Some(album) => println!("{:?}", album),
    ///     None => println!("No album found for the given ID"),
    /// }
    /// ```
    pub fn get_album_by_id(&mut self, id: &str) -> Result<Option<DjmdAlbum>> {
        let result: Option<DjmdAlbum> = djmdAlbum
            .filter(schema::djmdAlbum::ID.eq(id))
            .first(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves a [`DjmdAlbum`] entry by its name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the album.
    ///
    /// # Returns
    /// * `Result<Option<DjmdAlbum>>` - Returns an `Option` containing the [`DjmdAlbum`] object
    ///   if found, or `None` if no entry matches the given name. Returns an error if multiple entries
    ///   match the given name.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed or if more than one album
    ///   matches the given name.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let album = db.get_album_by_name("Album Name").unwrap();
    /// match album {
    ///     Some(album) => println!("{:?}", album),
    ///     None => println!("No album found for the given name"),
    /// }
    /// ```
    pub fn get_album_by_name(&mut self, name: &str) -> Result<Option<DjmdAlbum>> {
        let results: Vec<DjmdAlbum> = djmdAlbum
            .filter(schema::djmdAlbum::Name.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    /// Checks if an album with the given ID exists in the database.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the album ID.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the album exists, `false` otherwise.
    fn album_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdAlbum.filter(schema::djmdAlbum::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    /// Generates a unique album ID that does not exist in the database.
    ///
    /// # Returns
    /// * `Result<String>` - Returns a new unique album ID as a string.
    fn generate_album_id(&mut self) -> Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.album_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Inserts a new album into the [`DjmdAlbum`] table in the database.
    ///
    /// # Arguments
    /// * `name` - The name of the album.
    /// * `artist_id` - An optional identifier for the album artist.
    /// * `image_path` - An optional path to the album's image.
    /// * `compilation` - An optional integer indicating whether the album is a compilation.
    ///
    /// # Returns
    /// * `Result<DjmdAlbum>` - Returns the newly created [`DjmdAlbum`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let album = db.insert_album(
    ///     "Album Name".to_string(),
    ///     Some("ArtistID".to_string()),
    ///     Some("/path/to/image.jpg".to_string()),
    ///     Some(1),
    /// ).unwrap();
    /// println!("{:?}", album);
    /// ```
    pub fn insert_album(
        &mut self,
        name: String,
        artist_id: Option<String>,
        image_path: Option<String>,
        compilation: Option<i32>,
    ) -> Result<DjmdAlbum> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Generate ID/UUID
        let id: String = self.generate_album_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date
        let utcnow: DateTime<Utc> = Utc::now();
        // Create and insert model
        let item: DjmdAlbum = DjmdAlbum::new(
            id,
            uuid,
            usn,
            utcnow,
            name,
            artist_id,
            image_path,
            compilation,
        )?;
        let result: DjmdAlbum = diesel::insert_into(djmdAlbum)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    /// Inserts a new album into the [`DjmdAlbum`] table with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The name of the album to insert or retrieve.
    ///
    /// # Returns
    /// * `Result<DjmdAlbum>` - Returns the existing or newly created [`DjmdAlbum`] object.
    ///
    /// # Errors
    /// * Returns an error if the album cannot be inserted or retrieved.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// // Creates a new album
    /// let album = db.insert_album_if_not_exists("Album 1").unwrap();
    /// // Retrieves the existing album
    /// let album = db.insert_album_if_not_exists("Album 1").unwrap();
    /// ```
    fn insert_album_if_not_exists(&mut self, name: &str) -> Result<DjmdAlbum> {
        let album = self.get_album_by_name(name)?;
        if let Some(album) = album {
            Ok(album)
        } else {
            let new = self.insert_album(name.to_string(), None, None, None)?;
            Ok(new)
        }
    }

    /// Updates an existing [`DjmdAlbum`] entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdAlbum`] object to be updated.
    ///
    /// # Returns
    /// * `Result<DjmdAlbum>` - Returns the updated [`DjmdAlbum`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database update operation fails.
    ///
    /// # Behavior
    /// * Compares the fields of the provided [`DjmdAlbum`] object with the existing entry in the database.
    /// * If no differences are found, the existing entry is returned without making any updates.
    /// * If differences are found:
    ///   - Updates the `updated_at` timestamp to the current time.
    ///   - Increments the local update sequence number (USN) based on the number of differences.
    ///   - Updates the database entry with the modified fields.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let mut album = db.get_album_by_id("some_id").unwrap().unwrap();
    /// album.Name = "New Album Name".to_string();
    /// let updated_album = db.update_album(&mut album).unwrap();
    /// println!("{:?}", updated_album);
    /// ```
    pub fn update_album(&mut self, item: &mut DjmdAlbum) -> Result<DjmdAlbum> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Count differences
        let existing: DjmdAlbum = self.get_album_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.Name != existing.Name {
            n += 1
        }
        if item.AlbumArtistID != existing.AlbumArtistID {
            n += 1
        }
        if item.ImagePath != existing.ImagePath {
            n += 1
        }
        if item.Compilation != existing.Compilation {
            n += 1
        }
        if item.SearchStr != existing.SearchStr {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        item.updated_at = Utc::now();
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdAlbum = diesel::update(&*item)
            .set(item.clone())
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    /// Deletes an album entry from the [`DjmdAlbum`] table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the album to be deleted.
    ///
    /// # Returns
    /// * `Result<usize>` - Returns the number of rows affected by the delete operation.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database delete operation fails.
    ///
    /// # Behavior
    /// * Deletes the album entry with the specified ID from the [`DjmdAlbum`] table.
    /// * Removes any references to the album in the [`DjmdContent`] table by setting the `AlbumID` field to `None`.
    /// * Increments the local update sequence number (USN) after the operation.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let rows_deleted = db.delete_album("album_id").unwrap();
    /// println!("Number of rows deleted: {}", rows_deleted);
    /// ```
    pub fn delete_album(&mut self, id: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let result = diesel::delete(djmdAlbum.filter(schema::djmdAlbum::ID.eq(id)))
            .execute(&mut self.conn)?;

        // Remove any references to the album in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::AlbumID.eq(id)))
            .set(schema::djmdContent::AlbumID.eq(None::<String>))
            .execute(&mut self.conn)?;

        let _ = self.increment_local_usn(1);
        Ok(result)
    }

    // -- Artist -----------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdArtist`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdArtist>>` - A vector of [`DjmdArtist`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let artists = db.get_artist().unwrap();
    /// for artist in artists {
    ///     println!("{:?}", artist);
    /// }
    /// ```
    pub fn get_artist(&mut self) -> Result<Vec<DjmdArtist>> {
        let results: Vec<DjmdArtist> = djmdArtist.load(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdArtist`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the artist.
    ///
    /// # Returns
    /// * `Result<Option<DjmdArtist>>` - Returns an `Option` containing the [`DjmdArtist`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let artist = db.get_artist_by_id("some_id").unwrap();
    /// match artist {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_artist_by_id(&mut self, id: &str) -> Result<Option<DjmdArtist>> {
        let result: Option<DjmdArtist> = djmdArtist
            .filter(schema::djmdArtist::ID.eq(id))
            .first(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves a [`DjmdArtist`] entry by its name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the artist.
    ///
    /// # Returns
    /// * `Result<Option<DjmdArtist>>` - Returns an `Option` containing the [`DjmdArtist`] object
    ///   if found, or `None` if no entry matches the given name. Returns an error if multiple entries
    ///   match the given name.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed or if more than one artist
    ///   matches the given name.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let artist = db.get_artist_by_name("Artist Name").unwrap();
    /// match artist {
    ///     Some(artist) => println!("{:?}", artist),
    ///     None => println!("No artist found for the given name"),
    /// }
    /// ```
    pub fn get_artist_by_name(&mut self, name: &str) -> Result<Option<DjmdArtist>> {
        let results: Vec<DjmdArtist> = djmdArtist
            .filter(schema::djmdArtist::Name.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    /// Checks if a [`DjmdArtist`] with the given ID exists in the database.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the artist ID.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the artist exists, `false` otherwise.
    fn artist_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdArtist.filter(schema::djmdArtist::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    /// Generates a unique artist ID that does not exist in the database.
    ///
    /// # Returns
    /// * `Result<String>` - Returns a new unique artist ID as a string.
    fn generate_artist_id(&mut self) -> Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.artist_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Inserts a new artist into the [`DjmdArtist`] table in the database.
    ///
    /// # Arguments
    /// * `name` - The name of the artist to insert.
    ///
    /// # Returns
    /// * `Result<DjmdArtist>` - Returns the newly created [`DjmdArtist`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let artist = db.insert_artist("Artist Name".to_string()).unwrap();
    /// println!("{:?}", artist);
    /// ```
    pub fn insert_artist(&mut self, name: String) -> Result<DjmdArtist> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Generate ID/UUID
        let id: String = self.generate_artist_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date
        let utcnow: DateTime<Utc> = Utc::now();
        // Create and insert model
        let item: DjmdArtist = DjmdArtist::new(id, uuid, usn, utcnow, name)?;
        let result: DjmdArtist = diesel::insert_into(djmdArtist)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    /// Inserts a new [`DjmdArtist`] with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The name of the artist to insert or retrieve.
    ///
    /// # Returns
    /// * `Result<DjmdArtist>` - Returns the existing or newly created [`DjmdArtist`] object.
    ///
    /// # Errors
    /// * Returns an error if the artist cannot be inserted or retrieved.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// // Creates a new artist
    /// let artist = db.insert_artist_if_not_exists("Artist 1").unwrap();
    /// // Retrieves the existing artist
    /// let artist = db.insert_artist_if_not_exists("Artist 1").unwrap();
    /// ```
    fn insert_artist_if_not_exists(&mut self, name: &str) -> Result<DjmdArtist> {
        let artist = self.get_artist_by_name(name)?;
        if let Some(artist) = artist {
            Ok(artist)
        } else {
            let new = self.insert_artist(name.to_string())?;
            Ok(new)
        }
    }

    /// Updates an existing [`DjmdArtist`] entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdArtist`] object to be updated.
    ///
    /// # Returns
    /// * `Result<DjmdArtist>` - Returns the updated [`DjmdArtist`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database update operation fails.
    ///
    /// # Behavior
    /// * Compares the fields of the provided [`DjmdArtist`] object with the existing entry in the database.
    /// * If no differences are found, the existing entry is returned without making any updates.
    /// * If differences are found:
    ///   - Updates the `updated_at` timestamp to the current time.
    ///   - Increments the local update sequence number (USN) based on the number of differences.
    ///   - Updates the database entry with the modified fields.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let mut artist = db.get_artist_by_id("some_id").unwrap().unwrap();
    /// artist.Name = "New Artist Name".to_string();
    /// let updated_artist = db.update_artist(&mut artist).unwrap();
    /// println!("{:?}", updated_artist);
    /// ```
    pub fn update_artist(&mut self, item: &mut DjmdArtist) -> Result<DjmdArtist> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Count differences
        let existing: DjmdArtist = self.get_artist_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.Name != existing.Name {
            n += 1
        }
        if item.SearchStr != existing.SearchStr {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        item.updated_at = Utc::now();
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdArtist = diesel::update(&*item)
            .set(item.clone())
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    /// Deletes an artist entry from the [`DjmdArtist`] table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the artist to be deleted.
    ///
    /// # Returns
    /// * `Result<usize>` - Returns the number of rows affected by the delete operation.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database delete operation fails.
    ///
    /// # Behavior
    /// * Deletes the artist entry with the specified ID from the [`DjmdArtist`] table.
    /// * Removes any references to the artist in the [`DjmdContent`] table by setting the `ArtistID` and `OrgArtistID` fields to `None`.
    /// * Removes any references to the artist in the [`DjmdAlbum`] table by setting the `AlbumArtistID` field to `None`.
    /// * Increments the local update sequence number (USN) after the operation.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let rows_deleted = db.delete_artist("artist_id").unwrap();
    /// println!("Number of rows deleted: {}", rows_deleted);
    /// ```
    pub fn delete_artist(&mut self, id: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let result: usize = diesel::delete(djmdArtist.filter(schema::djmdArtist::ID.eq(id)))
            .execute(&mut self.conn)?;
        self.increment_local_usn(1)?;

        // Remove any references to the artist in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::ArtistID.eq(id)))
            .set(schema::djmdContent::ArtistID.eq(None::<String>))
            .execute(&mut self.conn)?;
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::OrgArtistID.eq(id)))
            .set(schema::djmdContent::OrgArtistID.eq(None::<String>))
            .execute(&mut self.conn)?;

        // Remove any references to the artist in DjmdAlbum
        let _ = diesel::update(djmdAlbum.filter(schema::djmdAlbum::AlbumArtistID.eq(id)))
            .set(schema::djmdAlbum::AlbumArtistID.eq(None::<String>))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- Category ---------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdCategory`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdCategory>>` - A vector of [`DjmdCategory`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let categories = db.get_category().unwrap();
    /// for category in categories {
    ///     println!("{:?}", category);
    /// }
    /// ```
    pub fn get_category(&mut self) -> Result<Vec<DjmdCategory>> {
        let results = djmdCategory
            .order_by(schema::djmdCategory::Seq)
            .load::<DjmdCategory>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdCategory`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the category.
    ///
    /// # Returns
    /// * `Result<Option<DjmdCategory>>` - Returns an `Option` containing the [`DjmdCategory`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let category = db.get_category_by_id("some_id").unwrap();
    /// match category {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_category_by_id(&mut self, id: &str) -> Result<Option<DjmdCategory>> {
        let result = djmdCategory
            .filter(schema::djmdCategory::ID.eq(id))
            .first::<DjmdCategory>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Color ------------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdColor`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdColor>>` - A vector of [`DjmdColor`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let colors = db.get_color().unwrap();
    /// for color in colors {
    ///     println!("{:?}", color);
    /// }
    /// ```
    pub fn get_color(&mut self) -> Result<Vec<DjmdColor>> {
        let results = djmdColor
            .order_by(schema::djmdColor::SortKey)
            .load::<DjmdColor>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdColor`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the color.
    ///
    /// # Returns
    /// * `Result<Option<DjmdColor>>` - Returns an `Option` containing the [`DjmdColor`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let color = db.get_color_by_id("some_id").unwrap();
    /// match color {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_color_by_id(&mut self, id: &str) -> Result<Option<DjmdColor>> {
        let result = djmdColor
            .filter(schema::djmdColor::ID.eq(id))
            .first::<DjmdColor>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Content ----------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdContent`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdContent>>` - A vector of [`DjmdContent`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let contents = db.get_content().unwrap();
    /// for content in contents {
    ///     println!("{:?}", content);
    /// }
    /// ```
    pub fn get_content(&mut self) -> Result<Vec<DjmdContent>> {
        let results = djmdContent.load::<DjmdContent>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdContent`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content.
    ///
    /// # Returns
    /// * `Result<Option<DjmdContent>>` - Returns an `Option` containing the [`DjmdContent`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let content = db.get_content_by_id("some_id").unwrap();
    /// match content {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_content_by_id(&mut self, id: &str) -> Result<Option<DjmdContent>> {
        let result = djmdContent
            .filter(schema::djmdContent::ID.eq(id))
            .first::<DjmdContent>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves multiple [`DjmdContent`] entries by their unique identifiers.
    ///
    /// # Arguments
    /// * `ids` - A vector of string slices representing the unique identifiers of the contents.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdContent>>` - A vector of [`DjmdContent`] objects found for the given IDs.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let contents = db.get_contents_by_ids(vec!["id1", "id2"]).unwrap();
    /// for content in contents {
    ///     println!("{:?}", content);
    /// }
    /// ```
    pub fn get_contents_by_ids(&mut self, ids: Vec<&str>) -> Result<Vec<DjmdContent>> {
        let mut result: Vec<DjmdContent> = Vec::new();
        for id in ids {
            let content = djmdContent
                .filter(schema::djmdContent::ID.eq(id))
                .first::<DjmdContent>(&mut self.conn)
                .optional()?;

            if let Some(content) = content {
                result.push(content);
            }
        }
        Ok(result)
    }

    /// Retrieves a [`DjmdContent`] entry by its folder path.
    ///
    /// # Arguments
    /// * `path` - A string slice representing the folder path of the content.
    ///
    /// # Returns
    /// * `Result<Option<DjmdContent>>` - Returns an `Option` containing the [`DjmdContent`] object
    ///   if found, or `None` if no entry matches the given path. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let content = db.get_content_by_path("/music/track.mp3").unwrap();
    /// match content {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given path"),
    /// }
    /// ```
    pub fn get_content_by_path(&mut self, path: &str) -> Result<Option<DjmdContent>> {
        let result = djmdContent
            .filter(schema::djmdContent::FolderPath.eq(path))
            .first::<DjmdContent>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Returns the path to the corresponding ANLZxxxx.DAT file for a given content ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content.
    ///
    /// # Returns
    /// * `Result<PathBuf>` - The canonicalized path to the analysis data file.
    ///
    /// # Errors
    /// * Returns an error if the share directory is not set or the path cannot be resolved.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let path = db.get_content_analysis_data_path("some_id").unwrap();
    /// println!("Analysis data path: {:?}", path);
    /// ```
    fn get_content_analysis_data_path(&mut self, id: &str) -> Result<PathBuf> {
        if self.share_dir.is_none() {
            return Err(anyhow::anyhow!("Share dir not set!"));
        }
        let result = djmdContent
            .filter(schema::djmdContent::ID.eq(id))
            .select(schema::djmdContent::AnalysisDataPath)
            .first::<Option<String>>(&mut self.conn)?;
        if let Some(result) = result {
            // Strip first "/" in result
            let striped = result.strip_prefix("/");
            if let Some(striped) = striped {
                let anlz_file = self.share_dir.clone().unwrap().join(striped);
                let anlz_files_canonicalized = dunce::canonicalize(&anlz_file);
                if let Err(e) = anlz_files_canonicalized {
                    return Err(anyhow::anyhow!("Failed to canonicalize path: {}", e));
                }
                return Ok(anlz_files_canonicalized?);
            }
        }
        Err(anyhow::anyhow!("Failed to get AnalysisDataPath"))
    }

    /// Returns the path to the directory containing the ANLZxxxx.xxx files for a given content ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content.
    ///
    /// # Returns
    /// * `Result<Option<PathBuf>>` - The path to the analysis directory, or `None` if not found.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let dir = db.get_content_anlz_dir("some_id").unwrap();
    /// println!("Analysis directory: {:?}", dir);
    /// ```
    pub fn get_content_anlz_dir(&mut self, id: &str) -> Result<Option<PathBuf>> {
        let anlz_file = self.get_content_analysis_data_path(id)?;
        let root = anlz_file.parent().unwrap();
        return Ok(Some(root.to_path_buf()));
    }

    /// Returns a struct containing the paths to ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 files.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content.
    ///
    /// # Returns
    /// * `Result<Option<AnlzPaths>>` - The [`AnlzPaths`] struct with analysis file paths, or `None` if not found.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let paths = db.get_content_anlz_paths("some_id").unwrap();
    /// println!("{:?}", paths);
    /// ```
    pub fn get_content_anlz_paths(&mut self, id: &str) -> Result<Option<AnlzPaths>> {
        let root = self.get_content_anlz_dir(id)?;
        if root.is_none() {
            return Ok(None);
        }
        find_anlz_files(root.unwrap())
    }

    /// Returns a struct containing the loaded ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 files.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content.
    ///
    /// # Returns
    /// * `Result<Option<AnlzFiles>>` - The [`AnlzFiles`] struct with loaded analysis files, or `None` if not found.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let files = db.get_content_anlz_files("some_id").unwrap();
    /// println!("{:?}", files);
    /// ```
    pub fn get_content_anlz_files(&mut self, id: &str) -> Result<Option<AnlzFiles>> {
        let paths = self.get_content_anlz_paths(id)?;
        if paths.is_none() {
            return Ok(None);
        }
        let paths = paths.unwrap();
        let mut files = AnlzFiles {
            dat: Anlz::load(paths.dat)?,
            ext: None,
            ex2: None,
        };
        if let Some(ext) = paths.ext {
            files.ext = Some(Anlz::load(ext)?);
        }
        if let Some(ex2) = paths.ex2 {
            files.ex2 = Some(Anlz::load(ex2)?);
        }
        Ok(Some(files))
    }

    /// Checks if a content entry with the given ID exists in the database.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the content ID.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the content exists, `false` otherwise.
    fn content_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdContent.filter(schema::djmdContent::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    /// Checks if a content entry with the given folder path exists in the database.
    ///
    /// # Arguments
    /// * `path` - A string slice representing the folder path.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the content exists, `false` otherwise.
    fn content_path_exists(&mut self, path: &str) -> Result<bool> {
        let exists: bool = select(exists(
            djmdContent.filter(schema::djmdContent::FolderPath.eq(path)),
        ))
        .get_result(&mut self.conn)?;
        Ok(exists)
    }

    /// Checks if a content entry with the given file ID exists in the database.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the file ID.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the file ID exists, `false` otherwise.
    fn content_file_id_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(
            djmdContent.filter(schema::djmdContent::rb_file_id.eq(id)),
        ))
        .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    /// Generates a unique content ID that does not exist in the database.
    ///
    /// # Returns
    /// * `Result<String>` - Returns a new unique content ID as a string.
    fn generate_content_id(&mut self) -> Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.content_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Generates a unique content file ID that does not exist in the database.
    ///
    /// # Returns
    /// * `Result<String>` - Returns a new unique content file ID as a string.
    fn generate_content_file_id(&mut self) -> Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.content_file_id_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Inserts a new entry into the [`DjmdContent`] table for the specified file path.
    ///
    /// **Note:** Not all fields of [`DjmdContent`] are set by this function. The user should update
    /// additional fields as needed after insertion. Also, after adding content via this method,
    /// you must run "reload tags" in Rekordbox to generate the corresponding analysis files
    /// (e.g., ANLZxxxx.DAT).
    ///
    /// # Arguments
    /// * `path` - The file path to be added as content. Accepts any type implementing [`AsRef<Path>`] and [`AsRef<OsStr>`].
    ///
    /// # Returns
    /// * `Result<DjmdContent>` - The newly inserted [`DjmdContent`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the file does not exist or is not a regular file.
    /// * Returns an error if the content already exists for the given path.
    /// * Returns an error if required metadata or IDs cannot be generated or retrieved.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let content = db.insert_content("/music/track.mp3").unwrap();
    /// // Update additional fields as needed
    /// // Run "reload tags" in Rekordbox to generate analysis files
    /// println!("{:?}", content);
    /// ```
    pub fn insert_content<P: AsRef<Path> + AsRef<OsStr>>(
        &mut self,
        path: P,
    ) -> Result<DjmdContent> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // prepare path and check if it exists
        let path = Path::new(&path);
        let rb_path = path.normalize_sep("/");
        let rb_path_str = rb_path
            .as_os_str()
            .to_str()
            .expect("Failed to convert path to string");
        if !path.is_file() || !path.exists() {
            return Err(anyhow::anyhow!(format!(
                "File {} is not a file or doesn't exist!",
                rb_path_str
            )));
        }
        if self.content_path_exists(&rb_path_str)? {
            return Err(anyhow::anyhow!(format!(
                "Content with path {} already exists",
                rb_path_str
            )));
        }
        // Generate ID/UUID
        let id: String = self.generate_content_id()?;
        let file_id: String = self.generate_content_file_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date
        let utcnow: DateTime<Utc> = Utc::now();
        // Get file metadata
        let meta = std::fs::metadata(&rb_path)?;
        let file_size: u64 = meta.len();
        let ext: &str = path.extension().unwrap().to_str().unwrap();
        let file_type = FileType::try_from_extension(ext)?;
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        // Get master DB ID
        let db_id = djmdDevice
            .select(schema::djmdDevice::MasterDBID)
            .first::<Option<String>>(&mut self.conn)?;
        if db_id.is_none() {
            return Err(anyhow::anyhow!("No master DB ID found in djmdDevice"));
        }
        // TODO: No clue what content link should be, for now we just choose the last value used
        let content_link = djmdContent
            .select(schema::djmdContent::ContentLink)
            .order(schema::djmdContent::rb_local_usn.desc())
            .limit(1)
            .first::<Option<i32>>(&mut self.conn)?;

        let item: DjmdContent = DjmdContent::new(
            id,
            uuid,
            usn,
            utcnow,
            rb_path_str.to_string(),
            file_name,
            file_type as i32,
            file_size.try_into().unwrap(),
            db_id.unwrap(),
            file_id,
            content_link,
        )?;

        let result: DjmdContent = diesel::insert_into(djmdContent)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    /// Updates an existing [`DjmdContent`] entry in the database.
    ///
    /// # Arguments
    /// * `item` - A reference to the [`DjmdContent`] object to be updated.
    ///
    /// # Returns
    /// * `Result<usize>` - The number of rows affected by the update operation.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database update operation fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let mut content = db.get_content_by_id("some_id").unwrap().unwrap();
    /// content.Title = "New Title".to_string();
    /// let rows_updated = db.update_content(&content).unwrap();
    /// println!("Rows updated: {}", rows_updated);
    /// ```
    pub fn update_content(&mut self, item: &DjmdContent) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let result = diesel::update(&*item)
            .set(item.clone())
            .execute(&mut self.conn)?;
        Ok(result)
    }

    /// Update the content album field.
    ///
    /// Sets the [DjmdContent.AlbumID] to the corresponding ID of the album.
    /// If the album does not exist yet, a new [DjmdAlbum] row will be created.
    pub fn update_content_album(&mut self, content_id: &str, name: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let album = self.insert_album_if_not_exists(name)?;

        let result = diesel::update(djmdContent.filter(schema::djmdContent::ID.eq(content_id)))
            .set(schema::djmdContent::AlbumID.eq(album.ID.clone()))
            .execute(&mut self.conn)?;
        Ok(result)
    }

    /// Update the content artist name.
    ///
    /// Sets the [DjmdContent.ArtistID] to the corresponding ID of the artist.
    /// If the artist does not exist yet, a new [DjmdArtist] row will be created.
    pub fn update_content_artist(&mut self, content_id: &str, name: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let artist = self.insert_artist_if_not_exists(name)?;

        let result = diesel::update(djmdContent.filter(schema::djmdContent::ID.eq(content_id)))
            .set(schema::djmdContent::ArtistID.eq(artist.ID.clone()))
            .execute(&mut self.conn)?;
        Ok(result)
    }

    /// Update the content remixer name.
    ///
    /// Sets the [DjmdContent.RemixerID] to the corresponding ID of the artist.
    /// If the artist does not exist yet, a new [DjmdArtist] row will be created.
    pub fn update_content_remixer(&mut self, content_id: &str, name: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let artist = self.insert_artist_if_not_exists(name)?;

        let result = diesel::update(djmdContent.filter(schema::djmdContent::ID.eq(content_id)))
            .set(schema::djmdContent::RemixerID.eq(artist.ID.clone()))
            .execute(&mut self.conn)?;
        Ok(result)
    }

    /// Update the content original artist name.
    ///
    /// Sets the [DjmdContent.OrgArtistID] to the corresponding ID of the artist.
    /// If the artist does not exist yet, a new [DjmdArtist] row will be created.
    pub fn update_content_original_artist(
        &mut self,
        content_id: &str,
        name: &str,
    ) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let artist = self.insert_artist_if_not_exists(name)?;

        let result = diesel::update(djmdContent.filter(schema::djmdContent::ID.eq(content_id)))
            .set(schema::djmdContent::OrgArtistID.eq(artist.ID.clone()))
            .execute(&mut self.conn)?;
        Ok(result)
    }

    /// Update the content composer name.
    ///
    /// Sets the [DjmdContent.ComposerID] to the corresponding ID of the artist.
    /// If the artist does not exist yet, a new [DjmdArtist] row will be created.
    pub fn update_content_composer(&mut self, content_id: &str, name: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let artist = self.insert_artist_if_not_exists(name)?;

        let result = diesel::update(djmdContent.filter(schema::djmdContent::ID.eq(content_id)))
            .set(schema::djmdContent::ComposerID.eq(artist.ID.clone()))
            .execute(&mut self.conn)?;
        Ok(result)
    }

    /// Update the content genre name.
    ///
    /// Sets the [DjmdContent.GenreID] to the corresponding ID of the genre.
    /// If the genre does not exist yet, a new [DjmdGenre] row will be created.
    pub fn update_content_genre(&mut self, content_id: &str, name: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let genre = self.insert_genre_if_not_exists(name)?;

        let result = diesel::update(djmdContent.filter(schema::djmdContent::ID.eq(content_id)))
            .set(schema::djmdContent::GenreID.eq(genre.ID.clone()))
            .execute(&mut self.conn)?;
        Ok(result)
    }

    /// Update the content label name.
    ///
    /// Sets the [DjmdContent.LabelID] to the corresponding ID of the label.
    /// If the label does not exist yet, a new [DjmdLabel] row will be created.
    pub fn update_content_label(&mut self, content_id: &str, name: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let label = self.insert_label_if_not_exists(name)?;

        let result = diesel::update(djmdContent.filter(schema::djmdContent::ID.eq(content_id)))
            .set(schema::djmdContent::LabelID.eq(label.ID.clone()))
            .execute(&mut self.conn)?;
        Ok(result)
    }

    /// Update the content key name.
    ///
    /// Sets the [DjmdContent.KeyID] to the corresponding ID of the label.
    /// If the key does not exist yet, a new [DjmdKey] row will be created.
    pub fn update_content_key(&mut self, content_id: &str, name: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let key = self.insert_key_if_not_exists(name)?;

        let result = diesel::update(djmdContent.filter(schema::djmdContent::ID.eq(content_id)))
            .set(schema::djmdContent::KeyID.eq(key.ID.clone()))
            .execute(&mut self.conn)?;
        Ok(result)
    }

    // pub fn delete_content(&mut self, id: &str) -> Result<usize> {
    //     // Check if Rekordbox is running
    //     if !self.unsafe_writes && is_rekordbox_running() {
    //         return Err(anyhow::anyhow!(
    //             "Rekordbox is running, unsafe writes are not allowed!"
    //         ));
    //     }
    //     let result = diesel::delete(djmdContent.filter(schema::djmdContent::ID.eq(id)))
    //         .execute(&mut self.conn)?;
    //     Ok(result)
    // }

    // -- Cue --------------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdCue`] table.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdCue>>` - Vector of all cue entries.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    ///
    /// # Example
    /// ```no_run
    /// let cues = db.get_cue()?;
    /// ```
    pub fn get_cue(&mut self) -> Result<Vec<DjmdCue>> {
        let results = djmdCue.load::<DjmdCue>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdCue`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - Cue entry ID.
    ///
    /// # Returns
    /// * `Result<Option<DjmdCue>>` - The cue entry if found, or `None`.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    ///
    /// # Example
    /// ```no_run
    /// let cue = db.get_cue_by_id("cue_id")?;
    /// ```
    pub fn get_cue_by_id(&mut self, id: &str) -> Result<Option<DjmdCue>> {
        let result = djmdCue
            .filter(schema::djmdCue::ID.eq(id))
            .first::<DjmdCue>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Device -----------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdDevice`] table.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdDevice>>` - Vector of all device entries.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    ///
    /// # Example
    /// ```no_run
    /// let devices = db.get_device()?;
    /// ```
    pub fn get_device(&mut self) -> Result<Vec<DjmdDevice>> {
        let results = djmdDevice.load::<DjmdDevice>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdDevice`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - Device entry ID.
    ///
    /// # Returns
    /// * `Result<Option<DjmdDevice>>` - The device entry if found, or `None`.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    ///
    /// # Example
    /// ```no_run
    /// let device = db.get_device_by_id("device_id")?;
    /// ```
    pub fn get_device_by_id(&mut self, id: &str) -> Result<Option<DjmdDevice>> {
        let result = djmdDevice
            .filter(schema::djmdDevice::ID.eq(id))
            .first::<DjmdDevice>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Genre ------------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdGenre`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdGenre>>` - A vector of [`DjmdGenre`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let genres = db.get_genre().unwrap();
    /// for genre in genres {
    ///     println!("{:?}", genre);
    /// }
    /// ```
    pub fn get_genre(&mut self) -> Result<Vec<DjmdGenre>> {
        let results = djmdGenre.load::<DjmdGenre>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdGenre`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the genre.
    ///
    /// # Returns
    /// * `Result<Option<DjmdGenre>>` - Returns an `Option` containing the [`DjmdGenre`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let genre = db.get_genre_by_id("some_id").unwrap();
    /// match genre {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_genre_by_id(&mut self, id: &str) -> Result<Option<DjmdGenre>> {
        let result = djmdGenre
            .filter(schema::djmdGenre::ID.eq(id))
            .first::<DjmdGenre>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves a [`DjmdGenre`] entry by its name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the genre.
    ///
    /// # Returns
    /// * `Result<Option<DjmdGenre>>` - Returns an `Option` containing the [`DjmdGenre`] object
    ///   if found, or `None` if no entry matches the given name. Returns an error if multiple entries
    ///   match the given name.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed or if more than one genre
    ///   matches the given name.
    ///
    /// # Example
    /// ```no_run
    /// let genre = db.get_genre_by_name("Genre Name").unwrap();
    /// match genre {
    ///     Some(genre) => println!("{:?}", genre),
    ///     None => println!("No genre found for the given name"),
    /// }
    /// ```
    pub fn get_genre_by_name(&mut self, name: &str) -> Result<Option<DjmdGenre>> {
        let results: Vec<DjmdGenre> = djmdGenre
            .filter(schema::djmdGenre::Name.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    /// Checks if a genre with the given ID exists in the database.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the genre ID.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the genre exists, `false` otherwise.
    fn genre_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdGenre.filter(schema::djmdGenre::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    /// Generates a unique genre ID that does not exist in the database.
    ///
    /// # Returns
    /// * `Result<String>` - Returns a new unique genre ID as a string.
    fn generate_genre_id(&mut self) -> Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.genre_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Inserts a new genre into the [`DjmdGenre`] table in the database.
    ///
    /// # Arguments
    /// * `name` - The name of the genre to insert.
    ///
    /// # Returns
    /// * `Result<DjmdGenre>` - Returns the newly created [`DjmdGenre`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let genre = db.insert_genre("Genre Name".to_string()).unwrap();
    /// println!("{:?}", genre);
    /// ```
    pub fn insert_genre(&mut self, name: String) -> Result<DjmdGenre> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Generate ID/UUID
        let id: String = self.generate_genre_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date
        let utcnow: DateTime<Utc> = Utc::now();
        // Create and insert model
        let item: DjmdGenre = DjmdGenre::new(id, uuid, usn, utcnow, name)?;
        let result: DjmdGenre = diesel::insert_into(djmdGenre)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    /// Inserts a new [`DjmdGenre`] with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The name of the genre to insert or retrieve.
    ///
    /// # Returns
    /// * `Result<DjmdGenre>` - Returns the existing or newly created [`DjmdGenre`] object.
    ///
    /// # Errors
    /// * Returns an error if the genre cannot be inserted or retrieved.
    ///
    /// # Example
    /// ```no_run
    /// let genre = db.insert_genre_if_not_exists("Genre 1").unwrap();
    /// ```
    fn insert_genre_if_not_exists(&mut self, name: &str) -> Result<DjmdGenre> {
        let genre = self.get_genre_by_name(name)?;
        if let Some(genre) = genre {
            Ok(genre)
        } else {
            let new = self.insert_genre(name.to_string())?;
            Ok(new)
        }
    }

    /// Updates an existing [`DjmdGenre`] entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdGenre`] object to be updated.
    ///
    /// # Returns
    /// * `Result<DjmdGenre>` - Returns the updated [`DjmdGenre`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database update operation fails.
    ///
    /// # Behavior
    /// * Compares the fields of the provided [`DjmdGenre`] object with the existing entry in the database.
    /// * If no differences are found, the existing entry is returned without making any updates.
    /// * If differences are found:
    ///   - Updates the `updated_at` timestamp to the current time.
    ///   - Increments the local update sequence number (USN) based on the number of differences.
    ///   - Updates the database entry with the modified fields.
    ///
    /// # Example
    /// ```no_run
    /// let mut genre = db.get_genre_by_id("some_id").unwrap().unwrap();
    /// genre.Name = "New Genre Name".to_string();
    /// let updated_genre = db.update_genre(&mut genre).unwrap();
    /// println!("{:?}", updated_genre);
    /// ```
    pub fn update_genre(&mut self, item: &mut DjmdGenre) -> Result<DjmdGenre> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Count differences
        let existing: DjmdGenre = self.get_genre_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.Name != existing.Name {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        item.updated_at = Utc::now();
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdGenre = diesel::update(&*item)
            .set(item.clone())
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    /// Deletes a genre entry from the [`DjmdGenre`] table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the genre to be deleted.
    ///
    /// # Returns
    /// * `Result<usize>` - Returns the number of rows affected by the delete operation.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database delete operation fails.
    ///
    /// # Behavior
    /// * Deletes the genre entry with the specified ID from the [`DjmdGenre`] table.
    /// * Removes any references to the genre in the [`DjmdContent`] table by setting the `GenreID` field to `None`.
    /// * Increments the local update sequence number (USN) after the operation.
    ///
    /// # Example
    /// ```no_run
    /// let rows_deleted = db.delete_genre("genre_id").unwrap();
    /// println!("Number of rows deleted: {}", rows_deleted);
    /// ```
    pub fn delete_genre(&mut self, id: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let result = diesel::delete(djmdGenre.filter(schema::djmdGenre::ID.eq(id)))
            .execute(&mut self.conn)?;
        self.increment_local_usn(1)?;

        // Remove any references to the genre in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::GenreID.eq(id)))
            .set(schema::djmdContent::GenreID.eq(None::<String>))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- History ----------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdHistory`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdHistory>>` - A vector of [`DjmdHistory`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let histories = db.get_history().unwrap();
    /// for history in histories {
    ///     println!("{:?}", history);
    /// }
    /// ```
    pub fn get_history(&mut self) -> Result<Vec<DjmdHistory>> {
        let results = djmdHistory
            .order_by(schema::djmdHistory::Seq)
            .load::<DjmdHistory>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdHistory`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the history entry.
    ///
    /// # Returns
    /// * `Result<Option<DjmdHistory>>` - Returns an `Option` containing the [`DjmdHistory`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let history = db.get_history_by_id("some_id").unwrap();
    /// match history {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_history_by_id(&mut self, id: &str) -> Result<Option<DjmdHistory>> {
        let result = djmdHistory
            .filter(schema::djmdHistory::ID.eq(id))
            .first::<DjmdHistory>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves all song history entries for a given history ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the history entry.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSongHistory>>` - A vector of [`DjmdSongHistory`] objects for the given history ID.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let songs = db.get_history_songs("history_id").unwrap();
    /// for song in songs {
    ///     println!("{:?}", song);
    /// }
    /// ```
    pub fn get_history_songs(&mut self, id: &str) -> Result<Vec<DjmdSongHistory>> {
        let results = schema::djmdSongHistory::table
            .filter(schema::djmdSongHistory::HistoryID.eq(id))
            .order_by(schema::djmdSongHistory::TrackNo)
            .load::<DjmdSongHistory>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all content entries referenced by the song history for a given history ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the history entry.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdContent>>` - A vector of [`DjmdContent`] objects referenced by the song history.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let contents = db.get_history_contents("history_id").unwrap();
    /// for content in contents {
    ///     println!("{:?}", content);
    /// }
    /// ```
    pub fn get_history_contents(&mut self, id: &str) -> Result<Vec<DjmdContent>> {
        let songs = self.get_history_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdHotCueBanklist`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdHotCueBanklist>>` - A vector of [`DjmdHotCueBanklist`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let banklists = db.get_hot_cue_banklist().unwrap();
    /// for banklist in banklists {
    ///     println!("{:?}", banklist);
    /// }
    /// ```
    pub fn get_hot_cue_banklist(&mut self) -> Result<Vec<DjmdHotCueBanklist>> {
        let results = djmdHotCueBanklist.load::<DjmdHotCueBanklist>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdHotCueBanklist`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the hot cue banklist entry.
    ///
    /// # Returns
    /// * `Result<Option<DjmdHotCueBanklist>>` - Returns an `Option` containing the [`DjmdHotCueBanklist`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let banklist = db.get_hot_cue_banklist_by_id("some_id").unwrap();
    /// match banklist {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_hot_cue_banklist_by_id(&mut self, id: &str) -> Result<Option<DjmdHotCueBanklist>> {
        let result = djmdHotCueBanklist
            .filter(schema::djmdHotCueBanklist::ID.eq(id))
            .first::<DjmdHotCueBanklist>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves all child [`DjmdHotCueBanklist`] entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent hot cue banklist ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdHotCueBanklist>>` - A vector of child [`DjmdHotCueBanklist`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let children = db.get_hot_cue_banklist_children("parent_id").unwrap();
    /// for child in children {
    ///     println!("{:?}", child);
    /// }
    /// ```
    pub fn get_hot_cue_banklist_children(
        &mut self,
        parent_id: &str,
    ) -> Result<Vec<DjmdHotCueBanklist>> {
        let results = djmdHotCueBanklist
            .filter(schema::djmdHotCueBanklist::ParentID.eq(parent_id))
            .order_by(schema::djmdHotCueBanklist::Seq)
            .load::<DjmdHotCueBanklist>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all [`DjmdSongHotCueBanklist`] entries for a given hot cue banklist ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the hot cue banklist ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSongHotCueBanklist>>` - A vector of [`DjmdSongHotCueBanklist`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let songs = db.get_hot_cue_banklist_songs("banklist_id").unwrap();
    /// for song in songs {
    ///     println!("{:?}", song);
    /// }
    /// ```
    pub fn get_hot_cue_banklist_songs(&mut self, id: &str) -> Result<Vec<DjmdSongHotCueBanklist>> {
        let results = schema::djmdSongHotCueBanklist::table
            .filter(schema::djmdSongHotCueBanklist::CueID.eq(id))
            .order_by(schema::djmdSongHotCueBanklist::TrackNo)
            .load::<DjmdSongHotCueBanklist>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all [`DjmdContent`] entries referenced by the song hot cue banklist for a given hot cue banklist ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the hot cue banklist ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdContent>>` - A vector of [`DjmdContent`] objects referenced by the song hot cue banklist.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let contents = db.get_hot_cue_banklist_contents("banklist_id").unwrap();
    /// for content in contents {
    ///     println!("{:?}", content);
    /// }
    /// ```
    pub fn get_hot_cue_banklist_contents(&mut self, id: &str) -> Result<Vec<DjmdContent>> {
        let songs = self.get_hot_cue_banklist_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    /// Retrieves all [`HotCueBanklistCue`] entries for a given hot cue banklist ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the hot cue banklist ID.
    ///
    /// # Returns
    /// * `Result<Vec<HotCueBanklistCue>>` - A vector of [`HotCueBanklistCue`] objects for the given banklist ID.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let cues = db.get_hot_cue_banklist_cues("banklist_id").unwrap();
    /// for cue in cues {
    ///     println!("{:?}", cue);
    /// }
    /// ```
    pub fn get_hot_cue_banklist_cues(&mut self, id: &str) -> Result<Vec<HotCueBanklistCue>> {
        let results = schema::hotCueBanklistCue::table
            .filter(schema::hotCueBanklistCue::HotCueBanklistID.eq(id))
            .load::<HotCueBanklistCue>(&mut self.conn)?;
        Ok(results)
    }

    // -- Key --------------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdKey`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdKey>>` - A vector of [`DjmdKey`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let keys = db.get_key().unwrap();
    /// for key in keys {
    ///     println!("{:?}", key);
    /// }
    /// ```
    pub fn get_key(&mut self) -> Result<Vec<DjmdKey>> {
        let results = djmdKey
            .order_by(schema::djmdKey::Seq)
            .load::<DjmdKey>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdKey`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the key.
    ///
    /// # Returns
    /// * `Result<Option<DjmdKey>>` - Returns an `Option` containing the [`DjmdKey`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let key = db.get_key_by_id("key_id").unwrap();
    /// match key {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_key_by_id(&mut self, id: &str) -> Result<Option<DjmdKey>> {
        let result = djmdKey
            .filter(schema::djmdKey::ID.eq(id))
            .first::<DjmdKey>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves a [`DjmdKey`] entry by its scale name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the scale name of the key.
    ///
    /// # Returns
    /// * `Result<Option<DjmdKey>>` - Returns an `Option` containing the [`DjmdKey`] object if found, or `None`.
    ///   Returns an error if multiple entries match the given name.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed or if more than one key matches the given name.
    ///
    /// # Example
    /// ```no_run
    /// let key = db.get_key_by_name("C#m").unwrap();
    /// match key {
    ///     Some(key) => println!("{:?}", key),
    ///     None => println!("No key found for the given name"),
    /// }
    /// ```
    pub fn get_key_by_name(&mut self, name: &str) -> Result<Option<DjmdKey>> {
        let results: Vec<DjmdKey> = djmdKey
            .filter(schema::djmdKey::ScaleName.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    /// Checks if a [`DjmdKey`] with the given ID exists in the database.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the key ID.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the key exists, `false` otherwise.
    fn key_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdKey.filter(schema::djmdKey::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    /// Generates a unique key ID that does not exist in the database.
    ///
    /// # Returns
    /// * `Result<String>` - Returns a new unique key ID as a string.
    fn generate_key_id(&mut self) -> Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.key_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Inserts a new key into the [`DjmdKey`] table in the database.
    ///
    /// # Arguments
    /// * `name` - The scale name of the key to insert.
    ///
    /// # Returns
    /// * `Result<DjmdKey>` - Returns the newly created [`DjmdKey`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let key = db.insert_key("C#m".to_string()).unwrap();
    /// println!("{:?}", key);
    /// ```
    pub fn insert_key(&mut self, name: String) -> Result<DjmdKey> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Generate ID/UUID
        let id: String = self.generate_key_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date
        let utcnow: DateTime<Utc> = Utc::now();
        // Count existing items for inserting at end
        let count = djmdKey.count().get_result::<i64>(&mut self.conn)? as i32;
        let seq: i32 = count + 1;

        // Create and insert model
        let item: DjmdKey = DjmdKey::new(id, uuid, usn, utcnow, name, seq)?;
        let result: DjmdKey = diesel::insert_into(djmdKey)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    /// Inserts a new [`DjmdKey`] with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The scale name of the key to insert or retrieve.
    ///
    /// # Returns
    /// * `Result<DjmdKey>` - Returns the existing or newly created [`DjmdKey`] object.
    ///
    /// # Errors
    /// * Returns an error if the key cannot be inserted or retrieved.
    fn insert_key_if_not_exists(&mut self, name: &str) -> Result<DjmdKey> {
        let key = self.get_key_by_name(name)?;
        if let Some(key) = key {
            Ok(key)
        } else {
            let new = self.insert_key(name.to_string())?;
            Ok(new)
        }
    }

    /// Updates an existing [`DjmdKey`] entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdKey`] object to be updated.
    ///
    /// # Returns
    /// * `Result<DjmdKey>` - Returns the updated [`DjmdKey`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database update operation fails.
    ///
    /// # Behavior
    /// * Compares the fields of the provided [`DjmdKey`] object with the existing entry in the database.
    /// * If no differences are found, the existing entry is returned without making any updates.
    /// * If differences are found:
    ///   - Updates the `updated_at` timestamp to the current time.
    ///   - Increments the local update sequence number (USN) based on the number of differences.
    ///   - Updates the database entry with the modified fields.
    ///
    /// # Example
    /// ```no_run
    /// let mut key = db.get_key_by_id("key_id").unwrap().unwrap();
    /// key.ScaleName = "Dm".to_string();
    /// let updated_key = db.update_key(&mut key).unwrap();
    /// println!("{:?}", updated_key);
    /// ```
    pub fn update_key(&mut self, item: &mut DjmdKey) -> Result<DjmdKey> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Count differences
        let existing: DjmdKey = self.get_key_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.ScaleName != existing.ScaleName {
            n += 1
        }
        if item.Seq != existing.Seq {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        item.updated_at = Utc::now();
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdKey = diesel::update(&*item)
            .set(item.clone())
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    /// Deletes a key entry from the [`DjmdKey`] table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the key to be deleted.
    ///
    /// # Returns
    /// * `Result<usize>` - Returns the number of rows affected by the delete operation.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database delete operation fails.
    ///
    /// # Behavior
    /// * Deletes the key entry with the specified ID from the [`DjmdKey`] table.
    /// * Removes any references to the key in the [`DjmdContent`] table by setting the `KeyID` field to `None`.
    /// * Increments the local update sequence number (USN) after the operation.
    ///
    /// # Example
    /// ```no_run
    /// let rows_deleted = db.delete_key("key_id").unwrap();
    /// println!("Number of rows deleted: {}", rows_deleted);
    /// ```
    pub fn delete_key(&mut self, id: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let result =
            diesel::delete(djmdKey.filter(schema::djmdKey::ID.eq(id))).execute(&mut self.conn)?;
        self.increment_local_usn(1)?;

        // Remove any references to the key in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::KeyID.eq(id)))
            .set(schema::djmdContent::KeyID.eq(None::<String>))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- Label ------------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdLabel`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdLabel>>` - A vector of [`DjmdLabel`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let labels = db.get_label().unwrap();
    /// for label in labels {
    ///     println!("{:?}", label);
    /// }
    /// ```
    pub fn get_label(&mut self) -> Result<Vec<DjmdLabel>> {
        let results = djmdLabel.load::<DjmdLabel>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdLabel`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the label.
    ///
    /// # Returns
    /// * `Result<Option<DjmdLabel>>` - Returns an `Option` containing the [`DjmdLabel`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let label = db.get_label_by_id("label_id").unwrap();
    /// match label {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_label_by_id(&mut self, id: &str) -> Result<Option<DjmdLabel>> {
        let result = djmdLabel
            .filter(schema::djmdLabel::ID.eq(id))
            .first::<DjmdLabel>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves a [`DjmdLabel`] entry by its name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the label.
    ///
    /// # Returns
    /// * `Result<Option<DjmdLabel>>` - Returns an `Option` containing the [`DjmdLabel`] object if found, or `None`.
    ///   Returns an error if multiple entries match the given name.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed or if more than one label matches the given name.
    ///
    /// # Example
    /// ```no_run
    /// let label = db.get_label_by_name("Label Name").unwrap();
    /// match label {
    ///     Some(label) => println!("{:?}", label),
    ///     None => println!("No label found for the given name"),
    /// }
    /// ```
    pub fn get_label_by_name(&mut self, name: &str) -> Result<Option<DjmdLabel>> {
        let results: Vec<DjmdLabel> = djmdLabel
            .filter(schema::djmdLabel::Name.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    /// Checks if a [`DjmdLabel`] with the given ID exists in the database.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the label ID.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the label exists, `false` otherwise.
    fn label_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdLabel.filter(schema::djmdLabel::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    /// Generates a unique label ID that does not exist in the database.
    ///
    /// # Returns
    /// * `Result<String>` - Returns a new unique label ID as a string.
    fn generate_label_id(&mut self) -> Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.label_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Inserts a new label into the [`DjmdLabel`] table in the database.
    ///
    /// # Arguments
    /// * `name` - The name of the label to insert.
    ///
    /// # Returns
    /// * `Result<DjmdLabel>` - Returns the newly created [`DjmdLabel`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let label = db.insert_label("Label Name".to_string()).unwrap();
    /// println!("{:?}", label);
    /// ```
    pub fn insert_label(&mut self, name: String) -> Result<DjmdLabel> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Generate ID/UUID
        let id: String = self.generate_label_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date
        let utcnow: DateTime<Utc> = Utc::now();
        // Create and insert model
        let item: DjmdLabel = DjmdLabel::new(id, uuid, usn, utcnow, name)?;
        let result: DjmdLabel = diesel::insert_into(djmdLabel)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    /// Inserts a new [`DjmdLabel`] with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The name of the label to insert or retrieve.
    ///
    /// # Returns
    /// * `Result<DjmdLabel>` - Returns the existing or newly created [`DjmdLabel`] object.
    ///
    /// # Errors
    /// * Returns an error if the label cannot be inserted or retrieved.
    fn insert_label_if_not_exists(&mut self, name: &str) -> Result<DjmdLabel> {
        let label = self.get_label_by_name(name)?;
        if let Some(label) = label {
            Ok(label)
        } else {
            let new = self.insert_label(name.to_string())?;
            Ok(new)
        }
    }

    /// Updates an existing [`DjmdLabel`] entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdLabel`] object to be updated.
    ///
    /// # Returns
    /// * `Result<DjmdLabel>` - Returns the updated [`DjmdLabel`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database update operation fails.
    ///
    /// # Behavior
    /// * Compares the fields of the provided [`DjmdLabel`] object with the existing entry in the database.
    /// * If no differences are found, the existing entry is returned without making any updates.
    /// * If differences are found:
    ///   - Updates the `updated_at` timestamp to the current time.
    ///   - Increments the local update sequence number (USN) based on the number of differences.
    ///   - Updates the database entry with the modified fields.
    ///
    /// # Example
    /// ```no_run
    /// let mut label = db.get_label_by_id("label_id").unwrap().unwrap();
    /// label.Name = "New Label Name".to_string();
    /// let updated_label = db.update_label(&mut label).unwrap();
    /// println
    pub fn update_label(&mut self, item: &mut DjmdLabel) -> Result<DjmdLabel> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Count differences
        let existing: DjmdLabel = self.get_label_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.Name != existing.Name {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        item.updated_at = Utc::now();
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdLabel = diesel::update(&*item)
            .set(item.clone())
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    /// Deletes a label entry from the [`DjmdLabel`] table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the label to be deleted.
    ///
    /// # Returns
    /// * `Result<usize>` - Returns the number of rows affected by the delete operation.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database delete operation fails.
    ///
    /// # Behavior
    /// * Deletes the label entry with the specified ID from the [`DjmdLabel`] table.
    /// * Removes any references to the label in the [`DjmdContent`] table by setting the `LabelID` field to `None`.
    /// * Increments the local update sequence number (USN) after the operation.
    ///
    /// # Example
    /// ```no_run
    /// let rows_deleted = db.delete_label("label_id").unwrap();
    /// println!("Number of rows deleted: {}", rows_deleted);
    /// ```
    pub fn delete_label(&mut self, id: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let result = diesel::delete(djmdLabel.filter(schema::djmdLabel::ID.eq(id)))
            .execute(&mut self.conn)?;
        self.increment_local_usn(1)?;

        // Remove any references to the key in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::LabelID.eq(id)))
            .set(schema::djmdContent::LabelID.eq(None::<String>))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- MenuItems --------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdMenuItems`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdMenuItems>>` - A vector of [`DjmdMenuItems`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let items = db.get_menu_item().unwrap();
    /// for item in items {
    ///     println!("{:?}", item);
    /// }
    /// ```
    pub fn get_menu_item(&mut self) -> Result<Vec<DjmdMenuItems>> {
        let results = djmdMenuItems.load::<DjmdMenuItems>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdMenuItems`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the menu item.
    ///
    /// # Returns
    /// * `Result<Option<DjmdMenuItems>>` - Returns an `Option` containing the [`DjmdMenuItems`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let item = db.get_menu_item_by_id("item_id").unwrap();
    /// match item {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_menu_item_by_id(&mut self, id: &str) -> Result<Option<DjmdMenuItems>> {
        let result = djmdMenuItems
            .filter(schema::djmdMenuItems::ID.eq(id))
            .first::<DjmdMenuItems>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- MixerParam -------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdMixerParam`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdMixerParam>>` - A vector of [`DjmdMixerParam`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let params = db.get_mixer_param().unwrap();
    /// for param in params {
    ///     println!("{:?}", param);
    /// }
    /// ```
    pub fn get_mixer_param(&mut self) -> Result<Vec<DjmdMixerParam>> {
        let results = djmdMixerParam.load::<DjmdMixerParam>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdMixerParam`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the mixer parameter.
    ///
    /// # Returns
    /// * `Result<Option<DjmdMixerParam>>` - Returns an `Option` containing the [`DjmdMixerParam`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let param = db.get_mixer_param_by_id("param_id").unwrap();
    /// match param {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_mixer_param_by_id(&mut self, id: &str) -> Result<Option<DjmdMixerParam>> {
        let result = djmdMixerParam
            .filter(schema::djmdMixerParam::ID.eq(id))
            .first::<DjmdMixerParam>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdMyTag`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdMyTag>>` - A vector of [`DjmdMyTag`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let tags = db.get_my_tag().unwrap();
    /// for tag in tags {
    ///     println!("{:?}", tag);
    /// }
    /// ```
    pub fn get_my_tag(&mut self) -> Result<Vec<DjmdMyTag>> {
        let results = djmdMyTag.load::<DjmdMyTag>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all child [`DjmdMyTag`] entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent tag ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdMyTag>>` - A vector of child [`DjmdMyTag`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let children = db.get_my_tag_children("parent_id").unwrap();
    /// for child in children {
    ///     println!("{:?}", child);
    /// }
    /// ```
    pub fn get_my_tag_children(&mut self, parent_id: &str) -> Result<Vec<DjmdMyTag>> {
        let results = djmdMyTag
            .filter(schema::djmdMyTag::ParentID.eq(parent_id))
            .order_by(schema::djmdMyTag::Seq)
            .load::<DjmdMyTag>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdMyTag`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the tag.
    ///
    /// # Returns
    /// * `Result<Option<DjmdMyTag>>` - Returns an `Option` containing the [`DjmdMyTag`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let tag = db.get_my_tag_by_id("tag_id").unwrap();
    /// match tag {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_my_tag_by_id(&mut self, id: &str) -> Result<Option<DjmdMyTag>> {
        let result = djmdMyTag
            .filter(schema::djmdMyTag::ID.eq(id))
            .first::<DjmdMyTag>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves all [`DjmdSongMyTag`] entries for a given tag ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the tag ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSongMyTag>>` - A vector of [`DjmdSongMyTag`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let songs = db.get_my_tag_songs("tag_id").unwrap();
    /// for song in songs {
    ///     println!("{:?}", song);
    /// }
    /// ```
    pub fn get_my_tag_songs(&mut self, id: &str) -> Result<Vec<DjmdSongMyTag>> {
        let results = schema::djmdSongMyTag::table
            .filter(schema::djmdSongMyTag::MyTagID.eq(id))
            .order_by(schema::djmdSongMyTag::TrackNo)
            .load::<DjmdSongMyTag>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all [`DjmdContent`] entries referenced by the songs for a given tag ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the tag ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdContent>>` - A vector of [`DjmdContent`] objects referenced by the tag.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let contents = db.get_my_tag_contents("tag_id").unwrap();
    /// for content in contents {
    ///     println!("{:?}", content);
    /// }
    /// ```
    pub fn get_my_tag_contents(&mut self, id: &str) -> Result<Vec<DjmdContent>> {
        let songs = self.get_my_tag_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    // -- Playlist ---------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdPlaylist`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdPlaylist>>` - A vector of [`DjmdPlaylist`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let playlists = db.get_playlist().unwrap();
    /// for playlist in playlists {
    ///     println!("{:?}", playlist);
    /// }
    /// ```
    pub fn get_playlist(&mut self) -> Result<Vec<DjmdPlaylist>> {
        let results = djmdPlaylist.load::<DjmdPlaylist>(&mut self.conn)?;
        Ok(results)
    }

    /// Returns a sorted tree of playlists as [`DjmdPlaylistTreeItem`] nodes.
    ///
    /// # Returns
    /// * `Result<Vec<Rc<RefCell<DjmdPlaylistTreeItem>>>>` - A vector of root nodes representing the playlist tree.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let tree = db.get_playlist_tree().unwrap();
    /// for root in tree {
    ///     println!("{:?}", root);
    /// }
    /// ```
    pub fn get_playlist_tree(&mut self) -> Result<Vec<Rc<RefCell<DjmdPlaylistTreeItem>>>> {
        let playlists: Vec<DjmdPlaylistTreeItem> = self
            .get_playlist()?
            .iter()
            .map(|p: &DjmdPlaylist| DjmdPlaylistTreeItem::from_playlist(p.clone()))
            .collect();

        let mut map = HashMap::<String, Rc<RefCell<DjmdPlaylistTreeItem>>>::new();
        let mut roots = Vec::new();

        // Populate the map with nodes
        for pl in playlists.iter() {
            let item = Rc::new(RefCell::new(pl.clone()));
            map.insert(pl.ID.clone(), item.clone());
            if pl.ParentID.is_none() || pl.ParentID == Some("root".to_string()) {
                roots.push(item.clone());
            }
        }

        // Build the tree structure
        for id in map.keys() {
            let node = map.get(id).unwrap();
            if let Some(parent_id) = node.borrow().ParentID.clone() {
                if let Some(parent_node) = map.get(&parent_id) {
                    parent_node.borrow_mut().Children.push(node.clone());
                }
            }
        }
        sort_tree_list(&mut roots);

        Ok(roots)
    }

    /// Retrieves all child [`DjmdPlaylist`] entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent playlist ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdPlaylist>>` - A vector of child [`DjmdPlaylist`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let children = db.get_playlist_children("parent_id").unwrap();
    /// for child in children {
    ///     println!("{:?}", child);
    /// }
    /// ```
    pub fn get_playlist_children(&mut self, parent_id: &str) -> Result<Vec<DjmdPlaylist>> {
        let results = djmdPlaylist
            .filter(schema::djmdPlaylist::ParentID.eq(parent_id))
            .order_by(schema::djmdPlaylist::Seq)
            .load::<DjmdPlaylist>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdPlaylist`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the playlist.
    ///
    /// # Returns
    /// * `Result<Option<DjmdPlaylist>>` - Returns an `Option` containing the [`DjmdPlaylist`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let playlist = db.get_playlist_by_id("playlist_id").unwrap();
    /// match playlist {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_playlist_by_id(&mut self, id: &str) -> Result<Option<DjmdPlaylist>> {
        let result = djmdPlaylist
            .filter(schema::djmdPlaylist::ID.eq(id))
            .first::<DjmdPlaylist>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves a [`DjmdPlaylist`] entry by its hierarchical path.
    ///
    /// # Arguments
    /// * `path` - A vector of string slices representing the playlist path.
    ///
    /// # Returns
    /// * `Result<Option<DjmdPlaylist>>` - Returns an `Option` containing the [`DjmdPlaylist`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let playlist = db.get_playlist_by_path(vec!["Folder", "Subfolder", "Playlist"]).unwrap();
    /// match playlist {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given path"),
    /// }
    /// ```
    pub fn get_playlist_by_path(&mut self, path: Vec<&str>) -> Result<Option<DjmdPlaylist>> {
        let mut parent_id: String = "root".to_string();
        let mut playlist: Option<DjmdPlaylist> = None;
        for name in path {
            let result = djmdPlaylist
                .filter(schema::djmdPlaylist::ParentID.eq(&parent_id))
                .filter(schema::djmdPlaylist::Name.eq(name))
                .first::<DjmdPlaylist>(&mut self.conn)
                .optional()?;
            playlist = result.clone();
            if let Some(result) = result {
                parent_id = result.ID.clone();
            } else {
                return Ok(None);
            }
        }
        Ok(playlist)
    }

    /// Retrieves all [`DjmdSongPlaylist`] entries for a given playlist ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the playlist ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSongPlaylist>>` - A vector of [`DjmdSongPlaylist`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let songs = db.get_playlist_songs("playlist_id").unwrap();
    /// for song in songs {
    ///     println!("{:?}", song);
    /// }
    /// ```
    pub fn get_playlist_songs(&mut self, id: &str) -> Result<Vec<DjmdSongPlaylist>> {
        let results = schema::djmdSongPlaylist::table
            .filter(schema::djmdSongPlaylist::PlaylistID.eq(id))
            .order_by(schema::djmdSongPlaylist::TrackNo)
            .load::<DjmdSongPlaylist>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all [`DjmdContent`] entries referenced by the songs for a given playlist ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the playlist ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdContent>>` - A vector of [`DjmdContent`] objects referenced by the playlist.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let contents = db.get_playlist_contents("playlist_id").unwrap();
    /// for content in contents {
    ///     println!("{:?}", content);
    /// }
    /// ```
    pub fn get_playlist_contents(&mut self, id: &str) -> Result<Vec<DjmdContent>> {
        let songs = self.get_playlist_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    /// Retrieves a [`DjmdSongPlaylist`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the song playlist entry.
    ///
    /// # Returns
    /// * `Result<Option<DjmdSongPlaylist>>` - Returns an `Option` containing the [`DjmdSongPlaylist`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let song = db.get_playlist_song_by_id("song_id").unwrap();
    /// match song {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_playlist_song_by_id(&mut self, id: &str) -> Result<Option<DjmdSongPlaylist>> {
        let result = djmdSongPlaylist
            .filter(schema::djmdSongPlaylist::ID.eq(id))
            .first::<DjmdSongPlaylist>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Returns the [`PlaylistType`] for a given playlist ID.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the playlist ID.
    ///
    /// # Returns
    /// * `Result<PlaylistType>` - The type of the playlist (e.g., folder, regular).
    ///
    /// # Errors
    /// * Returns an error if the playlist does not exist or has an invalid attribute.
    ///
    /// # Example
    /// ```no_run
    /// let ptype = db.playlist_type(&"playlist_id".to_string()).unwrap();
    /// println!("{:?}", ptype);
    /// ```
    fn playlist_type(&mut self, id: &String) -> Result<PlaylistType> {
        if id == "root" {
            return Ok(PlaylistType::Folder);
        }
        let result = djmdPlaylist
            .filter(schema::djmdPlaylist::ID.eq(id))
            .select(schema::djmdPlaylist::Attribute)
            .first::<Option<i32>>(&mut self.conn)?;
        let attr = result.expect("Playlist not found");
        let ptype = PlaylistType::try_from(attr).expect("Invalid playlist Attribute");
        Ok(ptype)
    }

    /// Checks if a playlist with the given ID exists in the database.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the playlist ID.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the playlist exists, `false` otherwise.
    ///
    /// # Example
    /// ```no_run
    /// let exists = db.playlist_exists(&"playlist_id".to_string()).unwrap();
    /// println!("Exists: {}", exists);
    /// ```
    fn playlist_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    /// Checks if a playlist song entry with the given ID exists in the database.
    ///
    /// # Arguments
    /// * `id` - A reference to a string representing the song playlist ID.
    ///
    /// # Returns
    /// * `Result<bool>` - Returns `true` if the song playlist entry exists, `false` otherwise.
    ///
    /// # Example
    /// ```no_run
    /// let exists = db.playlist_song_exists(&"song_id".to_string()).unwrap();
    /// println!("Exists: {}", exists);
    /// ```
    fn playlist_song_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(
            djmdSongPlaylist.filter(schema::djmdSongPlaylist::ID.eq(id)),
        ))
        .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    /// Generates a unique playlist ID that does not exist in the database.
    ///
    /// # Returns
    /// * `Result<String>` - Returns a new unique playlist ID as a string.
    ///
    /// # Example
    /// ```no_run
    /// let id = db.generate_playlist_id().unwrap();
    /// println!("Generated ID: {}", id);
    /// ```
    fn generate_playlist_id(&mut self) -> Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.playlist_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Generates a unique playlist song ID that does not exist in the database.
    ///
    /// # Returns
    /// * `Result<String>` - Returns a new unique playlist song ID as a string.
    ///
    /// # Example
    /// ```no_run
    /// let id = db.generate_playlist_song_id().unwrap();
    /// println!("Generated Song ID: {}", id);
    /// ```
    fn generate_playlist_song_id(&mut self) -> Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.playlist_song_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    /// Inserts a new playlist into the [`DjmdPlaylist`] table in the database.
    ///
    /// # Arguments
    /// * `name` - The name of the playlist to insert.
    /// * `attribute` - The [`PlaylistType`] attribute for the playlist (e.g., folder, regular).
    /// * `parent_id` - Optional parent playlist ID. Defaults to `"root"` if not provided.
    /// * `seq` - Optional sequence number for playlist ordering. If not provided, inserts at the end.
    /// * `image_path` - Optional image path for the playlist.
    /// * `smart_list` - Optional smart list configuration.
    ///
    /// # Returns
    /// * `Result<DjmdPlaylist>` - Returns the newly created [`DjmdPlaylist`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the parent playlist is not a folder or does not exist.
    /// * Returns an error if the sequence number is invalid or the database insertion fails.
    ///
    /// # Behavior
    /// * Validates the parent playlist and sequence.
    /// * Shifts existing playlists if inserting at a specific sequence.
    /// * Increments the local update sequence number (USN).
    /// * Updates the playlist XML file if configured.
    ///
    /// # Example
    /// ```no_run
    /// let playlist = db.insert_playlist(
    ///     "My Playlist".to_string(),
    ///     PlaylistType::Regular,
    ///     Some("parent_id".to_string()),
    ///     None,
    ///     None,
    ///     None,
    /// ).unwrap();
    /// println!("{:?}", playlist);
    /// ```
    pub fn insert_playlist(
        &mut self,
        name: String,
        attribute: PlaylistType,
        parent_id: Option<String>,
        seq: Option<i32>,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> Result<DjmdPlaylist> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Handle defaults
        let parent_id = parent_id.unwrap_or("root".to_string());
        if self.playlist_type(&parent_id)? != PlaylistType::Folder {
            return Err(anyhow!("Parent playlist must be a Folder!"));
        };
        if parent_id.as_str() != "root" && !self.playlist_exists(&parent_id)? {
            return Err(anyhow!("Parent playlist does not exist!"));
        };
        // Generate ID/UUID for new playlist
        let id: String = self.generate_playlist_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Generate date
        let utcnow = Utc::now();
        // Count existing playlists with same parent
        let count = djmdPlaylist
            .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
            .count()
            .get_result::<i64>(&mut self.conn)? as i32;

        // Handle seq
        let seq = if let Some(seq) = seq {
            // Playlist is not inserted at end
            if seq <= 0 {
                return Err(anyhow!("seq must be greater than 0!"));
            } else if seq > count + 1 {
                return Err(anyhow!("seq is too high!"));
            };
            // Shift playlists with seq higher than the new seq number by one
            // Shifting the other playlists increments the USN by 1 for *all* moved playlists
            let move_usn = self.increment_local_usn(1)?;
            diesel::update(
                djmdPlaylist
                    .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                    .filter(schema::djmdPlaylist::Seq.ge(seq)),
            )
            .set((
                schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq + 1),
                schema::djmdPlaylist::rb_local_usn.eq(move_usn.clone()),
                schema::djmdPlaylist::updated_at.eq(format_datetime(&utcnow)),
            ))
            .execute(&mut self.conn)?;
            seq
        } else {
            // Insert at end (count + 1)
            count + 1
        };

        // Get next USN: We increment by 2 (1 for creating, 1 for renaming from 'New Playlist')
        let usn: i32 = self.increment_local_usn(2)?;

        // Create and insert model
        let item: DjmdPlaylist = DjmdPlaylist::new(
            id.clone(),
            uuid,
            usn,
            utcnow,
            name,
            attribute.clone() as i32,
            parent_id.clone(),
            seq,
            image_path,
            smart_list,
        )?;
        let result: DjmdPlaylist = diesel::insert_into(djmdPlaylist)
            .values(item)
            .get_result(&mut self.conn)?;

        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            pl_xml.add(
                id.clone(),
                parent_id.clone(),
                attribute as i32,
                result.updated_at.naive_utc(),
            );
            let _ = pl_xml.dump();
        }

        Ok(result)
    }

    /// Creates a new regular playlist in the [`DjmdPlaylist`] table.
    ///
    /// # Arguments
    /// * `name` - The name of the playlist.
    /// * `parent_id` - Optional parent playlist ID.
    /// * `seq` - Optional sequence number for ordering.
    /// * `image_path` - Optional image path.
    /// * `smart_list` - Optional smart list configuration.
    ///
    /// # Returns
    /// * `Result<DjmdPlaylist>` - The newly created playlist.
    ///
    /// # Errors
    /// * Returns an error if the parent is invalid or database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let playlist = db.create_playlist("My Playlist".to_string(), None, None, None, None).unwrap();
    /// ```
    pub fn create_playlist(
        &mut self,
        name: String,
        parent_id: Option<String>,
        seq: Option<i32>,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> Result<DjmdPlaylist> {
        self.insert_playlist(
            name,
            PlaylistType::Playlist,
            parent_id,
            seq,
            image_path,
            smart_list,
        )
    }

    /// Creates a new playlist folder in the [`DjmdPlaylist`] table.
    ///
    /// # Arguments
    /// * `name` - The name of the folder.
    /// * `parent_id` - Optional parent folder ID.
    /// * `seq` - Optional sequence number for ordering.
    ///
    /// # Returns
    /// * `Result<DjmdPlaylist>` - The newly created folder.
    ///
    /// # Errors
    /// * Returns an error if the parent is invalid or database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let folder = db.create_playlist_folder("Folder".to_string(), None, None).unwrap();
    /// ```
    pub fn create_playlist_folder(
        &mut self,
        name: String,
        parent_id: Option<String>,
        seq: Option<i32>,
    ) -> Result<DjmdPlaylist> {
        self.insert_playlist(name, PlaylistType::Folder, parent_id, seq, None, None)
    }

    /// Renames an existing playlist.
    ///
    /// # Arguments
    /// * `id` - The playlist ID.
    /// * `name` - The new name for the playlist.
    ///
    /// # Returns
    /// * `Result<DjmdPlaylist>` - The updated playlist.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database update fails.
    ///
    /// # Example
    /// ```no_run
    /// let updated = db.rename_playlist(&"playlist_id".to_string(), "New Name".to_string()).unwrap();
    /// ```
    pub fn rename_playlist(&mut self, id: &String, name: String) -> Result<DjmdPlaylist> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let utcnow = Utc::now();
        let usn: i32 = self.increment_local_usn(1)?;
        let result: DjmdPlaylist =
            diesel::update(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id)))
                .set((
                    schema::djmdPlaylist::Name.eq(name),
                    schema::djmdPlaylist::rb_local_usn.eq(usn),
                    schema::djmdPlaylist::updated_at.eq(format_datetime(&utcnow)),
                ))
                .get_result(&mut self.conn)?;

        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            pl_xml.update(id.to_string(), result.updated_at.naive_utc());
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }

        Ok(result)
    }

    /// Moves a playlist to a new parent folder or changes its sequence within the same parent.
    ///
    /// # Arguments
    /// * `id` - The playlist ID to move.
    /// * `seq` - Optional new sequence number in the target parent.
    /// * `parent_id` - Optional new parent playlist ID. If not provided, keeps the current parent.
    ///
    /// # Returns
    /// * `Result<DjmdPlaylist>` - The updated [`DjmdPlaylist`] object.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the parent playlist is not a folder or does not exist.
    /// * Returns an error if the sequence number is invalid.
    ///
    /// # Behavior
    /// * Updates sequence numbers of affected playlists in both old and new parents.
    /// * Moves the playlist and updates its parent and sequence.
    /// * Increments the local update sequence number (USN).
    /// * Updates the playlist XML file if configured.
    ///
    /// # Example
    /// ```no_run
    /// let moved = db.move_playlist(&"playlist_id".to_string(), Some(2), Some("new_parent_id".to_string())).unwrap();
    /// println!("{:?}", moved);
    /// ```
    pub fn move_playlist(
        &mut self,
        id: &String,
        seq: Option<i32>,
        parent_id: Option<String>,
    ) -> Result<DjmdPlaylist> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Get playlist
        let playlist: DjmdPlaylist = self.get_playlist_by_id(id)?.expect("Playlist not found");

        // Get parentID (old parentID if None)
        let old_parent_id = playlist.ParentID.expect("No ParentID set");
        let old_seq = playlist.Seq.expect("No Seq set");
        let parent_id: String = parent_id.unwrap_or(old_parent_id.clone());

        // Moving increments USN by 1 for all changes
        let usn = self.increment_local_usn(1)?;
        // Generate date string
        let utcnow = Utc::now();
        let datestr: String = format_datetime(&utcnow);

        if parent_id != old_parent_id {
            let sequence: i32;
            // parent changed, move to new parent
            if self.playlist_type(&parent_id)? != PlaylistType::Folder {
                return Err(anyhow!("Parent playlist must be a Folder!"));
            };
            if parent_id.as_str() != "root" && !self.playlist_exists(&parent_id)? {
                return Err(anyhow!("Parent playlist does not exist!"));
            };

            // Update seq of playlists in old parent:
            // Decrease seq of playlists with seq higher than old seq
            diesel::update(
                djmdPlaylist
                    .filter(schema::djmdPlaylist::ParentID.eq(old_parent_id.clone()))
                    .filter(schema::djmdPlaylist::Seq.gt(old_seq)),
            )
            .set((
                schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq - 1),
                schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                schema::djmdPlaylist::updated_at.eq(datestr.clone()),
            ))
            .execute(&mut self.conn)?;
            if let Some(seq) = seq {
                let count = djmdPlaylist
                    .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                    .count()
                    .get_result::<i64>(&mut self.conn)? as i32;
                if seq <= 0 {
                    return Err(anyhow!("seq must be greater than 0!"));
                } else if seq > count {
                    return Err(anyhow!("seq is too high!"));
                }
                // Move to seq in new parent
                diesel::update(
                    djmdPlaylist
                        .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                        .filter(schema::djmdPlaylist::Seq.ge(seq)),
                )
                .set((
                    schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq + 1),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(datestr.clone()),
                ))
                .execute(&mut self.conn)?;

                sequence = seq;
            } else {
                // If no seq given, move to end of new parent:
                // No seq in new parent have to be updated
                let count = djmdPlaylist
                    .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                    .count()
                    .get_result::<i64>(&mut self.conn)? as i32;
                sequence = count + 1;
            };
            // Update Seq and parent of actual playlist
            diesel::update(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id)))
                .set((
                    schema::djmdPlaylist::Seq.eq(sequence),
                    schema::djmdPlaylist::ParentID.eq(parent_id.clone()),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(datestr.clone()),
                ))
                .execute(&mut self.conn)?;
        } else {
            let seq = seq.unwrap_or(old_seq.clone());
            let count = djmdPlaylist
                .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                .count()
                .get_result::<i64>(&mut self.conn)? as i32;
            if seq <= 0 {
                return Err(anyhow!("seq must be greater than 0!"));
            } else if seq > count {
                return Err(anyhow!("seq is too high!"));
            }

            // Move to seq in current parent
            if seq > old_seq {
                // Seq is increased (move down in playlist):
                // Decrease seq of all playlists with seq between old seq and new seq
                diesel::update(
                    djmdPlaylist
                        .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                        .filter(schema::djmdPlaylist::Seq.gt(old_seq))
                        .filter(schema::djmdPlaylist::Seq.le(seq)),
                )
                .set((
                    schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq - 1),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(datestr.clone()),
                ))
                .execute(&mut self.conn)?;
            } else if seq < old_seq {
                // Seq is decreased (moved up in playlist):
                // Increase seq of all playlists with seq between old seq and new seq
                diesel::update(
                    djmdPlaylist
                        .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                        .filter(schema::djmdPlaylist::Seq.lt(old_seq))
                        .filter(schema::djmdPlaylist::Seq.ge(seq)),
                )
                .set((
                    schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq + 1),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(datestr.clone()),
                ))
                .execute(&mut self.conn)?;
            };
            // Update Seq of actual playlist
            diesel::update(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id)))
                .set((
                    schema::djmdPlaylist::Seq.eq(seq),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(datestr.clone()),
                ))
                .execute(&mut self.conn)?;
        };

        let playlist: DjmdPlaylist = self.get_playlist_by_id(id)?.expect("Playlist not found");

        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            pl_xml.update_parent(
                id.to_string(),
                parent_id.clone(),
                playlist.updated_at.naive_utc(),
            );
            // Update update-time of all child items
            for pl in self.get_playlist_children(&parent_id)? {
                pl_xml.update(pl.ID, pl.updated_at.naive_utc());
            }
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }

        Ok(playlist)
    }

    /// Deletes a playlist entry from the [`DjmdPlaylist`] table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the playlist to be deleted.
    ///
    /// # Returns
    /// * `Result<usize>` - The number of rows affected by the delete operation.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the playlist does not exist or the database operation fails.
    ///
    /// # Behavior
    /// * Decreases the sequence of all sibling playlists with a higher sequence.
    /// * Deletes the playlist and cascades deletion to all child playlists (if folder) and songs (if regular playlist).
    /// * Updates the playlist XML file if configured.
    ///
    /// # Example
    /// ```no_run
    /// let rows_deleted = db.delete_playlist("playlist_id").unwrap();
    /// println!("Number of rows deleted: {}", rows_deleted);
    /// ```
    pub fn delete_playlist(&mut self, id: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let usn = self.increment_local_usn(1)?;
        // Generate date
        let utcnow: DateTime<Utc> = Utc::now();

        // Get parentID and seq number
        let playlist: DjmdPlaylist = self.get_playlist_by_id(id)?.expect("Playlist not found");
        let attr = playlist.Attribute.unwrap();
        let playlist_type = PlaylistType::try_from(attr).expect("Invalid playlist Attribute");
        let parent_id = playlist.ParentID.unwrap();
        let seq = playlist.Seq.unwrap();

        // Decrease seq of all playlists with seq higher then seq of deleted playlist
        diesel::update(
            djmdPlaylist
                .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                .filter(schema::djmdPlaylist::Seq.gt(seq)),
        )
        .set((
            schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq - 1),
            schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
            schema::djmdPlaylist::updated_at.eq(format_datetime(&utcnow)),
        ))
        .execute(&mut self.conn)?;

        // Delete playlist
        let mut result = diesel::delete(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id)))
            .execute(&mut self.conn)?;

        // Cascade delete references
        if playlist_type == PlaylistType::Playlist {
            // Delete all songs in playlist
            let n = diesel::delete(
                djmdSongPlaylist.filter(schema::djmdSongPlaylist::PlaylistID.eq(id)),
            )
            .execute(&mut self.conn)?;
            result += n;
        } else if playlist_type == PlaylistType::Folder {
            // Select child ids
            let children: Vec<String> = djmdPlaylist
                .filter(schema::djmdPlaylist::ParentID.eq(id))
                .select(schema::djmdPlaylist::ID)
                .load::<String>(&mut self.conn)?;
            // Delete all children recursively
            for child_id in children {
                let n = self.delete_playlist(&child_id)?;
                result += n;
            }
        }

        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            pl_xml.remove(id.to_string());
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }

        Ok(result)
    }

    /// Inserts a [`DjmdSongPlaylist`] song into a playlist at a specified sequence position.
    ///
    /// # Arguments
    /// * `playlist_id` - The ID of the target playlist.
    /// * `content_id` - The ID of the content (song) to insert.
    /// * `seq` - Optional sequence number for the song's position. If not provided, inserts at the end.
    ///
    /// # Returns
    /// * `Result<DjmdSongPlaylist>` - The newly created [`DjmdSongPlaylist`] object.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the playlist or content does not exist.
    /// * Returns an error if the sequence number is invalid.
    ///
    /// # Behavior
    /// * Shifts songs with a higher sequence if inserting at a specific position.
    /// * Increments the local update sequence number (USN).
    ///
    /// # Example
    /// ```no_run
    /// let song = db.insert_playlist_song(&"playlist_id".to_string(), &"content_id".to_string(), Some(2)).unwrap();
    /// println!("{:?}", song);
    /// ```
    pub fn insert_playlist_song(
        &mut self,
        playlist_id: &String,
        content_id: &String,
        seq: Option<i32>,
    ) -> Result<DjmdSongPlaylist> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        if self.playlist_type(&playlist_id)? != PlaylistType::Playlist {
            return Err(anyhow!(
                "Playlist with ID {} must be a Playlist!",
                playlist_id
            ));
        };
        if !self.playlist_exists(&playlist_id)? {
            return Err(anyhow!("Playlist with ID {} does not exist!", playlist_id));
        };
        if !self.content_exists(&content_id)? {
            return Err(anyhow!("Content with ID {} does not exist!", content_id));
        };

        // Generate ID/UUID/Date for new playlist song
        let id: String = self.generate_playlist_song_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        let utcnow: DateTime<Utc> = Utc::now();

        // Count existing songs in playlist
        let count = djmdSongPlaylist
            .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
            .count()
            .get_result::<i64>(&mut self.conn)? as i32;

        // Get next USN: We increment by 1 *before* moving others
        let usn: i32 = self.increment_local_usn(1)?;

        // Handle seq
        let seq = if let Some(seq) = seq {
            // Song is not inserted at end
            if seq <= 0 {
                return Err(anyhow!("seq must be greater than 0!"));
            } else if seq > count {
                return Err(anyhow!("seq is too high!"));
            };
            // Shift songs with seq higher than the new seq number by one
            // Shifting the other songs increments the USN by 1 for *all* moved songs
            let move_usn = self.increment_local_usn(1)?;
            diesel::update(
                djmdSongPlaylist
                    .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
                    .filter(schema::djmdSongPlaylist::TrackNo.ge(seq)),
            )
            .set((
                schema::djmdSongPlaylist::TrackNo.eq(schema::djmdSongPlaylist::TrackNo + 1),
                schema::djmdSongPlaylist::rb_local_usn.eq(move_usn.clone()),
                schema::djmdSongPlaylist::updated_at.eq(format_datetime(&utcnow)),
            ))
            .execute(&mut self.conn)?;
            seq
        } else {
            // Insert at end (count + 1)
            count + 1
        };

        let item = DjmdSongPlaylist::new(
            id,
            uuid,
            usn,
            utcnow,
            playlist_id.clone(),
            content_id.clone(),
            seq,
        )?;
        let result: DjmdSongPlaylist = diesel::insert_into(djmdSongPlaylist)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    /// Moves a song within a playlist to a new sequence position.
    ///
    /// # Arguments
    /// * `id` - The ID of the song playlist entry to move.
    /// * `seq` - The new sequence number for the song.
    ///
    /// # Returns
    /// * `Result<DjmdSongPlaylist>` - The updated [`DjmdSongPlaylist`] object.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the sequence number is invalid or the song does not exist.
    ///
    /// # Behavior
    /// * Updates sequence numbers of affected songs.
    /// * Moves the song and updates its sequence.
    /// * Increments the local update sequence number (USN).
    ///
    /// # Example
    /// ```no_run
    /// let moved = db.move_playlist_song(&"song_id".to_string(), 2).unwrap();
    /// println!("{:?}", moved);
    /// ```
    pub fn move_playlist_song(&mut self, id: &String, seq: i32) -> Result<DjmdSongPlaylist> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        // Get playlistID and seq number
        let song: DjmdSongPlaylist = self.get_playlist_song_by_id(id)?.expect("Song not found");
        if song.TrackNo.unwrap() == seq {
            // Nothing to change
            return Ok(song);
        }
        let count = djmdSongPlaylist
            .filter(schema::djmdSongPlaylist::PlaylistID.eq(song.PlaylistID.clone()))
            .count()
            .get_result::<i64>(&mut self.conn)? as i32;

        if seq <= 0 {
            return Err(anyhow!("seq must be greater than 0!"));
        } else if seq > count {
            return Err(anyhow!("seq is too high!"));
        }

        let playlist_id = song.PlaylistID.unwrap();
        let old_seq = song.TrackNo.unwrap();

        let usn = self.increment_local_usn(1)?;
        let utcnow: DateTime<Utc> = Utc::now();
        let datestr: String = format_datetime(&utcnow);
        if seq > old_seq {
            // Seq is increased (move down in playlist):
            // Decrease seq of all songs with seq between old seq and new seq
            diesel::update(
                djmdSongPlaylist
                    .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
                    .filter(schema::djmdSongPlaylist::TrackNo.gt(old_seq))
                    .filter(schema::djmdSongPlaylist::TrackNo.le(seq)),
            )
            .set((
                schema::djmdSongPlaylist::TrackNo.eq(schema::djmdSongPlaylist::TrackNo - 1),
                schema::djmdSongPlaylist::rb_local_usn.eq(usn.clone()),
                schema::djmdSongPlaylist::updated_at.eq(datestr.clone()),
            ))
            .execute(&mut self.conn)?;
        } else {
            // Seq is decreased (moved up in playlist):
            // Increase seq of all songs with seq between old seq and new seq
            diesel::update(
                djmdSongPlaylist
                    .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
                    .filter(schema::djmdSongPlaylist::TrackNo.le(old_seq))
                    .filter(schema::djmdSongPlaylist::TrackNo.ge(seq)),
            )
            .set((
                schema::djmdSongPlaylist::TrackNo.eq(schema::djmdSongPlaylist::TrackNo + 1),
                schema::djmdSongPlaylist::rb_local_usn.eq(usn.clone()),
                schema::djmdSongPlaylist::updated_at.eq(datestr.clone()),
            ))
            .execute(&mut self.conn)?;
        };
        // Update Seq of actual song
        diesel::update(djmdSongPlaylist.filter(schema::djmdSongPlaylist::ID.eq(id)))
            .set((
                schema::djmdSongPlaylist::TrackNo.eq(seq),
                schema::djmdSongPlaylist::rb_local_usn.eq(usn.clone()),
                schema::djmdSongPlaylist::updated_at.eq(datestr.clone()),
            ))
            .execute(&mut self.conn)?;

        let result: DjmdSongPlaylist = self.get_playlist_song_by_id(id)?.expect("Song not found");

        Ok(result)
    }

    /// Deletes a song entry from a playlist.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the song playlist entry to be deleted.
    ///
    /// # Returns
    /// * `Result<usize>` - The number of rows affected by the delete operation.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the song does not exist or the database operation fails.
    ///
    /// # Behavior
    /// * Decreases the sequence of all sibling songs with a higher sequence.
    /// * Deletes the song from the playlist.
    ///
    /// # Example
    /// ```no_run
    /// let rows_deleted = db.delete_playlist_song("song_id").unwrap();
    /// println!("Number of rows deleted: {}", rows_deleted);
    /// ```
    pub fn delete_playlist_song(&mut self, id: &str) -> Result<usize> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(anyhow::anyhow!(
                "Rekordbox is running, unsafe writes are not allowed!"
            ));
        }
        let usn = self.increment_local_usn(1)?;
        let utcnow: DateTime<Utc> = Utc::now();
        let datestr: String = format_datetime(&utcnow);

        // Get playlist and seq number
        let song: DjmdSongPlaylist = self.get_playlist_song_by_id(id)?.expect("Song not found");
        let playlist_id = song.PlaylistID.unwrap();
        let seq = song.TrackNo.unwrap();

        // Decrease seq of all songs with seq higher then seq of deleted song
        diesel::update(
            djmdSongPlaylist
                .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
                .filter(schema::djmdSongPlaylist::TrackNo.gt(seq)),
        )
        .set((
            schema::djmdSongPlaylist::TrackNo.eq(schema::djmdSongPlaylist::TrackNo - 1),
            schema::djmdSongPlaylist::rb_local_usn.eq(usn.clone()),
            schema::djmdSongPlaylist::updated_at.eq(datestr),
        ))
        .execute(&mut self.conn)?;

        let result = diesel::delete(djmdSongPlaylist.filter(schema::djmdSongPlaylist::ID.eq(id)))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- Property ---------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdProperty`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdProperty>>` - A vector of [`DjmdProperty`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let properties = db.get_property().unwrap();
    /// for property in properties {
    ///     println!("{:?}", property);
    /// }
    /// ```
    pub fn get_property(&mut self) -> Result<Vec<DjmdProperty>> {
        let results = djmdProperty.load::<DjmdProperty>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdProperty`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the property.
    ///
    /// # Returns
    /// * `Result<Option<DjmdProperty>>` - Returns an `Option` containing the [`DjmdProperty`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let property = db.get_property_by_id("property_id").unwrap();
    /// match property {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_property_by_id(&mut self, id: &str) -> Result<Option<DjmdProperty>> {
        let result = djmdProperty
            .filter(schema::djmdProperty::DBID.eq(id))
            .first::<DjmdProperty>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdCloudProperty`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdCloudProperty>>` - A vector of [`DjmdCloudProperty`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let cloud_properties = db.get_cloud_property().unwrap();
    /// for property in cloud_properties {
    ///     println!("{:?}", property);
    /// }
    /// ```
    pub fn get_cloud_property(&mut self) -> Result<Vec<DjmdCloudProperty>> {
        let results = djmdCloudProperty.load::<DjmdCloudProperty>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdCloudProperty`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the cloud property.
    ///
    /// # Returns
    /// * `Result<Option<DjmdCloudProperty>>` - Returns an `Option` containing the [`DjmdCloudProperty`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let cloud_property = db.get_cloud_property_by_id("cloud_property_id").unwrap();
    /// match cloud_property {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_cloud_property_by_id(&mut self, id: &str) -> Result<Option<DjmdCloudProperty>> {
        let result = djmdCloudProperty
            .filter(schema::djmdCloudProperty::ID.eq(id))
            .first::<DjmdCloudProperty>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdRecommendLike`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdRecommendLike>>` - A vector of [`DjmdRecommendLike`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let likes = db.get_recommend_like().unwrap();
    /// for like in likes {
    ///     println!("{:?}", like);
    /// }
    /// ```
    pub fn get_recommend_like(&mut self) -> Result<Vec<DjmdRecommendLike>> {
        let results = djmdRecommendLike.load::<DjmdRecommendLike>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdRecommendLike`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the recommend like entry.
    ///
    /// # Returns
    /// * `Result<Option<DjmdRecommendLike>>` - Returns an `Option` containing the [`DjmdRecommendLike`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let like = db.get_recommend_like_by_id("like_id").unwrap();
    /// match like {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_recommend_like_by_id(&mut self, id: &str) -> Result<Option<DjmdRecommendLike>> {
        let result = djmdRecommendLike
            .filter(schema::djmdRecommendLike::ID.eq(id))
            .first::<DjmdRecommendLike>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdRelatedTracks`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdRelatedTracks>>` - A vector of [`DjmdRelatedTracks`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let related_tracks = db.get_related_tracks().unwrap();
    /// for track in related_tracks {
    ///     println!("{:?}", track);
    /// }
    /// ```
    pub fn get_related_tracks(&mut self) -> Result<Vec<DjmdRelatedTracks>> {
        let results = djmdRelatedTracks.load::<DjmdRelatedTracks>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all child [`DjmdRelatedTracks`] entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent related tracks ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdRelatedTracks>>` - A vector of child [`DjmdRelatedTracks`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let children = db.get_related_tracks_children("parent_id").unwrap();
    /// for child in children {
    ///     println!("{:?}", child);
    /// }
    /// ```
    pub fn get_related_tracks_children(
        &mut self,
        parent_id: &str,
    ) -> Result<Vec<DjmdRelatedTracks>> {
        let results = djmdRelatedTracks
            .filter(schema::djmdRelatedTracks::ParentID.eq(parent_id))
            .order_by(schema::djmdRelatedTracks::Seq)
            .load::<DjmdRelatedTracks>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdRelatedTracks`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the related tracks entry.
    ///
    /// # Returns
    /// * `Result<Option<DjmdRelatedTracks>>` - Returns an `Option` containing the [`DjmdRelatedTracks`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let track = db.get_related_tracks_by_id("track_id").unwrap();
    /// match track {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_related_tracks_by_id(&mut self, id: &str) -> Result<Option<DjmdRelatedTracks>> {
        let result = djmdRelatedTracks
            .filter(schema::djmdRelatedTracks::ID.eq(id))
            .first::<DjmdRelatedTracks>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves all [`DjmdSongRelatedTracks`] entries for a given related tracks ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the related tracks ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSongRelatedTracks>>` - A vector of [`DjmdSongRelatedTracks`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let songs = db.get_related_tracks_songs("related_tracks_id").unwrap();
    /// for song in songs {
    ///     println!("{:?}", song);
    /// }
    /// ```
    pub fn get_related_tracks_songs(&mut self, id: &str) -> Result<Vec<DjmdSongRelatedTracks>> {
        let results = schema::djmdSongRelatedTracks::table
            .filter(schema::djmdSongRelatedTracks::RelatedTracksID.eq(id))
            .order_by(schema::djmdSongRelatedTracks::TrackNo)
            .load::<DjmdSongRelatedTracks>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all [`DjmdContent`] entries referenced by the songs for a given related tracks ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the related tracks ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdContent>>` - A vector of [`DjmdContent`] objects referenced by the related tracks.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let contents = db.get_related_tracks_contents("related_tracks_id").unwrap();
    /// for content in contents {
    ///     println!("{:?}", content);
    /// }
    /// ```
    pub fn get_related_tracks_contents(&mut self, id: &str) -> Result<Vec<DjmdContent>> {
        let songs = self.get_related_tracks_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    // -- Sampler ----------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdSampler`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSampler>>` - A vector of [`DjmdSampler`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let samplers = db.get_sampler().unwrap();
    /// for sampler in samplers {
    ///     println!("{:?}", sampler);
    /// }
    /// ```
    pub fn get_sampler(&mut self) -> Result<Vec<DjmdSampler>> {
        let results = djmdSampler.load::<DjmdSampler>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all child [`DjmdSampler`] entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent sampler ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSampler>>` - A vector of child [`DjmdSampler`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let children = db.get_sampler_children("parent_id").unwrap();
    /// for child in children {
    ///     println!("{:?}", child);
    /// }
    /// ```
    pub fn get_sampler_children(&mut self, parent_id: &str) -> Result<Vec<DjmdSampler>> {
        let results = djmdSampler
            .filter(schema::djmdSampler::ParentID.eq(parent_id))
            .order_by(schema::djmdSampler::Seq)
            .load::<DjmdSampler>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdSampler`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the sampler entry.
    ///
    /// # Returns
    /// * `Result<Option<DjmdSampler>>` - Returns an `Option` containing the [`DjmdSampler`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let sampler = db.get_sampler_by_id("sampler_id").unwrap();
    /// match sampler {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_sampler_by_id(&mut self, id: &str) -> Result<Option<DjmdSampler>> {
        let result = djmdSampler
            .filter(schema::djmdSampler::ID.eq(id))
            .first::<DjmdSampler>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Retrieves all [`DjmdSongSampler`] entries for a given sampler ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the sampler ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSongSampler>>` - A vector of [`DjmdSongSampler`] objects.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let songs = db.get_sampler_songs("sampler_id").unwrap();
    /// for song in songs {
    ///     println!("{:?}", song);
    /// }
    /// ```
    pub fn get_sampler_songs(&mut self, id: &str) -> Result<Vec<DjmdSongSampler>> {
        let results = schema::djmdSongSampler::table
            .filter(schema::djmdSongSampler::SamplerID.eq(id))
            .order_by(schema::djmdSongSampler::TrackNo)
            .load::<DjmdSongSampler>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves all [`DjmdContent`] entries referenced by the songs for a given sampler ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the sampler ID.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdContent>>` - A vector of [`DjmdContent`] objects referenced by the sampler.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let contents = db.get_sampler_contents("sampler_id").unwrap();
    /// for content in contents {
    ///     println!("{:?}", content);
    /// }
    /// ```
    pub fn get_sampler_contents(&mut self, id: &str) -> Result<Vec<DjmdContent>> {
        let songs = self.get_sampler_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    // -- SongTagList ------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdSongTagList`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSongTagList>>` - A vector of [`DjmdSongTagList`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let tags = db.get_song_tag_list().unwrap();
    /// for tag in tags {
    ///     println!("{:?}", tag);
    /// }
    /// ```
    pub fn get_song_tag_list(&mut self) -> Result<Vec<DjmdSongTagList>> {
        let results = djmdSongTagList
            .order_by(schema::djmdSongTagList::TrackNo)
            .load::<DjmdSongTagList>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdSongTagList`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the song tag list entry.
    ///
    /// # Returns
    /// * `Result<Option<DjmdSongTagList>>` - Returns an `Option` containing the [`DjmdSongTagList`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let tag = db.get_song_tag_list_by_id("tag_id").unwrap();
    /// match tag {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_song_tag_list_by_id(&mut self, id: &str) -> Result<Option<DjmdSongTagList>> {
        let result = djmdSongTagList
            .filter(schema::djmdSongTagList::ID.eq(id))
            .first::<DjmdSongTagList>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Sort -------------------------------------------------------------------------------------

    /// Retrieves all entries from the [`DjmdSort`] table in the database, ordered by sequence.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdSort>>` - A vector of [`DjmdSort`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let sorts = db.get_sort().unwrap();
    /// for sort in sorts {
    ///     println!("{:?}", sort);
    /// }
    /// ```
    pub fn get_sort(&mut self) -> Result<Vec<DjmdSort>> {
        let results = djmdSort
            .order_by(schema::djmdSort::Seq)
            .load::<DjmdSort>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`DjmdSort`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the sort entry.
    ///
    /// # Returns
    /// * `Result<Option<DjmdSort>>` - Returns an `Option` containing the [`DjmdSort`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let sort = db.get_sort_by_id("sort_id").unwrap();
    /// match sort {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_sort_by_id(&mut self, id: &str) -> Result<Option<DjmdSort>> {
        let result = djmdSort
            .filter(schema::djmdSort::ID.eq(id))
            .first::<DjmdSort>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ImageFile --------------------------------------------------------------------------------

    /// Retrieves all entries from the [`ImageFile`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<ImageFile>>` - A vector of [`ImageFile`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let images = db.get_image_file().unwrap();
    /// for image in images {
    ///     println!("{:?}", image);
    /// }
    /// ```
    pub fn get_image_file(&mut self) -> Result<Vec<ImageFile>> {
        let results = imageFile.load::<ImageFile>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves an [`ImageFile`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the image file.
    ///
    /// # Returns
    /// * `Result<Option<ImageFile>>` - Returns an `Option` containing the [`ImageFile`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let image = db.get_image_file_by_id("image_id").unwrap();
    /// match image {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_image_file_by_id(&mut self, id: &str) -> Result<Option<ImageFile>> {
        let result = imageFile
            .filter(schema::imageFile::ID.eq(id))
            .first::<ImageFile>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- SettingFile ------------------------------------------------------------------------------

    /// Retrieves all entries from the [`SettingFile`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<SettingFile>>` - A vector of [`SettingFile`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let settings = db.get_setting_file().unwrap();
    /// for setting in settings {
    ///     println!("{:?}", setting);
    /// }
    /// ```
    pub fn get_setting_file(&mut self) -> Result<Vec<SettingFile>> {
        let results = settingFile.load::<SettingFile>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`SettingFile`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the setting file.
    ///
    /// # Returns
    /// * `Result<Option<SettingFile>>` - Returns an `Option` containing the [`SettingFile`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let setting = db.get_setting_file_by_id("setting_id").unwrap();
    /// match setting {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_setting_file_by_id(&mut self, id: &str) -> Result<Option<SettingFile>> {
        let result = settingFile
            .filter(schema::settingFile::ID.eq(id))
            .first::<SettingFile>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    /// Retrieves all entries from the [`UuidIDMap`] table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<UuidIDMap>>` - A vector of [`UuidIDMap`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let maps = db.get_uuid_id_map().unwrap();
    /// for map in maps {
    ///     println!("{:?}", map);
    /// }
    /// ```
    pub fn get_uuid_id_map(&mut self) -> Result<Vec<UuidIDMap>> {
        let results = uuidIDMap.load::<UuidIDMap>(&mut self.conn)?;
        Ok(results)
    }

    /// Retrieves a [`UuidIDMap`] entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the UUID ID map entry.
    ///
    /// # Returns
    /// * `Result<Option<UuidIDMap>>` - Returns an `Option` containing the [`UuidIDMap`] object if found, or `None`.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let map = db.get_uuid_id_map_by_id("map_id").unwrap();
    /// match map {
    ///     Some(entry) => println!("{:?}", entry),
    ///     None => println!("No entry found for the given ID"),
    /// }
    /// ```
    pub fn get_uuid_id_map_by_id(&mut self, id: &str) -> Result<Option<UuidIDMap>> {
        let result = uuidIDMap
            .filter(schema::uuidIDMap::ID.eq(id))
            .first::<UuidIDMap>(&mut self.conn)
            .optional()?;
        Ok(result)
    }
}
