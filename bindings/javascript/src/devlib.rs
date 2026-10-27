// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: MIT

//! JavaScript Device Library lifecycle and model operations.

use std::path::Path;

use napi::bindgen_prelude::BigInt;
use napi::{Error, Result};
use napi_derive::napi;

use rbox::core::PlaylistType;
use rbox::devlib::models::content::{DevContentChanges, NewDevContentInput as NewDevContent};
use rbox::devlib::models::playlist::DevPlaylistTreeNode;
use rbox::devlib::models::*;
use rbox::devlib::{DeviceLibrary, LibraryCreateOptions};

use crate::models::Track;
use crate::util::binding_error;

/// Metadata for creating both export.pdb and exportExt.pdb.
#[napi(object, js_name = "LibraryCreateOptions")]
pub struct JsLibraryCreateOptions {
    /// Export date, conventionally YYYY-MM-DD.
    pub date: String,
    /// Device name stored in the export properties.
    pub device_name: String,
    /// Source library identifier stored in the My Tag properties.
    pub my_tag_master_db_id: i64,
}

/// Per-file write counts. Bigints preserve the complete unsigned 64-bit range.
#[napi(object, object_from_js = false, js_name = "DevSaveStats")]
pub struct JsDevSaveStats {
    pub pages_written: u64,
    pub bytes_written: u64,
}

/// Results of saving the standard file and optional extended file.
#[napi(object, object_from_js = false, js_name = "LibrarySaveStats")]
pub struct JsLibrarySaveStats {
    pub standard: JsDevSaveStats,
    pub extended: Option<JsDevSaveStats>,
}

/// Device Library backed by export.pdb and optionally exportExt.pdb.
/// Edits remain in memory until save/savePages; saveAs copies without switching files.
/// Saves run standard-then-extended and are not atomic across the pair.
#[napi(js_name = "DeviceLibrary")]
pub struct JsDeviceLibrary {
    inner: DeviceLibrary,
}

#[napi]
impl JsDeviceLibrary {
    /// Open the database directory, usually PIONEER/rekordbox (not the USB root).
    #[napi(constructor)]
    pub fn new(directory: String) -> Result<Self> {
        Ok(Self {
            inner: DeviceLibrary::new(directory).map_err(binding_error)?,
        })
    }

    /// Open explicit paths. Omit extended to open only the standard database.
    #[napi(factory)]
    pub fn from_paths(standard: String, extended: Option<String>) -> Result<Self> {
        Ok(Self {
            inner: DeviceLibrary::from_paths(standard, extended.as_deref().map(Path::new))
                .map_err(binding_error)?,
        })
    }

    /// Create both files without overwriting existing export files.
    #[napi(factory)]
    pub fn create(directory: String, options: JsLibraryCreateOptions) -> Result<Self> {
        let options = LibraryCreateOptions::new(
            options.date,
            options.device_name,
            options.my_tag_master_db_id,
        );
        Ok(Self {
            inner: DeviceLibrary::create(directory, &options).map_err(binding_error)?,
        })
    }

    /// Whether the extended database was opened.
    #[napi]
    pub fn has_extended(&self) -> bool {
        self.inner.has_extended()
    }

    /// Replace each changed file atomically; the pair is not atomic. Unchanged files write nothing.
    #[napi]
    pub fn save(&mut self) -> Result<JsLibrarySaveStats> {
        let stats = self.inner.save().map_err(binding_error)?;
        Ok(JsLibrarySaveStats {
            standard: JsDevSaveStats {
                pages_written: stats.standard.pages_written,
                bytes_written: stats.standard.bytes_written,
            },
            extended: stats.extended.map(|s| JsDevSaveStats {
                pages_written: s.pages_written,
                bytes_written: s.bytes_written,
            }),
        })
    }

    /// Write changed pages directly, non-atomically. After failure, discard the handler
    /// and reopen only after restoring or repairing the affected file.
    #[napi]
    pub fn save_pages(&mut self) -> Result<JsLibrarySaveStats> {
        let stats = self.inner.save_pages().map_err(binding_error)?;
        Ok(JsLibrarySaveStats {
            standard: JsDevSaveStats {
                pages_written: stats.standard.pages_written,
                bytes_written: stats.standard.bytes_written,
            },
            extended: stats.extended.map(|s| JsDevSaveStats {
                pages_written: s.pages_written,
                bytes_written: s.bytes_written,
            }),
        })
    }

    /// Copy database images without overwriting. Keeps source files and pending changes.
    /// Does not copy audio, artwork, or analysis assets.
    #[napi]
    pub fn save_as(&mut self, directory: String) -> Result<()> {
        self.inner.save_as(directory).map_err(binding_error)
    }

    /// Load all albums in physical order without resolving related rows.
    #[napi]
    pub fn get_albums(&mut self) -> Result<Vec<DevAlbum>> {
        self.inner.get_albums().map_err(binding_error)
    }

    /// Find an album by export ID; returns `null` when absent.
    #[napi]
    pub fn get_album_by_id(&mut self, id: i64) -> Result<Option<DevAlbum>> {
        self.inner.get_album_by_id(id).map_err(binding_error)
    }

    /// Find the first exact, case-sensitive name match in physical order.
    ///
    /// Albums with different artists can share a name; this does not filter by artist.
    #[napi]
    pub fn get_album_by_name(&mut self, name: String) -> Result<Option<DevAlbum>> {
        self.inner.get_album_by_name(&name).map_err(binding_error)
    }

    /// Load an album and its album artist. Missing albums return `null`.
    ///
    /// Missing or unassigned artists remain `null` in the detached view.
    #[napi]
    pub fn get_album_with_relations(&mut self, id: i64) -> Result<Option<DevAlbumView>> {
        self.inner
            .get_album_with_relations(id)
            .map_err(binding_error)
    }

    /// Load all albums in physical order and resolve shared album artists in a batch.
    #[napi]
    pub fn get_albums_with_relations(&mut self) -> Result<Vec<DevAlbumView>> {
        self.inner
            .get_albums_with_relations()
            .map_err(binding_error)
    }

    /// Allocate an album ID and insert it, enforcing unique `(name, artist_id)` pairs.
    ///
    /// Changes remain in memory until explicitly saved.
    #[napi]
    pub fn insert_album(&mut self, album: NewDevAlbum) -> Result<DevAlbum> {
        self.inner.insert_album(album).map_err(binding_error)
    }

    /// Create an album with an automatically allocated export ID and insert it in memory.
    ///
    /// `artist_id` is the album artist; `null` or zero means unassigned.
    #[napi]
    pub fn create_album(&mut self, name: String, artist_id: Option<i64>) -> Result<DevAlbum> {
        self.inner
            .create_album(name, artist_id)
            .map_err(binding_error)
    }

    /// Persist edited album fields in memory and return a refreshed snapshot.
    /// Preserves unedited binary data and enforces the model's uniqueness checks.
    #[napi]
    pub fn update_album(&mut self, album: &DevAlbum) -> Result<DevAlbum> {
        self.inner
            .update_album(album.clone())
            .map_err(binding_error)
    }

    /// Delete an album and clear referencing contents' album IDs atomically in memory.
    /// Returns the total affected rows, or zero for a missing album. Saving is explicit.
    #[napi]
    pub fn delete_album(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_album(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all artists in physical order without resolving related rows.
    #[napi]
    pub fn get_artists(&mut self) -> Result<Vec<DevArtist>> {
        self.inner.get_artists().map_err(binding_error)
    }

    /// Find an artist by export ID; returns `null` when absent.
    #[napi]
    pub fn get_artist_by_id(&mut self, id: i64) -> Result<Option<DevArtist>> {
        self.inner.get_artist_by_id(id).map_err(binding_error)
    }

    /// Find the first exact, case-sensitive name match; returns `null` when absent.
    #[napi]
    pub fn get_artist_by_name(&mut self, name: String) -> Result<Option<DevArtist>> {
        self.inner.get_artist_by_name(&name).map_err(binding_error)
    }

    /// Allocate an artist ID and insert it, enforcing unique, case-sensitive names.
    ///
    /// Changes remain in memory until explicitly saved.
    #[napi]
    pub fn insert_artist(&mut self, artist: NewDevArtist) -> Result<DevArtist> {
        self.inner.insert_artist(artist).map_err(binding_error)
    }

    /// Create an artist with an automatically allocated export ID and insert it in memory.
    #[napi]
    pub fn create_artist(&mut self, name: String) -> Result<DevArtist> {
        self.inner.create_artist(name).map_err(binding_error)
    }

    /// Persist an edited artist name in memory and return a refreshed snapshot.
    ///
    /// Preserves unedited binary data and enforces name uniqueness; saving is explicit.
    #[napi]
    pub fn update_artist(&mut self, artist: &DevArtist) -> Result<DevArtist> {
        self.inner
            .update_artist(artist.clone())
            .map_err(binding_error)
    }

    /// Delete an artist and clear its album and content references atomically in memory.
    ///
    /// Clears album artist, main artist, composer, original artist, and remixer IDs.
    /// Returns the total affected rows, or zero for a missing artist. Saving is explicit.
    #[napi]
    pub fn delete_artist(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_artist(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all artwork references in physical order without resolving related rows.
    #[napi]
    pub fn get_artworks(&mut self) -> Result<Vec<DevArtwork>> {
        self.inner.get_artworks().map_err(binding_error)
    }

    /// Find an artwork reference by export ID; returns `null` when absent.
    #[napi]
    pub fn get_artwork_by_id(&mut self, id: i64) -> Result<Option<DevArtwork>> {
        self.inner.get_artwork_by_id(id).map_err(binding_error)
    }

    /// Find the first exact, case-sensitive path match; returns `null` when absent.
    ///
    /// Paths are compared as stored, without normalization or filesystem access.
    #[napi]
    pub fn get_artwork_by_path(&mut self, path: String) -> Result<Option<DevArtwork>> {
        self.inner.get_artwork_by_path(&path).map_err(binding_error)
    }

    /// Allocate an artwork ID and insert a reference to a nonempty image path.
    ///
    /// Duplicate paths are allowed. Changes remain in memory; no image file is copied.
    #[napi]
    pub fn insert_artwork(&mut self, artwork: NewDevArtwork) -> Result<DevArtwork> {
        self.inner.insert_artwork(artwork).map_err(binding_error)
    }

    /// Create an artwork reference with an automatically allocated ID and insert it in memory.
    #[napi]
    pub fn create_artwork(&mut self, path: String) -> Result<DevArtwork> {
        self.inner.create_artwork(path).map_err(binding_error)
    }

    /// Persist an edited artwork path in memory and return a refreshed snapshot.
    ///
    /// Saving is explicit; this does not copy or rename image files.
    #[napi]
    pub fn update_artwork(&mut self, artwork: &DevArtwork) -> Result<DevArtwork> {
        self.inner
            .update_artwork(artwork.clone())
            .map_err(binding_error)
    }

    /// Delete an artwork reference and clear matching content artwork IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero when absent. Image files are retained;
    /// saving database changes remains explicit.
    #[napi]
    pub fn delete_artwork(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_artwork(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all predefined browse labels in physical order without resolving settings.
    #[napi]
    pub fn get_browse_columns(&mut self) -> Result<Vec<DevBrowseColumn>> {
        self.inner.get_browse_columns().map_err(binding_error)
    }

    /// Find a browse label by its 16-bit catalog ID; returns `null` when absent.
    #[napi]
    pub fn get_browse_column_by_id(&mut self, id: i64) -> Result<Option<DevBrowseColumn>> {
        self.inner
            .get_browse_column_by_id(id)
            .map_err(binding_error)
    }

    /// Find the first exact, case-sensitive plain-text label match, or `null`.
    ///
    /// Enclosing U+FFFA/U+FFFB markers are omitted from the compared label.
    #[napi]
    pub fn get_browse_column_by_name(&mut self, name: String) -> Result<Option<DevBrowseColumn>> {
        self.inner
            .get_browse_column_by_name(&name)
            .map_err(binding_error)
    }

    /// Update a browse label in memory, preserving its enclosing markers.
    ///
    /// Catalog IDs and codes cannot change; saving remains explicit.
    #[napi]
    pub fn update_browse_column(&mut self, column: &DevBrowseColumn) -> Result<DevBrowseColumn> {
        self.inner
            .update_browse_column(column.clone())
            .map_err(binding_error)
    }

    /// Load all category-menu settings in physical order, retaining repeated rows.
    #[napi]
    pub fn get_browse_categories(&mut self) -> Result<Vec<DevBrowseCategory>> {
        self.inner.get_browse_categories().map_err(binding_error)
    }

    /// Load all category settings for a column ID, retaining physical order and repetitions.
    ///
    /// Returns an empty list when absent; referenced labels are not loaded.
    #[napi]
    pub fn get_browse_categories_by_column(
        &mut self,
        column_id: i64,
    ) -> Result<Vec<DevBrowseCategory>> {
        self.inner
            .get_browse_categories_by_column(column_id)
            .map_err(binding_error)
    }

    /// Update one category-setting occurrence in memory; saving remains explicit.
    ///
    /// Column and selector cannot change. No automatic reordering or state coercion
    /// is performed. Stale snapshots are rejected; reload after rebuilding the database.
    #[napi]
    pub fn update_browse_category(
        &mut self,
        category: &DevBrowseCategory,
    ) -> Result<DevBrowseCategory> {
        self.inner
            .update_browse_category(category.clone())
            .map_err(binding_error)
    }

    /// Load all sort-option settings in physical order, retaining repeated rows.
    #[napi]
    pub fn get_browse_sorts(&mut self) -> Result<Vec<DevBrowseSort>> {
        self.inner.get_browse_sorts().map_err(binding_error)
    }

    /// Load all sort settings for a column ID, retaining physical order and repetitions.
    ///
    /// Returns an empty list when absent; referenced labels are not loaded.
    #[napi]
    pub fn get_browse_sorts_by_column(&mut self, column_id: i64) -> Result<Vec<DevBrowseSort>> {
        self.inner
            .get_browse_sorts_by_column(column_id)
            .map_err(binding_error)
    }

    /// Update one sort-setting occurrence's raw tail in memory; saving remains explicit.
    ///
    /// Column and selector cannot change. Stale snapshots are rejected; reload after
    /// rebuilding. Tail-bit meanings remain provisional; see `crate::models::browse`.
    #[napi]
    pub fn update_browse_sort(&mut self, sort: &DevBrowseSort) -> Result<DevBrowseSort> {
        self.inner
            .update_browse_sort(sort.clone())
            .map_err(binding_error)
    }

    /// Load all initialized color labels in physical order without resolving related rows.
    #[napi]
    pub fn get_colors(&mut self) -> Result<Vec<DevColor>> {
        self.inner.get_colors().map_err(binding_error)
    }

    /// Find a color label by its 16-bit export ID; returns `null` when absent.
    #[napi]
    pub fn get_color_by_id(&mut self, id: i64) -> Result<Option<DevColor>> {
        self.inner.get_color_by_id(id).map_err(binding_error)
    }

    /// Find the first exact, case-sensitive color name match, or `null` when absent.
    #[napi]
    pub fn get_color_by_name(&mut self, name: String) -> Result<Option<DevColor>> {
        self.inner.get_color_by_name(&name).map_err(binding_error)
    }

    /// Update a color name in memory and return a refreshed snapshot.
    ///
    /// Names must be unique (case-sensitive); existing IDs cannot change.
    /// Colors are initialized at creation and support updates only. Saving remains explicit.
    #[napi]
    pub fn update_color(&mut self, color: &DevColor) -> Result<DevColor> {
        self.inner
            .update_color(color.clone())
            .map_err(binding_error)
    }

    /// Load shared tracks in physical order with resolved metadata, including album artists.
    ///
    /// Each distinct related row is decoded once per call; missing metadata stays `null`.
    /// BPM is unscaled, duration is seconds, and file size is bytes. Source-library IDs
    /// come from each content row. Decoding and metadata conversion errors propagate.
    /// Results are detached snapshots; audio and analysis files are not read.
    #[napi]
    pub fn get_tracks(&mut self) -> Result<Vec<Track>> {
        self.inner
            .get_tracks()
            .map_err(binding_error)
            .map(|tracks| tracks.into_iter().map(Into::into).collect())
    }

    /// Load a shared track by its local export ID, or `null` if absent.
    ///
    /// See `Self::get_tracks` for relationship resolution, units and errors.
    #[napi]
    pub fn get_track_by_id(&mut self, id: i64) -> Result<Option<Track>> {
        self.inner
            .get_track_by_id(id)
            .map_err(binding_error)
            .map(|track| track.map(Into::into))
    }

    /// Load matching shared tracks in ascending local export-ID order.
    ///
    /// Missing IDs are omitted, repeated IDs occur once, and empty input reads no pages.
    /// See `Self::get_tracks` for relationship resolution, units and errors.
    #[napi]
    pub fn get_tracks_by_ids(&mut self, ids: Vec<i64>) -> Result<Vec<Track>> {
        self.inner
            .get_tracks_by_ids(&ids)
            .map_err(binding_error)
            .map(|tracks| tracks.into_iter().map(Into::into).collect())
    }

    /// Load a shared track by its stored audio path, or `null` if absent.
    ///
    /// Backslashes are converted to forward slashes without accessing the file;
    /// the path must otherwise match exactly. Repeated paths return the first
    /// physical occurrence. See `Self::get_tracks` for conversion errors.
    #[napi]
    pub fn get_track_by_path(&mut self, path: String) -> Result<Option<Track>> {
        self.inner
            .get_track_by_path(&path)
            .map_err(binding_error)
            .map(|track| track.map(Into::into))
    }

    /// Load all content rows in physical order without resolving related metadata.
    #[napi]
    pub fn get_contents(&mut self) -> Result<Vec<DevContent>> {
        self.inner.get_contents().map_err(binding_error)
    }

    /// Find content by its local export ID; returns `null` when absent.
    #[napi]
    pub fn get_content_by_id(&mut self, id: i64) -> Result<Option<DevContent>> {
        self.inner.get_content_by_id(id).map_err(binding_error)
    }

    /// Find content by both source-library IDs, rather than its local export ID.
    ///
    /// Missing or zero identities return `null`. Repeated pairs return the row
    /// with the lowest local export ID.
    #[napi]
    pub fn get_content_by_master_content_id(
        &mut self,
        master_content_id: i64,
        master_db_id: i64,
    ) -> Result<Option<DevContent>> {
        self.inner
            .get_content_by_master_content_id(master_content_id, master_db_id)
            .map_err(binding_error)
    }

    /// Load contents by source-content IDs within one source library.
    ///
    /// Returns all matches in local export-ID order, omitting missing/zero IDs and
    /// repeated input IDs. Repeated source identities retain all matching content rows.
    #[napi]
    pub fn get_contents_by_master_content_ids(
        &mut self,
        master_content_ids: Vec<i64>,
        master_db_id: i64,
    ) -> Result<Vec<DevContent>> {
        self.inner
            .get_contents_by_master_content_ids(&master_content_ids, master_db_id)
            .map_err(binding_error)
    }

    /// Load content and its metadata relationships, including its album artist.
    ///
    /// Missing content returns `null`; missing relationship targets remain `null`.
    #[napi]
    pub fn get_content_with_relations(&mut self, id: i64) -> Result<Option<DevContentView>> {
        self.inner
            .get_content_with_relations(id)
            .map_err(binding_error)
    }

    /// Load all content in physical order and resolve distinct metadata targets once.
    ///
    /// Returns detached snapshots without loading playlists, history, or My Tags.
    #[napi]
    pub fn get_contents_with_relations(&mut self) -> Result<Vec<DevContentView>> {
        self.inner
            .get_contents_with_relations()
            .map_err(binding_error)
    }

    /// Allocate a local export ID and insert content, updating existing export-property counts.
    ///
    /// Accepts a plain object with a required path. Omitted metadata uses defaults;
    /// the file name is derived from the path unless explicitly supplied.
    /// The operation commits atomically in memory; saving remains explicit.
    /// Audio, artwork and analysis files are not copied or generated.
    #[napi]
    pub fn insert_content(&mut self, content: NewDevContent) -> Result<DevContent> {
        self.inner
            .insert_content(content.into())
            .map_err(binding_error)
    }

    /// Create content from a nonempty audio path with forward slashes and insert it in memory.
    ///
    /// The file name is derived from the path; other metadata uses `NewDevContent` defaults.
    #[napi]
    pub fn create_content(&mut self, path: String) -> Result<DevContent> {
        self.inner.create_content(path).map_err(binding_error)
    }

    /// Apply partial changes to the current content row and return a plain object.
    ///
    /// Omitted or undefined fields retain their current values; null is rejected.
    /// Empty strings and zero are explicit changes. Private binary data is preserved.
    /// The ID cannot change. Errors roll back the operation; saving remains explicit.
    /// Editing paths does not move files or automatically update other path fields.
    #[napi]
    pub fn update_content(&mut self, id: i64, changes: DevContentChanges) -> Result<DevContent> {
        let mut content = self
            .inner
            .get_content_by_id(id)
            .map_err(binding_error)?
            .ok_or_else(|| {
                binding_error(rbox::devlib::Error::ModelNotFound {
                    table: DevContent::TABLE,
                    id,
                })
            })?;
        changes.apply_to(&mut content);
        self.inner.update_content(content).map_err(binding_error)
    }

    /// Delete content and its playlist/history occurrences, updating export-property counts.
    ///
    /// When the extended file is open, also delete its My Tag associations and
    /// recommendation pairs containing this ID on either side. Unknown tables remain intact.
    /// Returns deleted row count, including associations but excluding property updates;
    /// a missing ID returns zero. ID zero is rejected. Changes commit atomically in memory;
    /// saving the file pair remains explicit and is not atomic. Resource files are retained.
    #[napi]
    pub fn delete_content(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_content(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all genres in physical order without resolving related rows.
    #[napi]
    pub fn get_genres(&mut self) -> Result<Vec<DevGenre>> {
        self.inner.get_genres().map_err(binding_error)
    }

    /// Find a genre by export ID; returns `null` when absent.
    #[napi]
    pub fn get_genre_by_id(&mut self, id: i64) -> Result<Option<DevGenre>> {
        self.inner.get_genre_by_id(id).map_err(binding_error)
    }

    /// Find the first exact, case-sensitive name match; returns `null` when absent.
    #[napi]
    pub fn get_genre_by_name(&mut self, name: String) -> Result<Option<DevGenre>> {
        self.inner.get_genre_by_name(&name).map_err(binding_error)
    }

    /// Allocate a genre ID and insert it, enforcing unique, case-sensitive names.
    ///
    /// Changes remain in memory until explicitly saved.
    #[napi]
    pub fn insert_genre(&mut self, genre: NewDevGenre) -> Result<DevGenre> {
        self.inner.insert_genre(genre).map_err(binding_error)
    }

    /// Create a genre with an automatically allocated export ID and insert it in memory.
    #[napi]
    pub fn create_genre(&mut self, name: String) -> Result<DevGenre> {
        self.inner.create_genre(name).map_err(binding_error)
    }

    /// Persist an edited genre name in memory and return a refreshed snapshot.
    ///
    /// The ID cannot change; names must remain unique. Saving is explicit.
    #[napi]
    pub fn update_genre(&mut self, genre: &DevGenre) -> Result<DevGenre> {
        self.inner
            .update_genre(genre.clone())
            .map_err(binding_error)
    }

    /// Delete a genre and clear referencing contents' genre IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero for a missing genre. ID zero is rejected.
    /// Contents and export track counts are retained; saving remains explicit.
    #[napi]
    pub fn delete_genre(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_genre(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all histories in physical order without loading their occurrences.
    ///
    /// Device Library histories have no folder hierarchy or type attribute.
    #[napi]
    pub fn get_histories(&mut self) -> Result<Vec<DevHistory>> {
        self.inner.get_histories().map_err(binding_error)
    }

    /// Find a history by export ID; returns `null` when absent.
    #[napi]
    pub fn get_history_by_id(&mut self, id: i64) -> Result<Option<DevHistory>> {
        self.inner.get_history_by_id(id).map_err(binding_error)
    }

    /// Load a history's contents in stored sequence order, retaining repeated plays.
    ///
    /// Ties retain physical occurrence order; missing content references return errors.
    /// No entries returns an empty list. Content metadata relationships are not resolved.
    #[napi]
    pub fn get_history_contents(&mut self, id: i64) -> Result<Vec<DevContent>> {
        self.inner.get_history_contents(id).map_err(binding_error)
    }

    /// Load occurrence snapshots in stored sequence order without resolving contents.
    ///
    /// Repeated plays, sequence gaps and ties are retained. These snapshots can be
    /// passed to `Self::delete_history_content` to delete individual occurrences.
    #[napi]
    pub fn get_history_entries(&mut self, id: i64) -> Result<Vec<DevHistoryContent>> {
        self.inner.get_history_entries(id).map_err(binding_error)
    }

    /// Allocate a history ID and insert its name in memory; saving remains explicit.
    #[napi]
    pub fn insert_history(&mut self, history: NewDevHistory) -> Result<DevHistory> {
        self.inner.insert_history(history).map_err(binding_error)
    }

    /// Create a named history with an automatically allocated export ID.
    ///
    /// Changes remain in memory until explicitly saved.
    #[napi]
    pub fn create_history(&mut self, name: String) -> Result<DevHistory> {
        self.inner.create_history(name).map_err(binding_error)
    }

    /// Update a history name in memory and return a refreshed snapshot.
    /// The ID cannot change; saving remains explicit.
    #[napi]
    pub fn update_history(&mut self, history: &DevHistory) -> Result<DevHistory> {
        self.inner
            .update_history(history.clone())
            .map_err(binding_error)
    }

    /// Delete a history and its occurrences atomically in memory, retaining contents.
    ///
    /// Returns the total deleted rows, or zero for a missing history. ID zero is rejected.
    /// Other histories and playlists are unchanged; saving remains explicit.
    #[napi]
    pub fn delete_history(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_history(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Insert a history occurrence in memory, retaining its explicit stored position.
    ///
    /// Repeated tracks and position ties are allowed. Reference IDs must be nonzero;
    /// target existence is not checked. Saving remains explicit.
    #[napi]
    pub fn insert_history_content(
        &mut self,
        entry: NewDevHistoryContent,
    ) -> Result<DevHistoryContent> {
        self.inner
            .insert_history_content(entry)
            .map_err(binding_error)
    }

    /// Create an occurrence at an explicit stored position without shifting other entries.
    ///
    /// See `Self::insert_history_content` for validation and persistence semantics.
    #[napi]
    pub fn create_history_content(
        &mut self,
        history_id: i64,
        content_id: i64,
        seq: u32,
    ) -> Result<DevHistoryContent> {
        self.inner
            .create_history_content(history_id, content_id, seq)
            .map_err(binding_error)
    }

    /// Delete one occurrence snapshot, retaining other plays, contents and sequence gaps.
    ///
    /// Returns one on success; stale snapshots return an error. Re-fetch entries after
    /// structural edits. Changes remain in memory until explicitly saved.
    #[napi]
    pub fn delete_history_content(&mut self, entry: &DevHistoryContent) -> Result<f64> {
        self.inner
            .delete_history_content(entry.clone())
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all musical keys in physical order without resolving related rows.
    #[napi]
    pub fn get_keys(&mut self) -> Result<Vec<DevKey>> {
        self.inner.get_keys().map_err(binding_error)
    }

    /// Find a key by export ID; returns `null` when absent.
    #[napi]
    pub fn get_key_by_id(&mut self, id: i64) -> Result<Option<DevKey>> {
        self.inner.get_key_by_id(id).map_err(binding_error)
    }

    /// Find the first exact, case-sensitive name match; returns `null` when absent.
    #[napi]
    pub fn get_key_by_name(&mut self, name: String) -> Result<Option<DevKey>> {
        self.inner.get_key_by_name(&name).map_err(binding_error)
    }

    /// Allocate an export key ID and insert it, enforcing unique, case-sensitive names.
    ///
    /// The supplied secondary ID is retained independently; saving remains explicit.
    #[napi]
    pub fn insert_key(&mut self, key: NewDevKey) -> Result<DevKey> {
        self.inner.insert_key(key).map_err(binding_error)
    }

    /// Create a key with an automatically allocated export ID and an explicit secondary ID.
    ///
    /// The secondary ID's role remains unknown; no musical-key numbering is inferred.
    /// Changes remain in memory until explicitly saved.
    #[napi]
    pub fn create_key(&mut self, name: String, secondary_id: i64) -> Result<DevKey> {
        self.inner
            .create_key(name, secondary_id)
            .map_err(binding_error)
    }

    /// Persist an edited key name or secondary ID in memory and return a refreshed snapshot.
    ///
    /// The export ID cannot change; names must remain unique. Saving is explicit.
    #[napi]
    pub fn update_key(&mut self, key: &DevKey) -> Result<DevKey> {
        self.inner.update_key(key.clone()).map_err(binding_error)
    }

    /// Delete a key and clear referencing contents' key IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero for a missing key. ID zero is rejected.
    /// Contents and export track counts are retained; saving remains explicit.
    #[napi]
    pub fn delete_key(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_key(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all labels in physical order without resolving related rows.
    #[napi]
    pub fn get_labels(&mut self) -> Result<Vec<DevLabel>> {
        self.inner.get_labels().map_err(binding_error)
    }

    /// Find a label by export ID; returns `null` when absent.
    #[napi]
    pub fn get_label_by_id(&mut self, id: i64) -> Result<Option<DevLabel>> {
        self.inner.get_label_by_id(id).map_err(binding_error)
    }

    /// Find the first exact, case-sensitive name match; returns `null` when absent.
    #[napi]
    pub fn get_label_by_name(&mut self, name: String) -> Result<Option<DevLabel>> {
        self.inner.get_label_by_name(&name).map_err(binding_error)
    }

    /// Allocate a label ID and insert it, enforcing unique, case-sensitive names.
    /// Changes remain in memory until explicitly saved.
    #[napi]
    pub fn insert_label(&mut self, label: NewDevLabel) -> Result<DevLabel> {
        self.inner.insert_label(label).map_err(binding_error)
    }

    /// Create a label with an automatically allocated export ID and insert it in memory.
    #[napi]
    pub fn create_label(&mut self, name: String) -> Result<DevLabel> {
        self.inner.create_label(name).map_err(binding_error)
    }

    /// Persist an edited label name in memory and return a refreshed snapshot.
    ///
    /// The ID cannot change; names must remain unique. Saving is explicit.
    #[napi]
    pub fn update_label(&mut self, label: &DevLabel) -> Result<DevLabel> {
        self.inner
            .update_label(label.clone())
            .map_err(binding_error)
    }

    /// Delete a label and clear referencing contents' label IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero for a missing label. ID zero is rejected.
    /// Contents and export track counts are retained; saving remains explicit.
    #[napi]
    pub fn delete_label(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_label(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all tags/categories in physical order without resolving relations.
    ///
    /// All My Tag methods require the extended file; otherwise they return
    /// `Error::MissingExtendedDatabase`. Mutations remain in memory until saved.
    #[napi]
    pub fn get_my_tags(&mut self) -> Result<Vec<DevMyTag>> {
        self.inner.get_my_tags().map_err(binding_error)
    }

    /// Find a tag/category by its source-library ID; missing IDs return `null`.
    #[napi]
    pub fn get_my_tag_by_id(&mut self, id: i64) -> Result<Option<DevMyTag>> {
        self.inner.get_my_tag_by_id(id).map_err(binding_error)
    }

    /// Load immediate children in stored sequence order; zero selects roots.
    #[napi]
    pub fn get_my_tag_children(&mut self, id: i64) -> Result<Vec<DevMyTag>> {
        self.inner.get_my_tag_children(id).map_err(binding_error)
    }

    /// Load tags for a local content ID, retaining physical association order and repetitions.
    ///
    /// Missing tag targets return errors. The standard content row is not loaded.
    #[napi]
    pub fn get_my_tag_by_content_id(&mut self, content_id: i64) -> Result<Vec<DevMyTag>> {
        self.inner
            .get_my_tag_by_content_id(content_id)
            .map_err(binding_error)
    }

    /// Load a tag's contents across both files in physical association order, retaining repeats.
    ///
    /// Missing content targets return errors. Metadata relationships are not resolved.
    #[napi]
    pub fn get_my_tag_contents(&mut self, id: i64) -> Result<Vec<DevContent>> {
        self.inner.get_my_tag_contents(id).map_err(binding_error)
    }

    /// Insert a tag/category, validating ancestry and maintaining sibling order.
    ///
    /// IDs must be supplied explicitly and should match the source `master.db`.
    #[napi]
    pub fn insert_my_tag(&mut self, my_tag: NewDevMyTag) -> Result<DevMyTag> {
        self.inner.insert_my_tag(my_tag).map_err(binding_error)
    }

    /// Create a tag with its source `master.db` ID. `null` selects the root or appends.
    ///
    /// Explicit positions start at `DevMyTag::START_SEQ`. Saving remains explicit.
    #[napi]
    pub fn create_my_tag(
        &mut self,
        id: i64,
        name: String,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<DevMyTag> {
        self.inner
            .create_my_tag(id, name, parent_id, seq)
            .map_err(binding_error)
    }

    /// Create a category with its source `master.db` ID. `null` selects the root or appends.
    ///
    /// Explicit positions start at `DevMyTag::START_SEQ`. Saving remains explicit.
    #[napi]
    pub fn create_my_tag_folder(
        &mut self,
        id: i64,
        name: String,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<DevMyTag> {
        self.inner
            .create_my_tag_folder(id, name, parent_id, seq)
            .map_err(binding_error)
    }

    /// Update name, parent or position, validating ancestry and maintaining sibling order.
    ///
    /// IDs and attributes cannot change. Changes commit atomically in memory.
    #[napi]
    pub fn update_my_tag(&mut self, my_tag: &DevMyTag) -> Result<DevMyTag> {
        self.inner
            .update_my_tag(my_tag.clone())
            .map_err(binding_error)
    }

    /// Rename a tag/category, returning one on success or zero for a missing ID.
    #[napi]
    pub fn rename_my_tag(&mut self, id: i64, name: String) -> Result<f64> {
        self.inner
            .rename_my_tag(id, &name)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Count immediate children without decoding names; zero selects the root.
    #[napi]
    pub fn count_my_tag_children(&mut self, parent_id: i64) -> Result<f64> {
        self.inner
            .count_my_tag_children(parent_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Read sibling positions in ascending order, retaining imported gaps and ties.
    #[napi]
    pub fn get_my_tag_seq_numbers(&mut self, parent_id: i64) -> Result<Vec<u32>> {
        self.inner
            .get_my_tag_seq_numbers(parent_id)
            .map_err(binding_error)
    }

    /// Check contiguous sibling positions starting at `DevMyTag::START_SEQ`.
    #[napi]
    pub fn validate_my_tag_seq_numbers(&mut self, parent_id: i64) -> Result<bool> {
        self.inner
            .validate_my_tag_seq_numbers(parent_id)
            .map_err(binding_error)
    }

    /// Normalize sibling positions in stable sequence order, returning changed rows.
    ///
    /// The parent must be a category with valid ancestry, or zero for the root.
    #[napi]
    pub fn reset_my_tag_seq(&mut self, parent_id: i64) -> Result<f64> {
        self.inner
            .reset_my_tag_seq(parent_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Move one node, maintaining sibling order and rejecting cycles or invalid parents.
    ///
    /// `null` keeps the parent or appends. Returns one if the selected node moved, else zero.
    #[napi]
    pub fn move_my_tag(
        &mut self,
        id: i64,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<f64> {
        self.inner
            .move_my_tag(id, parent_id.as_ref(), seq)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Move distinct nodes as a block in caller order, atomically; `null` appends.
    ///
    /// Returns changed selected nodes, excluding shifts of other siblings.
    #[napi]
    pub fn move_my_tags(
        &mut self,
        ids: Vec<i64>,
        parent_id: i64,
        start_seq: Option<u32>,
    ) -> Result<f64> {
        self.inner
            .move_my_tags(&ids, parent_id, start_seq)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Delete a tag/category subtree and its associations, normalizing surviving siblings.
    ///
    /// Returns deleted rows; missing IDs return zero. Contents remain intact.
    #[napi]
    pub fn delete_my_tag(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_my_tag(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Delete selected subtrees atomically, counting overlapping descendants once.
    ///
    /// Duplicate or missing input IDs reject the batch. Returns deleted nodes and associations.
    #[napi]
    pub fn delete_my_tags(&mut self, ids: Vec<i64>) -> Result<f64> {
        self.inner
            .delete_my_tags(&ids)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all raw tag/content associations in physical order, retaining duplicates.
    #[napi]
    pub fn get_my_tag_associations(&mut self) -> Result<Vec<DevMyTagContent>> {
        self.inner.get_my_tag_associations().map_err(binding_error)
    }

    /// Load a tag's association snapshots without resolving contents or deduplicating pairs.
    #[napi]
    pub fn get_my_tag_entries(&mut self, id: i64) -> Result<Vec<DevMyTagContent>> {
        self.inner.get_my_tag_entries(id).map_err(binding_error)
    }

    /// Load a content's association snapshots without resolving tags or deduplicating pairs.
    #[napi]
    pub fn get_my_tag_associations_by_content_id(
        &mut self,
        content_id: i64,
    ) -> Result<Vec<DevMyTagContent>> {
        self.inner
            .get_my_tag_associations_by_content_id(content_id)
            .map_err(binding_error)
    }

    /// Insert an association with explicit reserved values. IDs must be nonzero.
    ///
    /// target existence is not checked and repeated pairs are allowed.
    #[napi]
    pub fn insert_my_tag_content(&mut self, entry: NewDevMyTagContent) -> Result<DevMyTagContent> {
        self.inner
            .insert_my_tag_content(entry)
            .map_err(binding_error)
    }

    /// Associate a source-library tag ID with a local export content ID using model defaults.
    ///
    /// Repeated pairs are allowed; referenced rows are not loaded or validated.
    #[napi]
    pub fn create_my_tag_content(
        &mut self,
        my_tag_id: i64,
        content_id: i64,
    ) -> Result<DevMyTagContent> {
        self.inner
            .create_my_tag_content(my_tag_id, content_id)
            .map_err(binding_error)
    }

    /// Delete all occurrences of a tag/content pair, returning the deleted count.
    /// Both IDs must be nonzero. Tags and contents remain intact.
    #[napi]
    pub fn delete_my_tag_content(&mut self, my_tag_id: i64, content_id: i64) -> Result<f64> {
        self.inner
            .delete_my_tag_content(my_tag_id, content_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Delete one association snapshot, retaining duplicate occurrences and referenced rows.
    ///
    /// Returns one on success. Re-fetch after structural edits; stale snapshots return errors.
    #[napi]
    pub fn delete_my_tag_content_entry(&mut self, entry: &DevMyTagContent) -> Result<f64> {
        self.inner
            .delete_my_tag_content_entry(entry.clone())
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all playlists and folders in physical order without resolving contents.
    ///
    /// All playlist mutations commit in memory; saving remains explicit.
    #[napi]
    pub fn get_playlists(&mut self) -> Result<Vec<DevPlaylist>> {
        self.inner.get_playlists().map_err(binding_error)
    }

    /// Find a playlist or folder by export ID; missing IDs return `null`.
    #[napi]
    pub fn get_playlist_by_id(&mut self, id: i64) -> Result<Option<DevPlaylist>> {
        self.inner.get_playlist_by_id(id).map_err(binding_error)
    }

    /// Load immediate children in sibling order; zero selects roots.
    #[napi]
    pub fn get_playlist_children(&mut self, id: i64) -> Result<Vec<DevPlaylist>> {
        self.inner.get_playlist_children(id).map_err(binding_error)
    }

    /// Load contents in playlist sequence order, retaining imported duplicate occurrences.
    ///
    /// Missing content references return errors; metadata relationships are not resolved.
    #[napi]
    pub fn get_playlist_contents(&mut self, id: i64) -> Result<Vec<DevContent>> {
        self.inner.get_playlist_contents(id).map_err(binding_error)
    }

    /// Find a node by exact path components. Empty, missing or ambiguous paths return `null`.
    ///
    /// Malformed ancestry and duplicate IDs return errors.
    #[napi]
    pub fn get_playlist_by_path(&mut self, path: Vec<String>) -> Result<Option<DevPlaylist>> {
        let path: Vec<&str> = path.iter().map(String::as_str).collect();
        self.inner
            .get_playlist_by_path(&path)
            .map_err(binding_error)
    }

    /// Build a detached tree with children in sibling order, without loading contents.
    ///
    /// Invalid parents, cycles and duplicate IDs return errors.
    #[napi]
    pub fn get_playlist_tree(&mut self) -> Result<Vec<DevPlaylistTreeNode>> {
        self.inner.get_playlist_tree().map_err(binding_error)
    }

    /// Read the node type; the root sentinel (zero) is treated as a folder.
    #[napi]
    pub fn get_playlist_attribute(&mut self, id: i64) -> Result<Option<PlaylistType>> {
        self.inner.get_playlist_attribute(id).map_err(binding_error)
    }

    /// Check whether an existing node is a regular playlist.
    #[napi]
    pub fn is_playlist(&mut self, id: i64) -> Result<bool> {
        self.inner.is_playlist(id).map_err(binding_error)
    }

    /// Allocate an export ID and insert a playlist/folder, validating ancestry and sibling order.
    #[napi]
    pub fn insert_playlist(&mut self, playlist: NewDevPlaylist) -> Result<DevPlaylist> {
        self.inner.insert_playlist(playlist).map_err(binding_error)
    }

    /// Create a regular playlist. `null` selects the root or appends among siblings.
    ///
    /// Explicit positions start at `DevPlaylist::START_SEQ`.
    #[napi]
    pub fn create_playlist(
        &mut self,
        name: String,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<DevPlaylist> {
        self.inner
            .create_playlist(name, parent_id, seq)
            .map_err(binding_error)
    }

    /// Create a folder. `null` selects the root or appends among siblings.
    ///
    /// Explicit positions start at `DevPlaylist::START_SEQ`.
    #[napi]
    pub fn create_playlist_folder(
        &mut self,
        name: String,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<DevPlaylist> {
        self.inner
            .create_playlist_folder(name, parent_id, seq)
            .map_err(binding_error)
    }

    /// Update name, parent or position, validating ancestry and maintaining sibling order.
    ///
    /// IDs and attributes cannot change. Changes commit atomically in memory.
    #[napi]
    pub fn update_playlist(&mut self, playlist: &DevPlaylist) -> Result<DevPlaylist> {
        self.inner
            .update_playlist(playlist.clone())
            .map_err(binding_error)
    }

    /// Rename a node while preserving hierarchy, position and reserved data.
    ///
    /// Returns one on success; missing IDs return `Error::ModelNotFound`.
    #[napi]
    pub fn rename_playlist(&mut self, id: i64, name: String) -> Result<f64> {
        self.inner
            .rename_playlist(id, &name)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Count immediate children without decoding names; zero selects roots.
    #[napi]
    pub fn count_playlist_children(&mut self, parent_id: i64) -> Result<f64> {
        self.inner
            .count_playlist_children(parent_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Read sibling positions in ascending order, retaining imported gaps and ties.
    #[napi]
    pub fn get_playlist_seq_numbers(&mut self, parent_id: i64) -> Result<Vec<u32>> {
        self.inner
            .get_playlist_seq_numbers(parent_id)
            .map_err(binding_error)
    }

    /// Check contiguous sibling positions starting at `DevPlaylist::START_SEQ`.
    #[napi]
    pub fn validate_playlist_seq_numbers(&mut self, parent_id: i64) -> Result<bool> {
        self.inner
            .validate_playlist_seq_numbers(parent_id)
            .map_err(binding_error)
    }

    /// Normalize siblings in stable sequence order, returning changed rows.
    ///
    /// The parent must be a folder with valid ancestry, or zero for the root.
    #[napi]
    pub fn reset_playlist_seq(&mut self, parent_id: i64) -> Result<f64> {
        self.inner
            .reset_playlist_seq(parent_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Move one node, rejecting cycles and invalid parents. `null` keeps the parent or appends.
    ///
    /// Positions start at `DevPlaylist::START_SEQ`. Returns one if the selected node moved, else zero.
    #[napi]
    pub fn move_playlist(
        &mut self,
        id: i64,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<f64> {
        self.inner
            .move_playlist(id, parent_id.as_ref(), seq)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Move distinct nodes as a block in caller order, atomically; `null` appends.
    ///
    /// The destination is measured after removing the selection. Returns changed selected nodes.
    #[napi]
    pub fn move_playlists(
        &mut self,
        ids: Vec<i64>,
        parent_id: i64,
        start_seq: Option<u32>,
    ) -> Result<f64> {
        self.inner
            .move_playlists(&ids, parent_id, start_seq)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Delete a playlist/folder subtree and its entries, normalizing surviving siblings.
    ///
    /// Returns deleted rows; missing IDs return zero. Contents remain intact.
    #[napi]
    pub fn delete_playlist(&mut self, id: i64) -> Result<f64> {
        self.inner
            .delete_playlist(id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Delete selected subtrees atomically, counting overlapping descendants once.
    ///
    /// Duplicate or missing input IDs reject the batch. Returns deleted nodes and entries.
    #[napi]
    pub fn delete_playlists(&mut self, ids: Vec<i64>) -> Result<f64> {
        self.inner
            .delete_playlists(&ids)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all playlist entry snapshots in physical order, retaining imported duplicates.
    #[napi]
    pub fn get_playlist_associations(&mut self) -> Result<Vec<DevPlaylistContent>> {
        self.inner
            .get_playlist_associations()
            .map_err(binding_error)
    }

    /// Load a playlist's entry snapshots in stored sequence order without resolving contents.
    #[napi]
    pub fn get_playlist_entries(&mut self, id: i64) -> Result<Vec<DevPlaylistContent>> {
        self.inner.get_playlist_entries(id).map_err(binding_error)
    }

    /// Load a content's entry snapshots ordered by playlist ID and sequence.
    #[napi]
    pub fn get_playlist_associations_by_content_id(
        &mut self,
        content_id: i64,
    ) -> Result<Vec<DevPlaylistContent>> {
        self.inner
            .get_playlist_associations_by_content_id(content_id)
            .map_err(binding_error)
    }

    /// Find a unique playlist/content pair; missing pairs return `null`, duplicates return errors.
    #[napi]
    pub fn get_playlist_content_by_id(
        &mut self,
        playlist_id: i64,
        content_id: i64,
    ) -> Result<Option<DevPlaylistContent>> {
        self.inner
            .get_playlist_content_by_id(playlist_id, content_id)
            .map_err(binding_error)
    }

    /// Insert a unique playlist/content pair, normalizing affected positions.
    ///
    /// The destination must be a regular playlist; positions start at `DevPlaylistContent::START_SEQ`.
    #[napi]
    pub fn insert_playlist_content(
        &mut self,
        entry: NewDevPlaylistContent,
    ) -> Result<DevPlaylistContent> {
        self.inner
            .insert_playlist_content(entry)
            .map_err(binding_error)
    }

    /// Create a unique playlist/content pair; `null` appends.
    ///
    /// Explicit positions start at `DevPlaylistContent::START_SEQ`.
    #[napi]
    pub fn create_playlist_content(
        &mut self,
        playlist_id: i64,
        content_id: i64,
        seq: Option<u32>,
    ) -> Result<DevPlaylistContent> {
        self.inner
            .create_playlist_content(playlist_id, content_id, seq)
            .map_err(binding_error)
    }

    /// Count live entries without resolving contents.
    #[napi]
    pub fn count_playlist_contents(&mut self, playlist_id: i64) -> Result<f64> {
        self.inner
            .count_playlist_contents(playlist_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Read stored entry positions in ascending order.
    #[napi]
    pub fn get_playlist_content_seq_numbers(&mut self, playlist_id: i64) -> Result<Vec<u32>> {
        self.inner
            .get_playlist_content_seq_numbers(playlist_id)
            .map_err(binding_error)
    }

    /// Check contiguous entry positions starting at `DevPlaylistContent::START_SEQ`.
    #[napi]
    pub fn validate_playlist_content_seq_numbers(&mut self, playlist_id: i64) -> Result<bool> {
        self.inner
            .validate_playlist_content_seq_numbers(playlist_id)
            .map_err(binding_error)
    }

    /// Normalize entry positions in stable sequence order, retaining imported duplicates.
    ///
    /// Returns changed rows; the destination must be a regular playlist.
    #[napi]
    pub fn reset_playlist_content_seq(&mut self, playlist_id: i64) -> Result<f64> {
        self.inner
            .reset_playlist_content_seq(playlist_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Move a unique pair within its playlist; `null` appends.
    ///
    /// Positions start at `DevPlaylistContent::START_SEQ`. Returns one if moved, else zero.
    #[napi]
    pub fn move_playlist_content(
        &mut self,
        playlist_id: i64,
        content_id: i64,
        seq: Option<u32>,
    ) -> Result<f64> {
        self.inner
            .move_playlist_content(playlist_id, content_id, seq)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Move distinct pairs from one playlist as a block in caller order, atomically.
    ///
    /// The destination follows removal of the selection; `null` appends. Returns changed selected entries.
    #[napi]
    pub fn move_playlist_contents(
        &mut self,
        ids: Vec<(i64, i64)>,
        start_seq: Option<u32>,
    ) -> Result<f64> {
        self.inner
            .move_playlist_contents(&ids, start_seq)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Delete a unique pair and close the sequence gap. Missing pairs return zero.
    ///
    /// imported duplicates return errors. The playlist and content remain intact.
    #[napi]
    pub fn delete_playlist_content(&mut self, playlist_id: i64, content_id: i64) -> Result<f64> {
        self.inner
            .delete_playlist_content(playlist_id, content_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Delete distinct pairs atomically and compact affected lists, retaining playlists and contents.
    ///
    /// Missing, repeated or ambiguous pairs reject the batch.
    #[napi]
    pub fn delete_playlist_contents(&mut self, ids: Vec<(i64, i64)>) -> Result<f64> {
        self.inner
            .delete_playlist_contents(&ids)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Delete one occurrence snapshot and normalize remaining positions, retaining duplicate pairs.
    ///
    /// Re-fetch after structural edits; stale snapshots return errors. Returns one on success.
    #[napi]
    pub fn delete_playlist_content_entry(&mut self, entry: &DevPlaylistContent) -> Result<f64> {
        self.inner
            .delete_playlist_content_entry(entry.clone())
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Remove every occurrence of a playlist/content pair, including imported duplicates.
    ///
    /// Both IDs must be nonzero. Returns deleted rows, retaining the playlist and content.
    #[napi]
    pub fn remove_playlist_content(&mut self, playlist_id: i64, content_id: i64) -> Result<f64> {
        self.inner
            .remove_playlist_content(playlist_id, content_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Remove all entries of a playlist, retaining the playlist and contents.
    #[napi]
    pub fn delete_playlist_contents_by_playlist_id(&mut self, playlist_id: i64) -> Result<f64> {
        self.inner
            .delete_playlist_contents_by_playlist_id(playlist_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Remove all entries of selected playlists atomically; missing IDs are ignored.
    ///
    /// Playlists and contents remain intact.
    #[napi]
    pub fn delete_playlist_contents_by_playlist_ids(&mut self, ids: Vec<i64>) -> Result<f64> {
        self.inner
            .delete_playlist_contents_by_playlist_ids(&ids)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Remove every playlist occurrence of a content ID and compact affected lists.
    ///
    /// The content itself and playlists remain intact.
    #[napi]
    pub fn delete_playlist_contents_by_content_id(&mut self, content_id: i64) -> Result<f64> {
        self.inner
            .delete_playlist_contents_by_content_id(content_id)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Remove every playlist occurrence of selected content IDs atomically and compact affected lists.
    ///
    /// Contents and playlists remain intact.
    #[napi]
    pub fn delete_playlist_contents_by_content_ids(&mut self, ids: Vec<i64>) -> Result<f64> {
        self.inner
            .delete_playlist_contents_by_content_ids(&ids)
            .map_err(binding_error)
            .and_then(js_count)
    }

    /// Load all standard export properties in physical order, validating unique device names.
    #[napi]
    pub fn get_properties(&mut self) -> Result<Vec<DevExportProperty>> {
        self.inner.get_properties().map_err(binding_error)
    }

    /// Find a standard export property by its exact, case-sensitive device name.
    ///
    /// Missing names return `null`; duplicate names return an error.
    #[napi]
    pub fn get_property_by_device(
        &mut self,
        device_name: String,
    ) -> Result<Option<DevExportProperty>> {
        self.inner
            .get_property_by_device(&device_name)
            .map_err(binding_error)
    }

    /// Load the first standard export property, validating all property names.
    ///
    /// Returns `null` when the table is empty.
    #[napi]
    pub fn get_first_property(&mut self) -> Result<Option<DevExportProperty>> {
        self.inner.get_first_property().map_err(binding_error)
    }

    /// Update a standard export property in memory, preserving unedited binary fields.
    ///
    /// Renaming requires an unused device name. Re-fetch after content mutations,
    /// which update track counts; stale snapshots are rejected. Saving remains explicit.
    #[napi]
    pub fn update_property(&mut self, property: &DevExportProperty) -> Result<DevExportProperty> {
        self.inner
            .update_property(property.clone())
            .map_err(binding_error)
    }

    /// Load all extended database properties in physical order, retaining repeated rows.
    ///
    /// Returns `Error::MissingExtendedDatabase` when the extended file is not open.
    #[napi]
    pub fn get_database_properties(&mut self) -> Result<Vec<DevDatabaseProperty>> {
        self.inner.get_database_properties().map_err(binding_error)
    }

    /// Load the first extended database property, or `null` when its table is empty.
    ///
    /// Returns `Error::MissingExtendedDatabase` when the extended file is not open.
    #[napi]
    pub fn get_first_database_property(&mut self) -> Result<Option<DevDatabaseProperty>> {
        self.inner
            .get_first_database_property()
            .map_err(binding_error)
    }

    /// Update one extended property's My Tag library identity, preserving reserved data.
    /// Does not remap tags or associations. Re-fetch after structural edits; saving is explicit.
    /// Returns `Error::MissingExtendedDatabase` when the extended file is not open.
    #[napi]
    pub fn update_database_property(
        &mut self,
        property: &DevDatabaseProperty,
    ) -> Result<DevDatabaseProperty> {
        self.inner
            .update_database_property(property.clone())
            .map_err(binding_error)
    }

    /// Load all extended recommendation rows in physical order, retaining imported duplicates.
    ///
    /// Returns `Error::MissingExtendedDatabase` when the extended file is not open.
    #[napi]
    pub fn get_recommend_likes(&mut self) -> Result<Vec<DevRecommendedLike>> {
        self.inner.get_recommend_likes().map_err(binding_error)
    }

    /// Load recommendations containing the content ID on either side, in physical order.
    ///
    /// Referenced contents are not loaded. Requires an open extended database.
    #[napi]
    pub fn get_recommend_like_by_content_id(
        &mut self,
        content_id: i64,
    ) -> Result<Vec<DevRecommendedLike>> {
        self.inner
            .get_recommend_like_by_content_id(content_id)
            .map_err(binding_error)
    }

    /// Insert a unique ordered content pair in the extended database, in memory.
    ///
    /// Reversed pairs remain distinct. IDs must be nonzero; target existence is not checked.
    /// Unknown words are retained verbatim. Requires an open extended database; saving is explicit.
    #[napi]
    pub fn insert_recommend_like(
        &mut self,
        recommend_like: NewDevRecommendedLike,
    ) -> Result<DevRecommendedLike> {
        self.inner
            .insert_recommend_like(recommend_like)
            .map_err(binding_error)
    }

    /// Create a recommendation row with explicit unknown words; their meanings are not inferred.
    ///
    /// See `Self::insert_recommend_like` for constraints and persistence semantics.
    #[napi]
    pub fn create_recommend_like(
        &mut self,
        content_id_1: i64,
        content_id_2: i64,
        unknown_00: BigInt,
        unknown_10: u32,
        unknown_14: u32,
    ) -> Result<DevRecommendedLike> {
        self.inner
            .create_recommend_like(
                content_id_1,
                content_id_2,
                unsigned_word(unknown_00)?,
                unknown_10,
                unknown_14,
            )
            .map_err(binding_error)
    }

    /// Delete one original recommendation occurrence, retaining contents and other occurrences.
    ///
    /// Returns one on success. Re-fetch snapshots after structural edits; stale rows return errors.
    /// Requires an open extended database; changes remain in memory until explicitly saved.
    #[napi]
    pub fn delete_recommend_like(&mut self, recommend_like: &DevRecommendedLike) -> Result<f64> {
        self.inner
            .delete_recommend_like(recommend_like.clone())
            .map_err(binding_error)
            .and_then(js_count)
    }
}

fn js_count(value: usize) -> Result<f64> {
    if value as u128 > 9_007_199_254_740_991 {
        return Err(Error::from_reason(
            "Row count exceeds JavaScript's safe integer range",
        ));
    }
    Ok(value as f64)
}

fn unsigned_word(value: BigInt) -> Result<u64> {
    if value.sign_bit || value.words.len() > 1 {
        return Err(Error::from_reason(
            "unknown00 must be a bigint between 0 and 18446744073709551615",
        ));
    }
    Ok(value.words.first().copied().unwrap_or(0))
}
