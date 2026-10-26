// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Python OneLibrary handler and database operations.

use pyo3::prelude::*;
use pyo3::types::PyType;
use rbox::core::Track;
use rbox::db::enums::HistoryType;
use rbox::one_library::{models::*, OneLibrary};

use super::errors::OneLibraryError;

#[inline]
fn db_error<E: ToString>(e: E) -> PyErr {
    PyErr::new::<OneLibraryError, _>(e.to_string())
}

#[pyclass(unsendable)]
pub struct PyOneLibrary {
    inner: OneLibrary,
}

#[pymethods]
impl PyOneLibrary {
    /// Load shared tracks with resolved names, including album artists.
    pub fn get_tracks(&self) -> PyResult<Vec<Track>> {
        self.inner.get_tracks().map_err(db_error)
    }

    /// Load a shared track by its numeric OneLibrary ID, or None if absent.
    pub fn get_track_by_id(&self, id: i32) -> PyResult<Option<Track>> {
        self.inner.get_track_by_id(id).map_err(db_error)
    }

    /// Load matching tracks in ID order, omitting missing and duplicate IDs.
    pub fn get_tracks_by_ids(&self, ids: Vec<i32>) -> PyResult<Vec<Track>> {
        self.inner.get_tracks_by_ids(&ids).map_err(db_error)
    }

    /// Load a shared track by stored path, accepting either separator style.
    pub fn get_track_by_path(&self, path: &str) -> PyResult<Option<Track>> {
        self.inner.get_track_by_path(path).map_err(db_error)
    }

    /// Open exportLibrary.db in the supplied directory, usually PIONEER/rekordbox.
    #[new]
    pub fn new(directory: &str) -> PyResult<Self> {
        let inner = OneLibrary::new(directory).map_err(db_error)?;
        Ok(PyOneLibrary { inner })
    }

    /// Open an existing OneLibrary database at an explicit file path.
    #[classmethod]
    pub fn from_path(_cls: &Bound<'_, PyType>, path: &str) -> PyResult<Self> {
        let inner = OneLibrary::from_path(path).map_err(db_error)?;
        Ok(PyOneLibrary { inner })
    }

    /// Create a new OneLibrary (Device Library Plus) database.
    #[classmethod]
    pub fn create(_cls: &Bound<'_, PyType>, path: &str, my_tag_master_dbid: i32) -> PyResult<Self> {
        let inner = OneLibrary::create(path, my_tag_master_dbid).map_err(db_error)?;
        Ok(PyOneLibrary { inner })
    }

    /// Create a new OneLibrary database if it does not exist, otherwise return the existing one.
    #[classmethod]
    pub fn create_if_not_exists(
        _cls: &Bound<'_, PyType>,
        path: &str,
        my_tag_master_dbid: i32,
    ) -> PyResult<Self> {
        let inner = OneLibrary::create_if_not_exists(path, my_tag_master_dbid).map_err(db_error)?;
        Ok(PyOneLibrary { inner })
    }

    /// Load a detached album snapshot with all supported direct relationships.
    /// Related rows are shallow snapshots; changes require explicit model updates.
    pub fn get_album_with_relations(&self, id: i32) -> PyResult<Option<AlbumView>> {
        self.inner.get_album_with_relations(id).map_err(db_error)
    }

    /// Load all albums with direct relationships using batched queries.
    /// Missing relationship targets remain absent; query errors propagate.
    pub fn get_albums_with_relations(&self) -> PyResult<Vec<AlbumView>> {
        self.inner.get_albums_with_relations().map_err(db_error)
    }

    /// Queries all records from the `album` table.
    pub fn get_albums(&self) -> PyResult<Vec<Album>> {
        let res = self.inner.get_albums().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `album` table by its primary key `id`.
    pub fn get_album_by_id(&self, id: i32) -> PyResult<Option<Album>> {
        let res = self.inner.get_album_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `album` table by its `name`.
    pub fn get_album_by_name(&self, name: &str) -> PyResult<Option<Album>> {
        let res = self.inner.get_album_by_name(name).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `album` table.
    pub fn insert_album(&self, album: NewAlbum) -> PyResult<Album> {
        let res = self.inner.insert_album(album).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `album` table.
    pub fn create_album(
        &self,
        name: &str,
        artist_id: Option<i32>,
        image_id: Option<i32>,
    ) -> PyResult<Album> {
        let res = self
            .inner
            .create_album(name, artist_id, image_id)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `album` table.
    pub fn update_album(&self, album: Album) -> PyResult<Album> {
        let res = self.inner.update_album(album).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `album` table by its primary key `id`.
    pub fn delete_album(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_album(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `artist` table.
    pub fn get_artists(&self) -> PyResult<Vec<Artist>> {
        let res = self.inner.get_artists().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `artist` table by its primary key `id`.
    pub fn get_artist_by_id(&self, id: i32) -> PyResult<Option<Artist>> {
        let res = self.inner.get_artist_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `artist` table by its `name`.
    pub fn get_artist_by_name(&self, name: &str) -> PyResult<Option<Artist>> {
        let res = self.inner.get_artist_by_name(name).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `artist` table.
    pub fn insert_artist(&self, artist: NewArtist) -> PyResult<Artist> {
        let res = self.inner.insert_artist(artist).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `artist` table.
    pub fn create_artist(&self, name: &str) -> PyResult<Artist> {
        let res = self.inner.create_artist(name).map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `artist` table.
    pub fn update_artist(&self, artist: Artist) -> PyResult<Artist> {
        let res = self.inner.update_artist(artist).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `artist` table by its primary key `id`.
    pub fn delete_artist(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_artist(id).map_err(db_error)?;
        Ok(res)
    }

    /// Load a detached category snapshot with all supported direct relationships.
    /// Related rows are shallow snapshots; changes require explicit model updates.
    pub fn get_category_with_relations(&self, id: i32) -> PyResult<Option<CategoryView>> {
        self.inner.get_category_with_relations(id).map_err(db_error)
    }

    /// Load all categories with direct relationships using batched queries.
    /// Missing relationship targets remain absent; query errors propagate.
    pub fn get_categories_with_relations(&self) -> PyResult<Vec<CategoryView>> {
        self.inner.get_categories_with_relations().map_err(db_error)
    }

    /// Queries all records from the `category` table.
    pub fn get_categories(&self) -> PyResult<Vec<Category>> {
        let res = self.inner.get_categories().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `category` table by its primary key `id`.
    pub fn get_category_by_id(&self, id: i32) -> PyResult<Option<Category>> {
        let res = self.inner.get_category_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `category` table.
    pub fn insert_category(&self, category: NewCategory) -> PyResult<Category> {
        let res = self.inner.insert_category(category).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `category` table.
    pub fn create_category(
        &self,
        menu_item_id: i32,
        seq: Option<i32>,
        is_visible: Option<i32>,
    ) -> PyResult<Category> {
        let res = self
            .inner
            .create_category(menu_item_id, seq, is_visible)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `category` table.
    pub fn update_category(&self, category: Category) -> PyResult<Category> {
        let res = self.inner.update_category(category).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `category` table by its primary key `id`.
    pub fn delete_category(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_category(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `color` table.
    pub fn get_colors(&self) -> PyResult<Vec<Color>> {
        let res = self.inner.get_colors().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `color` table by its primary key `id`.
    pub fn get_color_by_id(&self, id: i32) -> PyResult<Option<Color>> {
        let res = self.inner.get_color_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `color` table.
    pub fn insert_color(&self, color: NewColor) -> PyResult<Color> {
        let res = self.inner.insert_color(color).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `color` table.
    pub fn create_color(&self, name: &str) -> PyResult<Color> {
        let res = self.inner.create_color(name).map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `color` table.
    pub fn update_color(&self, color: Color) -> PyResult<Color> {
        let res = self.inner.update_color(color).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `color` table by its primary key `id`.
    pub fn delete_color(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_color(id).map_err(db_error)?;
        Ok(res)
    }

    /// Load a detached content snapshot with all supported direct relationships.
    /// Related rows are shallow snapshots; changes require explicit model updates.
    pub fn get_content_with_relations(&self, id: i32) -> PyResult<Option<ContentView>> {
        self.inner.get_content_with_relations(id).map_err(db_error)
    }

    /// Load all contents with direct relationships using batched queries.
    /// Missing relationship targets remain absent; query errors propagate.
    pub fn get_contents_with_relations(&self) -> PyResult<Vec<ContentView>> {
        self.inner.get_contents_with_relations().map_err(db_error)
    }

    /// Queries all records from the `color` table.
    pub fn get_contents(&self) -> PyResult<Vec<Content>> {
        let res = self.inner.get_contents().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `content` table by its primary key `id`.
    pub fn get_content_by_id(&self, id: i32) -> PyResult<Option<Content>> {
        let res = self.inner.get_content_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Find content by both stored signed source IDs, or None if absent or zero.
    /// Repeated pairs return the content with the lowest local export ID.
    pub fn get_content_by_master_content_id(
        &self,
        master_content_id: i32,
        master_db_id: i32,
    ) -> PyResult<Option<Content>> {
        self.inner
            .get_content_by_master_content_id(master_content_id, master_db_id)
            .map_err(db_error)
    }

    /// Load all matching contents for signed source-content IDs in one source library.
    /// Results use local export-ID order; missing/zero IDs are omitted and input
    /// duplicates do not repeat rows. Empty input or a zero library ID returns [].
    pub fn get_contents_by_master_content_ids(
        &self,
        master_content_ids: Vec<i32>,
        master_db_id: i32,
    ) -> PyResult<Vec<Content>> {
        self.inner
            .get_contents_by_master_content_ids(&master_content_ids, master_db_id)
            .map_err(db_error)
    }

    /// Inserts a new record into the `content` table.
    pub fn insert_content(&self, content: NewContent) -> PyResult<Content> {
        let res = self.inner.insert_content(content).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `content` table.
    pub fn create_content(&self, path: &str) -> PyResult<Content> {
        let res = self.inner.create_content(path).map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `content` table.
    pub fn update_content(&self, content: Content) -> PyResult<Content> {
        let res = self.inner.update_content(content).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `content` table by its primary key `id`.
    pub fn delete_content(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_content(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `color` table.
    pub fn get_cues(&self) -> PyResult<Vec<Cue>> {
        let res = self.inner.get_cues().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `cue` table by its primary key `id`.
    pub fn get_cue_by_id(&self, id: i32) -> PyResult<Option<Cue>> {
        let res = self.inner.get_cue_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `cue` table.
    pub fn update_cue(&self, cue: Cue) -> PyResult<Cue> {
        let res = self.inner.update_cue(cue).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `cue` table by its primary key `id`.
    pub fn delete_cue(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_cue(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `genre` table.
    pub fn get_genres(&self) -> PyResult<Vec<Genre>> {
        let res = self.inner.get_genres().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `genre` table by its primary key `id`.
    pub fn get_genre_by_id(&self, id: i32) -> PyResult<Option<Genre>> {
        let res = self.inner.get_genre_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `genre` table by its `name`.
    pub fn get_genre_by_name(&self, name: &str) -> PyResult<Option<Genre>> {
        let res = self.inner.get_genre_by_name(name).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `genre` table.
    pub fn insert_genre(&self, genre: NewGenre) -> PyResult<Genre> {
        let res = self.inner.insert_genre(genre).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `genre` table.
    pub fn create_genre(&self, name: &str) -> PyResult<Genre> {
        let res = self.inner.create_genre(name).map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `genre` table.
    pub fn update_genre(&self, genre: Genre) -> PyResult<Genre> {
        let res = self.inner.update_genre(genre).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `genre` table by its primary key `id`.
    pub fn delete_genre(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_genre(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `history` table.
    pub fn get_histories(&self) -> PyResult<Vec<History>> {
        let res = self.inner.get_histories().map_err(db_error)?;
        Ok(res)
    }

    /// Queries all child records from the `history` table by parent `id`.
    pub fn get_history_children(&self, id: i32) -> PyResult<Vec<History>> {
        let res = self.inner.get_history_children(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all `content` records associated with a `history` record.
    pub fn get_history_contents(&self, id: i32) -> PyResult<Vec<Content>> {
        let res = self.inner.get_history_contents(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `history` table by its primary key `id`.
    pub fn get_history_by_id(&self, id: i32) -> PyResult<Option<History>> {
        let res = self.inner.get_history_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `history` table.
    pub fn insert_history(&self, history: NewHistory) -> PyResult<History> {
        let res = self.inner.insert_history(history).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `history` table.
    pub fn create_history(&self, name: &str, attribute: HistoryType) -> PyResult<History> {
        let res = self
            .inner
            .create_history(name, attribute)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `history` table by its primary key `id`.
    pub fn delete_history(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_history(id).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `history_content` table.
    pub fn create_history_content(
        &self,
        history_id: i32,
        content_id: i32,
    ) -> PyResult<HistoryContent> {
        let res = self
            .inner
            .create_history_content(history_id, content_id)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `history_content` table by its primary key.
    pub fn delete_history_content(&self, history_id: i32, content_id: i32) -> PyResult<usize> {
        let res = self
            .inner
            .delete_history_content(history_id, content_id)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `color` table.
    pub fn get_hot_cue_bank_lists(&self) -> PyResult<Vec<HotCueBankList>> {
        let res = self.inner.get_hot_cue_bank_lists().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `hot_cue_bank_list` table by its primary key `id`.
    pub fn get_hot_cue_bank_list_by_id(&self, id: i32) -> PyResult<Option<HotCueBankList>> {
        let res = self
            .inner
            .get_hot_cue_bank_list_by_id(id)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `image` table.
    pub fn get_images(&self) -> PyResult<Vec<Image>> {
        let res = self.inner.get_images().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `image` table by its primary key `id`.
    pub fn get_image_by_id(&self, id: i32) -> PyResult<Option<Image>> {
        let res = self.inner.get_image_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `image` table by its `path`.
    pub fn get_image_by_path(&self, path: &str) -> PyResult<Option<Image>> {
        let res = self.inner.get_image_by_path(path).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `image` table.
    pub fn insert_image(&self, image: NewImage) -> PyResult<Image> {
        let res = self.inner.insert_image(image).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `image` table.
    pub fn create_image(&self, path: &str) -> PyResult<Image> {
        let res = self.inner.create_image(path).map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `image` table.
    pub fn update_image(&self, image: Image) -> PyResult<Image> {
        let res = self.inner.update_image(image).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `image` table by its primary key `id`.
    pub fn delete_image(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_image(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `key` table.
    pub fn get_keys(&self) -> PyResult<Vec<Key>> {
        let res = self.inner.get_keys().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `key` table by its primary key `id`.
    pub fn get_key_by_id(&self, id: i32) -> PyResult<Option<Key>> {
        let res = self.inner.get_key_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `key` table by its `name`.
    pub fn get_key_by_name(&self, name: &str) -> PyResult<Option<Key>> {
        let res = self.inner.get_key_by_name(name).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `key` table.
    pub fn insert_key(&self, key: NewKey) -> PyResult<Key> {
        let res = self.inner.insert_key(key).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `key` table.
    pub fn create_key(&self, name: &str) -> PyResult<Key> {
        let res = self.inner.create_key(name).map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `key` table.
    pub fn update_key(&self, key: Key) -> PyResult<Key> {
        let res = self.inner.update_key(key).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `key` table by its primary key `id`.
    pub fn delete_key(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_key(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `label` table.
    pub fn get_labels(&self) -> PyResult<Vec<Label>> {
        let res = self.inner.get_labels().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `label` table by its primary key `id`.
    pub fn get_label_by_id(&self, id: i32) -> PyResult<Option<Label>> {
        let res = self.inner.get_label_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `label` table by its `name`.
    pub fn get_label_by_name(&self, name: &str) -> PyResult<Option<Label>> {
        let res = self.inner.get_label_by_name(name).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `label` table.
    pub fn insert_label(&self, label: NewLabel) -> PyResult<Label> {
        let res = self.inner.insert_label(label).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `label` table.
    pub fn create_label(&self, name: &str) -> PyResult<Label> {
        let res = self.inner.create_label(name).map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `label` table.
    pub fn update_label(&self, label: Label) -> PyResult<Label> {
        let res = self.inner.update_label(label).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `label` table by its primary key `id`.
    pub fn delete_label(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_label(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `menuItem` table.
    pub fn get_menu_items(&self) -> PyResult<Vec<MenuItem>> {
        let res = self.inner.get_menu_items().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `menuItem` table by its primary key `id`.
    pub fn get_menu_item_by_id(&self, id: i32) -> PyResult<Option<MenuItem>> {
        let res = self.inner.get_menu_item_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `menuItem` table by its `name`.
    pub fn get_menu_item_by_name(&self, name: &str) -> PyResult<Option<MenuItem>> {
        let res = self.inner.get_menu_item_by_name(name).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `menuItem` table.
    pub fn insert_menu_item(&self, menu_item: NewMenuItem) -> PyResult<MenuItem> {
        let res = self.inner.insert_menu_item(menu_item).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `menuItem` table.
    pub fn create_menu_item(&self, name: &str, kind: i32) -> PyResult<MenuItem> {
        let res = self.inner.create_menu_item(name, kind).map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `menuItem` table.
    pub fn update_menu_item(&self, menu_item: MenuItem) -> PyResult<MenuItem> {
        let res = self.inner.update_menu_item(menu_item).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `menuItem` table by its primary key `id`.
    pub fn delete_menu_item(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_menu_item(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `myTag` table.
    pub fn get_my_tags(&self) -> PyResult<Vec<MyTag>> {
        let res = self.inner.get_my_tags().map_err(db_error)?;
        Ok(res)
    }

    /// Queries all child records from the `myTag` table by parent `id`.
    pub fn get_my_tag_children(&self, id: i32) -> PyResult<Vec<MyTag>> {
        let res = self.inner.get_my_tag_children(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all `content` records associated with a `myTag` record.
    pub fn get_my_tag_contents(&self, id: i32) -> PyResult<Vec<Content>> {
        let res = self.inner.get_my_tag_contents(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `myTag` table by its primary key `id`.
    pub fn get_my_tag_by_id(&self, id: i32) -> PyResult<Option<MyTag>> {
        let res = self.inner.get_my_tag_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `myTag` table their associated `content_id` via `myTag_content`
    pub fn get_my_tag_by_content_id(&self, cid: i32) -> PyResult<Vec<MyTag>> {
        let res = self.inner.get_my_tag_by_content_id(cid).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `myTag` table.
    pub fn insert_my_tag(&self, my_tag: NewMyTag) -> PyResult<MyTag> {
        let res = self.inner.insert_my_tag(my_tag).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new tag record and inserts it into the `myTag` table.
    pub fn create_my_tag(
        &self,
        name: &str,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> PyResult<MyTag> {
        let res = self
            .inner
            .create_my_tag(name, parent_id, seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new my_tag folder record and inserts it into the `myTag` table.
    pub fn create_my_tag_folder(
        &self,
        name: &str,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> PyResult<MyTag> {
        let res = self
            .inner
            .create_my_tag_folder(name, parent_id, seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Renames an existing my_tag record in the `myTag` table.
    pub fn rename_my_tag(&self, my_tag_id: i32, name: &str) -> PyResult<usize> {
        let res = self
            .inner
            .rename_my_tag(my_tag_id, name)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Moves a tag to a new parent folder or changes its sequence within the same parent.
    pub fn move_my_tag(
        &self,
        my_tag_id: i32,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> PyResult<usize> {
        let res = self
            .inner
            .move_my_tag(my_tag_id, parent_id.as_ref(), seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Moves multiple tags to a new parent folder or changes its sequence within the same parent.
    pub fn move_my_tags(
        &self,
        my_tag_ids: Vec<i32>,
        parent_id: i32,
        start_seq: Option<i32>,
    ) -> PyResult<usize> {
        let res = self
            .inner
            .move_my_tags(my_tag_ids.iter().map(|i| i).collect(), parent_id, start_seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `myTag` table by its primary key `id`.
    pub fn delete_my_tag(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_my_tag(id).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes multiple records from the `myTag` table by its primary keys `id`.
    pub fn delete_my_tags(&self, ids: Vec<i32>) -> PyResult<usize> {
        let res = self
            .inner
            .delete_my_tags(ids.iter().map(|i| i).collect())
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `myTag_content` table by its primary key.
    pub fn delete_my_tag_content(&self, my_tag_id: i32, content_id: i32) -> PyResult<usize> {
        let res = self
            .inner
            .delete_my_tag_content(my_tag_id, content_id)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `myTag_content` table by its primary key.
    pub fn delete_my_tag_contents(&self, ids: Vec<(i32, i32)>) -> PyResult<usize> {
        let res = self
            .inner
            .delete_my_tag_contents(ids.iter().map(|i| i).collect())
            .map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `playlist` table.
    pub fn get_playlists(&self) -> PyResult<Vec<Playlist>> {
        let res = self.inner.get_playlists().map_err(db_error)?;
        Ok(res)
    }

    /// Queries all child records from the `playlist` table by parent `id`.
    pub fn get_playlist_children(&self, id: i32) -> PyResult<Vec<Playlist>> {
        let res = self.inner.get_playlist_children(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all `content` records associated with a `playlist` record.
    pub fn get_playlist_contents(&self, id: i32) -> PyResult<Vec<Content>> {
        let res = self.inner.get_playlist_contents(id).map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `playlist` table by its primary key `id`.
    pub fn get_playlist_by_id(&self, id: i32) -> PyResult<Option<Playlist>> {
        let res = self.inner.get_playlist_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `playlist` table.
    pub fn insert_playlist(&self, playlist: NewPlaylist) -> PyResult<Playlist> {
        let res = self.inner.insert_playlist(playlist).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new playlist record and inserts it into the `playlist` table.
    pub fn create_playlist(
        &self,
        name: &str,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> PyResult<Playlist> {
        let res = self
            .inner
            .create_playlist(name, parent_id, seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new playlist folder record and inserts it into the `playlist` table.
    pub fn create_playlist_folder(
        &self,
        name: &str,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> PyResult<Playlist> {
        let res = self
            .inner
            .create_playlist_folder(name, parent_id, seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Renames an existing playlist record in the `playlist` table.
    pub fn rename_playlist(&self, playlist_id: i32, name: &str) -> PyResult<usize> {
        let res = self
            .inner
            .rename_playlist(playlist_id, name)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Moves a playlist to a new parent folder or changes its sequence within the same parent.
    pub fn move_playlist(
        &self,
        playlist_id: i32,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> PyResult<usize> {
        let res = self
            .inner
            .move_playlist(playlist_id, parent_id.as_ref(), seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Moves multiple playlists to a new parent folder or changes its sequence within the same parent.
    pub fn move_playlists(
        &self,
        playlist_ids: Vec<i32>,
        parent_id: i32,
        start_seq: Option<i32>,
    ) -> PyResult<usize> {
        let res = self
            .inner
            .move_playlists(
                playlist_ids.iter().map(|i| i).collect(),
                parent_id,
                start_seq,
            )
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `playlist` table by its primary key `id`.
    pub fn delete_playlist(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_playlist(id).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes multiple records from the `playlist` table by its primary keys `id`.
    pub fn delete_playlists(&self, ids: Vec<i32>) -> PyResult<usize> {
        let res = self
            .inner
            .delete_playlists(ids.iter().map(|i| i).collect())
            .map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `playlist_content` table.
    /// Positions start at one; an omitted position appends.
    pub fn create_playlist_content(
        &self,
        playlist_id: i32,
        content_id: i32,
        seq: Option<i32>,
    ) -> PyResult<PlaylistContent> {
        let res = self
            .inner
            .create_playlist_content(playlist_id, content_id, seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Moves a playlist content to a new sequence number.
    /// Positions start at one; an omitted position moves to the end.
    pub fn move_playlist_content(
        &self,
        playlist_id: i32,
        content_id: i32,
        seq: Option<i32>,
    ) -> PyResult<usize> {
        let res = self
            .inner
            .move_playlist_content(playlist_id, content_id, seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Moves multiple playlist contents to a new sequence number.
    /// The one-based destination is measured after removing the selected entries;
    /// an omitted position appends, preserving the supplied entry order.
    pub fn move_playlist_contents(
        &self,
        ids: Vec<(i32, i32)>,
        start_seq: Option<i32>,
    ) -> PyResult<usize> {
        let res = self
            .inner
            .move_playlist_contents(ids.iter().map(|i| i).collect(), start_seq)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `playlist_content` table by its primary key.
    pub fn delete_playlist_content(&self, playlist_id: i32, content_id: i32) -> PyResult<usize> {
        let res = self
            .inner
            .delete_playlist_content(playlist_id, content_id)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `playlist_content` table by its primary key.
    pub fn delete_playlist_contents(&self, ids: Vec<(i32, i32)>) -> PyResult<usize> {
        let res = self
            .inner
            .delete_playlist_contents(ids.iter().map(|i| i).collect())
            .map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `property` table.
    pub fn get_properties(&self) -> PyResult<Vec<Property>> {
        let res = self.inner.get_properties().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `property` table by its primary key `device_name`.
    pub fn get_property_by_device(&self, device_name: &str) -> PyResult<Option<Property>> {
        let res = self
            .inner
            .get_property_by_device(device_name)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `property` table by its primary key `device_name`.
    pub fn get_first_property(&self) -> PyResult<Option<Property>> {
        let res = self.inner.get_first_property().map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `property` table.
    pub fn insert_property(&self, property: Property) -> PyResult<Property> {
        let res = self.inner.insert_property(property).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `property` table.
    pub fn create_property(
        &self,
        device_name: &str,
        my_tag_master_dbid: i32,
    ) -> PyResult<Property> {
        let res = self
            .inner
            .create_property(device_name, my_tag_master_dbid)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `property` table.
    pub fn update_property(&self, property: Property) -> PyResult<Property> {
        let res = self.inner.update_property(property).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `property` table by its primary key `device_name`.
    pub fn delete_property(&self, device_name: &str) -> PyResult<usize> {
        let res = self.inner.delete_property(device_name).map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `recommendLike` table.
    pub fn get_recommend_likes(&self) -> PyResult<Vec<RecommendedLike>> {
        let res = self.inner.get_recommend_likes().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `recommendLike` table by its primary key `id`.
    pub fn get_recommend_like_by_id(
        &self,
        content_id_1: i32,
        content_id_2: i32,
    ) -> PyResult<Option<RecommendedLike>> {
        let res = self
            .inner
            .get_recommend_like_by_id(content_id_1, content_id_2)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `recommendedLike` table that are linked to the given `content`.
    pub fn get_recommend_like_by_content_id(
        &self,
        content_id: i32,
    ) -> PyResult<Vec<RecommendedLike>> {
        let res = self
            .inner
            .get_recommend_like_by_content_id(content_id)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Get related `content` ids for a given `content`.
    pub fn get_recommend_like_contents(&self, content_id: i32) -> PyResult<Vec<i32>> {
        let res = self
            .inner
            .get_recommend_like_contents(content_id)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `recommendLike` table.
    pub fn insert_recommend_like(
        &self,
        recommend_like: RecommendedLike,
    ) -> PyResult<RecommendedLike> {
        let res = self
            .inner
            .insert_recommend_like(recommend_like)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `recommendLike` table.
    pub fn create_recommend_like(
        &self,
        content_id_1: i32,
        content_id_2: i32,
        rating: Option<i32>,
    ) -> PyResult<RecommendedLike> {
        let res = self
            .inner
            .create_recommend_like(content_id_1, content_id_2, rating)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `recommendLike` table.
    pub fn update_recommend_like(
        &self,
        recommend_like: RecommendedLike,
    ) -> PyResult<RecommendedLike> {
        let res = self
            .inner
            .update_recommend_like(recommend_like)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `recommendLike` table by its primary key.
    pub fn delete_recommend_like(&self, content_id_1: i32, content_id_2: i32) -> PyResult<usize> {
        let res = self
            .inner
            .delete_recommend_like(content_id_1, content_id_2)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `recommendLike` table by its primary key.
    pub fn delete_recommend_like_by_content(&self, content_id: i32) -> PyResult<usize> {
        let res = self
            .inner
            .delete_recommend_like_by_content(content_id)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Queries all records from the `sort` table.
    pub fn get_sorts(&self) -> PyResult<Vec<Sort>> {
        let res = self.inner.get_sorts().map_err(db_error)?;
        Ok(res)
    }

    /// Queries a record from the `sort` table by its primary key `id`.
    pub fn get_sort_by_id(&self, id: i32) -> PyResult<Option<Sort>> {
        let res = self.inner.get_sort_by_id(id).map_err(db_error)?;
        Ok(res)
    }

    /// Inserts a new record into the `sort` table.
    pub fn insert_sort(&self, sort: NewSort) -> PyResult<Sort> {
        let res = self.inner.insert_sort(sort).map_err(db_error)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `sort` table.
    pub fn create_sort(
        &self,
        menu_item_id: i32,
        seq: Option<i32>,
        is_visible: Option<i32>,
    ) -> PyResult<Sort> {
        let res = self
            .inner
            .create_sort(menu_item_id, seq, is_visible)
            .map_err(db_error)?;
        Ok(res)
    }

    /// Updates an existing record in the `sort` table.
    pub fn update_sort(&self, sort: Sort) -> PyResult<Sort> {
        let res = self.inner.update_sort(sort).map_err(db_error)?;
        Ok(res)
    }

    /// Deletes a record from the `sort` table by its primary key `id`.
    pub fn delete_sort(&self, id: i32) -> PyResult<usize> {
        let res = self.inner.delete_sort(id).map_err(db_error)?;
        Ok(res)
    }
}
