// Author: Dylan Jones
// Date:   2025-05-01
/// This module serves as the handler for the Rekordbox `master.db` database.
///
/// # Purpose
/// Provides functionality for interacting with the Rekordbox database, including
/// querying, updating, and managing various database tables and entries.
///
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
///
/// * `path` - A string slice that holds the path to the SQLite database file.
///
/// # Returns
///
/// * `Result<SqliteConnection>` - Returns a `SqliteConnection` object if successful, or an error.
///
/// # Errors
///
/// * Returns an error if the database connection cannot be established or if any of the
///   configuration commands fail.
///
/// # Configuration
///
/// The function applies the following settings to the SQLite connection:
/// - Sets the encryption key using the `PRAGMA key` command.
/// - Enables foreign key constraints using the `PRAGMA foreign_keys = ON` command.
/// - Sets the journal mode to Write-Ahead Logging (WAL) using the `PRAGMA journal_mode = WAL` command.
/// - Sets the synchronous mode to NORMAL using the `PRAGMA synchronous = NORMAL` command.
///
/// # Example
///
/// ```rust
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
    ///
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
    ///
    /// * `Result<Vec<AgentRegistry>>` - A vector of [`AgentRegistry`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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
    ///
    /// * `id` - A string slice representing the unique identifier of the agent registry entry.
    ///
    /// # Returns
    ///
    /// * `Result<Option<AgentRegistry>>` - Returns an `Option` containing the [`AgentRegistry`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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
    ///
    /// * `Result<i32>` - Returns the local USN as an integer if found, or an error if the entry
    ///   does not exist.
    ///
    /// # Errors
    ///
    /// * Returns an error if the `localUpdateCount` entry is not found in the [`AgentRegistry`] table
    ///   or if the database query fails.
    ///
    /// # Example
    ///
    /// ```rust
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
    ///
    /// * `usn` - An integer value representing the amount to increment the local USN.
    ///
    /// # Returns
    ///
    /// * `Result<i32>` - Returns the updated USN as an integer if successful, or an error.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database update operation fails or if the updated USN is not found.
    ///
    /// # Example
    ///
    /// ```rust
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
    ///
    /// * `Result<Vec<CloudAgentRegistry>>` - A vector of [`CloudAgentRegistry`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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
    ///
    /// * `id` - A string slice representing the unique identifier of the cloud agent registry entry.
    ///
    /// # Returns
    ///
    /// * `Result<Option<CloudAgentRegistry>>` - Returns an `Option` containing the [`CloudAgentRegistry`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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
    ///
    /// * `Result<Vec<ContentActiveCensor>>` - A vector of [`ContentActiveCensor`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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
    ///
    /// * `id` - A string slice representing the unique identifier of the content active censor entry.
    ///
    /// # Returns
    ///
    /// * `Result<Option<ContentActiveCensor>>` - Returns an `Option` containing the [`ContentActiveCensor`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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

    /// Retrieves all entries from the `ContentCue` table in the database.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<ContentCue>>` - A vector of [`ContentCue`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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

    /// Retrieves a `ContentCue` entry by its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - A string slice representing the unique identifier of the content cue entry.
    ///
    /// # Returns
    ///
    /// * `Result<Option<ContentCue>>` - Returns an `Option` containing the [`ContentCue`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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

    /// Retrieves all entries from the `contentFile` table in the database.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<ContentFile>>` - A vector of [`ContentFile`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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

    /// Retrieves a `ContentFile` entry by its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - A string slice representing the unique identifier of the content file entry.
    ///
    /// # Returns
    ///
    /// * `Result<Option<ContentFile>>` - Returns an `Option` containing the [`ContentFile`] object
    ///   if found, or `None` if no entry matches the given identifier. Returns an error if the query fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    ///
    /// ```rust
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

    /// Retrieves all entries from the `DjmdActiveCensor` table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdActiveCensor>>` - A vector of [`DjmdActiveCensor`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```rust
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

    /// Retrieves a `DjmdActiveCensor` entry by its unique identifier.
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
    /// ```rust
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

    /// Retrieves all entries from the `DjmdAlbum` table in the database.
    ///
    /// # Returns
    /// * `Result<Vec<DjmdAlbum>>` - A vector of [`DjmdAlbum`] objects if the query is successful,
    ///   or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```rust
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

    /// Retrieves a `DjmdAlbum` entry by its unique identifier.
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
    /// ```rust
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

    /// Retrieves a `DjmdAlbum` entry by its name.
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
    /// ```rust
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

    fn album_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdAlbum.filter(schema::djmdAlbum::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

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

    /// Inserts a new album into the `DjmdAlbum` table in the database.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the album.
    /// * `artist_id` - An optional identifier for the album artist.
    /// * `image_path` - An optional path to the album's image.
    /// * `compilation` - An optional integer indicating whether the album is a compilation.
    ///
    /// # Returns
    ///
    /// * `Result<DjmdAlbum>` - Returns the newly created [`DjmdAlbum`] object if successful, or an error.
    ///
    /// # Errors
    ///
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    ///
    /// ```rust
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

    fn insert_album_if_not_exists(&mut self, name: &str) -> Result<DjmdAlbum> {
        let album = self.get_album_by_name(name)?;
        if let Some(album) = album {
            Ok(album)
        } else {
            let new = self.insert_album(name.to_string(), None, None, None)?;
            Ok(new)
        }
    }

    /// Updates an existing `DjmdAlbum` entry in the database.
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
    /// ```rust
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

    /// Deletes an album entry from the `DjmdAlbum` table in the database.
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
    /// ```rust
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

    pub fn get_artist(&mut self) -> Result<Vec<DjmdArtist>> {
        let results: Vec<DjmdArtist> = djmdArtist.load(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_artist_by_id(&mut self, id: &str) -> Result<Option<DjmdArtist>> {
        let result: Option<DjmdArtist> = djmdArtist
            .filter(schema::djmdArtist::ID.eq(id))
            .first(&mut self.conn)
            .optional()?;
        Ok(result)
    }

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

    fn artist_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdArtist.filter(schema::djmdArtist::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

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

    fn insert_artist_if_not_exists(&mut self, name: &str) -> Result<DjmdArtist> {
        let artist = self.get_artist_by_name(name)?;
        if let Some(artist) = artist {
            Ok(artist)
        } else {
            let new = self.insert_artist(name.to_string())?;
            Ok(new)
        }
    }

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

    pub fn get_category(&mut self) -> Result<Vec<DjmdCategory>> {
        let results = djmdCategory
            .order_by(schema::djmdCategory::Seq)
            .load::<DjmdCategory>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_category_by_id(&mut self, id: &str) -> Result<Option<DjmdCategory>> {
        let result = djmdCategory
            .filter(schema::djmdCategory::ID.eq(id))
            .first::<DjmdCategory>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Color ------------------------------------------------------------------------------------

    pub fn get_color(&mut self) -> Result<Vec<DjmdColor>> {
        let results = djmdColor
            .order_by(schema::djmdColor::SortKey)
            .load::<DjmdColor>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_color_by_id(&mut self, id: &str) -> Result<Option<DjmdColor>> {
        let result = djmdColor
            .filter(schema::djmdColor::ID.eq(id))
            .first::<DjmdColor>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Content ----------------------------------------------------------------------------------

    pub fn get_content(&mut self) -> Result<Vec<DjmdContent>> {
        let results = djmdContent.load::<DjmdContent>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_content_by_id(&mut self, id: &str) -> Result<Option<DjmdContent>> {
        let result = djmdContent
            .filter(schema::djmdContent::ID.eq(id))
            .first::<DjmdContent>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

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

    pub fn get_content_by_path(&mut self, path: &str) -> Result<Option<DjmdContent>> {
        let result = djmdContent
            .filter(schema::djmdContent::FolderPath.eq(path))
            .first::<DjmdContent>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    /// Returns the path to the corresponding ANLZxxxx.DAT file
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

    /// Returns the path to the directory containing the ANLZxxxx.xxx files
    pub fn get_content_anlz_dir(&mut self, id: &str) -> Result<Option<PathBuf>> {
        let anlz_file = self.get_content_analysis_data_path(id)?;
        let root = anlz_file.parent().unwrap();
        return Ok(Some(root.to_path_buf()));
    }

    /// Returns a struct containing the  ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 paths
    pub fn get_content_anlz_paths(&mut self, id: &str) -> Result<Option<AnlzPaths>> {
        let root = self.get_content_anlz_dir(id)?;
        if root.is_none() {
            return Ok(None);
        }
        find_anlz_files(root.unwrap())
    }

    /// Returns a struct containing the loaded ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 files
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

    fn content_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdContent.filter(schema::djmdContent::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    fn content_path_exists(&mut self, path: &str) -> Result<bool> {
        let exists: bool = select(exists(
            djmdContent.filter(schema::djmdContent::FolderPath.eq(path)),
        ))
        .get_result(&mut self.conn)?;
        Ok(exists)
    }

    fn content_file_id_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(
            djmdContent.filter(schema::djmdContent::rb_file_id.eq(id)),
        ))
        .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

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

    pub fn get_cue(&mut self) -> Result<Vec<DjmdCue>> {
        let results = djmdCue.load::<DjmdCue>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_cue_by_id(&mut self, id: &str) -> Result<Option<DjmdCue>> {
        let result = djmdCue
            .filter(schema::djmdCue::ID.eq(id))
            .first::<DjmdCue>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Device -----------------------------------------------------------------------------------

    pub fn get_device(&mut self) -> Result<Vec<DjmdDevice>> {
        let results = djmdDevice.load::<DjmdDevice>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_device_by_id(&mut self, id: &str) -> Result<Option<DjmdDevice>> {
        let result = djmdDevice
            .filter(schema::djmdDevice::ID.eq(id))
            .first::<DjmdDevice>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Genre ------------------------------------------------------------------------------------

    pub fn get_genre(&mut self) -> Result<Vec<DjmdGenre>> {
        let results = djmdGenre.load::<DjmdGenre>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_genre_by_id(&mut self, id: &str) -> Result<Option<DjmdGenre>> {
        let result = djmdGenre
            .filter(schema::djmdGenre::ID.eq(id))
            .first::<DjmdGenre>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

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

    fn genre_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdGenre.filter(schema::djmdGenre::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

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

    fn insert_genre_if_not_exists(&mut self, name: &str) -> Result<DjmdGenre> {
        let genre = self.get_genre_by_name(name)?;
        if let Some(genre) = genre {
            Ok(genre)
        } else {
            let new = self.insert_genre(name.to_string())?;
            Ok(new)
        }
    }

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

    pub fn get_history(&mut self) -> Result<Vec<DjmdHistory>> {
        let results = djmdHistory
            .order_by(schema::djmdHistory::Seq)
            .load::<DjmdHistory>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_history_by_id(&mut self, id: &str) -> Result<Option<DjmdHistory>> {
        let result = djmdHistory
            .filter(schema::djmdHistory::ID.eq(id))
            .first::<DjmdHistory>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_history_songs(&mut self, id: &str) -> Result<Vec<DjmdSongHistory>> {
        let results = schema::djmdSongHistory::table
            .filter(schema::djmdSongHistory::HistoryID.eq(id))
            .order_by(schema::djmdSongHistory::TrackNo)
            .load::<DjmdSongHistory>(&mut self.conn)?;
        Ok(results)
    }

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

    pub fn get_hot_cue_banklist(&mut self) -> Result<Vec<DjmdHotCueBanklist>> {
        let results = djmdHotCueBanklist.load::<DjmdHotCueBanklist>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_hot_cue_banklist_by_id(&mut self, id: &str) -> Result<Option<DjmdHotCueBanklist>> {
        let result = djmdHotCueBanklist
            .filter(schema::djmdHotCueBanklist::ID.eq(id))
            .first::<DjmdHotCueBanklist>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

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

    pub fn get_hot_cue_banklist_songs(&mut self, id: &str) -> Result<Vec<DjmdSongHotCueBanklist>> {
        let results = schema::djmdSongHotCueBanklist::table
            .filter(schema::djmdSongHotCueBanklist::CueID.eq(id))
            .order_by(schema::djmdSongHotCueBanklist::TrackNo)
            .load::<DjmdSongHotCueBanklist>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_hot_cue_banklist_contents(&mut self, id: &str) -> Result<Vec<DjmdContent>> {
        let songs = self.get_hot_cue_banklist_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    pub fn get_hot_cue_banklist_cues(&mut self, id: &str) -> Result<Vec<HotCueBanklistCue>> {
        let results = schema::hotCueBanklistCue::table
            .filter(schema::hotCueBanklistCue::HotCueBanklistID.eq(id))
            .load::<HotCueBanklistCue>(&mut self.conn)?;
        Ok(results)
    }

    // -- Key --------------------------------------------------------------------------------------

    pub fn get_key(&mut self) -> Result<Vec<DjmdKey>> {
        let results = djmdKey
            .order_by(schema::djmdKey::Seq)
            .load::<DjmdKey>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_key_by_id(&mut self, id: &str) -> Result<Option<DjmdKey>> {
        let result = djmdKey
            .filter(schema::djmdKey::ID.eq(id))
            .first::<DjmdKey>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

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

    fn key_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdKey.filter(schema::djmdKey::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

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

    fn insert_key_if_not_exists(&mut self, name: &str) -> Result<DjmdKey> {
        let key = self.get_key_by_name(name)?;
        if let Some(key) = key {
            Ok(key)
        } else {
            let new = self.insert_key(name.to_string())?;
            Ok(new)
        }
    }

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

    pub fn get_label(&mut self) -> Result<Vec<DjmdLabel>> {
        let results = djmdLabel.load::<DjmdLabel>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_label_by_id(&mut self, id: &str) -> Result<Option<DjmdLabel>> {
        let result = djmdLabel
            .filter(schema::djmdLabel::ID.eq(id))
            .first::<DjmdLabel>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

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

    fn label_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdLabel.filter(schema::djmdLabel::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

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

    fn insert_label_if_not_exists(&mut self, name: &str) -> Result<DjmdLabel> {
        let label = self.get_label_by_name(name)?;
        if let Some(label) = label {
            Ok(label)
        } else {
            let new = self.insert_label(name.to_string())?;
            Ok(new)
        }
    }

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

    pub fn get_menu_item(&mut self) -> Result<Vec<DjmdMenuItems>> {
        let results = djmdMenuItems.load::<DjmdMenuItems>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_menu_item_by_id(&mut self, id: &str) -> Result<Option<DjmdMenuItems>> {
        let result = djmdMenuItems
            .filter(schema::djmdMenuItems::ID.eq(id))
            .first::<DjmdMenuItems>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- MixerParam -------------------------------------------------------------------------------

    pub fn get_mixer_param(&mut self) -> Result<Vec<DjmdMixerParam>> {
        let results = djmdMixerParam.load::<DjmdMixerParam>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_mixer_param_by_id(&mut self, id: &str) -> Result<Option<DjmdMixerParam>> {
        let result = djmdMixerParam
            .filter(schema::djmdMixerParam::ID.eq(id))
            .first::<DjmdMixerParam>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    pub fn get_my_tag(&mut self) -> Result<Vec<DjmdMyTag>> {
        let results = djmdMyTag.load::<DjmdMyTag>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_my_tag_children(&mut self, parent_id: &str) -> Result<Vec<DjmdMyTag>> {
        let results = djmdMyTag
            .filter(schema::djmdMyTag::ParentID.eq(parent_id))
            .order_by(schema::djmdMyTag::Seq)
            .load::<DjmdMyTag>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_my_tag_by_id(&mut self, id: &str) -> Result<Option<DjmdMyTag>> {
        let result = djmdMyTag
            .filter(schema::djmdMyTag::ID.eq(id))
            .first::<DjmdMyTag>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_my_tag_songs(&mut self, id: &str) -> Result<Vec<DjmdSongMyTag>> {
        let results = schema::djmdSongMyTag::table
            .filter(schema::djmdSongMyTag::MyTagID.eq(id))
            .order_by(schema::djmdSongMyTag::TrackNo)
            .load::<DjmdSongMyTag>(&mut self.conn)?;
        Ok(results)
    }

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

    pub fn get_playlist(&mut self) -> Result<Vec<DjmdPlaylist>> {
        let results = djmdPlaylist.load::<DjmdPlaylist>(&mut self.conn)?;
        Ok(results)
    }

    /// Return a sorted tree of playlists
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

    pub fn get_playlist_children(&mut self, parent_id: &str) -> Result<Vec<DjmdPlaylist>> {
        let results = djmdPlaylist
            .filter(schema::djmdPlaylist::ParentID.eq(parent_id))
            .order_by(schema::djmdPlaylist::Seq)
            .load::<DjmdPlaylist>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_playlist_by_id(&mut self, id: &str) -> Result<Option<DjmdPlaylist>> {
        let result = djmdPlaylist
            .filter(schema::djmdPlaylist::ID.eq(id))
            .first::<DjmdPlaylist>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

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

    pub fn get_playlist_songs(&mut self, id: &str) -> Result<Vec<DjmdSongPlaylist>> {
        let results = schema::djmdSongPlaylist::table
            .filter(schema::djmdSongPlaylist::PlaylistID.eq(id))
            .order_by(schema::djmdSongPlaylist::TrackNo)
            .load::<DjmdSongPlaylist>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_playlist_contents(&mut self, id: &str) -> Result<Vec<DjmdContent>> {
        let songs = self.get_playlist_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    pub fn get_playlist_song_by_id(&mut self, id: &str) -> Result<Option<DjmdSongPlaylist>> {
        let result = djmdSongPlaylist
            .filter(schema::djmdSongPlaylist::ID.eq(id))
            .first::<DjmdSongPlaylist>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

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

    fn playlist_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    fn playlist_song_exists(&mut self, id: &String) -> Result<bool> {
        let id_exists: bool = select(exists(
            djmdSongPlaylist.filter(schema::djmdSongPlaylist::ID.eq(id)),
        ))
        .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

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
                utcnow.naive_utc(),
            );
            let _ = pl_xml.dump();
        }

        Ok(result)
    }

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
            pl_xml.update(id.to_string(), utcnow.naive_utc());
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }

        Ok(result)
    }

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
            pl_xml.update_parent(id.to_string(), parent_id.clone(), utcnow.naive_utc());
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }

        Ok(playlist)
    }

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
            } else if seq > count + 1 {
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
                    .filter(schema::djmdSongPlaylist::TrackNo.lt(old_seq))
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

    pub fn get_property(&mut self) -> Result<Vec<DjmdProperty>> {
        let results = djmdProperty.load::<DjmdProperty>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_property_by_id(&mut self, id: &str) -> Result<Option<DjmdProperty>> {
        let result = djmdProperty
            .filter(schema::djmdProperty::DBID.eq(id))
            .first::<DjmdProperty>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    pub fn get_cloud_property(&mut self) -> Result<Vec<DjmdCloudProperty>> {
        let results = djmdCloudProperty.load::<DjmdCloudProperty>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_cloud_property_by_id(&mut self, id: &str) -> Result<Option<DjmdCloudProperty>> {
        let result = djmdCloudProperty
            .filter(schema::djmdCloudProperty::ID.eq(id))
            .first::<DjmdCloudProperty>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    pub fn get_recommend_like(&mut self) -> Result<Vec<DjmdRecommendLike>> {
        let results = djmdRecommendLike.load::<DjmdRecommendLike>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_recommend_like_by_id(&mut self, id: &str) -> Result<Option<DjmdRecommendLike>> {
        let result = djmdRecommendLike
            .filter(schema::djmdRecommendLike::ID.eq(id))
            .first::<DjmdRecommendLike>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    pub fn get_related_tracks(&mut self) -> Result<Vec<DjmdRelatedTracks>> {
        let results = djmdRelatedTracks.load::<DjmdRelatedTracks>(&mut self.conn)?;
        Ok(results)
    }

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

    pub fn get_related_tracks_by_id(&mut self, id: &str) -> Result<Option<DjmdRelatedTracks>> {
        let result = djmdRelatedTracks
            .filter(schema::djmdRelatedTracks::ID.eq(id))
            .first::<DjmdRelatedTracks>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_related_tracks_songs(&mut self, id: &str) -> Result<Vec<DjmdSongRelatedTracks>> {
        let results = schema::djmdSongRelatedTracks::table
            .filter(schema::djmdSongRelatedTracks::RelatedTracksID.eq(id))
            .order_by(schema::djmdSongRelatedTracks::TrackNo)
            .load::<DjmdSongRelatedTracks>(&mut self.conn)?;
        Ok(results)
    }

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

    pub fn get_sampler(&mut self) -> Result<Vec<DjmdSampler>> {
        let results = djmdSampler.load::<DjmdSampler>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_sampler_children(&mut self, parent_id: &str) -> Result<Vec<DjmdSampler>> {
        let results = djmdSampler
            .filter(schema::djmdSampler::ParentID.eq(parent_id))
            .order_by(schema::djmdSampler::Seq)
            .load::<DjmdSampler>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_sampler_by_id(&mut self, id: &str) -> Result<Option<DjmdSampler>> {
        let result = djmdSampler
            .filter(schema::djmdSampler::ID.eq(id))
            .first::<DjmdSampler>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_sampler_songs(&mut self, id: &str) -> Result<Vec<DjmdSongSampler>> {
        let results = schema::djmdSongSampler::table
            .filter(schema::djmdSongSampler::SamplerID.eq(id))
            .order_by(schema::djmdSongSampler::TrackNo)
            .load::<DjmdSongSampler>(&mut self.conn)?;
        Ok(results)
    }

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

    pub fn get_song_tag_list(&mut self) -> Result<Vec<DjmdSongTagList>> {
        let results = djmdSongTagList
            .order_by(schema::djmdSongTagList::TrackNo)
            .load::<DjmdSongTagList>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_song_tag_list_by_id(&mut self, id: &str) -> Result<Option<DjmdSongTagList>> {
        let result = djmdSongTagList
            .filter(schema::djmdSongTagList::ID.eq(id))
            .first::<DjmdSongTagList>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Sort -------------------------------------------------------------------------------------

    pub fn get_sort(&mut self) -> Result<Vec<DjmdSort>> {
        let results = djmdSort
            .order_by(schema::djmdSort::Seq)
            .load::<DjmdSort>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_sort_by_id(&mut self, id: &str) -> Result<Option<DjmdSort>> {
        let result = djmdSort
            .filter(schema::djmdSort::ID.eq(id))
            .first::<DjmdSort>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ImageFile --------------------------------------------------------------------------------

    pub fn get_image_file(&mut self) -> Result<Vec<ImageFile>> {
        let results = imageFile.load::<ImageFile>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_image_file_by_id(&mut self, id: &str) -> Result<Option<ImageFile>> {
        let result = imageFile
            .filter(schema::imageFile::ID.eq(id))
            .first::<ImageFile>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- SettingFile ------------------------------------------------------------------------------

    pub fn get_setting_file(&mut self) -> Result<Vec<SettingFile>> {
        let results = settingFile.load::<SettingFile>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_setting_file_by_id(&mut self, id: &str) -> Result<Option<SettingFile>> {
        let result = settingFile
            .filter(schema::settingFile::ID.eq(id))
            .first::<SettingFile>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    pub fn get_uuid_id_map(&mut self) -> Result<Vec<UuidIDMap>> {
        let results = uuidIDMap.load::<UuidIDMap>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_uuid_id_map_by_id(&mut self, id: &str) -> Result<Option<UuidIDMap>> {
        let result = uuidIDMap
            .filter(schema::uuidIDMap::ID.eq(id))
            .first::<UuidIDMap>(&mut self.conn)
            .optional()?;
        Ok(result)
    }
}
