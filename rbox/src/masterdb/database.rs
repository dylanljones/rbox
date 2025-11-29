// Author: Dylan Jones
// Date:   2025-05-01

//! # Rekordbox Master Database Handler
//!
//! This module provides a high-level interface for interacting with the Rekordbox `master.db` SQLite database.
//! It enables querying, updating, and managing various tables such as playlists, songs, tags, and settings.
//!
//! The main struct [`MasterDb`] encapsulates the database connection and provides methods for
//! accessing and modifying database entries. The queries are handled by the individual model structs.
//! For a lower level interface, these models can be used directly to perform database operations.
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
//! By default, write operations are restricted if Rekordbox is running. Use `set_unsafe_writes(true)`
//! to override this behavior if necessary.
//!
//! ## Tables Supported
//! - AgentRegistry, CloudAgentRegistry, ContentActiveCensor, DjmdPlaylist, DjmdSongPlaylist,
//!   DjmdProperty, DjmdCloudProperty, DjmdRecommendLike, DjmdRelatedTracks, DjmdSampler, DjmdSongTagList,
//!   DjmdSort, ImageFile, SettingFile, UuidIDMap, and more.
//!
//! See individual method documentation for details on arguments, return values, and error handling.

use dunce;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::conn::{create_connection_pool, DbPool};
use super::enums::*;
use super::models::*;
use super::playlist_xml::MasterPlaylistXml;
use crate::anlz::{find_anlz_files, Anlz, AnlzFiles, AnlzPaths};
use crate::error::{Error, Result};
use crate::model_traits::{
    Model, ModelDelete, ModelInsert, ModelList, ModelTree, ModelUpdate, NodeRef,
};
use crate::options::RekordboxOptions;
use crate::pathlib::NormalizePath;
use crate::util::is_rekordbox_running;

#[derive(Clone, Debug)]
pub struct MasterDb {
    /// A connection pool used for interacting with the database.
    pub pool: DbPool,
    // /// Represents the SQLite database connection used for interacting with the database.
    // conn: SqliteConnection,
    /// Stores the path to the PIONEER share directory, which contains analysis and other files.
    /// This is optional and may not be set if the directory is not found.
    share_dir: Option<PathBuf>,
    /// Stores the path to the `masterPlaylist6.xml` file located in the same directory as the database.
    /// This is optional and may not be set if the file is not found.
    plxml_path: Option<PathBuf>,
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
            return Err(Error::FileNotFound(path_obj.to_str().unwrap().to_string()));
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
        let url = path_obj.to_str().unwrap();
        Ok(Self {
            pool: create_connection_pool(url, 8)?,
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
        let plxml_path = options.get_db_dir().normalize();
        let url = options.db_path.to_str().unwrap();

        Ok(Self {
            pool: create_connection_pool(url, 8)?,
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

    /// Checks if write operations to the database are allowed.
    fn assert_write_mode(&self) -> Result<()> {
        // Check if Rekordbox is running
        if !self.unsafe_writes && is_rekordbox_running() {
            return Err(Error::RekordboxRunning);
        }
        Ok(())
    }

    /// Returns the path to the configured share directory.
    pub fn share_directory(&self) -> Option<PathBuf> {
        self.share_dir.clone()
    }

    /// Returns the path to the configured playlist XML file.
    pub fn playlist_xml_path(&self) -> Option<PathBuf> {
        self.plxml_path.clone()
    }

    // -- AgentRegistry ----------------------------------------------------------------------------

    /// Retrieves all entries from the `agentRegistry` table in the database.
    ///
    /// # Returns
    /// A vector of [`AgentRegistry`] objects if the query is successful, or an error if the
    /// query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let registries = db.get_agent_registries().unwrap();
    /// for registry in registries {
    ///     println!("{:?}", registry);
    /// }
    /// ```
    pub fn get_agent_registries(&mut self) -> Result<Vec<AgentRegistry>> {
        let mut conn = self.pool.get()?;
        Ok(AgentRegistry::all(&mut conn)?)
    }

    /// Retrieves an `agentRegistry` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the agent registry entry.
    ///
    /// # Returns
    /// An `Option` containing the [`AgentRegistry`] object if found, or `None` if no entry matches
    /// the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(AgentRegistry::find(&mut conn, id)?)
    }

    /// Retrieves the local update sequence number (USN) from the `agentRegistry` table.
    ///
    /// # Returns
    /// The local USN as an integer if found, or an error if the entry does not exist.
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
        let mut conn = self.pool.get()?;
        Ok(AgentRegistry::local_usn(&mut conn)?)
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    /// Retrieves all entries from the `cloudAgentRegistry` table in the database.
    ///
    /// # Returns
    /// A vector of [`CloudAgentRegistry`] objects if the query is successful, or an error if
    /// the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let registries = db.get_cloud_agent_registries().unwrap();
    /// for registry in registries {
    ///     println!("{:?}", registry);
    /// }
    /// ```
    pub fn get_cloud_agent_registries(&mut self) -> Result<Vec<CloudAgentRegistry>> {
        let mut conn = self.pool.get()?;
        Ok(CloudAgentRegistry::all(&mut conn)?)
    }

    /// Retrieves a `cloudAgentRegistry` entry by its unique identifier.
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
        let mut conn = self.pool.get()?;
        Ok(CloudAgentRegistry::find(&mut conn, id)?)
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    /// Retrieves all entries from the `contentActiveCensor` table in the database.
    ///
    /// # Returns
    /// A vector of [`ContentActiveCensor`] objects if the query is successful, or an error if
    /// the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let active_censors = db.get_content_active_censors().unwrap();
    /// for censor in active_censors {
    ///     println!("{:?}", censor);
    /// }
    /// ```
    pub fn get_content_active_censors(&mut self) -> Result<Vec<ContentActiveCensor>> {
        let mut conn = self.pool.get()?;
        Ok(ContentActiveCensor::all(&mut conn)?)
    }

    /// Retrieves a `contentActiveCensor` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content active censor entry.
    ///
    /// # Returns
    /// An `Option` containing the [`ContentActiveCensor`] object if found, or `None` if no entry
    /// matches the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(ContentActiveCensor::find(&mut conn, id)?)
    }

    // -- ContentCue -------------------------------------------------------------------------------

    /// Retrieves all entries from the `contentCue` table in the database.
    ///
    /// # Returns
    /// A vector of [`ContentCue`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let cues = db.get_content_cues().unwrap();
    /// for cue in cues {
    ///     println!("{:?}", cue);
    /// }
    /// ```
    pub fn get_content_cues(&mut self) -> Result<Vec<ContentCue>> {
        let mut conn = self.pool.get()?;
        Ok(ContentCue::all(&mut conn)?)
    }

    /// Retrieves a `contentCue` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content cue entry.
    ///
    /// # Returns
    /// An `Option` containing the [`ContentCue`] object if found, or `None` if no entry matches
    /// the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(ContentCue::find(&mut conn, id)?)
    }

    // -- ContentFile ------------------------------------------------------------------------------

    /// Retrieves all entries from the `contentFile` table in the database.
    ///
    /// # Returns
    /// A vector of [`ContentFile`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let files = db.get_content_files().unwrap();
    /// for file in files {
    ///     println!("{:?}", file);
    /// }
    /// ```
    pub fn get_content_files(&mut self) -> Result<Vec<ContentFile>> {
        let mut conn = self.pool.get()?;
        Ok(ContentFile::all(&mut conn)?)
    }

    /// Retrieves a `contentFile` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content file entry.
    ///
    /// # Returns
    /// An `Option` containing the [`ContentFile`] object if found, or `None` if no entry matches
    /// the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(ContentFile::find(&mut conn, id)?)
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdActiveCensor` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdActiveCensor`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let active_censors = db.get_active_censors().unwrap();
    /// for censor in active_censors {
    ///     println!("{:?}", censor);
    /// }
    /// ```
    pub fn get_active_censors(&mut self) -> Result<Vec<DjmdActiveCensor>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdActiveCensor::all(&mut conn)?)
    }

    /// Retrieves a `djmdActiveCensor` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the active censor entry.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdActiveCensor`] object if found, or `None` if no entry
    /// matches the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdActiveCensor::find(&mut conn, id)?)
    }

    // -- Album ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdAlbum` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdAlbum`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let albums = db.get_albums().unwrap();
    /// for album in albums {
    ///     println!("{:?}", album);
    /// }
    /// ```
    pub fn get_albums(&mut self) -> Result<Vec<DjmdAlbum>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdAlbum::all(&mut conn)?)
    }

    /// Retrieves a `djmdAlbum` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the album.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdAlbum`] object if found, or `None` if no entry matches
    /// the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdAlbum::find(&mut conn, id)?)
    }

    /// Retrieves a `djmdAlbum` entry by its name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the album.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdAlbum`] object if found, or `None` if no entry matches the
    /// given name. Returns an error if multiple entries match the given name.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdAlbum::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new album into the `djmdAlbum` table in the database.
    ///
    /// # Arguments
    /// * `item` - A [`NewDjmdAlbum`] object containing the album details to be inserted.
    ///
    /// # Returns
    /// The newly created [`DjmdAlbum`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let item = NewDjmdAlbum::new("name").album_artist_id("artist_id");
    /// let album = db.insert_album(item).unwrap();
    /// println!("{:?}", album);
    /// ```
    pub fn insert_album(&mut self, item: NewDjmdAlbum) -> Result<DjmdAlbum> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.insert(&mut conn)?)
    }

    /// Creates a new album and inserts it into the `djmdAlbum` table in the database.
    ///
    /// # Arguments
    /// * `name` - The name of the album.
    /// * `artist_id` - An optional identifier for the album artist.
    /// * `image_path` - An optional path to the album's image.
    /// * `compilation` - An optional integer indicating whether the album is a compilation.
    ///
    /// # Returns
    /// The newly created [`DjmdAlbum`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let album = db.create_album(
    ///     "Album Name".to_string(),
    ///     Some("ArtistID".to_string()),
    ///     Some("/path/to/image.jpg".to_string()),
    ///     Some(1),
    /// ).unwrap();
    /// println!("{:?}", album);
    /// ```
    pub fn create_album(
        &mut self,
        name: String,
        artist_id: Option<String>,
        image_path: Option<String>,
        compilation: Option<i32>,
    ) -> Result<DjmdAlbum> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdAlbum {
            name,
            album_artist_id: artist_id,
            image_path,
            compilation,
            search_str: None,
        };
        Ok(item.insert(&mut conn)?)
    }

    /// Inserts a new album into the `djmdAlbum` table with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The name of the album to insert or retrieve.
    ///
    /// # Returns
    /// The existing or newly created [`DjmdAlbum`] object.
    ///
    /// # Errors
    /// * Returns an error if the album cannot be inserted or retrieved.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// // Creates a new album
    /// let album = db.create_album_if_not_exists("Album 1").unwrap();
    /// // Retrieves the existing album
    /// let album = db.create_album_if_not_exists("Album 1").unwrap();
    /// ```
    fn create_album_if_not_exists(&mut self, name: &str) -> Result<DjmdAlbum> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdAlbum::new(name.to_string());
        Ok(item.insert_if_not_exists(&mut conn)?)
    }

    /// Updates an existing `djmdAlbum` entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdAlbum`] object to be updated.
    ///
    /// # Returns
    /// The updated [`DjmdAlbum`] object if successful, or an error.
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
    /// album.name = "New Album Name".to_string();
    /// let updated_album = db.update_album(&mut album).unwrap();
    /// println!("{:?}", updated_album);
    /// ```
    pub fn update_album(&mut self, item: &mut DjmdAlbum) -> Result<DjmdAlbum> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.clone().update(&mut conn)?)
    }

    /// Deletes an album entry from the `djmdAlbum` table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the album to be deleted.
    ///
    /// # Returns
    /// The number of rows affected by the delete operation.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(DjmdAlbum::delete(&mut conn, id)?)
    }

    // -- Artist -----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdArtist` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdArtist`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let artists = db.get_artists().unwrap();
    /// for artist in artists {
    ///     println!("{:?}", artist);
    /// }
    /// ```
    pub fn get_artists(&mut self) -> Result<Vec<DjmdArtist>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdArtist::all(&mut conn)?)
    }

    /// Retrieves a `djmdArtist` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the artist.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdArtist`] object if found, or `None` if no entry matches
    /// the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdArtist::find(&mut conn, id)?)
    }

    /// Retrieves a `djmdArtist` entry by its name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the artist.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdArtist`] object if found, or `None` if no entry matches
    /// the given name. Returns an error if multiple entries match the given name.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdArtist::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new artist into the `djmdArtist` table in the database.
    ///
    /// # Arguments
    /// * `item` - A [`NewDjmdArtist`] object containing the album details to be inserted.
    ///
    /// # Returns
    /// The newly created [`DjmdArtist`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let item = NewDjmdArtist::new("name");
    /// let album = db.insert_artist(item).unwrap();
    /// println!("{:?}", album);
    /// ```
    pub fn insert_artist(&mut self, item: NewDjmdArtist) -> Result<DjmdArtist> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.insert(&mut conn)?)
    }

    /// Creates a new artist and inserts it into the `djmdArtist` table in the database.
    ///
    /// # Arguments
    /// * `name` - The name of the artist to insert.
    ///
    /// # Returns
    /// The newly created [`DjmdArtist`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let artist = db.create_artist("Artist Name".to_string()).unwrap();
    /// println!("{:?}", artist);
    /// ```
    pub fn create_artist(&mut self, name: String) -> Result<DjmdArtist> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdArtist::new(name);
        Ok(item.insert(&mut conn)?)
    }

    /// Inserts a new `djmdArtist` with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The name of the artist to insert or retrieve.
    ///
    /// # Returns
    /// The existing or newly created [`DjmdArtist`] object.
    ///
    /// # Errors
    /// * Returns an error if the artist cannot be inserted or retrieved.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// // Creates a new artist
    /// let artist = db.create_artist_if_not_exists("Artist 1").unwrap();
    /// // Retrieves the existing artist
    /// let artist = db.create_artist_if_not_exists("Artist 1").unwrap();
    /// ```
    fn create_artist_if_not_exists(&mut self, name: &str) -> Result<DjmdArtist> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdArtist::new(name.to_string());
        Ok(item.insert_if_not_exists(&mut conn)?)
    }

    /// Updates an existing `djmdArtist` entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdArtist`] object to be updated.
    ///
    /// # Returns
    /// The updated [`DjmdArtist`] object if successful, or an error.
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
    /// artist.name = "New Artist Name".to_string();
    /// let updated_artist = db.update_artist(&mut artist).unwrap();
    /// println!("{:?}", updated_artist);
    /// ```
    pub fn update_artist(&mut self, item: &mut DjmdArtist) -> Result<DjmdArtist> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.clone().update(&mut conn)?)
    }

    /// Deletes an artist entry from the `djmdArtist` table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the artist to be deleted.
    ///
    /// # Returns
    /// The number of rows affected by the delete operation.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(DjmdArtist::delete(&mut conn, id)?)
    }

    // -- Category ---------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdCategory` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdCategory`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let categories = db.get_categories().unwrap();
    /// for category in categories {
    ///     println!("{:?}", category);
    /// }
    /// ```
    pub fn get_categories(&mut self) -> Result<Vec<DjmdCategory>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdCategory::all(&mut conn)?)
    }

    /// Retrieves a `djmdCategory` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the category.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdCategory`] object if found, or `None` if no entry matches
    /// the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdCategory::find(&mut conn, id)?)
    }

    // -- Color ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdColor` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdColor`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let colors = db.get_colors().unwrap();
    /// for color in colors {
    ///     println!("{:?}", color);
    /// }
    /// ```
    pub fn get_colors(&mut self) -> Result<Vec<DjmdColor>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdColor::all(&mut conn)?)
    }

    /// Retrieves a `djmdColor` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the color.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdColor`] object if found, or `None` if no entry matches the
    /// given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdColor::find(&mut conn, id)?)
    }

    // -- Content ----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdContent` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdContent`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let contents = db.get_contents().unwrap();
    /// for content in contents {
    ///     println!("{:?}", content);
    /// }
    /// ```
    pub fn get_contents(&mut self) -> Result<Vec<DjmdContent>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdContent::all(&mut conn)?)
    }

    /// Retrieves a `djmdContent` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdContent`] object if found, or `None` if no entry matches
    /// the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdContent::find(&mut conn, id)?)
    }

    /// Retrieves multiple `djmdContent` entries by their unique identifiers.
    ///
    /// # Arguments
    /// * `ids` - A vector of string slices representing the unique identifiers of the contents.
    ///
    /// # Returns
    /// A vector of [`DjmdContent`] objects found for the given IDs.
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
    pub fn get_contents_by_ids(&mut self, ids: &[&str]) -> Result<Vec<DjmdContent>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdContent::by_ids(&mut conn, ids)?)
    }

    /// Retrieves a `djmdContent` entry by its folder path.
    ///
    /// # Arguments
    /// * `path` - A string slice representing the folder path of the content.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdContent`] object if found, or `None` if no entry matches
    /// the given path. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdContent::find_by_path(&mut conn, path)?)
    }

    /// Returns the path to the corresponding ANLZxxxx.DAT file for a given content ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content.
    ///
    /// # Returns
    /// The canonicalized path to the analysis data file.
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
        let share_dir = match &self.share_dir {
            Some(s) => s,
            None => return Err(Error::Database("Share dir not set!".into())),
        };

        let mut conn = self.pool.get()?;
        if let Some(result) = DjmdContent::find_anlz_path(&mut conn, id)? {
            // Strip first "/" in result
            let striped = result.strip_prefix("/").unwrap();
            let anlz_file = share_dir.join(striped);
            let anlz_files_canonicalized = dunce::canonicalize(&anlz_file);
            if let Err(e) = anlz_files_canonicalized {
                return Err(Error::Database(format!(
                    "Failed to canonicalize path: {}",
                    e
                )));
            }
            return Ok(anlz_files_canonicalized?);
        }
        Err(Error::Database("Failed to get AnalysisDataPath".into()))
    }

    /// Returns the path to the directory containing the ANLZxxxx.xxx files for a given content ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content.
    ///
    /// # Returns
    /// The path to the analysis directory, or `None` if not found.
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
        Ok(Some(root.to_path_buf()))
    }

    /// Returns a struct containing the paths to ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 files.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the content.
    ///
    /// # Returns
    /// The [`AnlzPaths`] struct with analysis file paths, or `None` if not found.
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
    /// The [`AnlzFiles`] struct with loaded analysis files, or `None` if not found.
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

    /// Creates a new content entry and inserts it into the `djmdContent` table.
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
    /// The newly inserted [`DjmdContent`] object if successful.
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
    /// let content = db.create_content("/music/track.mp3").unwrap();
    /// // Update additional fields as needed
    /// // Run "reload tags" in Rekordbox to generate analysis files
    /// println!("{:?}", content);
    /// ```
    pub fn create_content<P: AsRef<Path> + AsRef<OsStr>>(
        &mut self,
        path: P,
    ) -> Result<DjmdContent> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let path = Path::new(&path).as_os_str().to_str().unwrap().to_string();
        let item = NewDjmdContent::new(path);
        Ok(item.insert(&mut conn)?)
    }

    /// Updates an existing `djmdContent` entry in the database.
    ///
    /// # Arguments
    /// * `item` - A reference to the [`DjmdContent`] object to be updated.
    ///
    /// # Returns
    /// The number of rows affected by the update operation.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database update operation fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let mut content = db.get_content_by_id("some_id").unwrap().unwrap();
    /// content.title = "New Title".to_string();
    /// let rows_updated = db.update_content(&content).unwrap();
    /// println!("Rows updated: {}", rows_updated);
    /// ```
    pub fn update_content(&mut self, item: &DjmdContent) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let _ = item.clone().update(&mut conn)?;
        Ok(1)
    }

    /// Update the content album field.
    ///
    /// Sets the [DjmdContent.album_id] to the corresponding ID of the album.
    /// If the album does not exist yet, a new [DjmdAlbum] row will be created.
    pub fn update_content_album(&mut self, content_id: &str, name: &str) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let album = self.create_album_if_not_exists(name)?;
        Ok(DjmdContent::set_album_id(&mut conn, content_id, &album.id)?)
    }

    /// Update the content artist name.
    ///
    /// Sets the [DjmdContent.artist_id] to the corresponding ID of the artist.
    /// If the artist does not exist yet, a new [DjmdArtist] row will be created.
    pub fn update_content_artist(&mut self, content_id: &str, name: &str) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let artist = self.create_artist_if_not_exists(name)?;
        Ok(DjmdContent::set_artist_id(
            &mut conn, content_id, &artist.id,
        )?)
    }

    /// Update the content remixer name.
    ///
    /// Sets the [DjmdContent.remixer_id] to the corresponding ID of the artist.
    /// If the artist does not exist yet, a new [DjmdArtist] row will be created.
    pub fn update_content_remixer(&mut self, content_id: &str, name: &str) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let artist = self.create_artist_if_not_exists(name)?;
        Ok(DjmdContent::set_remixer_id(
            &mut conn, content_id, &artist.id,
        )?)
    }

    /// Update the content original artist name.
    ///
    /// Sets the [DjmdContent.org_artist_id] to the corresponding ID of the artist.
    /// If the artist does not exist yet, a new [DjmdArtist] row will be created.
    pub fn update_content_original_artist(
        &mut self,
        content_id: &str,
        name: &str,
    ) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let artist = self.create_artist_if_not_exists(name)?;
        Ok(DjmdContent::set_original_artist_id(
            &mut conn, content_id, &artist.id,
        )?)
    }

    /// Update the content composer name.
    ///
    /// Sets the [DjmdContent.composer_id] to the corresponding ID of the artist.
    /// If the artist does not exist yet, a new [DjmdArtist] row will be created.
    pub fn update_content_composer(&mut self, content_id: &str, name: &str) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let artist = self.create_artist_if_not_exists(name)?;
        Ok(DjmdContent::set_composer_id(
            &mut conn, content_id, &artist.id,
        )?)
    }

    /// Update the content genre name.
    ///
    /// Sets the [DjmdContent.genre_id] to the corresponding ID of the genre.
    /// If the genre does not exist yet, a new [DjmdGenre] row will be created.
    pub fn update_content_genre(&mut self, content_id: &str, name: &str) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let genre = self.create_genre_if_not_exists(name)?;
        Ok(DjmdContent::set_genre_id(&mut conn, content_id, &genre.id)?)
    }

    /// Update the content label name.
    ///
    /// Sets the [DjmdContent.label_id] to the corresponding ID of the label.
    /// If the label does not exist yet, a new [DjmdLabel] row will be created.
    pub fn update_content_label(&mut self, content_id: &str, name: &str) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let label = self.create_label_if_not_exists(name)?;
        Ok(DjmdContent::set_label_id(&mut conn, content_id, &label.id)?)
    }

    /// Update the content key name.
    ///
    /// Sets the [DjmdContent.key_id] to the corresponding ID of the label.
    /// If the key does not exist yet, a new [DjmdKey] row will be created.
    pub fn update_content_key(&mut self, content_id: &str, name: &str) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let key = self.create_key_if_not_exists(name)?;
        Ok(DjmdContent::set_key_id(&mut conn, content_id, &key.id)?)
    }

    /// Updates the file path of a content entry in the database.
    ///
    /// This function performs the following steps:
    /// 1. Checks if unsafe writes are allowed when Rekordbox is running.
    /// 2. Validates the provided file path to ensure it exists and is a file.
    /// 3. Ensures the new path does not already exist in the database.
    /// 4. Updates the analysis data files (if any) with the new path.
    /// 5. Updates the database entry with the new path.
    ///
    /// # Arguments
    /// * `content_id` - A string slice representing the unique identifier of the [`DjmdContent`] entry.
    /// * `path` - A string slice representing the new file path to be set.
    ///
    /// # Returns
    /// The number of rows affected by the update operation.
    ///
    /// # Errors
    /// * Returns an error if:
    ///   - Rekordbox is running and unsafe writes are not allowed.
    ///   - The provided path does not exist or is not a file.
    ///   - The new path already exists in the database.
    ///   - Updating the analysis data or database fails.
    ///
    /// # Example
    /// ```no_run
    /// let mut db = MasterDb::open().unwrap();
    /// let rows_updated = db.update_content_path("content_id", "/new/path/to/file.mp3").unwrap();
    /// println!("Rows updated: {}", rows_updated);
    /// ```
    pub fn update_content_path(&mut self, content_id: &str, path: &str) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;

        // prepare path and check if it exists
        let path = Path::new(&path);
        let rb_path = path.normalize_sep("/");
        let rb_path_str = rb_path
            .as_os_str()
            .to_str()
            .expect("Failed to convert path to string");
        if !path.is_file() || !path.exists() {
            return Err(Error::FileNotFound(rb_path_str.to_string()));
        }
        if DjmdContent::path_exists(&mut conn, &rb_path_str)? {
            return Err(Error::Database(format!(
                "Content with path {} already exists",
                rb_path_str
            )));
        }

        // Update path in analysis data
        let anlz_files: Option<AnlzFiles> = self.get_content_anlz_files(content_id)?;
        if anlz_files.is_some() {
            let mut anlz_files = anlz_files.unwrap();
            anlz_files.dat.set_path(&rb_path_str)?;
            anlz_files.dat.dump()?;

            if let Some(ext) = &mut anlz_files.ext {
                ext.set_path(&rb_path_str)?;
                ext.dump()?;
            }
            if let Some(ex2) = &mut anlz_files.ex2 {
                ex2.set_path(&rb_path_str)?;
                ex2.dump()?;
            }
        }

        Ok(DjmdContent::set_path(&mut conn, content_id, &rb_path_str)?)
    }

    // -- Cue --------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdCue` table.
    ///
    /// # Returns
    /// A vector of all cue entries.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    ///
    /// # Example
    /// ```no_run
    /// let cues = db.get_cues()?;
    /// ```
    pub fn get_cues(&mut self) -> Result<Vec<DjmdCue>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdCue::all(&mut conn)?)
    }

    /// Retrieves a `djmdCue` entry by its unique identifier.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdCue::find(&mut conn, id)?)
    }

    // -- Device -----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdDevice` table.
    ///
    /// # Returns
    /// A vector of all device entries.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    ///
    /// # Example
    /// ```no_run
    /// let devices = db.get_devices()?;
    /// ```
    pub fn get_devices(&mut self) -> Result<Vec<DjmdDevice>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdDevice::all(&mut conn)?)
    }

    /// Retrieves a `djmdDevice` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - Device entry ID.
    ///
    /// # Returns
    /// The device entry if found, or `None`.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    ///
    /// # Example
    /// ```no_run
    /// let device = db.get_device_by_id("device_id")?;
    /// ```
    pub fn get_device_by_id(&mut self, id: &str) -> Result<Option<DjmdDevice>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdDevice::find(&mut conn, id)?)
    }

    // -- Genre ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdGenre` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdGenre`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let genres = db.get_genres().unwrap();
    /// for genre in genres {
    ///     println!("{:?}", genre);
    /// }
    /// ```
    pub fn get_genres(&mut self) -> Result<Vec<DjmdGenre>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdGenre::all(&mut conn)?)
    }

    /// Retrieves a `djmdGenre` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the genre.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdGenre`] object if found, or `None` if no entry matches
    /// the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdGenre::find(&mut conn, id)?)
    }

    /// Retrieves a `djmdGenre` entry by its name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the genre.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdGenre`] object if found, or `None` if no entry matches
    /// the given name. Returns an error if multiple entries match the given name.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdGenre::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new genre into the `djmdGenre` table in the database.
    ///
    /// # Arguments
    /// * `item` - The [`NewDjmdGenre`] object representing the genre to insert.
    ///
    /// # Returns
    /// The newly created [`DjmdGenre`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let item = NewDjmdGenre::new("name");
    /// let genre = db.insert_genre(item).unwrap();
    /// println!("{:?}", genre);
    /// ```
    pub fn insert_genre(&mut self, item: NewDjmdGenre) -> Result<DjmdGenre> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.insert(&mut conn)?)
    }

    /// Creates a new genre and inserts it into the `djmdGenre` table in the database.
    ///
    /// # Arguments
    /// * `name` - The name of the genre to insert.
    ///
    /// # Returns
    /// The newly created [`DjmdGenre`] object if successful, or an error.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let genre = db.create_genre("Genre Name".to_string()).unwrap();
    /// println!("{:?}", genre);
    /// ```
    pub fn create_genre(&mut self, name: String) -> Result<DjmdGenre> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdGenre::new(name);
        Ok(item.insert(&mut conn)?)
    }

    /// Inserts a new `djmdGenre` with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The name of the genre to insert or retrieve.
    ///
    /// # Returns
    /// The existing or newly created [`DjmdGenre`] object.
    ///
    /// # Errors
    /// * Returns an error if the genre cannot be inserted or retrieved.
    ///
    /// # Example
    /// ```no_run
    /// let genre = db.create_genre_if_not_exists("Genre 1").unwrap();
    /// ```
    fn create_genre_if_not_exists(&mut self, name: &str) -> Result<DjmdGenre> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdGenre::new(name.to_string());
        Ok(item.insert_if_not_exists(&mut conn)?)
    }

    /// Updates an existing `djmdGenre` entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdGenre`] object to be updated.
    ///
    /// # Returns
    /// The updated [`DjmdGenre`] object if successful, or an error.
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
    /// genre.name = "New Genre Name".to_string();
    /// let updated_genre = db.update_genre(&mut genre).unwrap();
    /// println!("{:?}", updated_genre);
    /// ```
    pub fn update_genre(&mut self, item: &mut DjmdGenre) -> Result<DjmdGenre> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.clone().update(&mut conn)?)
    }

    /// Deletes a genre entry from the `djmdGenre` table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the genre to be deleted.
    ///
    /// # Returns
    /// The number of rows affected by the delete operation.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(DjmdGenre::delete(&mut conn, id)?)
    }

    // -- History ----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdHistory` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdHistory`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let histories = db.get_histories().unwrap();
    /// for history in histories {
    ///     println!("{:?}", history);
    /// }
    /// ```
    pub fn get_histories(&mut self) -> Result<Vec<DjmdHistory>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdHistory::all(&mut conn)?)
    }

    /// Retrieves a `djmdHistory` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the history entry.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdHistory`] object if found, or `None` if no entry matches
    /// the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdHistory::find(&mut conn, id)?)
    }

    /// Retrieves all song history entries for a given history ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the history entry.
    ///
    /// # Returns
    /// A vector of [`DjmdSongHistory`] objects for the given history ID.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSongHistory::by_history_id(&mut conn, id)?)
    }

    /// Retrieves all content entries referenced by the song history for a given history ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the history entry.
    ///
    /// # Returns
    /// A vector of [`DjmdContent`] objects referenced by the song history.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdHistory::get_contents(&mut conn, id)?)
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdHotCueBanklist` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdHotCueBanklist`] objects if the query is successful, or an error if the query fails.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let banklists = db.get_hot_cue_banklists().unwrap();
    /// for banklist in banklists {
    ///     println!("{:?}", banklist);
    /// }
    /// ```
    pub fn get_hot_cue_banklists(&mut self) -> Result<Vec<DjmdHotCueBanklist>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdHotCueBanklist::all(&mut conn)?)
    }

    /// Retrieves a `djmdHotCueBanklist` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the hot cue banklist entry.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdHotCueBanklist`] objectif found, or `None` if no entry
    /// matches the given identifier. Returns an error if the query fails.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdHotCueBanklist::find(&mut conn, id)?)
    }

    /// Retrieves all child `djmdHotCueBanklist` entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent hot cue banklist ID.
    ///
    /// # Returns
    /// A vector of child [`DjmdHotCueBanklist`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdHotCueBanklist::by_parent_id(&mut conn, parent_id)?)
    }

    /// Retrieves all `djmdSongHotCueBanklist` entries for a given hot cue banklist ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the hot cue banklist ID.
    ///
    /// # Returns
    /// A vector of [`DjmdSongHotCueBanklist`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSongHotCueBanklist::by_hot_cue_banklist_id(
            &mut conn, id,
        )?)
    }

    /// Retrieves all `djmdContent` entries referenced by the song hot cue banklist for a given hot cue banklist ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the hot cue banklist ID.
    ///
    /// # Returns
    /// A vector of [`DjmdContent`] objects referenced by the song hot cue banklist.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdHotCueBanklist::get_contents(&mut conn, id)?)
    }

    /// Retrieves all `hotCueBanklistCue` entries for a given hot cue banklist ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the hot cue banklist ID.
    ///
    /// # Returns
    /// A vector of [`HotCueBanklistCue`] objects for the given banklist ID.
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
        let mut conn = self.pool.get()?;
        Ok(HotCueBanklistCue::by_hot_cue_banklist_id(&mut conn, id)?)
    }

    // -- Key --------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdKey` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdKey`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let keys = db.get_keys().unwrap();
    /// for key in keys {
    ///     println!("{:?}", key);
    /// }
    /// ```
    pub fn get_keys(&mut self) -> Result<Vec<DjmdKey>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdKey::all(&mut conn)?)
    }

    /// Retrieves a `djmdKey` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the key.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdKey`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdKey::find(&mut conn, id)?)
    }

    /// Retrieves a `djmdKey` entry by its scale name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the scale name of the key.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdKey`] object if found, or `None`. Returns an error if
    /// multiple entries match the given name.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdKey::find_by_name(&mut conn, name)?)
    }

    /// Inserts it into the `djmdKey` table in the database.
    ///
    /// # Arguments
    /// * `item` - The [`NewDjmdKey`] object representing the key to insert.
    ///
    /// # Returns
    /// The newly created [`DjmdKey`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let item = NewDjmdKey::new("C#m");
    /// let key = db.insert_key(item).unwrap();
    /// println!("{:?}", key);
    /// ```
    pub fn insert_key(&mut self, item: NewDjmdKey) -> Result<DjmdKey> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.insert(&mut conn)?)
    }

    /// Creates a new key and inserts it into the `djmdKey` table in the database.
    ///
    /// # Arguments
    /// * `name` - The scale name of the key to insert.
    ///
    /// # Returns
    /// The newly created [`DjmdKey`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let key = db.create_key("C#m".to_string()).unwrap();
    /// println!("{:?}", key);
    /// ```
    pub fn create_key(&mut self, name: String) -> Result<DjmdKey> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdKey::new(name);
        Ok(item.insert(&mut conn)?)
    }

    /// Inserts a new `djmdKey` with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The scale name of the key to insert or retrieve.
    ///
    /// # Returns
    /// The existing or newly created [`DjmdKey`] object.
    ///
    /// # Errors
    /// * Returns an error if the key cannot be inserted or retrieved.
    fn create_key_if_not_exists(&mut self, name: &str) -> Result<DjmdKey> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdKey::new(name.to_string());
        Ok(item.insert_if_not_exists(&mut conn)?)
    }

    /// Updates an existing `djmdKey` entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdKey`] object to be updated.
    ///
    /// # Returns
    /// The updated [`DjmdKey`] object if successful.
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
    /// key.scale_name = "Dm".to_string();
    /// let updated_key = db.update_key(&mut key).unwrap();
    /// println!("{:?}", updated_key);
    /// ```
    pub fn update_key(&mut self, item: &mut DjmdKey) -> Result<DjmdKey> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.clone().update(&mut conn)?)
    }

    /// Deletes a key entry from the `djmdKey` table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the key to be deleted.
    ///
    /// # Returns
    /// The number of rows affected by the delete operation.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(DjmdKey::delete(&mut conn, id)?)
    }

    // -- Label ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdLabel` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdLabel`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let labels = db.get_labels().unwrap();
    /// for label in labels {
    ///     println!("{:?}", label);
    /// }
    /// ```
    pub fn get_labels(&mut self) -> Result<Vec<DjmdLabel>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdLabel::all(&mut conn)?)
    }

    /// Retrieves a `djmdLabel` entry by its unique identifier.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdLabel::find(&mut conn, id)?)
    }

    /// Retrieves a `djmdLabel` entry by its name.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the label.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdLabel`] object if found, or `None`. Returns an error if
    /// multiple entries match the given name.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdLabel::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new label into the `djmdLabel` table in the database.
    ///
    /// # Arguments
    /// * `item` - The [`NewDjmdLabel`] object representing the label to insert.
    ///
    /// # Returns
    /// The newly created [`DjmdLabel`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let item = NewDjmdLabel::new("name");
    /// let label = db.insert_label(item).unwrap();
    /// println!("{:?}", label);
    /// ```
    pub fn insert_label(&mut self, item: NewDjmdLabel) -> Result<DjmdLabel> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.insert(&mut conn)?)
    }

    /// Creates a new label and inserts it into the `djmdLabel` table in the database.
    ///
    /// # Arguments
    /// * `name` - The name of the label to insert.
    ///
    /// # Returns
    /// The newly created [`DjmdLabel`] object if successful.
    ///
    /// # Errors
    /// * Returns an error if Rekordbox is running and unsafe writes are not allowed.
    /// * Returns an error if the database insertion fails.
    ///
    /// # Example
    /// ```no_run
    /// let label = db.create_label("Label Name".to_string()).unwrap();
    /// println!("{:?}", label);
    /// ```
    pub fn create_label(&mut self, name: String) -> Result<DjmdLabel> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdLabel::new(name);
        Ok(item.insert(&mut conn)?)
    }

    /// Inserts a new `djmdLabel` with the given name if it does not already exist.
    ///
    /// # Arguments
    /// * `name` - The name of the label to insert or retrieve.
    ///
    /// # Returns
    /// The existing or newly created [`DjmdLabel`] object.
    ///
    /// # Errors
    /// * Returns an error if the label cannot be inserted or retrieved.
    fn create_label_if_not_exists(&mut self, name: &str) -> Result<DjmdLabel> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let item = NewDjmdLabel::new(name.to_string());
        Ok(item.insert_if_not_exists(&mut conn)?)
    }

    /// Updates an existing `djmdLabel` entry in the database.
    ///
    /// # Arguments
    /// * `item` - A mutable reference to the [`DjmdLabel`] object to be updated.
    ///
    /// # Returns
    /// The updated [`DjmdLabel`] object if successful.
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
    /// label.name = "New Label Name".to_string();
    /// let updated_label = db.update_label(&mut label).unwrap();
    /// println
    pub fn update_label(&mut self, item: &mut DjmdLabel) -> Result<DjmdLabel> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(item.clone().update(&mut conn)?)
    }

    /// Deletes a label entry from the `djmdLabel` table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the label to be deleted.
    ///
    /// # Returns
    /// The number of rows affected by the delete operation.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(DjmdLabel::delete(&mut conn, id)?)
    }

    // -- MenuItems --------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdMenuItems` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdMenuItems`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let items = db.get_menu_items().unwrap();
    /// for item in items {
    ///     println!("{:?}", item);
    /// }
    /// ```
    pub fn get_menu_items(&mut self) -> Result<Vec<DjmdMenuItems>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdMenuItems::all(&mut conn)?)
    }

    /// Retrieves a `djmdMenuItems` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the menu item.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdMenuItems`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdMenuItems::find(&mut conn, id)?)
    }

    // -- MixerParam -------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdMixerParam` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdMixerParam`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let params = db.get_mixer_params().unwrap();
    /// for param in params {
    ///     println!("{:?}", param);
    /// }
    /// ```
    pub fn get_mixer_params(&mut self) -> Result<Vec<DjmdMixerParam>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdMixerParam::all(&mut conn)?)
    }

    /// Retrieves a `djmdMixerParam` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the mixer parameter.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdMixerParam`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdMixerParam::find(&mut conn, id)?)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdMyTag` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdMyTag`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let tags = db.get_my_tags().unwrap();
    /// for tag in tags {
    ///     println!("{:?}", tag);
    /// }
    /// ```
    pub fn get_my_tags(&mut self) -> Result<Vec<DjmdMyTag>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdMyTag::all(&mut conn)?)
    }

    /// Retrieves all child `djmdMyTag` entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent tag ID.
    ///
    /// # Returns
    /// A vector of child [`DjmdMyTag`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdMyTag::by_parent_id(&mut conn, parent_id)?)
    }

    /// Retrieves a `djmdMyTag` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the tag.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdMyTag`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdMyTag::find(&mut conn, id)?)
    }

    /// Retrieves all `djmdSongMyTag` entries for a given tag ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the tag ID.
    ///
    /// # Returns
    /// A vector of [`DjmdSongMyTag`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSongMyTag::by_tag_id(&mut conn, id)?)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given tag ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the tag ID.
    ///
    /// # Returns
    /// A vector of [`DjmdContent`] objects referenced by the tag.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdMyTag::get_contents(&mut conn, id)?)
    }

    // -- Playlist ---------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdPlaylist` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdPlaylist`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let playlists = db.get_playlists().unwrap();
    /// for playlist in playlists {
    ///     println!("{:?}", playlist);
    /// }
    /// ```
    pub fn get_playlists(&mut self) -> Result<Vec<DjmdPlaylist>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdPlaylist::all(&mut conn)?)
    }

    /// Returns a sorted tree of playlists as [`DjmdPlaylistTreeItem`] nodes.
    ///
    /// # Returns
    /// A vector of root nodes representing the playlist tree.
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
    pub fn get_playlist_tree(&mut self) -> Result<Vec<NodeRef<DjmdPlaylistTreeNode>>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdPlaylist::tree(&mut conn)?)
    }

    /// Retrieves all child `djmdPlaylist` entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent playlist ID.
    ///
    /// # Returns
    /// A vector of child [`DjmdPlaylist`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdPlaylist::by_parent_id(&mut conn, parent_id)?)
    }

    /// Retrieves a `djmdPlaylist` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the playlist.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdPlaylist`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdPlaylist::find(&mut conn, id)?)
    }

    /// Retrieves a `djmdPlaylist` entry by its hierarchical path.
    ///
    /// # Arguments
    /// * `path` - A vector of string slices representing the playlist path.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdPlaylist`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdPlaylist::find_by_path(&mut conn, path)?)
    }

    /// Retrieves all `djmdSongPlaylist` entries for a given playlist ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the playlist ID.
    ///
    /// # Returns
    /// A vector of [`DjmdSongPlaylist`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSongPlaylist::by_playlist_id(&mut conn, id)?)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given playlist ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the playlist ID.
    ///
    /// # Returns
    /// A vector of [`DjmdContent`] objects referenced by the playlist.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdPlaylist::get_contents(&mut conn, id)?)
    }

    /// Retrieves a `djmdSongPlaylist` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the song playlist entry.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdSongPlaylist`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSongPlaylist::find(&mut conn, id)?)
    }

    /// Inserts a new playlist into the `djmdPlaylist` table in the database.
    ///
    /// # Arguments
    /// * `item` - The [`NewDjmdPlaylist`] object representing the playlist to insert.
    ///
    /// # Returns
    /// The newly created [`DjmdPlaylist`] object if successful.
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
    /// let item = NewDjmdPlaylist::playlist("name").seq(1);
    /// let playlist = db.insert_playlist(item).unwrap();
    /// println!("{:?}", playlist);
    /// ```
    pub fn insert_playlist(&mut self, item: NewDjmdPlaylist) -> Result<DjmdPlaylist> {
        let mut conn = self.pool.get()?;
        self.assert_write_mode()?;
        let result = item.insert(&mut conn)?;
        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            pl_xml.add(
                result.id.clone(),
                result.parent_id.clone(),
                result.attribute,
                result.updated_at.naive_utc(),
            );
            let _ = pl_xml.dump();
        }
        Ok(result)
    }

    /// Creates a new playlist and inserts it into the `djmdPlaylist` table in the database.
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
    /// The newly created [`DjmdPlaylist`] object if successful.
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
    /// let playlist = db.create_playlist_node(
    ///     "My Playlist".to_string(),
    ///     PlaylistType::Regular,
    ///     Some("parent_id".to_string()),
    ///     None,
    ///     None,
    ///     None,
    /// ).unwrap();
    /// println!("{:?}", playlist);
    /// ```
    pub fn create_playlist_node(
        &mut self,
        name: String,
        attribute: PlaylistType,
        parent_id: Option<String>,
        seq: Option<i32>,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> Result<DjmdPlaylist> {
        let item = NewDjmdPlaylist {
            name,
            attribute: attribute as i32,
            seq,
            parent_id,
            image_path,
            smart_list,
        };
        self.insert_playlist(item)
    }

    /// Creates a new regular playlist in the `djmdPlaylist` table.
    ///
    /// # Arguments
    /// * `name` - The name of the playlist.
    /// * `parent_id` - Optional parent playlist ID.
    /// * `seq` - Optional sequence number for ordering.
    /// * `image_path` - Optional image path.
    /// * `smart_list` - Optional smart list configuration.
    ///
    /// # Returns
    /// The newly created playlist.
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
        self.create_playlist_node(
            name,
            PlaylistType::Playlist,
            parent_id,
            seq,
            image_path,
            smart_list,
        )
    }

    /// Creates a new playlist folder in the `djmdPlaylist` table.
    ///
    /// # Arguments
    /// * `name` - The name of the folder.
    /// * `parent_id` - Optional parent folder ID.
    /// * `seq` - Optional sequence number for ordering.
    ///
    /// # Returns
    /// The newly created folder.
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
        self.create_playlist_node(name, PlaylistType::Folder, parent_id, seq, None, None)
    }

    /// Renames an existing playlist.
    ///
    /// # Arguments
    /// * `id` - The playlist ID.
    /// * `name` - The new name for the playlist.
    ///
    /// # Returns
    /// The updated playlist.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let result = DjmdPlaylist::rename(&mut conn, id.as_str(), &name)?;
        if let Some(pl_xml_path) = self.plxml_path.as_ref() {
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
    /// The updated [`DjmdPlaylist`] object.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;

        // Move playlist
        let playlist: DjmdPlaylist = self.get_playlist_by_id(id)?.expect("Playlist not found");
        let parent_id = parent_id.unwrap_or(playlist.parent_id.clone());
        let result = DjmdPlaylist::move_to(&mut conn, id, Some(&parent_id), seq)?;
        if result == 0 {
            return Ok(playlist);
        }
        // Update playlist XML
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
                pl_xml.update(pl.id, pl.updated_at.naive_utc());
            }
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }
        Ok(playlist)
    }

    /// Moves multiple playlists to a new parent folder or changes its sequence within the same parent.
    ///
    /// # Arguments
    /// * `ids` - The playlist IDs to move.
    /// * `start_seq` - Optional new starting sequence number in the target parent.
    /// * `parent_id` - Optional new parent playlist ID. If not provided, keeps the current parent.
    ///
    /// # Returns
    /// The updated [`DjmdPlaylist`] objects.
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
    /// let moved = db.move_playlists(vec!["pid1", "pid2"], "new_parent_id", Some(2))).unwrap();
    /// println!("{:?}", moved);
    /// ```
    pub fn move_playlists(
        &mut self,
        ids: Vec<&str>,
        parent_id: &str,
        start_seq: Option<i32>,
    ) -> Result<Vec<DjmdPlaylist>> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;

        let playlists = DjmdPlaylist::find_all(&mut conn, &ids)?;
        let old_parent_ids: Vec<&str> = playlists.iter().map(|p| p.parent_id.as_str()).collect();

        // Move playlists
        let result = DjmdPlaylist::move_all_to(&mut conn, &ids, parent_id, start_seq)?;
        if result == 0 {
            return Ok(DjmdPlaylist::find_all(&mut conn, &ids)?);
        }

        // Update playlist XML
        let playlists = DjmdPlaylist::find_all(&mut conn, &ids)?;
        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            for pl in &playlists {
                pl_xml.update_parent(
                    pl.id.clone(),
                    pl.parent_id.clone(),
                    pl.updated_at.naive_utc(),
                );
            }
            // Update update-time of all child items
            let mut parent_ids: HashSet<&str> = HashSet::from_iter(old_parent_ids);
            parent_ids.insert(parent_id);
            for parent_id in parent_ids {
                for pl in self.get_playlist_children(&parent_id)? {
                    pl_xml.update(pl.id, pl.updated_at.naive_utc());
                }
            }
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Couldn't update playlist XML, file not found!");
        }
        Ok(playlists)
    }

    /// Deletes a playlist entry from the `djmdPlaylist` table in the database.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the playlist to be deleted.
    ///
    /// # Returns
    /// The number of rows affected by the delete operation.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;

        let deleted_ids = DjmdPlaylist::delete(&mut conn, id)?;

        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            for id in &deleted_ids {
                pl_xml.remove(id.clone());
            }
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }

        Ok(deleted_ids.len())
    }

    /// Inserts a new `djmdSongPlaylist` song into a playlist.
    ///
    /// # Arguments
    /// * `item` - The [`NewDjmdSongPlaylist`] object representing the song to insert.
    ///
    /// # Returns
    /// The newly created [`DjmdSongPlaylist`] object.
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
    /// let item = NewDjmdSongPlaylist::new("playlist_id", "content_id").seq(1);
    /// let song = db.insert_playlist_song(item).unwrap();
    /// println!("{:?}", song);
    /// ```
    pub fn insert_playlist_song(&mut self, item: NewDjmdSongPlaylist) -> Result<DjmdSongPlaylist> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        let result = item.insert(&mut conn)?;
        Ok(result)
    }

    /// Creates a new playlist song and inserts it into the `djmdSongPlaylist` table.
    ///
    /// # Arguments
    /// * `playlist_id` - The ID of the target playlist.
    /// * `content_id` - The ID of the content (song) to insert.
    /// * `seq` - Optional sequence number for the song's position. If not provided, inserts at the end.
    ///
    /// # Returns
    /// The newly created [`DjmdSongPlaylist`] object.
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
    /// let song = db.create_playlist_song(&"playlist_id".to_string(), &"content_id".to_string(), Some(2)).unwrap();
    /// println!("{:?}", song);
    /// ```
    pub fn create_playlist_song(
        &mut self,
        playlist_id: &String,
        content_id: &String,
        seq: Option<i32>,
    ) -> Result<DjmdSongPlaylist> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;

        let item = NewDjmdSongPlaylist {
            playlist_id: playlist_id.clone(),
            content_id: content_id.clone(),
            track_no: seq,
        };
        let result = item.insert(&mut conn)?;
        Ok(result)
    }

    /// Moves a song within a playlist to a new sequence position.
    ///
    /// # Arguments
    /// * `id` - The ID of the song playlist entry to move.
    /// * `seq` - The new sequence number for the song.
    ///
    /// # Returns
    /// The updated [`DjmdSongPlaylist`] object.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        DjmdSongPlaylist::move_to(&mut conn, id, Some(seq))?;
        let result: DjmdSongPlaylist = self.get_playlist_song_by_id(id)?.expect("Song not found");
        Ok(result)
    }

    /// Moves multiple songs within a playlist to a new sequence position.
    ///
    /// # Arguments
    /// * `ids` - The ID of the songs playlist entry to move.
    /// * `start_seq` - The starting sequence number for the songs.
    ///
    /// # Returns
    /// The updated [`DjmdSongPlaylist`] objects.
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
    /// let moved = db.move_playlist_songs(vec!["sid1", "sid2"], 2).unwrap();
    /// println!("{:?}", moved);
    /// ```
    pub fn move_playlist_songs(
        &mut self,
        ids: Vec<&str>,
        start_seq: i32,
    ) -> Result<Vec<DjmdSongPlaylist>> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        DjmdSongPlaylist::move_all_to(&mut conn, &ids, Some(start_seq))?;
        let results = DjmdSongPlaylist::find_all(&mut conn, &ids)?;
        Ok(results)
    }

    /// Deletes a song entry from a playlist.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the song playlist entry to be deleted.
    ///
    /// # Returns
    /// The number of rows affected by the delete operation.
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
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(DjmdSongPlaylist::delete(&mut conn, id)?)
    }

    /// Deletes multiple song entries from a playlist.
    pub fn delete_playlist_songs(&mut self, ids: Vec<&str>) -> Result<usize> {
        self.assert_write_mode()?;
        let mut conn = self.pool.get()?;
        Ok(DjmdSongPlaylist::delete_all(&mut conn, ids)?)
    }

    // -- Property ---------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdProperty` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdProperty`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let properties = db.get_properties().unwrap();
    /// for property in properties {
    ///     println!("{:?}", property);
    /// }
    /// ```
    pub fn get_properties(&mut self) -> Result<Vec<DjmdProperty>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdProperty::all(&mut conn)?)
    }

    /// Retrieves a `djmdProperty` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the property.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdProperty`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdProperty::find(&mut conn, id)?)
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdCloudProperty` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdCloudProperty`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let cloud_properties = db.get_cloud_properties().unwrap();
    /// for property in cloud_properties {
    ///     println!("{:?}", property);
    /// }
    /// ```
    pub fn get_cloud_properties(&mut self) -> Result<Vec<DjmdCloudProperty>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdCloudProperty::all(&mut conn)?)
    }

    /// Retrieves a `djmdCloudProperty` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the cloud property.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdCloudProperty`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdCloudProperty::find(&mut conn, id)?)
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdRecommendLike` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdRecommendLike`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let likes = db.get_recommend_likes().unwrap();
    /// for like in likes {
    ///     println!("{:?}", like);
    /// }
    /// ```
    pub fn get_recommend_likes(&mut self) -> Result<Vec<DjmdRecommendLike>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdRecommendLike::all(&mut conn)?)
    }

    /// Retrieves a `djmdRecommendLike` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the recommend like entry.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdRecommendLike`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdRecommendLike::find(&mut conn, id)?)
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdRelatedTracks` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdRelatedTracks`] objects if the query is successful.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdRelatedTracks::all(&mut conn)?)
    }

    /// Retrieves all child `djmdRelatedTracks` entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent related tracks ID.
    ///
    /// # Returns
    /// A vector of child [`DjmdRelatedTracks`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdRelatedTracks::by_parent_id(&mut conn, parent_id)?)
    }

    /// Retrieves a `djmdRelatedTracks` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the related tracks entry.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdRelatedTracks`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdRelatedTracks::find(&mut conn, id)?)
    }

    /// Retrieves all `djmdSongRelatedTracks` entries for a given related tracks ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the related tracks ID.
    ///
    /// # Returns
    /// A vector of [`DjmdSongRelatedTracks`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSongRelatedTracks::by_related_tracks_id(&mut conn, id)?)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given related tracks ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the related tracks ID.
    ///
    /// # Returns
    /// A vector of [`DjmdContent`] objects referenced by the related tracks.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdRelatedTracks::get_contents(&mut conn, id)?)
    }

    // -- Sampler ----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdSampler` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdSampler`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let samplers = db.get_samplers().unwrap();
    /// for sampler in samplers {
    ///     println!("{:?}", sampler);
    /// }
    /// ```
    pub fn get_samplers(&mut self) -> Result<Vec<DjmdSampler>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdSampler::all(&mut conn)?)
    }

    /// Retrieves all child `djmdSampler` entries for a given parent ID, ordered by sequence.
    ///
    /// # Arguments
    /// * `parent_id` - A string slice representing the parent sampler ID.
    ///
    /// # Returns
    /// A vector of child [`DjmdSampler`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSampler::by_parent_id(&mut conn, parent_id)?)
    }

    /// Retrieves a `djmdSampler` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the sampler entry.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdSampler`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSampler::find(&mut conn, id)?)
    }

    /// Retrieves all `djmdSongSampler` entries for a given sampler ID, ordered by track number.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the sampler ID.
    ///
    /// # Returns
    /// A vector of [`DjmdSongSampler`] objects.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSongSampler::by_sampler_id(&mut conn, id)?)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given sampler ID.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the sampler ID.
    ///
    /// # Returns
    /// A vector of [`DjmdContent`] objects referenced by the sampler.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSampler::get_contents(&mut conn, id)?)
    }

    // -- SongTagList ------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdSongTagList` table in the database.
    ///
    /// # Returns
    /// A vector of [`DjmdSongTagList`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let tags = db.get_song_tag_lists().unwrap();
    /// for tag in tags {
    ///     println!("{:?}", tag);
    /// }
    /// ```
    pub fn get_song_tag_lists(&mut self) -> Result<Vec<DjmdSongTagList>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdSongTagList::all(&mut conn)?)
    }

    /// Retrieves a `djmdSongTagList` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the song tag list entry.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdSongTagList`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSongTagList::find(&mut conn, id)?)
    }

    // -- Sort -------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdSort` table in the database, ordered by sequence.
    ///
    /// # Returns
    /// A vector of [`DjmdSort`] objects if the query is successful.
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
    pub fn get_sorts(&mut self) -> Result<Vec<DjmdSort>> {
        let mut conn = self.pool.get()?;
        Ok(DjmdSort::all(&mut conn)?)
    }

    /// Retrieves a `djmdSort` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the sort entry.
    ///
    /// # Returns
    /// An `Option` containing the [`DjmdSort`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(DjmdSort::find(&mut conn, id)?)
    }

    // -- ImageFile --------------------------------------------------------------------------------

    /// Retrieves all entries from the `imageFile` table in the database.
    ///
    /// # Returns
    /// A vector of [`ImageFile`] objects if the query is successful.
    ///
    /// # Errors
    /// * Returns an error if the database query cannot be executed.
    ///
    /// # Example
    /// ```no_run
    /// let images = db.get_image_files().unwrap();
    /// for image in images {
    ///     println!("{:?}", image);
    /// }
    /// ```
    pub fn get_image_files(&mut self) -> Result<Vec<ImageFile>> {
        let mut conn = self.pool.get()?;
        Ok(ImageFile::all(&mut conn)?)
    }

    /// Retrieves an `imageFile` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the image file.
    ///
    /// # Returns
    /// An `Option` containing the [`ImageFile`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(ImageFile::find(&mut conn, id)?)
    }

    // -- SettingFile ------------------------------------------------------------------------------

    /// Retrieves all entries from the `settingFile` table in the database.
    ///
    /// # Returns
    /// A vector of [`SettingFile`] objects if the query is successful.
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
    pub fn get_setting_files(&mut self) -> Result<Vec<SettingFile>> {
        let mut conn = self.pool.get()?;
        Ok(SettingFile::all(&mut conn)?)
    }

    /// Retrieves a `settingFile` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the setting file.
    ///
    /// # Returns
    /// An `Option` containing the [`SettingFile`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(SettingFile::find(&mut conn, id)?)
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    /// Retrieves all entries from the `uuidIDMap` table in the database.
    ///
    /// # Returns
    /// A vector of [`UuidIDMap`] objects if the query is successful.
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
    pub fn get_uuid_id_maps(&mut self) -> Result<Vec<UuidIDMap>> {
        let mut conn = self.pool.get()?;
        Ok(UuidIDMap::all(&mut conn)?)
    }

    /// Retrieves a `uuidIDMap` entry by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - A string slice representing the unique identifier of the UUID ID map entry.
    ///
    /// # Returns
    /// An `Option` containing the [`UuidIDMap`] object if found, or `None`.
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
        let mut conn = self.pool.get()?;
        Ok(UuidIDMap::find(&mut conn, id)?)
    }
}
