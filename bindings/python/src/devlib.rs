// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Python Device Library lifecycle and model operations.

use std::path::PathBuf;

use pyo3::prelude::*;
use pyo3::types::PyType;
use rbox::core::{PlaylistType, Track};
use rbox::devlib::models::playlist::DevPlaylistTreeNode;
use rbox::devlib::models::*;
use rbox::devlib::{DeviceLibrary, LibraryCreateOptions};

use super::errors::DeviceLibraryError;

fn db_error(error: impl std::fmt::Display) -> PyErr {
    DeviceLibraryError::new_err(error.to_string())
}

/// Metadata for creating both export.pdb and exportExt.pdb.
#[derive(Clone)]
#[pyclass(name = "LibraryCreateOptions", from_py_object, get_all, set_all)]
pub struct PyLibraryCreateOptions {
    /// Export date, conventionally YYYY-MM-DD.
    pub date: String,
    /// Device name stored in the export properties.
    pub device_name: String,
    /// Source library identifier stored in the My Tag properties.
    pub my_tag_master_db_id: i64,
}

#[pymethods]
impl PyLibraryCreateOptions {
    #[new]
    pub fn new(date: String, device_name: String, my_tag_master_db_id: i64) -> Self {
        Self {
            date,
            device_name,
            my_tag_master_db_id,
        }
    }
}

/// Per-file write counts, represented without narrowing as Python integers.
#[derive(Clone)]
#[pyclass(name = "DevSaveStats", from_py_object, frozen, get_all)]
pub struct PyDevSaveStats {
    pub pages_written: u64,
    pub bytes_written: u64,
}

/// Results of saving the standard file and optional extended file.
#[derive(Clone)]
#[pyclass(name = "LibrarySaveStats", from_py_object, frozen, get_all)]
pub struct PyLibrarySaveStats {
    pub standard: PyDevSaveStats,
    pub extended: Option<PyDevSaveStats>,
}

/// Device Library backed by export.pdb and optionally exportExt.pdb.
/// Edits remain in memory until save/save_pages; save_as copies without switching files.
/// Saves run standard-then-extended and are not atomic across the pair.
#[pyclass(unsendable)]
pub struct PyDeviceLibrary {
    inner: DeviceLibrary,
}

#[pymethods]
impl PyDeviceLibrary {
    /// Open the database directory, usually PIONEER/rekordbox (not the USB root).
    #[new]
    pub fn new(directory: PathBuf) -> PyResult<Self> {
        Ok(Self {
            inner: DeviceLibrary::new(directory).map_err(db_error)?,
        })
    }

    /// Open explicit paths. Omit extended to open only the standard database.
    #[classmethod]
    #[pyo3(signature = (standard, extended=None))]
    pub fn from_paths(
        _cls: &Bound<'_, PyType>,
        standard: PathBuf,
        extended: Option<PathBuf>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: DeviceLibrary::from_paths(standard, extended.as_deref()).map_err(db_error)?,
        })
    }

    /// Create both files without overwriting existing export files.
    #[classmethod]
    pub fn create(
        _cls: &Bound<'_, PyType>,
        directory: PathBuf,
        options: PyLibraryCreateOptions,
    ) -> PyResult<Self> {
        let options = LibraryCreateOptions::new(
            options.date,
            options.device_name,
            options.my_tag_master_db_id,
        );
        Ok(Self {
            inner: DeviceLibrary::create(directory, &options).map_err(db_error)?,
        })
    }

    /// Whether the extended database was opened.
    pub fn has_extended(&self) -> bool {
        self.inner.has_extended()
    }

    /// Replace each changed file atomically; the pair is not atomic. Unchanged files write nothing.
    pub fn save(&mut self) -> PyResult<PyLibrarySaveStats> {
        let stats = self.inner.save().map_err(db_error)?;
        Ok(PyLibrarySaveStats {
            standard: PyDevSaveStats {
                pages_written: stats.standard.pages_written,
                bytes_written: stats.standard.bytes_written,
            },
            extended: stats.extended.map(|s| PyDevSaveStats {
                pages_written: s.pages_written,
                bytes_written: s.bytes_written,
            }),
        })
    }

    /// Write changed pages directly, non-atomically. After failure, discard the handler
    /// and reopen only after restoring or repairing the affected file.
    pub fn save_pages(&mut self) -> PyResult<PyLibrarySaveStats> {
        let stats = self.inner.save_pages().map_err(db_error)?;
        Ok(PyLibrarySaveStats {
            standard: PyDevSaveStats {
                pages_written: stats.standard.pages_written,
                bytes_written: stats.standard.bytes_written,
            },
            extended: stats.extended.map(|s| PyDevSaveStats {
                pages_written: s.pages_written,
                bytes_written: s.bytes_written,
            }),
        })
    }

    /// Copy database images without overwriting. Keeps source files and pending changes.
    /// Does not copy audio, artwork, or analysis assets.
    pub fn save_as(&mut self, directory: PathBuf) -> PyResult<()> {
        self.inner.save_as(directory).map_err(db_error)
    }

    /// Load all albums in physical order without resolving related rows.
    pub fn get_albums(&mut self) -> PyResult<Vec<DevAlbum>> {
        self.inner.get_albums().map_err(db_error)
    }

    /// Find an album by export ID; returns `None` when absent.
    pub fn get_album_by_id(&mut self, id: i64) -> PyResult<Option<DevAlbum>> {
        self.inner.get_album_by_id(id).map_err(db_error)
    }

    /// Find the first exact, case-sensitive name match in physical order.
    ///
    /// Albums with different artists can share a name; this does not filter by artist.
    pub fn get_album_by_name(&mut self, name: &str) -> PyResult<Option<DevAlbum>> {
        self.inner.get_album_by_name(name).map_err(db_error)
    }

    /// Load an album and its album artist. Missing albums return `None`.
    ///
    /// Missing or unassigned artists remain `None` in the detached view.
    pub fn get_album_with_relations(&mut self, id: i64) -> PyResult<Option<DevAlbumView>> {
        self.inner.get_album_with_relations(id).map_err(db_error)
    }

    /// Load all albums in physical order and resolve shared album artists in a batch.
    pub fn get_albums_with_relations(&mut self) -> PyResult<Vec<DevAlbumView>> {
        self.inner.get_albums_with_relations().map_err(db_error)
    }

    /// Allocate an album ID and insert it, enforcing unique `(name, artist_id)` pairs.
    ///
    /// Changes remain in memory until explicitly saved.
    pub fn insert_album(&mut self, album: NewDevAlbum) -> PyResult<DevAlbum> {
        self.inner.insert_album(album).map_err(db_error)
    }

    /// Create an album with an automatically allocated export ID and insert it in memory.
    ///
    /// `artist_id` is the album artist; `None` or zero means unassigned.
    #[pyo3(signature = (name, artist_id=None))]
    pub fn create_album(&mut self, name: String, artist_id: Option<i64>) -> PyResult<DevAlbum> {
        self.inner.create_album(name, artist_id).map_err(db_error)
    }

    /// Persist edited album fields in memory and return a refreshed snapshot.
    /// Preserves unedited binary data and enforces the model's uniqueness checks.
    pub fn update_album(&mut self, album: DevAlbum) -> PyResult<DevAlbum> {
        self.inner.update_album(album).map_err(db_error)
    }

    /// Delete an album and clear referencing contents' album IDs atomically in memory.
    /// Returns the total affected rows, or zero for a missing album. Saving is explicit.
    pub fn delete_album(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_album(id).map_err(db_error)
    }

    /// Load all artists in physical order without resolving related rows.
    pub fn get_artists(&mut self) -> PyResult<Vec<DevArtist>> {
        self.inner.get_artists().map_err(db_error)
    }

    /// Find an artist by export ID; returns `None` when absent.
    pub fn get_artist_by_id(&mut self, id: i64) -> PyResult<Option<DevArtist>> {
        self.inner.get_artist_by_id(id).map_err(db_error)
    }

    /// Find the first exact, case-sensitive name match; returns `None` when absent.
    pub fn get_artist_by_name(&mut self, name: &str) -> PyResult<Option<DevArtist>> {
        self.inner.get_artist_by_name(name).map_err(db_error)
    }

    /// Allocate an artist ID and insert it, enforcing unique, case-sensitive names.
    ///
    /// Changes remain in memory until explicitly saved.
    pub fn insert_artist(&mut self, artist: NewDevArtist) -> PyResult<DevArtist> {
        self.inner.insert_artist(artist).map_err(db_error)
    }

    /// Create an artist with an automatically allocated export ID and insert it in memory.
    pub fn create_artist(&mut self, name: String) -> PyResult<DevArtist> {
        self.inner.create_artist(name).map_err(db_error)
    }

    /// Persist an edited artist name in memory and return a refreshed snapshot.
    ///
    /// Preserves unedited binary data and enforces name uniqueness; saving is explicit.
    pub fn update_artist(&mut self, artist: DevArtist) -> PyResult<DevArtist> {
        self.inner.update_artist(artist).map_err(db_error)
    }

    /// Delete an artist and clear its album and content references atomically in memory.
    ///
    /// Clears album artist, main artist, composer, original artist, and remixer IDs.
    /// Returns the total affected rows, or zero for a missing artist. Saving is explicit.
    pub fn delete_artist(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_artist(id).map_err(db_error)
    }

    /// Load all artwork references in physical order without resolving related rows.
    pub fn get_artworks(&mut self) -> PyResult<Vec<DevArtwork>> {
        self.inner.get_artworks().map_err(db_error)
    }

    /// Find an artwork reference by export ID; returns `None` when absent.
    pub fn get_artwork_by_id(&mut self, id: i64) -> PyResult<Option<DevArtwork>> {
        self.inner.get_artwork_by_id(id).map_err(db_error)
    }

    /// Find the first exact, case-sensitive path match; returns `None` when absent.
    ///
    /// Paths are compared as stored, without normalization or filesystem access.
    pub fn get_artwork_by_path(&mut self, path: &str) -> PyResult<Option<DevArtwork>> {
        self.inner.get_artwork_by_path(path).map_err(db_error)
    }

    /// Allocate an artwork ID and insert a reference to a nonempty image path.
    ///
    /// Duplicate paths are allowed. Changes remain in memory; no image file is copied.
    pub fn insert_artwork(&mut self, artwork: NewDevArtwork) -> PyResult<DevArtwork> {
        self.inner.insert_artwork(artwork).map_err(db_error)
    }

    /// Create an artwork reference with an automatically allocated ID and insert it in memory.
    pub fn create_artwork(&mut self, path: String) -> PyResult<DevArtwork> {
        self.inner.create_artwork(path).map_err(db_error)
    }

    /// Persist an edited artwork path in memory and return a refreshed snapshot.
    ///
    /// Saving is explicit; this does not copy or rename image files.
    pub fn update_artwork(&mut self, artwork: DevArtwork) -> PyResult<DevArtwork> {
        self.inner.update_artwork(artwork).map_err(db_error)
    }

    /// Delete an artwork reference and clear matching content artwork IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero when absent. Image files are retained;
    /// saving database changes remains explicit.
    pub fn delete_artwork(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_artwork(id).map_err(db_error)
    }

    /// Load all predefined browse labels in physical order without resolving settings.
    pub fn get_browse_columns(&mut self) -> PyResult<Vec<DevBrowseColumn>> {
        self.inner.get_browse_columns().map_err(db_error)
    }

    /// Find a browse label by its 16-bit catalog ID; returns `None` when absent.
    pub fn get_browse_column_by_id(&mut self, id: i64) -> PyResult<Option<DevBrowseColumn>> {
        self.inner.get_browse_column_by_id(id).map_err(db_error)
    }

    /// Find the first exact, case-sensitive plain-text label match, or `None`.
    ///
    /// Enclosing U+FFFA/U+FFFB markers are omitted from the compared label.
    pub fn get_browse_column_by_name(&mut self, name: &str) -> PyResult<Option<DevBrowseColumn>> {
        self.inner.get_browse_column_by_name(name).map_err(db_error)
    }

    /// Update a browse label in memory, preserving its enclosing markers.
    ///
    /// Catalog IDs and codes cannot change; saving remains explicit.
    pub fn update_browse_column(&mut self, column: DevBrowseColumn) -> PyResult<DevBrowseColumn> {
        self.inner.update_browse_column(column).map_err(db_error)
    }

    /// Load all category-menu settings in physical order, retaining repeated rows.
    pub fn get_browse_categories(&mut self) -> PyResult<Vec<DevBrowseCategory>> {
        self.inner.get_browse_categories().map_err(db_error)
    }

    /// Load all category settings for a column ID, retaining physical order and repetitions.
    ///
    /// Returns an empty list when absent; referenced labels are not loaded.
    pub fn get_browse_categories_by_column(
        &mut self,
        column_id: i64,
    ) -> PyResult<Vec<DevBrowseCategory>> {
        self.inner
            .get_browse_categories_by_column(column_id)
            .map_err(db_error)
    }

    /// Update one category-setting occurrence in memory; saving remains explicit.
    ///
    /// Column and selector cannot change. No automatic reordering or state coercion
    /// is performed. Stale snapshots are rejected; reload after rebuilding the database.
    pub fn update_browse_category(
        &mut self,
        category: DevBrowseCategory,
    ) -> PyResult<DevBrowseCategory> {
        self.inner
            .update_browse_category(category)
            .map_err(db_error)
    }

    /// Load all sort-option settings in physical order, retaining repeated rows.
    pub fn get_browse_sorts(&mut self) -> PyResult<Vec<DevBrowseSort>> {
        self.inner.get_browse_sorts().map_err(db_error)
    }

    /// Load all sort settings for a column ID, retaining physical order and repetitions.
    ///
    /// Returns an empty list when absent; referenced labels are not loaded.
    pub fn get_browse_sorts_by_column(&mut self, column_id: i64) -> PyResult<Vec<DevBrowseSort>> {
        self.inner
            .get_browse_sorts_by_column(column_id)
            .map_err(db_error)
    }

    /// Update one sort-setting occurrence's raw tail in memory; saving remains explicit.
    ///
    /// Column and selector cannot change. Stale snapshots are rejected; reload after
    /// rebuilding. Tail-bit meanings remain provisional; see `crate::models::browse`.
    pub fn update_browse_sort(&mut self, sort: DevBrowseSort) -> PyResult<DevBrowseSort> {
        self.inner.update_browse_sort(sort).map_err(db_error)
    }

    /// Load all initialized color labels in physical order without resolving related rows.
    pub fn get_colors(&mut self) -> PyResult<Vec<DevColor>> {
        self.inner.get_colors().map_err(db_error)
    }

    /// Find a color label by its 16-bit export ID; returns `None` when absent.
    pub fn get_color_by_id(&mut self, id: i64) -> PyResult<Option<DevColor>> {
        self.inner.get_color_by_id(id).map_err(db_error)
    }

    /// Find the first exact, case-sensitive color name match, or `None` when absent.
    pub fn get_color_by_name(&mut self, name: &str) -> PyResult<Option<DevColor>> {
        self.inner.get_color_by_name(name).map_err(db_error)
    }

    /// Update a color name in memory and return a refreshed snapshot.
    ///
    /// Names must be unique (case-sensitive); existing IDs cannot change.
    /// Colors are initialized at creation and support updates only. Saving remains explicit.
    pub fn update_color(&mut self, color: DevColor) -> PyResult<DevColor> {
        self.inner.update_color(color).map_err(db_error)
    }

    /// Load shared tracks in physical order with resolved metadata, including album artists.
    ///
    /// Each distinct related row is decoded once per call; missing metadata stays `None`.
    /// BPM is unscaled, duration is seconds, and file size is bytes. Source-library IDs
    /// come from each content row. Decoding and metadata conversion errors propagate.
    /// Results are detached snapshots; audio and analysis files are not read.
    pub fn get_tracks(&mut self) -> PyResult<Vec<Track>> {
        self.inner.get_tracks().map_err(db_error)
    }

    /// Load a shared track by its local export ID, or `None` if absent.
    ///
    /// See `Self::get_tracks` for relationship resolution, units and errors.
    pub fn get_track_by_id(&mut self, id: i64) -> PyResult<Option<Track>> {
        self.inner.get_track_by_id(id).map_err(db_error)
    }

    /// Load matching shared tracks in ascending local export-ID order.
    ///
    /// Missing IDs are omitted, repeated IDs occur once, and empty input reads no pages.
    /// See `Self::get_tracks` for relationship resolution, units and errors.
    pub fn get_tracks_by_ids(&mut self, ids: Vec<i64>) -> PyResult<Vec<Track>> {
        self.inner.get_tracks_by_ids(&ids).map_err(db_error)
    }

    /// Load a shared track by its stored audio path, or `None` if absent.
    ///
    /// Backslashes are converted to forward slashes without accessing the file;
    /// the path must otherwise match exactly. Repeated paths return the first
    /// physical occurrence. See `Self::get_tracks` for conversion errors.
    pub fn get_track_by_path(&mut self, path: &str) -> PyResult<Option<Track>> {
        self.inner.get_track_by_path(path).map_err(db_error)
    }

    /// Load all content rows in physical order without resolving related metadata.
    pub fn get_contents(&mut self) -> PyResult<Vec<DevContent>> {
        self.inner.get_contents().map_err(db_error)
    }

    /// Find content by its local export ID; returns `None` when absent.
    pub fn get_content_by_id(&mut self, id: i64) -> PyResult<Option<DevContent>> {
        self.inner.get_content_by_id(id).map_err(db_error)
    }

    /// Find content by both source-library IDs, rather than its local export ID.
    ///
    /// Missing or zero identities return `None`. Repeated pairs return the row
    /// with the lowest local export ID.
    pub fn get_content_by_master_content_id(
        &mut self,
        master_content_id: i64,
        master_db_id: i64,
    ) -> PyResult<Option<DevContent>> {
        self.inner
            .get_content_by_master_content_id(master_content_id, master_db_id)
            .map_err(db_error)
    }

    /// Load contents by source-content IDs within one source library.
    ///
    /// Returns all matches in local export-ID order, omitting missing/zero IDs and
    /// repeated input IDs. Repeated source identities retain all matching content rows.
    pub fn get_contents_by_master_content_ids(
        &mut self,
        master_content_ids: Vec<i64>,
        master_db_id: i64,
    ) -> PyResult<Vec<DevContent>> {
        self.inner
            .get_contents_by_master_content_ids(&master_content_ids, master_db_id)
            .map_err(db_error)
    }

    /// Load content and its metadata relationships, including its album artist.
    ///
    /// Missing content returns `None`; missing relationship targets remain `None`.
    pub fn get_content_with_relations(&mut self, id: i64) -> PyResult<Option<DevContentView>> {
        self.inner.get_content_with_relations(id).map_err(db_error)
    }

    /// Load all content in physical order and resolve distinct metadata targets once.
    ///
    /// Returns detached snapshots without loading playlists, history, or My Tags.
    pub fn get_contents_with_relations(&mut self) -> PyResult<Vec<DevContentView>> {
        self.inner.get_contents_with_relations().map_err(db_error)
    }

    /// Allocate a local export ID and insert content, updating existing export-property counts.
    ///
    /// The operation commits atomically in memory; saving remains explicit.
    /// Audio, artwork and analysis files are not copied or generated.
    pub fn insert_content(&mut self, content: NewDevContent) -> PyResult<DevContent> {
        self.inner.insert_content(content).map_err(db_error)
    }

    /// Create content from a nonempty audio path with forward slashes and insert it in memory.
    ///
    /// The file name is derived from the path; other metadata uses `NewDevContent` defaults.
    pub fn create_content(&mut self, path: String) -> PyResult<DevContent> {
        self.inner.create_content(path).map_err(db_error)
    }

    /// Persist edited content in memory, preserving unedited binary fields.
    ///
    /// The ID cannot change. Errors roll back the operation; saving remains explicit.
    /// Editing paths does not move files or automatically update other path fields.
    pub fn update_content(&mut self, content: DevContent) -> PyResult<DevContent> {
        self.inner.update_content(content).map_err(db_error)
    }

    /// Delete content and its playlist/history occurrences, updating export-property counts.
    ///
    /// When the extended file is open, also delete its My Tag associations and
    /// recommendation pairs containing this ID on either side. Unknown tables remain intact.
    /// Returns deleted row count, including associations but excluding property updates;
    /// a missing ID returns zero. ID zero is rejected. Changes commit atomically in memory;
    /// saving the file pair remains explicit and is not atomic. Resource files are retained.
    pub fn delete_content(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_content(id).map_err(db_error)
    }

    /// Load all genres in physical order without resolving related rows.
    pub fn get_genres(&mut self) -> PyResult<Vec<DevGenre>> {
        self.inner.get_genres().map_err(db_error)
    }

    /// Find a genre by export ID; returns `None` when absent.
    pub fn get_genre_by_id(&mut self, id: i64) -> PyResult<Option<DevGenre>> {
        self.inner.get_genre_by_id(id).map_err(db_error)
    }

    /// Find the first exact, case-sensitive name match; returns `None` when absent.
    pub fn get_genre_by_name(&mut self, name: &str) -> PyResult<Option<DevGenre>> {
        self.inner.get_genre_by_name(name).map_err(db_error)
    }

    /// Allocate a genre ID and insert it, enforcing unique, case-sensitive names.
    ///
    /// Changes remain in memory until explicitly saved.
    pub fn insert_genre(&mut self, genre: NewDevGenre) -> PyResult<DevGenre> {
        self.inner.insert_genre(genre).map_err(db_error)
    }

    /// Create a genre with an automatically allocated export ID and insert it in memory.
    pub fn create_genre(&mut self, name: String) -> PyResult<DevGenre> {
        self.inner.create_genre(name).map_err(db_error)
    }

    /// Persist an edited genre name in memory and return a refreshed snapshot.
    ///
    /// The ID cannot change; names must remain unique. Saving is explicit.
    pub fn update_genre(&mut self, genre: DevGenre) -> PyResult<DevGenre> {
        self.inner.update_genre(genre).map_err(db_error)
    }

    /// Delete a genre and clear referencing contents' genre IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero for a missing genre. ID zero is rejected.
    /// Contents and export track counts are retained; saving remains explicit.
    pub fn delete_genre(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_genre(id).map_err(db_error)
    }

    /// Load all histories in physical order without loading their occurrences.
    ///
    /// Device Library histories have no folder hierarchy or type attribute.
    pub fn get_histories(&mut self) -> PyResult<Vec<DevHistory>> {
        self.inner.get_histories().map_err(db_error)
    }

    /// Find a history by export ID; returns `None` when absent.
    pub fn get_history_by_id(&mut self, id: i64) -> PyResult<Option<DevHistory>> {
        self.inner.get_history_by_id(id).map_err(db_error)
    }

    /// Load a history's contents in stored sequence order, retaining repeated plays.
    ///
    /// Ties retain physical occurrence order; missing content references return errors.
    /// No entries returns an empty list. Content metadata relationships are not resolved.
    pub fn get_history_contents(&mut self, id: i64) -> PyResult<Vec<DevContent>> {
        self.inner.get_history_contents(id).map_err(db_error)
    }

    /// Load occurrence snapshots in stored sequence order without resolving contents.
    ///
    /// Repeated plays, sequence gaps and ties are retained. These snapshots can be
    /// passed to `Self::delete_history_content` to delete individual occurrences.
    pub fn get_history_entries(&mut self, id: i64) -> PyResult<Vec<DevHistoryContent>> {
        self.inner.get_history_entries(id).map_err(db_error)
    }

    /// Allocate a history ID and insert its name in memory; saving remains explicit.
    pub fn insert_history(&mut self, history: NewDevHistory) -> PyResult<DevHistory> {
        self.inner.insert_history(history).map_err(db_error)
    }

    /// Create a named history with an automatically allocated export ID.
    ///
    /// Changes remain in memory until explicitly saved.
    pub fn create_history(&mut self, name: String) -> PyResult<DevHistory> {
        self.inner.create_history(name).map_err(db_error)
    }

    /// Update a history name in memory and return a refreshed snapshot.
    /// The ID cannot change; saving remains explicit.
    pub fn update_history(&mut self, history: DevHistory) -> PyResult<DevHistory> {
        self.inner.update_history(history).map_err(db_error)
    }

    /// Delete a history and its occurrences atomically in memory, retaining contents.
    ///
    /// Returns the total deleted rows, or zero for a missing history. ID zero is rejected.
    /// Other histories and playlists are unchanged; saving remains explicit.
    pub fn delete_history(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_history(id).map_err(db_error)
    }

    /// Insert a history occurrence in memory, retaining its explicit stored position.
    ///
    /// Repeated tracks and position ties are allowed. Reference IDs must be nonzero;
    /// target existence is not checked. Saving remains explicit.
    pub fn insert_history_content(
        &mut self,
        entry: NewDevHistoryContent,
    ) -> PyResult<DevHistoryContent> {
        self.inner.insert_history_content(entry).map_err(db_error)
    }

    /// Create an occurrence at an explicit stored position without shifting other entries.
    ///
    /// See `Self::insert_history_content` for validation and persistence semantics.
    pub fn create_history_content(
        &mut self,
        history_id: i64,
        content_id: i64,
        seq: u32,
    ) -> PyResult<DevHistoryContent> {
        self.inner
            .create_history_content(history_id, content_id, seq)
            .map_err(db_error)
    }

    /// Delete one occurrence snapshot, retaining other plays, contents and sequence gaps.
    ///
    /// Returns one on success; stale snapshots return an error. Re-fetch entries after
    /// structural edits. Changes remain in memory until explicitly saved.
    pub fn delete_history_content(&mut self, entry: DevHistoryContent) -> PyResult<usize> {
        self.inner.delete_history_content(entry).map_err(db_error)
    }

    /// Load all musical keys in physical order without resolving related rows.
    pub fn get_keys(&mut self) -> PyResult<Vec<DevKey>> {
        self.inner.get_keys().map_err(db_error)
    }

    /// Find a key by export ID; returns `None` when absent.
    pub fn get_key_by_id(&mut self, id: i64) -> PyResult<Option<DevKey>> {
        self.inner.get_key_by_id(id).map_err(db_error)
    }

    /// Find the first exact, case-sensitive name match; returns `None` when absent.
    pub fn get_key_by_name(&mut self, name: &str) -> PyResult<Option<DevKey>> {
        self.inner.get_key_by_name(name).map_err(db_error)
    }

    /// Allocate an export key ID and insert it, enforcing unique, case-sensitive names.
    ///
    /// The supplied secondary ID is retained independently; saving remains explicit.
    pub fn insert_key(&mut self, key: NewDevKey) -> PyResult<DevKey> {
        self.inner.insert_key(key).map_err(db_error)
    }

    /// Create a key with an automatically allocated export ID and an explicit secondary ID.
    ///
    /// The secondary ID's role remains unknown; no musical-key numbering is inferred.
    /// Changes remain in memory until explicitly saved.
    pub fn create_key(&mut self, name: String, secondary_id: i64) -> PyResult<DevKey> {
        self.inner.create_key(name, secondary_id).map_err(db_error)
    }

    /// Persist an edited key name or secondary ID in memory and return a refreshed snapshot.
    ///
    /// The export ID cannot change; names must remain unique. Saving is explicit.
    pub fn update_key(&mut self, key: DevKey) -> PyResult<DevKey> {
        self.inner.update_key(key).map_err(db_error)
    }

    /// Delete a key and clear referencing contents' key IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero for a missing key. ID zero is rejected.
    /// Contents and export track counts are retained; saving remains explicit.
    pub fn delete_key(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_key(id).map_err(db_error)
    }

    /// Load all labels in physical order without resolving related rows.
    pub fn get_labels(&mut self) -> PyResult<Vec<DevLabel>> {
        self.inner.get_labels().map_err(db_error)
    }

    /// Find a label by export ID; returns `None` when absent.
    pub fn get_label_by_id(&mut self, id: i64) -> PyResult<Option<DevLabel>> {
        self.inner.get_label_by_id(id).map_err(db_error)
    }

    /// Find the first exact, case-sensitive name match; returns `None` when absent.
    pub fn get_label_by_name(&mut self, name: &str) -> PyResult<Option<DevLabel>> {
        self.inner.get_label_by_name(name).map_err(db_error)
    }

    /// Allocate a label ID and insert it, enforcing unique, case-sensitive names.
    /// Changes remain in memory until explicitly saved.
    pub fn insert_label(&mut self, label: NewDevLabel) -> PyResult<DevLabel> {
        self.inner.insert_label(label).map_err(db_error)
    }

    /// Create a label with an automatically allocated export ID and insert it in memory.
    pub fn create_label(&mut self, name: String) -> PyResult<DevLabel> {
        self.inner.create_label(name).map_err(db_error)
    }

    /// Persist an edited label name in memory and return a refreshed snapshot.
    ///
    /// The ID cannot change; names must remain unique. Saving is explicit.
    pub fn update_label(&mut self, label: DevLabel) -> PyResult<DevLabel> {
        self.inner.update_label(label).map_err(db_error)
    }

    /// Delete a label and clear referencing contents' label IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero for a missing label. ID zero is rejected.
    /// Contents and export track counts are retained; saving remains explicit.
    pub fn delete_label(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_label(id).map_err(db_error)
    }

    /// Load all tags/categories in physical order without resolving relations.
    ///
    /// All My Tag methods require the extended file; otherwise they return
    /// `Error::MissingExtendedDatabase`. Mutations remain in memory until saved.
    pub fn get_my_tags(&mut self) -> PyResult<Vec<DevMyTag>> {
        self.inner.get_my_tags().map_err(db_error)
    }

    /// Find a tag/category by its source-library ID; missing IDs return `None`.
    pub fn get_my_tag_by_id(&mut self, id: i64) -> PyResult<Option<DevMyTag>> {
        self.inner.get_my_tag_by_id(id).map_err(db_error)
    }

    /// Load immediate children in stored sequence order; zero selects roots.
    pub fn get_my_tag_children(&mut self, id: i64) -> PyResult<Vec<DevMyTag>> {
        self.inner.get_my_tag_children(id).map_err(db_error)
    }

    /// Load tags for a local content ID, retaining physical association order and repetitions.
    ///
    /// Missing tag targets return errors. The standard content row is not loaded.
    pub fn get_my_tag_by_content_id(&mut self, content_id: i64) -> PyResult<Vec<DevMyTag>> {
        self.inner
            .get_my_tag_by_content_id(content_id)
            .map_err(db_error)
    }

    /// Load a tag's contents across both files in physical association order, retaining repeats.
    ///
    /// Missing content targets return errors. Metadata relationships are not resolved.
    pub fn get_my_tag_contents(&mut self, id: i64) -> PyResult<Vec<DevContent>> {
        self.inner.get_my_tag_contents(id).map_err(db_error)
    }

    /// Insert a tag/category, validating ancestry and maintaining sibling order.
    ///
    /// IDs must be supplied explicitly and should match the source `master.db`.
    pub fn insert_my_tag(&mut self, my_tag: NewDevMyTag) -> PyResult<DevMyTag> {
        self.inner.insert_my_tag(my_tag).map_err(db_error)
    }

    /// Create a tag with its source `master.db` ID. `None` selects the root or appends.
    ///
    /// Explicit positions start at `DevMyTag::START_SEQ`. Saving remains explicit.
    #[pyo3(signature = (id, name, parent_id=None, seq=None))]
    pub fn create_my_tag(
        &mut self,
        id: i64,
        name: String,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> PyResult<DevMyTag> {
        self.inner
            .create_my_tag(id, name, parent_id, seq)
            .map_err(db_error)
    }

    /// Create a category with its source `master.db` ID. `None` selects the root or appends.
    ///
    /// Explicit positions start at `DevMyTag::START_SEQ`. Saving remains explicit.
    #[pyo3(signature = (id, name, parent_id=None, seq=None))]
    pub fn create_my_tag_folder(
        &mut self,
        id: i64,
        name: String,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> PyResult<DevMyTag> {
        self.inner
            .create_my_tag_folder(id, name, parent_id, seq)
            .map_err(db_error)
    }

    /// Update name, parent or position, validating ancestry and maintaining sibling order.
    ///
    /// IDs and attributes cannot change. Changes commit atomically in memory.
    pub fn update_my_tag(&mut self, my_tag: DevMyTag) -> PyResult<DevMyTag> {
        self.inner.update_my_tag(my_tag).map_err(db_error)
    }

    /// Rename a tag/category, returning one on success or zero for a missing ID.
    pub fn rename_my_tag(&mut self, id: i64, name: &str) -> PyResult<usize> {
        self.inner.rename_my_tag(id, name).map_err(db_error)
    }

    /// Count immediate children without decoding names; zero selects the root.
    pub fn count_my_tag_children(&mut self, parent_id: i64) -> PyResult<usize> {
        self.inner
            .count_my_tag_children(parent_id)
            .map_err(db_error)
    }

    /// Read sibling positions in ascending order, retaining imported gaps and ties.
    pub fn get_my_tag_seq_numbers(&mut self, parent_id: i64) -> PyResult<Vec<u32>> {
        self.inner
            .get_my_tag_seq_numbers(parent_id)
            .map_err(db_error)
    }

    /// Check contiguous sibling positions starting at `DevMyTag::START_SEQ`.
    pub fn validate_my_tag_seq_numbers(&mut self, parent_id: i64) -> PyResult<bool> {
        self.inner
            .validate_my_tag_seq_numbers(parent_id)
            .map_err(db_error)
    }

    /// Normalize sibling positions in stable sequence order, returning changed rows.
    ///
    /// The parent must be a category with valid ancestry, or zero for the root.
    pub fn reset_my_tag_seq(&mut self, parent_id: i64) -> PyResult<usize> {
        self.inner.reset_my_tag_seq(parent_id).map_err(db_error)
    }

    /// Move one node, maintaining sibling order and rejecting cycles or invalid parents.
    ///
    /// `None` keeps the parent or appends. Returns one if the selected node moved, else zero.
    #[pyo3(signature = (id, parent_id=None, seq=None))]
    pub fn move_my_tag(
        &mut self,
        id: i64,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> PyResult<usize> {
        self.inner
            .move_my_tag(id, parent_id.as_ref(), seq)
            .map_err(db_error)
    }

    /// Move distinct nodes as a block in caller order, atomically; `None` appends.
    ///
    /// Returns changed selected nodes, excluding shifts of other siblings.
    #[pyo3(signature = (ids, parent_id, start_seq=None))]
    pub fn move_my_tags(
        &mut self,
        ids: Vec<i64>,
        parent_id: i64,
        start_seq: Option<u32>,
    ) -> PyResult<usize> {
        self.inner
            .move_my_tags(&ids, parent_id, start_seq)
            .map_err(db_error)
    }

    /// Delete a tag/category subtree and its associations, normalizing surviving siblings.
    ///
    /// Returns deleted rows; missing IDs return zero. Contents remain intact.
    pub fn delete_my_tag(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_my_tag(id).map_err(db_error)
    }

    /// Delete selected subtrees atomically, counting overlapping descendants once.
    ///
    /// Duplicate or missing input IDs reject the batch. Returns deleted nodes and associations.
    pub fn delete_my_tags(&mut self, ids: Vec<i64>) -> PyResult<usize> {
        self.inner.delete_my_tags(&ids).map_err(db_error)
    }

    /// Load all raw tag/content associations in physical order, retaining duplicates.
    pub fn get_my_tag_associations(&mut self) -> PyResult<Vec<DevMyTagContent>> {
        self.inner.get_my_tag_associations().map_err(db_error)
    }

    /// Load a tag's association snapshots without resolving contents or deduplicating pairs.
    pub fn get_my_tag_entries(&mut self, id: i64) -> PyResult<Vec<DevMyTagContent>> {
        self.inner.get_my_tag_entries(id).map_err(db_error)
    }

    /// Load a content's association snapshots without resolving tags or deduplicating pairs.
    pub fn get_my_tag_associations_by_content_id(
        &mut self,
        content_id: i64,
    ) -> PyResult<Vec<DevMyTagContent>> {
        self.inner
            .get_my_tag_associations_by_content_id(content_id)
            .map_err(db_error)
    }

    /// Insert an association with explicit reserved values. IDs must be nonzero.
    ///
    /// target existence is not checked and repeated pairs are allowed.
    pub fn insert_my_tag_content(
        &mut self,
        entry: NewDevMyTagContent,
    ) -> PyResult<DevMyTagContent> {
        self.inner.insert_my_tag_content(entry).map_err(db_error)
    }

    /// Associate a source-library tag ID with a local export content ID using model defaults.
    ///
    /// Repeated pairs are allowed; referenced rows are not loaded or validated.
    pub fn create_my_tag_content(
        &mut self,
        my_tag_id: i64,
        content_id: i64,
    ) -> PyResult<DevMyTagContent> {
        self.inner
            .create_my_tag_content(my_tag_id, content_id)
            .map_err(db_error)
    }

    /// Delete all occurrences of a tag/content pair, returning the deleted count.
    /// Both IDs must be nonzero. Tags and contents remain intact.
    pub fn delete_my_tag_content(&mut self, my_tag_id: i64, content_id: i64) -> PyResult<usize> {
        self.inner
            .delete_my_tag_content(my_tag_id, content_id)
            .map_err(db_error)
    }

    /// Delete one association snapshot, retaining duplicate occurrences and referenced rows.
    ///
    /// Returns one on success. Re-fetch after structural edits; stale snapshots return errors.
    pub fn delete_my_tag_content_entry(&mut self, entry: DevMyTagContent) -> PyResult<usize> {
        self.inner
            .delete_my_tag_content_entry(entry)
            .map_err(db_error)
    }

    /// Load all playlists and folders in physical order without resolving contents.
    ///
    /// All playlist mutations commit in memory; saving remains explicit.
    pub fn get_playlists(&mut self) -> PyResult<Vec<DevPlaylist>> {
        self.inner.get_playlists().map_err(db_error)
    }

    /// Find a playlist or folder by export ID; missing IDs return `None`.
    pub fn get_playlist_by_id(&mut self, id: i64) -> PyResult<Option<DevPlaylist>> {
        self.inner.get_playlist_by_id(id).map_err(db_error)
    }

    /// Load immediate children in sibling order; zero selects roots.
    pub fn get_playlist_children(&mut self, id: i64) -> PyResult<Vec<DevPlaylist>> {
        self.inner.get_playlist_children(id).map_err(db_error)
    }

    /// Load contents in playlist sequence order, retaining imported duplicate occurrences.
    ///
    /// Missing content references return errors; metadata relationships are not resolved.
    pub fn get_playlist_contents(&mut self, id: i64) -> PyResult<Vec<DevContent>> {
        self.inner.get_playlist_contents(id).map_err(db_error)
    }

    /// Find a node by exact path components. Empty, missing or ambiguous paths return `None`.
    ///
    /// Malformed ancestry and duplicate IDs return errors.
    pub fn get_playlist_by_path(&mut self, path: Vec<String>) -> PyResult<Option<DevPlaylist>> {
        let path: Vec<&str> = path.iter().map(String::as_str).collect();
        self.inner.get_playlist_by_path(&path).map_err(db_error)
    }

    /// Build a detached tree with children in sibling order, without loading contents.
    ///
    /// Invalid parents, cycles and duplicate IDs return errors.
    pub fn get_playlist_tree(&mut self) -> PyResult<Vec<DevPlaylistTreeNode>> {
        self.inner.get_playlist_tree().map_err(db_error)
    }

    /// Read the node type; the root sentinel (zero) is treated as a folder.
    pub fn get_playlist_attribute(&mut self, id: i64) -> PyResult<Option<PlaylistType>> {
        self.inner.get_playlist_attribute(id).map_err(db_error)
    }

    /// Check whether an existing node is a regular playlist.
    pub fn is_playlist(&mut self, id: i64) -> PyResult<bool> {
        self.inner.is_playlist(id).map_err(db_error)
    }

    /// Allocate an export ID and insert a playlist/folder, validating ancestry and sibling order.
    pub fn insert_playlist(&mut self, playlist: NewDevPlaylist) -> PyResult<DevPlaylist> {
        self.inner.insert_playlist(playlist).map_err(db_error)
    }

    /// Create a regular playlist. `None` selects the root or appends among siblings.
    ///
    /// Explicit positions start at `DevPlaylist::START_SEQ`.
    #[pyo3(signature = (name, parent_id=None, seq=None))]
    pub fn create_playlist(
        &mut self,
        name: String,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> PyResult<DevPlaylist> {
        self.inner
            .create_playlist(name, parent_id, seq)
            .map_err(db_error)
    }

    /// Create a folder. `None` selects the root or appends among siblings.
    ///
    /// Explicit positions start at `DevPlaylist::START_SEQ`.
    #[pyo3(signature = (name, parent_id=None, seq=None))]
    pub fn create_playlist_folder(
        &mut self,
        name: String,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> PyResult<DevPlaylist> {
        self.inner
            .create_playlist_folder(name, parent_id, seq)
            .map_err(db_error)
    }

    /// Update name, parent or position, validating ancestry and maintaining sibling order.
    ///
    /// IDs and attributes cannot change. Changes commit atomically in memory.
    pub fn update_playlist(&mut self, playlist: DevPlaylist) -> PyResult<DevPlaylist> {
        self.inner.update_playlist(playlist).map_err(db_error)
    }

    /// Rename a node while preserving hierarchy, position and reserved data.
    ///
    /// Returns one on success; missing IDs return `Error::ModelNotFound`.
    pub fn rename_playlist(&mut self, id: i64, name: &str) -> PyResult<usize> {
        self.inner.rename_playlist(id, name).map_err(db_error)
    }

    /// Count immediate children without decoding names; zero selects roots.
    pub fn count_playlist_children(&mut self, parent_id: i64) -> PyResult<usize> {
        self.inner
            .count_playlist_children(parent_id)
            .map_err(db_error)
    }

    /// Read sibling positions in ascending order, retaining imported gaps and ties.
    pub fn get_playlist_seq_numbers(&mut self, parent_id: i64) -> PyResult<Vec<u32>> {
        self.inner
            .get_playlist_seq_numbers(parent_id)
            .map_err(db_error)
    }

    /// Check contiguous sibling positions starting at `DevPlaylist::START_SEQ`.
    pub fn validate_playlist_seq_numbers(&mut self, parent_id: i64) -> PyResult<bool> {
        self.inner
            .validate_playlist_seq_numbers(parent_id)
            .map_err(db_error)
    }

    /// Normalize siblings in stable sequence order, returning changed rows.
    ///
    /// The parent must be a folder with valid ancestry, or zero for the root.
    pub fn reset_playlist_seq(&mut self, parent_id: i64) -> PyResult<usize> {
        self.inner.reset_playlist_seq(parent_id).map_err(db_error)
    }

    /// Move one node, rejecting cycles and invalid parents. `None` keeps the parent or appends.
    ///
    /// Positions start at `DevPlaylist::START_SEQ`. Returns one if the selected node moved, else zero.
    #[pyo3(signature = (id, parent_id=None, seq=None))]
    pub fn move_playlist(
        &mut self,
        id: i64,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> PyResult<usize> {
        self.inner
            .move_playlist(id, parent_id.as_ref(), seq)
            .map_err(db_error)
    }

    /// Move distinct nodes as a block in caller order, atomically; `None` appends.
    ///
    /// The destination is measured after removing the selection. Returns changed selected nodes.
    #[pyo3(signature = (ids, parent_id, start_seq=None))]
    pub fn move_playlists(
        &mut self,
        ids: Vec<i64>,
        parent_id: i64,
        start_seq: Option<u32>,
    ) -> PyResult<usize> {
        self.inner
            .move_playlists(&ids, parent_id, start_seq)
            .map_err(db_error)
    }

    /// Delete a playlist/folder subtree and its entries, normalizing surviving siblings.
    ///
    /// Returns deleted rows; missing IDs return zero. Contents remain intact.
    pub fn delete_playlist(&mut self, id: i64) -> PyResult<usize> {
        self.inner.delete_playlist(id).map_err(db_error)
    }

    /// Delete selected subtrees atomically, counting overlapping descendants once.
    ///
    /// Duplicate or missing input IDs reject the batch. Returns deleted nodes and entries.
    pub fn delete_playlists(&mut self, ids: Vec<i64>) -> PyResult<usize> {
        self.inner.delete_playlists(&ids).map_err(db_error)
    }

    /// Load all playlist entry snapshots in physical order, retaining imported duplicates.
    pub fn get_playlist_associations(&mut self) -> PyResult<Vec<DevPlaylistContent>> {
        self.inner.get_playlist_associations().map_err(db_error)
    }

    /// Load a playlist's entry snapshots in stored sequence order without resolving contents.
    pub fn get_playlist_entries(&mut self, id: i64) -> PyResult<Vec<DevPlaylistContent>> {
        self.inner.get_playlist_entries(id).map_err(db_error)
    }

    /// Load a content's entry snapshots ordered by playlist ID and sequence.
    pub fn get_playlist_associations_by_content_id(
        &mut self,
        content_id: i64,
    ) -> PyResult<Vec<DevPlaylistContent>> {
        self.inner
            .get_playlist_associations_by_content_id(content_id)
            .map_err(db_error)
    }

    /// Find a unique playlist/content pair; missing pairs return `None`, duplicates return errors.
    pub fn get_playlist_content_by_id(
        &mut self,
        playlist_id: i64,
        content_id: i64,
    ) -> PyResult<Option<DevPlaylistContent>> {
        self.inner
            .get_playlist_content_by_id(playlist_id, content_id)
            .map_err(db_error)
    }

    /// Insert a unique playlist/content pair, normalizing affected positions.
    ///
    /// The destination must be a regular playlist; positions start at `DevPlaylistContent::START_SEQ`.
    pub fn insert_playlist_content(
        &mut self,
        entry: NewDevPlaylistContent,
    ) -> PyResult<DevPlaylistContent> {
        self.inner.insert_playlist_content(entry).map_err(db_error)
    }

    /// Create a unique playlist/content pair; `None` appends.
    ///
    /// Explicit positions start at `DevPlaylistContent::START_SEQ`.
    #[pyo3(signature = (playlist_id, content_id, seq=None))]
    pub fn create_playlist_content(
        &mut self,
        playlist_id: i64,
        content_id: i64,
        seq: Option<u32>,
    ) -> PyResult<DevPlaylistContent> {
        self.inner
            .create_playlist_content(playlist_id, content_id, seq)
            .map_err(db_error)
    }

    /// Count live entries without resolving contents.
    pub fn count_playlist_contents(&mut self, playlist_id: i64) -> PyResult<usize> {
        self.inner
            .count_playlist_contents(playlist_id)
            .map_err(db_error)
    }

    /// Read stored entry positions in ascending order.
    pub fn get_playlist_content_seq_numbers(&mut self, playlist_id: i64) -> PyResult<Vec<u32>> {
        self.inner
            .get_playlist_content_seq_numbers(playlist_id)
            .map_err(db_error)
    }

    /// Check contiguous entry positions starting at `DevPlaylistContent::START_SEQ`.
    pub fn validate_playlist_content_seq_numbers(&mut self, playlist_id: i64) -> PyResult<bool> {
        self.inner
            .validate_playlist_content_seq_numbers(playlist_id)
            .map_err(db_error)
    }

    /// Normalize entry positions in stable sequence order, retaining imported duplicates.
    ///
    /// Returns changed rows; the destination must be a regular playlist.
    pub fn reset_playlist_content_seq(&mut self, playlist_id: i64) -> PyResult<usize> {
        self.inner
            .reset_playlist_content_seq(playlist_id)
            .map_err(db_error)
    }

    /// Move a unique pair within its playlist; `None` appends.
    ///
    /// Positions start at `DevPlaylistContent::START_SEQ`. Returns one if moved, else zero.
    #[pyo3(signature = (playlist_id, content_id, seq=None))]
    pub fn move_playlist_content(
        &mut self,
        playlist_id: i64,
        content_id: i64,
        seq: Option<u32>,
    ) -> PyResult<usize> {
        self.inner
            .move_playlist_content(playlist_id, content_id, seq)
            .map_err(db_error)
    }

    /// Move distinct pairs from one playlist as a block in caller order, atomically.
    ///
    /// The destination follows removal of the selection; `None` appends. Returns changed selected entries.
    #[pyo3(signature = (ids, start_seq=None))]
    pub fn move_playlist_contents(
        &mut self,
        ids: Vec<(i64, i64)>,
        start_seq: Option<u32>,
    ) -> PyResult<usize> {
        self.inner
            .move_playlist_contents(&ids, start_seq)
            .map_err(db_error)
    }

    /// Delete a unique pair and close the sequence gap. Missing pairs return zero.
    ///
    /// imported duplicates return errors. The playlist and content remain intact.
    pub fn delete_playlist_content(
        &mut self,
        playlist_id: i64,
        content_id: i64,
    ) -> PyResult<usize> {
        self.inner
            .delete_playlist_content(playlist_id, content_id)
            .map_err(db_error)
    }

    /// Delete distinct pairs atomically and compact affected lists, retaining playlists and contents.
    ///
    /// Missing, repeated or ambiguous pairs reject the batch.
    pub fn delete_playlist_contents(&mut self, ids: Vec<(i64, i64)>) -> PyResult<usize> {
        self.inner.delete_playlist_contents(&ids).map_err(db_error)
    }

    /// Delete one occurrence snapshot and normalize remaining positions, retaining duplicate pairs.
    ///
    /// Re-fetch after structural edits; stale snapshots return errors. Returns one on success.
    pub fn delete_playlist_content_entry(&mut self, entry: DevPlaylistContent) -> PyResult<usize> {
        self.inner
            .delete_playlist_content_entry(entry)
            .map_err(db_error)
    }

    /// Remove every occurrence of a playlist/content pair, including imported duplicates.
    ///
    /// Both IDs must be nonzero. Returns deleted rows, retaining the playlist and content.
    pub fn remove_playlist_content(
        &mut self,
        playlist_id: i64,
        content_id: i64,
    ) -> PyResult<usize> {
        self.inner
            .remove_playlist_content(playlist_id, content_id)
            .map_err(db_error)
    }

    /// Remove all entries of a playlist, retaining the playlist and contents.
    pub fn delete_playlist_contents_by_playlist_id(&mut self, playlist_id: i64) -> PyResult<usize> {
        self.inner
            .delete_playlist_contents_by_playlist_id(playlist_id)
            .map_err(db_error)
    }

    /// Remove all entries of selected playlists atomically; missing IDs are ignored.
    ///
    /// Playlists and contents remain intact.
    pub fn delete_playlist_contents_by_playlist_ids(&mut self, ids: Vec<i64>) -> PyResult<usize> {
        self.inner
            .delete_playlist_contents_by_playlist_ids(&ids)
            .map_err(db_error)
    }

    /// Remove every playlist occurrence of a content ID and compact affected lists.
    ///
    /// The content itself and playlists remain intact.
    pub fn delete_playlist_contents_by_content_id(&mut self, content_id: i64) -> PyResult<usize> {
        self.inner
            .delete_playlist_contents_by_content_id(content_id)
            .map_err(db_error)
    }

    /// Remove every playlist occurrence of selected content IDs atomically and compact affected lists.
    ///
    /// Contents and playlists remain intact.
    pub fn delete_playlist_contents_by_content_ids(&mut self, ids: Vec<i64>) -> PyResult<usize> {
        self.inner
            .delete_playlist_contents_by_content_ids(&ids)
            .map_err(db_error)
    }

    /// Load all standard export properties in physical order, validating unique device names.
    pub fn get_properties(&mut self) -> PyResult<Vec<DevExportProperty>> {
        self.inner.get_properties().map_err(db_error)
    }

    /// Find a standard export property by its exact, case-sensitive device name.
    ///
    /// Missing names return `None`; duplicate names return an error.
    pub fn get_property_by_device(
        &mut self,
        device_name: &str,
    ) -> PyResult<Option<DevExportProperty>> {
        self.inner
            .get_property_by_device(device_name)
            .map_err(db_error)
    }

    /// Load the first standard export property, validating all property names.
    ///
    /// Returns `None` when the table is empty.
    pub fn get_first_property(&mut self) -> PyResult<Option<DevExportProperty>> {
        self.inner.get_first_property().map_err(db_error)
    }

    /// Update a standard export property in memory, preserving unedited binary fields.
    ///
    /// Renaming requires an unused device name. Re-fetch after content mutations,
    /// which update track counts; stale snapshots are rejected. Saving remains explicit.
    pub fn update_property(&mut self, property: DevExportProperty) -> PyResult<DevExportProperty> {
        self.inner.update_property(property).map_err(db_error)
    }

    /// Load all extended database properties in physical order, retaining repeated rows.
    ///
    /// Returns `Error::MissingExtendedDatabase` when the extended file is not open.
    pub fn get_database_properties(&mut self) -> PyResult<Vec<DevDatabaseProperty>> {
        self.inner.get_database_properties().map_err(db_error)
    }

    /// Load the first extended database property, or `None` when its table is empty.
    ///
    /// Returns `Error::MissingExtendedDatabase` when the extended file is not open.
    pub fn get_first_database_property(&mut self) -> PyResult<Option<DevDatabaseProperty>> {
        self.inner.get_first_database_property().map_err(db_error)
    }

    /// Update one extended property's My Tag library identity, preserving reserved data.
    /// Does not remap tags or associations. Re-fetch after structural edits; saving is explicit.
    /// Returns `Error::MissingExtendedDatabase` when the extended file is not open.
    pub fn update_database_property(
        &mut self,
        property: DevDatabaseProperty,
    ) -> PyResult<DevDatabaseProperty> {
        self.inner
            .update_database_property(property)
            .map_err(db_error)
    }

    /// Load all extended recommendation rows in physical order, retaining imported duplicates.
    ///
    /// Returns `Error::MissingExtendedDatabase` when the extended file is not open.
    pub fn get_recommend_likes(&mut self) -> PyResult<Vec<DevRecommendedLike>> {
        self.inner.get_recommend_likes().map_err(db_error)
    }

    /// Load recommendations containing the content ID on either side, in physical order.
    ///
    /// Referenced contents are not loaded. Requires an open extended database.
    pub fn get_recommend_like_by_content_id(
        &mut self,
        content_id: i64,
    ) -> PyResult<Vec<DevRecommendedLike>> {
        self.inner
            .get_recommend_like_by_content_id(content_id)
            .map_err(db_error)
    }

    /// Insert a unique ordered content pair in the extended database, in memory.
    ///
    /// Reversed pairs remain distinct. IDs must be nonzero; target existence is not checked.
    /// Unknown words are retained verbatim. Requires an open extended database; saving is explicit.
    pub fn insert_recommend_like(
        &mut self,
        recommend_like: NewDevRecommendedLike,
    ) -> PyResult<DevRecommendedLike> {
        self.inner
            .insert_recommend_like(recommend_like)
            .map_err(db_error)
    }

    /// Create a recommendation row with explicit unknown words; their meanings are not inferred.
    ///
    /// See `Self::insert_recommend_like` for constraints and persistence semantics.
    pub fn create_recommend_like(
        &mut self,
        content_id_1: i64,
        content_id_2: i64,
        unknown_00: u64,
        unknown_10: u32,
        unknown_14: u32,
    ) -> PyResult<DevRecommendedLike> {
        self.inner
            .create_recommend_like(
                content_id_1,
                content_id_2,
                unknown_00,
                unknown_10,
                unknown_14,
            )
            .map_err(db_error)
    }

    /// Delete one original recommendation occurrence, retaining contents and other occurrences.
    ///
    /// Returns one on success. Re-fetch snapshots after structural edits; stale rows return errors.
    /// Requires an open extended database; changes remain in memory until explicitly saved.
    pub fn delete_recommend_like(&mut self, recommend_like: DevRecommendedLike) -> PyResult<usize> {
        self.inner
            .delete_recommend_like(recommend_like)
            .map_err(db_error)
    }
}
