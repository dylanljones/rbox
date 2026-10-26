// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

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
//! use rbox_onelib::OneLibrary;
//!
//! // Open the database
//! let mut db = OneLibrary::new("PIONEER/rekordbox").unwrap();
//!
//! // Query all playlists
//! let playlists = db.get_playlists().unwrap();
//!
//! // Insert a new playlist
//! let new_playlist = db.create_playlist("My Playlist", None, None).unwrap();
//! ```
//!

use diesel::Connection;
use rbox_core::{PathExt, Track};
use std::ffi::OsStr;
use std::path::Path;

use rbox_db::{
    ConnectionOptions, Model, ModelDelete, ModelInsert, ModelList, ModelRelations, ModelTree,
    ModelUpdate,
};

use crate::connection::{create_connection_pool, establish_connection, initialize_schema, DbPool};
use crate::enums::HistoryType;
use crate::error::{Error, Result};
use crate::models::*;

/// Create a new device library plus database at the specified path and initialize it with default data.
fn create(url: &str, my_tag_master_dbid: i32, options: &ConnectionOptions) -> Result<()> {
    let mut conn = establish_connection(url, options)?;
    // create tables and indexes for the One Library database schema.
    // This also populates the DB with some default values.
    initialize_schema(&mut conn)?;

    // Default property
    let property = Property::new("", my_tag_master_dbid);
    property.insert(&mut conn)?;

    Ok(())
}

#[derive(Clone, Debug)]
pub struct OneLibrary {
    /// A connection pool used for interacting with the database.
    pool: DbPool,
}

impl OneLibrary {
    /// Open `exportLibrary.db` in the supplied directory.
    ///
    /// Pass the database directory, usually `PIONEER/rekordbox`, not the USB root.
    /// Use [`Self::from_path`] to open an explicit database filename.
    pub fn new<P: AsRef<OsStr>>(directory: P) -> Result<Self> {
        Self::from_path(Path::new(&directory).join("exportLibrary.db"))
    }

    /// Open an existing OneLibrary database at an explicit file path.
    ///
    /// The filename may differ from `exportLibrary.db`. Missing paths and
    /// directories return [`Error::FileNotFound`]; I/O and connection errors
    /// propagate. This does not create a database.
    pub fn from_path<P: AsRef<OsStr>>(path: P) -> Result<Self> {
        let path = Path::new(&path);
        match std::fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => (),
            Ok(_) => return Err(Error::FileNotFound(path.to_path_buf())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::FileNotFound(path.to_path_buf()));
            }
            Err(error) => return Err(error.into()),
        }
        Ok(Self {
            pool: create_connection_pool(path, &ConnectionOptions::default(), 8)?,
        })
    }

    /// Create a new OneLibrary database at an explicit file path.
    pub fn create<P: AsRef<OsStr>>(path: P, my_tag_master_dbid: i32) -> Result<Self> {
        let path = Path::new(&path);
        create(
            path.to_str().unwrap(),
            my_tag_master_dbid,
            &ConnectionOptions::default(),
        )?;
        Self::from_path(path)
    }

    /// Create a database at an explicit file path, or open it if it already exists.
    pub fn create_if_not_exists<P: AsRef<OsStr>>(path: P, my_tag_master_dbid: i32) -> Result<Self> {
        let path = Path::new(&path);
        if path.exists() {
            Self::from_path(path)
        } else {
            Self::create(path, my_tag_master_dbid)
        }
    }

    // -- Album ------------------------------------------------------------------------------------

    /// Load an album and all direct relationships in one read transaction.
    /// Returns a detached snapshot; missing relationship targets remain `None`.
    pub fn get_album_with_relations(&self, id: i32) -> Result<Option<AlbumView>> {
        let mut conn = self.pool.get()?;
        Ok(Album::find_with_relations(&mut conn, id)?)
    }

    /// Load all albums and their direct relationships using batched queries.
    /// Base and related rows are read within the same transaction.
    pub fn get_albums_with_relations(&self) -> Result<Vec<AlbumView>> {
        let mut conn = self.pool.get()?;
        Ok(Album::all_with_relations(&mut conn)?)
    }

    /// Queries all records from the `album` table.
    pub fn get_albums(&self) -> Result<Vec<Album>> {
        let mut conn = self.pool.get()?;
        Ok(Album::all(&mut conn)?)
    }

    /// Queries a record from the `album` table by its primary key `id`.
    pub fn get_album_by_id(&self, id: i32) -> Result<Option<Album>> {
        let mut conn = self.pool.get()?;
        Ok(Album::find(&mut conn, &id)?)
    }

    /// Queries a record from the `album` table by its `name`.
    pub fn get_album_by_name(&self, name: &str) -> Result<Option<Album>> {
        let mut conn = self.pool.get()?;
        Ok(Album::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new record into the `album` table.
    pub fn insert_album(&self, album: NewAlbum) -> Result<Album> {
        let mut conn = self.pool.get()?;
        Ok(album.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `album` table.
    pub fn create_album<S: Into<String>>(
        &self,
        name: S,
        artist_id: Option<i32>,
        image_id: Option<i32>,
    ) -> Result<Album> {
        let mut album = NewAlbum::new(name);
        album.artist_id = artist_id;
        album.image_id = image_id;
        self.insert_album(album)
    }

    /// Updates an existing record in the `album` table.
    pub fn update_album(&self, album: Album) -> Result<Album> {
        let mut conn = self.pool.get()?;
        Ok(album.update(&mut conn)?)
    }

    /// Deletes a record from the `album` table by its primary key `id`.
    pub fn delete_album(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Album::delete(&mut conn, &id)?)
    }

    // -- Artist -----------------------------------------------------------------------------------

    /// Queries all records from the `artist` table.
    pub fn get_artists(&self) -> Result<Vec<Artist>> {
        let mut conn = self.pool.get()?;
        Ok(Artist::all(&mut conn)?)
    }

    /// Queries a record from the `artist` table by its primary key `id`.
    pub fn get_artist_by_id(&self, id: i32) -> Result<Option<Artist>> {
        let mut conn = self.pool.get()?;
        Ok(Artist::find(&mut conn, &id)?)
    }

    /// Queries a record from the `artist` table by its `name`.
    pub fn get_artist_by_name(&self, name: &str) -> Result<Option<Artist>> {
        let mut conn = self.pool.get()?;
        Ok(Artist::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new record into the `artist` table.
    pub fn insert_artist(&self, artist: NewArtist) -> Result<Artist> {
        let mut conn = self.pool.get()?;
        Ok(artist.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `artist` table.
    pub fn create_artist<S: Into<String>>(&self, name: S) -> Result<Artist> {
        let artist = NewArtist::new(name);
        self.insert_artist(artist)
    }

    /// Updates an existing record in the `artist` table.
    pub fn update_artist(&self, artist: Artist) -> Result<Artist> {
        let mut conn = self.pool.get()?;
        Ok(artist.update(&mut conn)?)
    }

    /// Deletes a record from the `artist` table by its primary key `id`.
    pub fn delete_artist(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Artist::delete(&mut conn, &id)?)
    }

    // -- Category ---------------------------------------------------------------------------------

    /// Load a category and all direct relationships in one read transaction.
    /// Returns a detached snapshot; missing relationship targets remain `None`.
    pub fn get_category_with_relations(&self, id: i32) -> Result<Option<CategoryView>> {
        let mut conn = self.pool.get()?;
        Ok(Category::find_with_relations(&mut conn, id)?)
    }

    /// Load all categories and their direct relationships using batched queries.
    /// Base and related rows are read within the same transaction.
    pub fn get_categories_with_relations(&self) -> Result<Vec<CategoryView>> {
        let mut conn = self.pool.get()?;
        Ok(Category::all_with_relations(&mut conn)?)
    }

    /// Queries all records from the `category` table.
    pub fn get_categories(&self) -> Result<Vec<Category>> {
        let mut conn = self.pool.get()?;
        Ok(Category::all(&mut conn)?)
    }

    /// Queries a record from the `category` table by its primary key `id`.
    pub fn get_category_by_id(&self, id: i32) -> Result<Option<Category>> {
        let mut conn = self.pool.get()?;
        Ok(Category::find(&mut conn, &id)?)
    }

    /// Inserts a new record into the `category` table.
    pub fn insert_category(&self, category: NewCategory) -> Result<Category> {
        let mut conn = self.pool.get()?;
        Ok(category.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `category` table.
    pub fn create_category(
        &self,
        menu_item_id: i32,
        seq: Option<i32>,
        is_visible: Option<i32>,
    ) -> Result<Category> {
        let category = NewCategory {
            menu_item_id,
            seq: seq.unwrap_or_default(),
            is_visible: is_visible.unwrap_or_default(),
        };
        self.insert_category(category)
    }

    /// Updates an existing record in the `category` table.
    pub fn update_category(&self, category: Category) -> Result<Category> {
        let mut conn = self.pool.get()?;
        Ok(category.update(&mut conn)?)
    }

    /// Deletes a record from the `category` table by its primary key `id`.
    pub fn delete_category(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Category::delete(&mut conn, &id)?)
    }

    // -- Color ------------------------------------------------------------------------------------

    /// Queries all records from the `color` table.
    pub fn get_colors(&self) -> Result<Vec<Color>> {
        let mut conn = self.pool.get()?;
        Ok(Color::all(&mut conn)?)
    }

    /// Queries a record from the `color` table by its primary key `id`.
    pub fn get_color_by_id(&self, id: i32) -> Result<Option<Color>> {
        let mut conn = self.pool.get()?;
        Ok(Color::find(&mut conn, &id)?)
    }

    /// Inserts a new record into the `color` table.
    pub fn insert_color(&self, color: NewColor) -> Result<Color> {
        let mut conn = self.pool.get()?;
        Ok(color.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `color` table.
    pub fn create_color<S: Into<String>>(&self, name: S) -> Result<Color> {
        let color = NewColor::new(name);
        self.insert_color(color)
    }

    /// Updates an existing record in the `color` table.
    pub fn update_color(&self, color: Color) -> Result<Color> {
        let mut conn = self.pool.get()?;
        Ok(color.update(&mut conn)?)
    }

    /// Deletes a record from the `color` table by its primary key `id`.
    pub fn delete_color(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Color::delete(&mut conn, &id)?)
    }

    // -- Content ----------------------------------------------------------------------------------

    /// Load shared tracks with resolved metadata, including album artists.
    ///
    /// Content and relationships share one read transaction. Results are detached
    /// snapshots; absent metadata stays `None`. BPM is unscaled, duration is seconds,
    /// and file size is bytes. Source-library IDs come from each content row.
    /// Database and metadata conversion errors propagate.
    pub fn get_tracks(&self) -> Result<Vec<Track>> {
        self.read_tracks(Content::all)
    }

    /// Load a shared track by its OneLibrary content ID, or `None` if absent.
    /// See [`Self::get_tracks`] for resolution, units and errors.
    pub fn get_track_by_id(&self, id: i32) -> Result<Option<Track>> {
        Ok(self
            .read_tracks(|conn| Ok(Content::find(conn, &id)?.into_iter().collect()))?
            .pop())
    }

    /// Load matching tracks in ascending ID order with batched queries.
    ///
    /// Missing IDs are omitted, repeated IDs occur once, and empty input returns
    /// an empty vector. See [`Self::get_tracks`] for resolution, units and errors.
    pub fn get_tracks_by_ids(&self, ids: &[i32]) -> Result<Vec<Track>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        self.read_tracks(|conn| {
            let mut ids = ids.to_vec();
            ids.sort_unstable();
            ids.dedup();
            let mut rows = Vec::new();
            for chunk in ids.chunks(500) {
                rows.extend(Content::by_ids(conn, chunk)?);
            }
            rows.sort_unstable_by_key(|row| row.id);
            Ok(rows)
        })
    }

    /// Load a shared track by its stored path, or `None` if absent.
    ///
    /// Backslashes are converted to forward slashes without accessing the file;
    /// the path must otherwise match exactly. See [`Self::get_tracks`] for errors.
    pub fn get_track_by_path(&self, path: &str) -> Result<Option<Track>> {
        let path = Path::new(path).to_unix();
        Ok(self
            .read_tracks(|conn| {
                Ok(Content::find_by_path(conn, &path.to_string_lossy())?
                    .into_iter()
                    .collect())
            })?
            .pop())
    }

    fn read_tracks(
        &self,
        load: impl FnOnce(&mut crate::connection::DbConn) -> diesel::QueryResult<Vec<Content>>,
    ) -> Result<Vec<Track>> {
        let mut conn = self.pool.get()?;
        conn.transaction(|conn| {
            let rows = load(conn)?;
            Content::resolve_many(conn, rows)?
                .into_iter()
                .map(Track::try_from)
                .collect()
        })
    }

    /// Queries all records from the `color` table.
    pub fn get_contents(&self) -> Result<Vec<Content>> {
        let mut conn = self.pool.get()?;
        Ok(Content::all(&mut conn)?)
    }

    /// Queries a record from the `content` table by its primary key `id`.
    pub fn get_content_by_id(&self, id: i32) -> Result<Option<Content>> {
        let mut conn = self.pool.get()?;
        Ok(Content::find(&mut conn, &id)?)
    }

    /// Find content by both source-library IDs in OneLibrary's stored signed form.
    ///
    /// Returns `None` for missing or zero identities. If the pair occurs more than
    /// once, returns the content with the lowest local export ID.
    pub fn get_content_by_master_content_id(
        &self,
        master_content_id: i32,
        master_db_id: i32,
    ) -> Result<Option<Content>> {
        let mut conn = self.pool.get()?;
        Ok(Content::find_by_master_content_id(
            &mut conn,
            master_content_id,
            master_db_id,
        )?)
    }

    /// Load contents by source-content IDs within one source library, using signed IDs.
    ///
    /// Returns all matches in local export-ID order, omitting missing/zero IDs and
    /// repeated input IDs. See [`Content::by_master_content_ids`] for batch semantics.
    pub fn get_contents_by_master_content_ids(
        &self,
        master_content_ids: &[i32],
        master_db_id: i32,
    ) -> Result<Vec<Content>> {
        if master_db_id == 0 || master_content_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut conn = self.pool.get()?;
        Ok(Content::by_master_content_ids(
            &mut conn,
            master_content_ids,
            master_db_id,
        )?)
    }

    /// Load a track and all direct metadata relationships in one read transaction.
    /// Returns a detached snapshot; missing relationship targets remain `None`.
    pub fn get_content_with_relations(&self, id: i32) -> Result<Option<ContentView>> {
        let mut conn = self.pool.get()?;
        Ok(Content::find_with_relations(&mut conn, id)?)
    }

    /// Load all tracks and their direct metadata using batched relationship queries.
    /// Content and related rows are read within the same transaction.
    pub fn get_contents_with_relations(&self) -> Result<Vec<ContentView>> {
        let mut conn = self.pool.get()?;
        Ok(Content::all_with_relations(&mut conn)?)
    }

    /// Inserts a new record into the `content` table.
    pub fn insert_content(&self, content: NewContent) -> Result<Content> {
        let mut conn = self.pool.get()?;
        Ok(content.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `content` table.
    pub fn create_content<S: Into<String>>(&self, path: S) -> Result<Content> {
        let content = NewContent::new(path);
        self.insert_content(content)
    }

    /// Updates an existing record in the `content` table.
    pub fn update_content(&self, content: Content) -> Result<Content> {
        let mut conn = self.pool.get()?;
        Ok(content.update(&mut conn)?)
    }

    /// Deletes a record from the `content` table by its primary key `id`.
    pub fn delete_content(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Content::delete(&mut conn, &id)?)
    }

    // -- Cue --------------------------------------------------------------------------------------

    /// Queries all records from the `color` table.
    pub fn get_cues(&self) -> Result<Vec<Cue>> {
        let mut conn = self.pool.get()?;
        Ok(Cue::all(&mut conn)?)
    }

    /// Queries a record from the `cue` table by its primary key `id`.
    pub fn get_cue_by_id(&self, id: i32) -> Result<Option<Cue>> {
        let mut conn = self.pool.get()?;
        Ok(Cue::find(&mut conn, &id)?)
    }

    /// Updates an existing record in the `cue` table.
    pub fn update_cue(&self, cue: Cue) -> Result<Cue> {
        let mut conn = self.pool.get()?;
        Ok(cue.update(&mut conn)?)
    }

    /// Deletes a record from the `cue` table by its primary key `id`.
    pub fn delete_cue(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Cue::delete(&mut conn, &id)?)
    }

    // -- Genre ------------------------------------------------------------------------------------

    /// Queries all records from the `genre` table.
    pub fn get_genres(&self) -> Result<Vec<Genre>> {
        let mut conn = self.pool.get()?;
        Ok(Genre::all(&mut conn)?)
    }

    /// Queries a record from the `genre` table by its primary key `id`.
    pub fn get_genre_by_id(&self, id: i32) -> Result<Option<Genre>> {
        let mut conn = self.pool.get()?;
        Ok(Genre::find(&mut conn, &id)?)
    }

    /// Queries a record from the `genre` table by its `name`.
    pub fn get_genre_by_name(&self, name: &str) -> Result<Option<Genre>> {
        let mut conn = self.pool.get()?;
        Ok(Genre::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new record into the `genre` table.
    pub fn insert_genre(&self, genre: NewGenre) -> Result<Genre> {
        let mut conn = self.pool.get()?;
        Ok(genre.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `genre` table.
    pub fn create_genre<S: Into<String>>(&self, name: S) -> Result<Genre> {
        let genre = NewGenre::new(name);
        self.insert_genre(genre)
    }

    /// Updates an existing record in the `genre` table.
    pub fn update_genre(&self, genre: Genre) -> Result<Genre> {
        let mut conn = self.pool.get()?;
        Ok(genre.update(&mut conn)?)
    }

    /// Deletes a record from the `genre` table by its primary key `id`.
    pub fn delete_genre(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Genre::delete(&mut conn, &id)?)
    }

    // -- History ----------------------------------------------------------------------------------

    /// Queries all records from the `history` table.
    pub fn get_histories(&self) -> Result<Vec<History>> {
        let mut conn = self.pool.get()?;
        Ok(History::all(&mut conn)?)
    }

    /// Queries all child records from the `history` table by parent `id`.
    pub fn get_history_children(&self, id: i32) -> Result<Vec<History>> {
        let mut conn = self.pool.get()?;
        Ok(History::by_parent_id(&mut conn, id)?)
    }

    /// Queries all `content` records associated with a `history` record.
    pub fn get_history_contents(&self, id: i32) -> Result<Vec<Content>> {
        let mut conn = self.pool.get()?;
        Ok(History::get_contents(&mut conn, id)?)
    }

    /// Queries a record from the `history` table by its primary key `id`.
    pub fn get_history_by_id(&self, id: i32) -> Result<Option<History>> {
        let mut conn = self.pool.get()?;
        Ok(History::find(&mut conn, &id)?)
    }

    /// Inserts a new record into the `history` table.
    pub fn insert_history(&self, history: NewHistory) -> Result<History> {
        let mut conn = self.pool.get()?;
        Ok(history.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `history` table.
    pub fn create_history<S: Into<String>>(
        &self,
        name: S,
        attribute: HistoryType,
    ) -> Result<History> {
        let history = NewHistory::new(name, attribute);
        self.insert_history(history)
    }

    /// Deletes a record from the `history` table by its primary key `id`.
    pub fn delete_history(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(History::delete(&mut conn, &id)?)
    }

    /// Creates a new record and inserts it into the `history_content` table.
    pub fn create_history_content(
        &self,
        history_id: i32,
        content_id: i32,
    ) -> Result<HistoryContent> {
        let mut conn = self.pool.get()?;
        let history_content = HistoryContent::new(history_id, content_id);
        Ok(history_content.insert(&mut conn)?)
    }

    /// Deletes a record from the `history_content` table by its primary key.
    pub fn delete_history_content(&self, history_id: i32, content_id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(HistoryContent::delete(
            &mut conn,
            &(history_id, content_id),
        )?)
    }

    // -- HotCueBankList ---------------------------------------------------------------------------

    /// Queries all records from the `color` table.
    pub fn get_hot_cue_bank_lists(&self) -> Result<Vec<HotCueBankList>> {
        let mut conn = self.pool.get()?;
        Ok(HotCueBankList::all(&mut conn)?)
    }

    /// Queries a record from the `hot_cue_bank_list` table by its primary key `id`.
    pub fn get_hot_cue_bank_list_by_id(&self, id: i32) -> Result<Option<HotCueBankList>> {
        let mut conn = self.pool.get()?;
        Ok(HotCueBankList::find(&mut conn, &id)?)
    }

    // -- Image ------------------------------------------------------------------------------------

    /// Queries all records from the `image` table.
    pub fn get_images(&self) -> Result<Vec<Image>> {
        let mut conn = self.pool.get()?;
        Ok(Image::all(&mut conn)?)
    }

    /// Queries a record from the `image` table by its primary key `id`.
    pub fn get_image_by_id(&self, id: i32) -> Result<Option<Image>> {
        let mut conn = self.pool.get()?;
        Ok(Image::find(&mut conn, &id)?)
    }

    /// Queries a record from the `image` table by its `path`.
    pub fn get_image_by_path(&self, path: &str) -> Result<Option<Image>> {
        let mut conn = self.pool.get()?;
        Ok(Image::find_by_path(&mut conn, path)?)
    }

    /// Inserts a new record into the `image` table.
    pub fn insert_image(&self, image: NewImage) -> Result<Image> {
        let mut conn = self.pool.get()?;
        Ok(image.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `image` table.
    pub fn create_image<S: Into<String>>(&self, path: S) -> Result<Image> {
        let image = NewImage::new(path);
        self.insert_image(image)
    }

    /// Updates an existing record in the `image` table.
    pub fn update_image(&self, image: Image) -> Result<Image> {
        let mut conn = self.pool.get()?;
        Ok(image.update(&mut conn)?)
    }

    /// Deletes a record from the `image` table by its primary key `id`.
    pub fn delete_image(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Image::delete(&mut conn, &id)?)
    }

    // -- Key --------------------------------------------------------------------------------------

    /// Queries all records from the `key` table.
    pub fn get_keys(&self) -> Result<Vec<Key>> {
        let mut conn = self.pool.get()?;
        Ok(Key::all(&mut conn)?)
    }

    /// Queries a record from the `key` table by its primary key `id`.
    pub fn get_key_by_id(&self, id: i32) -> Result<Option<Key>> {
        let mut conn = self.pool.get()?;
        Ok(Key::find(&mut conn, &id)?)
    }

    /// Queries a record from the `key` table by its `name`.
    pub fn get_key_by_name(&self, name: &str) -> Result<Option<Key>> {
        let mut conn = self.pool.get()?;
        Ok(Key::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new record into the `key` table.
    pub fn insert_key(&self, key: NewKey) -> Result<Key> {
        let mut conn = self.pool.get()?;
        Ok(key.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `key` table.
    pub fn create_key<S: Into<String>>(&self, name: S) -> Result<Key> {
        let key = NewKey::new(name);
        self.insert_key(key)
    }

    /// Updates an existing record in the `key` table.
    pub fn update_key(&self, key: Key) -> Result<Key> {
        let mut conn = self.pool.get()?;
        Ok(key.update(&mut conn)?)
    }

    /// Deletes a record from the `key` table by its primary key `id`.
    pub fn delete_key(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Key::delete(&mut conn, &id)?)
    }

    // -- Label ------------------------------------------------------------------------------------

    /// Queries all records from the `label` table.
    pub fn get_labels(&self) -> Result<Vec<Label>> {
        let mut conn = self.pool.get()?;
        Ok(Label::all(&mut conn)?)
    }

    /// Queries a record from the `label` table by its primary key `id`.
    pub fn get_label_by_id(&self, id: i32) -> Result<Option<Label>> {
        let mut conn = self.pool.get()?;
        Ok(Label::find(&mut conn, &id)?)
    }

    /// Queries a record from the `label` table by its `name`.
    pub fn get_label_by_name(&self, name: &str) -> Result<Option<Label>> {
        let mut conn = self.pool.get()?;
        Ok(Label::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new record into the `label` table.
    pub fn insert_label(&self, label: NewLabel) -> Result<Label> {
        let mut conn = self.pool.get()?;
        Ok(label.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `label` table.
    pub fn create_label<S: Into<String>>(&self, name: S) -> Result<Label> {
        let label = NewLabel::new(name);
        self.insert_label(label)
    }

    /// Updates an existing record in the `label` table.
    pub fn update_label(&self, label: Label) -> Result<Label> {
        let mut conn = self.pool.get()?;
        Ok(label.update(&mut conn)?)
    }

    /// Deletes a record from the `label` table by its primary key `id`.
    pub fn delete_label(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Label::delete(&mut conn, &id)?)
    }

    // -- MenuItem ---------------------------------------------------------------------------------

    /// Queries all records from the `menuItem` table.
    pub fn get_menu_items(&self) -> Result<Vec<MenuItem>> {
        let mut conn = self.pool.get()?;
        Ok(MenuItem::all(&mut conn)?)
    }

    /// Queries a record from the `menuItem` table by its primary key `id`.
    pub fn get_menu_item_by_id(&self, id: i32) -> Result<Option<MenuItem>> {
        let mut conn = self.pool.get()?;
        Ok(MenuItem::find(&mut conn, &id)?)
    }

    /// Queries a record from the `menuItem` table by its `name`.
    pub fn get_menu_item_by_name(&self, name: &str) -> Result<Option<MenuItem>> {
        let mut conn = self.pool.get()?;
        Ok(MenuItem::find_by_name(&mut conn, name)?)
    }

    /// Inserts a new record into the `menuItem` table.
    pub fn insert_menu_item(&self, menu_item: NewMenuItem) -> Result<MenuItem> {
        let mut conn = self.pool.get()?;
        Ok(menu_item.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `menuItem` table.
    pub fn create_menu_item<S: Into<String>>(&self, name: S, kind: i32) -> Result<MenuItem> {
        let menu_item = NewMenuItem::new(name, kind);
        self.insert_menu_item(menu_item)
    }

    /// Updates an existing record in the `menuItem` table.
    pub fn update_menu_item(&self, menu_item: MenuItem) -> Result<MenuItem> {
        let mut conn = self.pool.get()?;
        Ok(menu_item.update(&mut conn)?)
    }

    /// Deletes a record from the `menuItem` table by its primary key `id`.
    pub fn delete_menu_item(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(MenuItem::delete(&mut conn, &id)?)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    /// Queries all records from the `myTag` table.
    pub fn get_my_tags(&self) -> Result<Vec<MyTag>> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::all(&mut conn)?)
    }

    /// Queries all child records from the `myTag` table by parent `id`.
    pub fn get_my_tag_children(&self, id: i32) -> Result<Vec<MyTag>> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::by_parent_id(&mut conn, id)?)
    }

    /// Queries all `content` records associated with a `myTag` record.
    pub fn get_my_tag_contents(&self, id: i32) -> Result<Vec<Content>> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::get_contents(&mut conn, id)?)
    }

    /// Queries a record from the `myTag` table by its primary key `id`.
    pub fn get_my_tag_by_id(&self, id: i32) -> Result<Option<MyTag>> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::find(&mut conn, &id)?)
    }

    /// Queries all records from the `myTag` table their associated `content_id` via `myTag_content`
    pub fn get_my_tag_by_content_id(&self, cid: i32) -> Result<Vec<MyTag>> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::by_content_id(&mut conn, cid)?)
    }

    /// Inserts a new record into the `myTag` table.
    pub fn insert_my_tag(&self, my_tag: NewMyTag) -> Result<MyTag> {
        let mut conn = self.pool.get()?;
        Ok(my_tag.insert(&mut conn)?)
    }

    /// Creates a new tag record and inserts it into the `myTag` table.
    pub fn create_my_tag<S: Into<String>>(
        &self,
        name: S,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<MyTag> {
        let mut my_tag = NewMyTag::tag(name);
        my_tag.parent_id = parent_id.unwrap_or_default();
        my_tag.seq = seq;
        self.insert_my_tag(my_tag)
    }

    /// Creates a new my_tag folder record and inserts it into the `myTag` table.
    pub fn create_my_tag_folder<S: Into<String>>(
        &self,
        name: S,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<MyTag> {
        let mut my_tag = NewMyTag::folder(name);
        my_tag.parent_id = parent_id.unwrap_or_default();
        my_tag.seq = seq;
        self.insert_my_tag(my_tag)
    }

    /// Renames an existing my_tag record in the `myTag` table.
    pub fn rename_my_tag(&self, my_tag_id: i32, name: &str) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::rename(&mut conn, my_tag_id, name)?)
    }

    /// Moves a tag to a new parent folder or changes its sequence within the same parent.
    pub fn move_my_tag(
        &self,
        my_tag_id: i32,
        parent_id: Option<&i32>,
        seq: Option<i32>,
    ) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::move_to(&mut conn, &my_tag_id, parent_id, seq)?)
    }

    /// Moves multiple tags to a new parent folder or changes its sequence within the same parent.
    pub fn move_my_tags(
        &self,
        my_tag_ids: Vec<&i32>,
        parent_id: i32,
        start_seq: Option<i32>,
    ) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::move_all_to(
            &mut conn,
            &my_tag_ids,
            &parent_id,
            start_seq,
        )?)
    }

    /// Deletes a record from the `myTag` table by its primary key `id`.
    pub fn delete_my_tag(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::delete(&mut conn, &id)?)
    }

    /// Deletes multiple records from the `myTag` table by its primary keys `id`.
    pub fn delete_my_tags(&self, ids: Vec<&i32>) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(MyTag::delete_all(&mut conn, ids)?)
    }

    /// Deletes a record from the `myTag_content` table by its primary key.
    pub fn delete_my_tag_content(&self, my_tag_id: i32, content_id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(MyTagContent::delete(&mut conn, &(my_tag_id, content_id))?)
    }

    /// Deletes a record from the `myTag_content` table by its primary key.
    pub fn delete_my_tag_contents(&self, ids: Vec<&(i32, i32)>) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(MyTagContent::delete_all(&mut conn, ids)?)
    }

    // -- Playlist ---------------------------------------------------------------------------------

    /// Queries all records from the `playlist` table.
    pub fn get_playlists(&self) -> Result<Vec<Playlist>> {
        let mut conn = self.pool.get()?;
        Ok(Playlist::all(&mut conn)?)
    }

    /// Queries all child records from the `playlist` table by parent `id`.
    pub fn get_playlist_children(&self, id: i32) -> Result<Vec<Playlist>> {
        let mut conn = self.pool.get()?;
        Ok(Playlist::by_parent_id(&mut conn, id)?)
    }

    /// Queries all `content` records associated with a `playlist` record.
    pub fn get_playlist_contents(&self, id: i32) -> Result<Vec<Content>> {
        let mut conn = self.pool.get()?;
        Ok(Playlist::get_contents(&mut conn, id)?)
    }

    /// Queries a record from the `playlist` table by its primary key `id`.
    pub fn get_playlist_by_id(&self, id: i32) -> Result<Option<Playlist>> {
        let mut conn = self.pool.get()?;
        Ok(Playlist::find(&mut conn, &id)?)
    }

    /// Inserts a new record into the `playlist` table.
    pub fn insert_playlist(&self, playlist: NewPlaylist) -> Result<Playlist> {
        let mut conn = self.pool.get()?;
        Ok(playlist.insert(&mut conn)?)
    }

    /// Creates a new playlist record and inserts it into the `playlist` table.
    pub fn create_playlist<S: Into<String>>(
        &self,
        name: S,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<Playlist> {
        let mut playlist = NewPlaylist::playlist(name);
        playlist.parent_id = parent_id.unwrap_or_default();
        playlist.seq = seq;
        self.insert_playlist(playlist)
    }

    /// Creates a new playlist folder record and inserts it into the `playlist` table.
    pub fn create_playlist_folder<S: Into<String>>(
        &self,
        name: S,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<Playlist> {
        let mut playlist = NewPlaylist::folder(name);
        playlist.parent_id = parent_id.unwrap_or_default();
        playlist.seq = seq;
        self.insert_playlist(playlist)
    }

    /// Renames an existing playlist record in the `playlist` table.
    pub fn rename_playlist(&self, playlist_id: i32, name: &str) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Playlist::rename(&mut conn, playlist_id, name)?)
    }

    /// Moves a playlist to a new parent folder or changes its sequence within the same parent.
    pub fn move_playlist(
        &self,
        playlist_id: i32,
        parent_id: Option<&i32>,
        seq: Option<i32>,
    ) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Playlist::move_to(&mut conn, &playlist_id, parent_id, seq)?)
    }

    /// Moves multiple playlists to a new parent folder or changes its sequence within the same parent.
    pub fn move_playlists(
        &self,
        playlist_ids: Vec<&i32>,
        parent_id: i32,
        start_seq: Option<i32>,
    ) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Playlist::move_all_to(
            &mut conn,
            &playlist_ids,
            &parent_id,
            start_seq,
        )?)
    }

    /// Deletes a record from the `playlist` table by its primary key `id`.
    pub fn delete_playlist(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Playlist::delete(&mut conn, &id)?)
    }

    /// Deletes multiple records from the `playlist` table by its primary keys `id`.
    pub fn delete_playlists(&self, ids: Vec<&i32>) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Playlist::delete_all(&mut conn, ids)?)
    }

    /// Creates a new record and inserts it into the `playlist_content` table.
    /// Positions start at one; an omitted position appends.
    pub fn create_playlist_content(
        &self,
        playlist_id: i32,
        content_id: i32,
        seq: Option<i32>,
    ) -> Result<PlaylistContent> {
        let mut conn = self.pool.get()?;
        let mut playlist_content = PlaylistContent::new(playlist_id, content_id);
        playlist_content.seq = seq.unwrap_or(-1);
        Ok(playlist_content.insert(&mut conn)?)
    }

    /// Moves a playlist content to a new sequence number.
    /// Positions start at one; an omitted position moves to the end.
    pub fn move_playlist_content(
        &self,
        playlist_id: i32,
        content_id: i32,
        seq: Option<i32>,
    ) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(PlaylistContent::move_to(
            &mut conn,
            &(playlist_id, content_id),
            seq,
        )?)
    }

    /// Moves multiple playlist contents to a new sequence number.
    /// The one-based destination is measured after removing the selected entries;
    /// an omitted position appends, preserving the supplied entry order.
    pub fn move_playlist_contents(
        &self,
        ids: Vec<&(i32, i32)>,
        start_seq: Option<i32>,
    ) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(PlaylistContent::move_all_to(&mut conn, &ids, start_seq)?)
    }

    /// Deletes a record from the `playlist_content` table by its primary key.
    pub fn delete_playlist_content(&self, playlist_id: i32, content_id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(PlaylistContent::delete(
            &mut conn,
            &(playlist_id, content_id),
        )?)
    }

    /// Deletes a record from the `playlist_content` table by its primary key.
    pub fn delete_playlist_contents(&self, ids: Vec<&(i32, i32)>) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(PlaylistContent::delete_all(&mut conn, ids)?)
    }

    // -- Property ---------------------------------------------------------------------------------

    /// Queries all records from the `property` table.
    pub fn get_properties(&self) -> Result<Vec<Property>> {
        let mut conn = self.pool.get()?;
        Ok(Property::all(&mut conn)?)
    }

    /// Queries a record from the `property` table by its primary key `device_name`.
    pub fn get_property_by_device(&self, device_name: &str) -> Result<Option<Property>> {
        let mut conn = self.pool.get()?;
        Ok(Property::find(&mut conn, device_name)?)
    }

    /// Queries a record from the `property` table by its primary key `device_name`.
    pub fn get_first_property(&self) -> Result<Option<Property>> {
        let mut conn = self.pool.get()?;
        Ok(Property::first(&mut conn)?)
    }

    /// Inserts a new record into the `property` table.
    pub fn insert_property(&self, property: Property) -> Result<Property> {
        let mut conn = self.pool.get()?;
        Ok(property.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `property` table.
    pub fn create_property<S: Into<String>>(
        &self,
        device_name: S,
        my_tag_master_dbid: i32,
    ) -> Result<Property> {
        let property = Property::new(device_name, my_tag_master_dbid);
        self.insert_property(property)
    }

    /// Updates an existing record in the `property` table.
    pub fn update_property(&self, property: Property) -> Result<Property> {
        let mut conn = self.pool.get()?;
        Ok(property.update(&mut conn)?)
    }

    /// Deletes a record from the `property` table by its primary key `device_name`.
    pub fn delete_property(&self, device_name: &str) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Property::delete(&mut conn, device_name)?)
    }

    // -- RecommendedLike --------------------------------------------------------------------------

    /// Queries all records from the `recommendLike` table.
    pub fn get_recommend_likes(&self) -> Result<Vec<RecommendedLike>> {
        let mut conn = self.pool.get()?;
        Ok(RecommendedLike::all(&mut conn)?)
    }

    /// Queries a record from the `recommendLike` table by its primary key `id`.
    pub fn get_recommend_like_by_id(
        &self,
        content_id_1: i32,
        content_id_2: i32,
    ) -> Result<Option<RecommendedLike>> {
        let mut conn = self.pool.get()?;
        Ok(RecommendedLike::find(
            &mut conn,
            &(content_id_1, content_id_2),
        )?)
    }

    /// Queries all records from the `recommendedLike` table that are linked to the given `content`.
    pub fn get_recommend_like_by_content_id(
        &self,
        content_id: i32,
    ) -> Result<Vec<RecommendedLike>> {
        let mut conn = self.pool.get()?;
        Ok(RecommendedLike::by_content_id(&mut conn, content_id)?)
    }

    /// Get related `content` ids for a given `content`.
    pub fn get_recommend_like_contents(&self, content_id: i32) -> Result<Vec<i32>> {
        let mut conn = self.pool.get()?;
        Ok(RecommendedLike::related_content_ids(&mut conn, content_id)?)
    }

    /// Inserts a new record into the `recommendLike` table.
    pub fn insert_recommend_like(
        &self,
        recommend_like: RecommendedLike,
    ) -> Result<RecommendedLike> {
        let mut conn = self.pool.get()?;
        Ok(recommend_like.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `recommendLike` table.
    pub fn create_recommend_like(
        &self,
        content_id_1: i32,
        content_id_2: i32,
        rating: Option<i32>,
    ) -> Result<RecommendedLike> {
        let recommend_like =
            RecommendedLike::new(content_id_1, content_id_2, rating.unwrap_or_default());
        self.insert_recommend_like(recommend_like)
    }

    /// Updates an existing record in the `recommendLike` table.
    pub fn update_recommend_like(
        &self,
        recommend_like: RecommendedLike,
    ) -> Result<RecommendedLike> {
        let mut conn = self.pool.get()?;
        Ok(recommend_like.update(&mut conn)?)
    }

    /// Deletes a record from the `recommendLike` table by its primary key.
    pub fn delete_recommend_like(&self, content_id_1: i32, content_id_2: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(RecommendedLike::delete(
            &mut conn,
            &(content_id_1, content_id_2),
        )?)
    }

    /// Deletes a record from the `recommendLike` table by its primary key.
    pub fn delete_recommend_like_by_content(&self, content_id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(RecommendedLike::delete_by_content_id(
            &mut conn, content_id,
        )?)
    }

    // -- Sort -------------------------------------------------------------------------------------

    /// Queries all records from the `sort` table.
    pub fn get_sorts(&self) -> Result<Vec<Sort>> {
        let mut conn = self.pool.get()?;
        Ok(Sort::all(&mut conn)?)
    }

    /// Queries a record from the `sort` table by its primary key `id`.
    pub fn get_sort_by_id(&self, id: i32) -> Result<Option<Sort>> {
        let mut conn = self.pool.get()?;
        Ok(Sort::find(&mut conn, &id)?)
    }

    /// Inserts a new record into the `sort` table.
    pub fn insert_sort(&self, sort: NewSort) -> Result<Sort> {
        let mut conn = self.pool.get()?;
        Ok(sort.insert(&mut conn)?)
    }

    /// Creates a new record and inserts it into the `sort` table.
    pub fn create_sort(
        &self,
        menu_item_id: i32,
        seq: Option<i32>,
        is_visible: Option<i32>,
    ) -> Result<Sort> {
        let sort = NewSort {
            menu_item_id,
            seq: seq.unwrap_or_default(),
            is_visible: is_visible.unwrap_or_default(),
            ..Default::default()
        };
        self.insert_sort(sort)
    }

    /// Updates an existing record in the `sort` table.
    pub fn update_sort(&self, sort: Sort) -> Result<Sort> {
        let mut conn = self.pool.get()?;
        Ok(sort.update(&mut conn)?)
    }

    /// Deletes a record from the `sort` table by its primary key `id`.
    pub fn delete_sort(&self, id: i32) -> Result<usize> {
        let mut conn = self.pool.get()?;
        Ok(Sort::delete(&mut conn, &id)?)
    }
}
