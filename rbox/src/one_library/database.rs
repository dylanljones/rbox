// Author: Dylan Jones
// Date:   2025-11-16

//! # Rekordbox One Library (Device Library Plus) Database Handler
//!
//! This module provides a high-level interface for interacting with the Rekordbox
//! device library plus export SQLite database.
//!
//! The main struct [`OneLibrary`] encapsulates the database connection and provides methods for
//! accessing and modifying database entries. The queries are handled by the individual model structs.
//! For a lower level interface, these models can be used directly to perform database operations.
//!
//! ## Basic Usage
//! ```no_run
//! use rbox::OneLibrary;
//!
//! // Open the database
//! let mut db = OneLibrary::new("exportLibrary.db").unwrap();
//!
//! // Query all playlists
//! let playlists = db.get_playlists().unwrap();
//!
//! // Insert a new playlist
//! let new_playlist = db.create_playlist("My Playlist".to_string(), None, None, None, None).unwrap();
//! ```
//!

use std::ffi::OsStr;
use std::path::Path;

use super::conn::{create_connection_pool, establish_connection, run_migrations, DbPool};
use super::models::*;
use crate::error::{Error, Result};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};

#[derive(Clone, Debug)]
pub struct OneLibrary {
    /// A connection pool used for interacting with the database.
    pool: DbPool,
}

impl OneLibrary {
    /// Open a device library plus database specified by path.
    ///
    /// The `path` must be a valid database file.
    pub fn new<P: AsRef<OsStr>>(path: P) -> Result<Self> {
        let path = Path::new(&path);
        if !path.exists() {
            return Err(Error::FileNotFound(path.to_str().unwrap().to_string()));
        }
        let url = path.to_str().unwrap();
        Ok(Self {
            pool: create_connection_pool(url, 8)?,
        })
    }

    /// Create a new device library plus database.
    pub fn create<P: AsRef<OsStr>>(path: P) -> Result<Self> {
        let path = Path::new(&path);
        // Create simple connection and run migrtions
        let url = path.to_str().unwrap();
        let mut conn = establish_connection(url)?;
        run_migrations(&mut conn);

        Self::new(path)
    }

    pub fn create_tables(&self) -> Result<()> {
        let mut conn = self.pool.get()?;
        run_migrations(&mut conn);
        Ok(())
    }

    // -- Album ------------------------------------------------------------------------------------

    pub fn get_albums(&self) -> Result<Vec<Album>> {
        let mut conn = self.pool.get()?;
        Ok(Album::all(&mut conn)?)
    }

    pub fn get_album_by_id(&self, id: i32) -> Result<Option<Album>> {
        let mut conn = self.pool.get()?;
        Ok(Album::find(&mut conn, &id)?)
    }

    pub fn get_album_by_name(&self, name: &str) -> Result<Option<Album>> {
        let mut conn = self.pool.get()?;
        Ok(Album::find_by_name(&mut conn, name)?)
    }

    pub fn insert_album(&self, album: NewAlbum) -> Result<Album> {
        let mut conn = self.pool.get()?;
        Ok(album.insert(&mut conn)?)
    }

    pub fn create_album(
        &self,
        name: String,
        artist_id: Option<i32>,
        image_id: Option<i32>,
    ) -> Result<Album> {
        let mut album = NewAlbum::new(name);
        album.artist_id = artist_id;
        album.image_id = image_id;
        self.insert_album(album)
    }

    pub fn update_album(&self, album: Album) -> Result<Album> {
        let mut conn = self.pool.get()?;
        Ok(album.update(&mut conn)?)
    }

    pub fn delete_album(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Album::delete(&mut conn, &id)?)
    }
}
