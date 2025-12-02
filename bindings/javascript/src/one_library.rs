use napi::Result;
use napi_derive::napi;

use rbox::one_library::{models::*, OneLibrary};

#[napi(js_name = "OneLibrary")]
pub struct JsOneLibrary {
    inner: OneLibrary,
}

#[napi]
impl JsOneLibrary {
    /// Open a OneLibrary (Device Library Plus) database specified by path.
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        Ok(Self {
            inner: OneLibrary::new(path.as_str())?,
        })
    }

    /// Open a OneLibrary (Device Library Plus) database specified by path.
    #[napi(factory)]
    pub fn create(path: String, my_tag_master_dbid: i32) -> Result<Self> {
        Ok(Self {
            inner: OneLibrary::create(path.as_str(), my_tag_master_dbid)?,
        })
    }

    /// Create a new OneLibrary database if it does not exist, otherwise return the existing one.
    #[napi(factory)]
    pub fn create_if_not_exists(path: String, my_tag_master_dbid: i32) -> Result<Self> {
        Ok(Self {
            inner: OneLibrary::create_if_not_exists(path.as_str(), my_tag_master_dbid)?,
        })
    }

    /// Queries all records from the `album` table.
    pub fn get_albums(&self) -> Result<Vec<Album>> {
        let res = self.inner.get_albums()?;
        Ok(res)
    }

    /// Queries a record from the `album` table by its primary key `id`.
    pub fn get_album_by_id(&self, id: i32) -> Result<Option<Album>> {
        let res = self.inner.get_album_by_id(id)?;
        Ok(res)
    }

    /// Queries a record from the `album` table by its `name`.
    pub fn get_album_by_name(&self, name: &str) -> Result<Option<Album>> {
        let res = self.inner.get_album_by_name(name)?;
        Ok(res)
    }

    /// Inserts a new record into the `album` table.
    pub fn insert_album(&self, album: NewAlbum) -> Result<Album> {
        let res = self.inner.insert_album(album)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `album` table.
    pub fn create_album(
        &self,
        name: &str,
        artist_id: Option<i32>,
        image_id: Option<i32>,
    ) -> Result<Album> {
        let res = self.inner.create_album(name, artist_id, image_id)?;
        Ok(res)
    }

    /// Updates an existing record in the `album` table.
    pub fn update_album(&self, album: Album) -> Result<Album> {
        let res = self.inner.update_album(album)?;
        Ok(res)
    }

    /// Deletes a record from the `album` table by its primary key `id`.
    pub fn delete_album(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_album(id)?;
        Ok(res)
    }

    /// Queries all records from the `artist` table.
    pub fn get_artists(&self) -> Result<Vec<Artist>> {
        let res = self.inner.get_artists()?;
        Ok(res)
    }

    /// Queries a record from the `artist` table by its primary key `id`.
    pub fn get_artist_by_id(&self, id: i32) -> Result<Option<Artist>> {
        let res = self.inner.get_artist_by_id(id)?;
        Ok(res)
    }

    /// Queries a record from the `artist` table by its `name`.
    pub fn get_artist_by_name(&self, name: &str) -> Result<Option<Artist>> {
        let res = self.inner.get_artist_by_name(name)?;
        Ok(res)
    }

    /// Inserts a new record into the `artist` table.
    pub fn insert_artist(&self, artist: NewArtist) -> Result<Artist> {
        let res = self.inner.insert_artist(artist)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `artist` table.
    pub fn create_artist(&self, name: &str) -> Result<Artist> {
        let res = self.inner.create_artist(name)?;
        Ok(res)
    }

    /// Updates an existing record in the `artist` table.
    pub fn update_artist(&self, artist: Artist) -> Result<Artist> {
        let res = self.inner.update_artist(artist)?;
        Ok(res)
    }

    /// Deletes a record from the `artist` table by its primary key `id`.
    pub fn delete_artist(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_artist(id)?;
        Ok(res)
    }

    /// Queries all records from the `category` table.
    pub fn get_categories(&self) -> Result<Vec<Category>> {
        let res = self.inner.get_categories()?;
        Ok(res)
    }

    /// Queries a record from the `category` table by its primary key `id`.
    pub fn get_category_by_id(&self, id: i32) -> Result<Option<Category>> {
        let res = self.inner.get_category_by_id(id)?;
        Ok(res)
    }

    /// Inserts a new record into the `category` table.
    pub fn insert_category(&self, category: NewCategory) -> Result<Category> {
        let res = self.inner.insert_category(category)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `category` table.
    pub fn create_category(
        &self,
        menu_item_id: i32,
        seq: Option<i32>,
        is_visible: Option<i32>,
    ) -> Result<Category> {
        let res = self.inner.create_category(menu_item_id, seq, is_visible)?;
        Ok(res)
    }

    /// Updates an existing record in the `category` table.
    pub fn update_category(&self, category: Category) -> Result<Category> {
        let res = self.inner.update_category(category)?;
        Ok(res)
    }

    /// Deletes a record from the `category` table by its primary key `id`.
    pub fn delete_category(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_category(id)?;
        Ok(res)
    }

    /// Queries all records from the `color` table.
    pub fn get_colors(&self) -> Result<Vec<Color>> {
        let res = self.inner.get_colors()?;
        Ok(res)
    }

    /// Queries a record from the `color` table by its primary key `id`.
    pub fn get_color_by_id(&self, id: i32) -> Result<Option<Color>> {
        let res = self.inner.get_color_by_id(id)?;
        Ok(res)
    }

    /// Inserts a new record into the `color` table.
    pub fn insert_color(&self, color: NewColor) -> Result<Color> {
        let res = self.inner.insert_color(color)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `color` table.
    pub fn create_color(&self, name: &str) -> Result<Color> {
        let res = self.inner.create_color(name)?;
        Ok(res)
    }

    /// Updates an existing record in the `color` table.
    pub fn update_color(&self, color: Color) -> Result<Color> {
        let res = self.inner.update_color(color)?;
        Ok(res)
    }

    /// Deletes a record from the `color` table by its primary key `id`.
    pub fn delete_color(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_color(id)?;
        Ok(res)
    }

    /// Queries all records from the `color` table.
    pub fn get_contents(&self) -> Result<Vec<Content>> {
        let res = self.inner.get_contents()?;
        Ok(res)
    }

    /// Queries a record from the `content` table by its primary key `id`.
    pub fn get_content_by_id(&self, id: i32) -> Result<Option<Content>> {
        let res = self.inner.get_content_by_id(id)?;
        Ok(res)
    }

    /// Inserts a new record into the `content` table.
    pub fn insert_content(&self, content: NewContent) -> Result<Content> {
        let res = self.inner.insert_content(content)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `content` table.
    pub fn create_content(&self, path: &str) -> Result<Content> {
        let res = self.inner.create_content(path)?;
        Ok(res)
    }

    /// Updates an existing record in the `content` table.
    pub fn update_content(&self, content: Content) -> Result<Content> {
        let res = self.inner.update_content(content)?;
        Ok(res)
    }

    /// Deletes a record from the `content` table by its primary key `id`.
    pub fn delete_content(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_content(id)?;
        Ok(res)
    }

    /// Queries all records from the `color` table.
    pub fn get_cues(&self) -> Result<Vec<Cue>> {
        let res = self.inner.get_cues()?;
        Ok(res)
    }

    /// Queries a record from the `cue` table by its primary key `id`.
    pub fn get_cue_by_id(&self, id: i32) -> Result<Option<Cue>> {
        let res = self.inner.get_cue_by_id(id)?;
        Ok(res)
    }

    /// Updates an existing record in the `cue` table.
    pub fn update_cue(&self, cue: Cue) -> Result<Cue> {
        let res = self.inner.update_cue(cue)?;
        Ok(res)
    }

    /// Deletes a record from the `cue` table by its primary key `id`.
    pub fn delete_cue(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_cue(id)?;
        Ok(res)
    }

    /// Queries all records from the `genre` table.
    pub fn get_genres(&self) -> Result<Vec<Genre>> {
        let res = self.inner.get_genres()?;
        Ok(res)
    }

    /// Queries a record from the `genre` table by its primary key `id`.
    pub fn get_genre_by_id(&self, id: i32) -> Result<Option<Genre>> {
        let res = self.inner.get_genre_by_id(id)?;
        Ok(res)
    }

    /// Queries a record from the `genre` table by its `name`.
    pub fn get_genre_by_name(&self, name: &str) -> Result<Option<Genre>> {
        let res = self.inner.get_genre_by_name(name)?;
        Ok(res)
    }

    /// Inserts a new record into the `genre` table.
    pub fn insert_genre(&self, genre: NewGenre) -> Result<Genre> {
        let res = self.inner.insert_genre(genre)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `genre` table.
    pub fn create_genre(&self, name: &str) -> Result<Genre> {
        let res = self.inner.create_genre(name)?;
        Ok(res)
    }

    /// Updates an existing record in the `genre` table.
    pub fn update_genre(&self, genre: Genre) -> Result<Genre> {
        let res = self.inner.update_genre(genre)?;
        Ok(res)
    }

    /// Deletes a record from the `genre` table by its primary key `id`.
    pub fn delete_genre(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_genre(id)?;
        Ok(res)
    }

    /// Queries all records from the `history` table.
    pub fn get_histories(&self) -> Result<Vec<History>> {
        let res = self.inner.get_histories()?;
        Ok(res)
    }

    /// Queries all child records from the `history` table by parent `id`.
    pub fn get_history_children(&self, id: i32) -> Result<Vec<History>> {
        let res = self.inner.get_history_children(id)?;
        Ok(res)
    }

    /// Queries all `content` records associated with a `history` record.
    pub fn get_history_contents(&self, id: i32) -> Result<Vec<Content>> {
        let res = self.inner.get_history_contents(id)?;
        Ok(res)
    }

    /// Queries a record from the `history` table by its primary key `id`.
    pub fn get_history_by_id(&self, id: i32) -> Result<Option<History>> {
        let res = self.inner.get_history_by_id(id)?;
        Ok(res)
    }

    /// Inserts a new record into the `history` table.
    pub fn insert_history(&self, history: NewHistory) -> Result<History> {
        let res = self.inner.insert_history(history)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `history` table.
    pub fn create_history(&self, name: &str) -> Result<History> {
        let res = self.inner.create_history(name)?;
        Ok(res)
    }

    /// Deletes a record from the `history` table by its primary key `id`.
    pub fn delete_history(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_history(id)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `history_content` table.
    pub fn create_history_content(
        &self,
        history_id: i32,
        content_id: i32,
    ) -> Result<HistoryContent> {
        let res = self.inner.create_history_content(history_id, content_id)?;
        Ok(res)
    }

    /// Deletes a record from the `history_content` table by its primary key.
    pub fn delete_history_content(&self, history_id: i32, content_id: i32) -> Result<usize> {
        let res = self.inner.delete_history_content(history_id, content_id)?;
        Ok(res)
    }

    /// Queries all records from the `color` table.
    pub fn get_hot_cue_bank_lists(&self) -> Result<Vec<HotCueBankList>> {
        let res = self.inner.get_hot_cue_bank_lists()?;
        Ok(res)
    }

    /// Queries a record from the `hot_cue_bank_list` table by its primary key `id`.
    pub fn get_hot_cue_bank_list_by_id(&self, id: i32) -> Result<Option<HotCueBankList>> {
        let res = self.inner.get_hot_cue_bank_list_by_id(id)?;
        Ok(res)
    }

    /// Queries all records from the `image` table.
    pub fn get_images(&self) -> Result<Vec<Image>> {
        let res = self.inner.get_images()?;
        Ok(res)
    }

    /// Queries a record from the `image` table by its primary key `id`.
    pub fn get_image_by_id(&self, id: i32) -> Result<Option<Image>> {
        let res = self.inner.get_image_by_id(id)?;
        Ok(res)
    }

    /// Queries a record from the `image` table by its `path`.
    pub fn get_image_by_path(&self, path: &str) -> Result<Option<Image>> {
        let res = self.inner.get_image_by_path(path)?;
        Ok(res)
    }

    /// Inserts a new record into the `image` table.
    pub fn insert_image(&self, image: NewImage) -> Result<Image> {
        let res = self.inner.insert_image(image)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `image` table.
    pub fn create_image(&self, path: &str) -> Result<Image> {
        let res = self.inner.create_image(path)?;
        Ok(res)
    }

    /// Updates an existing record in the `image` table.
    pub fn update_image(&self, image: Image) -> Result<Image> {
        let res = self.inner.update_image(image)?;
        Ok(res)
    }

    /// Deletes a record from the `image` table by its primary key `id`.
    pub fn delete_image(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_image(id)?;
        Ok(res)
    }

    /// Queries all records from the `key` table.
    pub fn get_keys(&self) -> Result<Vec<Key>> {
        let res = self.inner.get_keys()?;
        Ok(res)
    }

    /// Queries a record from the `key` table by its primary key `id`.
    pub fn get_key_by_id(&self, id: i32) -> Result<Option<Key>> {
        let res = self.inner.get_key_by_id(id)?;
        Ok(res)
    }

    /// Queries a record from the `key` table by its `name`.
    pub fn get_key_by_name(&self, name: &str) -> Result<Option<Key>> {
        let res = self.inner.get_key_by_name(name)?;
        Ok(res)
    }

    /// Inserts a new record into the `key` table.
    pub fn insert_key(&self, key: NewKey) -> Result<Key> {
        let res = self.inner.insert_key(key)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `key` table.
    pub fn create_key(&self, name: &str) -> Result<Key> {
        let res = self.inner.create_key(name)?;
        Ok(res)
    }

    /// Updates an existing record in the `key` table.
    pub fn update_key(&self, key: Key) -> Result<Key> {
        let res = self.inner.update_key(key)?;
        Ok(res)
    }

    /// Deletes a record from the `key` table by its primary key `id`.
    pub fn delete_key(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_key(id)?;
        Ok(res)
    }

    /// Queries all records from the `label` table.
    pub fn get_labels(&self) -> Result<Vec<Label>> {
        let res = self.inner.get_labels()?;
        Ok(res)
    }

    /// Queries a record from the `label` table by its primary key `id`.
    pub fn get_label_by_id(&self, id: i32) -> Result<Option<Label>> {
        let res = self.inner.get_label_by_id(id)?;
        Ok(res)
    }

    /// Queries a record from the `label` table by its `name`.
    pub fn get_label_by_name(&self, name: &str) -> Result<Option<Label>> {
        let res = self.inner.get_label_by_name(name)?;
        Ok(res)
    }

    /// Inserts a new record into the `label` table.
    pub fn insert_label(&self, label: NewLabel) -> Result<Label> {
        let res = self.inner.insert_label(label)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `label` table.
    pub fn create_label(&self, name: &str) -> Result<Label> {
        let res = self.inner.create_label(name)?;
        Ok(res)
    }

    /// Updates an existing record in the `label` table.
    pub fn update_label(&self, label: Label) -> Result<Label> {
        let res = self.inner.update_label(label)?;
        Ok(res)
    }

    /// Deletes a record from the `label` table by its primary key `id`.
    pub fn delete_label(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_label(id)?;
        Ok(res)
    }

    /// Queries all records from the `menuItem` table.
    pub fn get_menu_items(&self) -> Result<Vec<MenuItem>> {
        let res = self.inner.get_menu_items()?;
        Ok(res)
    }

    /// Queries a record from the `menuItem` table by its primary key `id`.
    pub fn get_menu_item_by_id(&self, id: i32) -> Result<Option<MenuItem>> {
        let res = self.inner.get_menu_item_by_id(id)?;
        Ok(res)
    }

    /// Queries a record from the `menuItem` table by its `name`.
    pub fn get_menu_item_by_name(&self, name: &str) -> Result<Option<MenuItem>> {
        let res = self.inner.get_menu_item_by_name(name)?;
        Ok(res)
    }

    /// Inserts a new record into the `menuItem` table.
    pub fn insert_menu_item(&self, menu_item: NewMenuItem) -> Result<MenuItem> {
        let res = self.inner.insert_menu_item(menu_item)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `menuItem` table.
    pub fn create_menu_item(&self, name: &str, kind: i32) -> Result<MenuItem> {
        let res = self.inner.create_menu_item(name, kind)?;
        Ok(res)
    }

    /// Updates an existing record in the `menuItem` table.
    pub fn update_menu_item(&self, menu_item: MenuItem) -> Result<MenuItem> {
        let res = self.inner.update_menu_item(menu_item)?;
        Ok(res)
    }

    /// Deletes a record from the `menuItem` table by its primary key `id`.
    pub fn delete_menu_item(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_menu_item(id)?;
        Ok(res)
    }

    /// Queries all records from the `myTag` table.
    pub fn get_my_tags(&self) -> Result<Vec<MyTag>> {
        let res = self.inner.get_my_tags()?;
        Ok(res)
    }

    /// Queries all child records from the `myTag` table by parent `id`.
    pub fn get_my_tag_children(&self, id: i32) -> Result<Vec<MyTag>> {
        let res = self.inner.get_my_tag_children(id)?;
        Ok(res)
    }

    /// Queries all `content` records associated with a `myTag` record.
    pub fn get_my_tag_contents(&self, id: i32) -> Result<Vec<Content>> {
        let res = self.inner.get_my_tag_contents(id)?;
        Ok(res)
    }

    /// Queries a record from the `myTag` table by its primary key `id`.
    pub fn get_my_tag_by_id(&self, id: i32) -> Result<Option<MyTag>> {
        let res = self.inner.get_my_tag_by_id(id)?;
        Ok(res)
    }

    /// Queries all records from the `myTag` table their associated `content_id` via `myTag_content`
    pub fn get_my_tag_by_content_id(&self, cid: i32) -> Result<Vec<MyTag>> {
        let res = self.inner.get_my_tag_by_content_id(cid)?;
        Ok(res)
    }

    /// Inserts a new record into the `myTag` table.
    pub fn insert_my_tag(&self, my_tag: NewMyTag) -> Result<MyTag> {
        let res = self.inner.insert_my_tag(my_tag)?;
        Ok(res)
    }

    /// Creates a new tag record and inserts it into the `myTag` table.
    pub fn create_my_tag(
        &self,
        name: &str,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<MyTag> {
        let res = self.inner.create_my_tag(name, parent_id, seq)?;
        Ok(res)
    }

    /// Creates a new my_tag folder record and inserts it into the `myTag` table.
    pub fn create_my_tag_folder(
        &self,
        name: &str,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<MyTag> {
        let res = self.inner.create_my_tag_folder(name, parent_id, seq)?;
        Ok(res)
    }

    /// Renames an existing my_tag record in the `myTag` table.
    pub fn rename_my_tag(&self, my_tag_id: i32, name: &str) -> Result<usize> {
        let res = self.inner.rename_my_tag(my_tag_id, name)?;
        Ok(res)
    }

    /// Moves a tag to a new parent folder or changes its sequence within the same parent.
    pub fn move_my_tag(
        &self,
        my_tag_id: i32,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<usize> {
        let res = self.inner.move_my_tag(my_tag_id, parent_id.as_ref(), seq)?;
        Ok(res)
    }

    /// Moves multiple tags to a new parent folder or changes its sequence within the same parent.
    pub fn move_my_tags(
        &self,
        my_tag_ids: Vec<i32>,
        parent_id: i32,
        start_seq: Option<i32>,
    ) -> Result<usize> {
        let res = self.inner.move_my_tags(
            my_tag_ids.iter().map(|i| i).collect(),
            parent_id,
            start_seq,
        )?;
        Ok(res)
    }

    /// Deletes a record from the `myTag` table by its primary key `id`.
    pub fn delete_my_tag(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_my_tag(id)?;
        Ok(res)
    }

    /// Deletes multiple records from the `myTag` table by its primary keys `id`.
    pub fn delete_my_tags(&self, ids: Vec<i32>) -> Result<usize> {
        let res = self.inner.delete_my_tags(ids.iter().map(|i| i).collect())?;
        Ok(res)
    }

    /// Deletes a record from the `myTag_content` table by its primary key.
    pub fn delete_my_tag_content(&self, my_tag_id: i32, content_id: i32) -> Result<usize> {
        let res = self.inner.delete_my_tag_content(my_tag_id, content_id)?;
        Ok(res)
    }

    /// Deletes a record from the `myTag_content` table by its primary key.
    pub fn delete_my_tag_contents(&self, ids: Vec<(i32, i32)>) -> Result<usize> {
        let res = self
            .inner
            .delete_my_tag_contents(ids.iter().map(|i| i).collect())?;
        Ok(res)
    }

    /// Queries all records from the `playlist` table.
    pub fn get_playlists(&self) -> Result<Vec<Playlist>> {
        let res = self.inner.get_playlists()?;
        Ok(res)
    }

    /// Queries all child records from the `playlist` table by parent `id`.
    pub fn get_playlist_children(&self, id: i32) -> Result<Vec<Playlist>> {
        let res = self.inner.get_playlist_children(id)?;
        Ok(res)
    }

    /// Queries all `content` records associated with a `playlist` record.
    pub fn get_playlist_contents(&self, id: i32) -> Result<Vec<Content>> {
        let res = self.inner.get_playlist_contents(id)?;
        Ok(res)
    }

    /// Queries a record from the `playlist` table by its primary key `id`.
    pub fn get_playlist_by_id(&self, id: i32) -> Result<Option<Playlist>> {
        let res = self.inner.get_playlist_by_id(id)?;
        Ok(res)
    }

    /// Inserts a new record into the `playlist` table.
    pub fn insert_playlist(&self, playlist: NewPlaylist) -> Result<Playlist> {
        let res = self.inner.insert_playlist(playlist)?;
        Ok(res)
    }

    /// Creates a new playlist record and inserts it into the `playlist` table.
    pub fn create_playlist(
        &self,
        name: &str,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<Playlist> {
        let res = self.inner.create_playlist(name, parent_id, seq)?;
        Ok(res)
    }

    /// Creates a new playlist folder record and inserts it into the `playlist` table.
    pub fn create_playlist_folder(
        &self,
        name: &str,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<Playlist> {
        let res = self.inner.create_playlist_folder(name, parent_id, seq)?;
        Ok(res)
    }

    /// Renames an existing playlist record in the `playlist` table.
    pub fn rename_playlist(&self, playlist_id: i32, name: &str) -> Result<usize> {
        let res = self.inner.rename_playlist(playlist_id, name)?;
        Ok(res)
    }

    /// Moves a playlist to a new parent folder or changes its sequence within the same parent.
    pub fn move_playlist(
        &self,
        playlist_id: i32,
        parent_id: Option<i32>,
        seq: Option<i32>,
    ) -> Result<usize> {
        let res = self
            .inner
            .move_playlist(playlist_id, parent_id.as_ref(), seq)?;
        Ok(res)
    }

    /// Moves multiple playlists to a new parent folder or changes its sequence within the same parent.
    pub fn move_playlists(
        &self,
        playlist_ids: Vec<i32>,
        parent_id: i32,
        start_seq: Option<i32>,
    ) -> Result<usize> {
        let res = self.inner.move_playlists(
            playlist_ids.iter().map(|i| i).collect(),
            parent_id,
            start_seq,
        )?;
        Ok(res)
    }

    /// Deletes a record from the `playlist` table by its primary key `id`.
    pub fn delete_playlist(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_playlist(id)?;
        Ok(res)
    }

    /// Deletes multiple records from the `playlist` table by its primary keys `id`.
    pub fn delete_playlists(&self, ids: Vec<i32>) -> Result<usize> {
        let res = self
            .inner
            .delete_playlists(ids.iter().map(|i| i).collect())?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `playlist_content` table.
    pub fn create_playlist_content(
        &self,
        playlist_id: i32,
        content_id: i32,
        seq: Option<i32>,
    ) -> Result<PlaylistContent> {
        let res = self
            .inner
            .create_playlist_content(playlist_id, content_id, seq)?;
        Ok(res)
    }

    /// Moves a playlist content to a new sequence number.
    pub fn move_playlist_content(
        &self,
        playlist_id: i32,
        content_id: i32,
        seq: Option<i32>,
    ) -> Result<usize> {
        let res = self
            .inner
            .move_playlist_content(playlist_id, content_id, seq)?;
        Ok(res)
    }

    /// Moves multiple playlist contents to a new sequence number.
    pub fn move_playlist_contents(
        &self,
        ids: Vec<(i32, i32)>,
        start_seq: Option<i32>,
    ) -> Result<usize> {
        let res = self
            .inner
            .move_playlist_contents(ids.iter().map(|i| i).collect(), start_seq)?;
        Ok(res)
    }

    /// Deletes a record from the `playlist_content` table by its primary key.
    pub fn delete_playlist_content(&self, playlist_id: i32, content_id: i32) -> Result<usize> {
        let res = self
            .inner
            .delete_playlist_content(playlist_id, content_id)?;
        Ok(res)
    }

    /// Deletes a record from the `playlist_content` table by its primary key.
    pub fn delete_playlist_contents(&self, ids: Vec<(i32, i32)>) -> Result<usize> {
        let res = self
            .inner
            .delete_playlist_contents(ids.iter().map(|i| i).collect())?;
        Ok(res)
    }

    /// Queries all records from the `property` table.
    pub fn get_properties(&self) -> Result<Vec<Property>> {
        let res = self.inner.get_properties()?;
        Ok(res)
    }

    /// Queries a record from the `property` table by its primary key `device_name`.
    pub fn get_property_by_device(&self, device_name: &str) -> Result<Option<Property>> {
        let res = self.inner.get_property_by_device(device_name)?;
        Ok(res)
    }

    /// Queries a record from the `property` table by its primary key `device_name`.
    pub fn get_first_property(&self) -> Result<Option<Property>> {
        let res = self.inner.get_first_property()?;
        Ok(res)
    }

    /// Inserts a new record into the `property` table.
    pub fn insert_property(&self, property: Property) -> Result<Property> {
        let res = self.inner.insert_property(property)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `property` table.
    pub fn create_property(&self, device_name: &str, my_tag_master_dbid: i32) -> Result<Property> {
        let res = self
            .inner
            .create_property(device_name, my_tag_master_dbid)?;
        Ok(res)
    }

    /// Updates an existing record in the `property` table.
    pub fn update_property(&self, property: Property) -> Result<Property> {
        let res = self.inner.update_property(property)?;
        Ok(res)
    }

    /// Deletes a record from the `property` table by its primary key `device_name`.
    pub fn delete_property(&self, device_name: &str) -> Result<usize> {
        let res = self.inner.delete_property(device_name)?;
        Ok(res)
    }

    /// Queries all records from the `recommendLike` table.
    pub fn get_recommend_likes(&self) -> Result<Vec<RecommendedLike>> {
        let res = self.inner.get_recommend_likes()?;
        Ok(res)
    }

    /// Queries a record from the `recommendLike` table by its primary key `id`.
    pub fn get_recommend_like_by_id(
        &self,
        content_id_1: i32,
        content_id_2: i32,
    ) -> Result<Option<RecommendedLike>> {
        let res = self
            .inner
            .get_recommend_like_by_id(content_id_1, content_id_2)?;
        Ok(res)
    }

    /// Queries all records from the `recommendedLike` table that are linked to the given `content`.
    pub fn get_recommend_like_by_content_id(
        &self,
        content_id: i32,
    ) -> Result<Vec<RecommendedLike>> {
        let res = self.inner.get_recommend_like_by_content_id(content_id)?;
        Ok(res)
    }

    /// Get related `content` ids for a given `content`.
    pub fn get_recommend_like_contents(&self, content_id: i32) -> Result<Vec<i32>> {
        let res = self.inner.get_recommend_like_contents(content_id)?;
        Ok(res)
    }

    /// Inserts a new record into the `recommendLike` table.
    pub fn insert_recommend_like(
        &self,
        recommend_like: RecommendedLike,
    ) -> Result<RecommendedLike> {
        let res = self.inner.insert_recommend_like(recommend_like)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `recommendLike` table.
    pub fn create_recommend_like(
        &self,
        content_id_1: i32,
        content_id_2: i32,
        rating: Option<i32>,
    ) -> Result<RecommendedLike> {
        let res = self
            .inner
            .create_recommend_like(content_id_1, content_id_2, rating)?;
        Ok(res)
    }

    /// Updates an existing record in the `recommendLike` table.
    pub fn update_recommend_like(
        &self,
        recommend_like: RecommendedLike,
    ) -> Result<RecommendedLike> {
        let res = self.inner.update_recommend_like(recommend_like)?;
        Ok(res)
    }

    /// Deletes a record from the `recommendLike` table by its primary key.
    pub fn delete_recommend_like(&self, content_id_1: i32, content_id_2: i32) -> Result<usize> {
        let res = self
            .inner
            .delete_recommend_like(content_id_1, content_id_2)?;
        Ok(res)
    }

    /// Deletes a record from the `recommendLike` table by its primary key.
    pub fn delete_recommend_like_by_content(&self, content_id: i32) -> Result<usize> {
        let res = self.inner.delete_recommend_like_by_content(content_id)?;
        Ok(res)
    }

    /// Queries all records from the `sort` table.
    pub fn get_sorts(&self) -> Result<Vec<Sort>> {
        let res = self.inner.get_sorts()?;
        Ok(res)
    }

    /// Queries a record from the `sort` table by its primary key `id`.
    pub fn get_sort_by_id(&self, id: i32) -> Result<Option<Sort>> {
        let res = self.inner.get_sort_by_id(id)?;
        Ok(res)
    }

    /// Inserts a new record into the `sort` table.
    pub fn insert_sort(&self, sort: NewSort) -> Result<Sort> {
        let res = self.inner.insert_sort(sort)?;
        Ok(res)
    }

    /// Creates a new record and inserts it into the `sort` table.
    pub fn create_sort(
        &self,
        menu_item_id: i32,
        seq: Option<i32>,
        is_visible: Option<i32>,
    ) -> Result<Sort> {
        let res = self.inner.create_sort(menu_item_id, seq, is_visible)?;
        Ok(res)
    }

    /// Updates an existing record in the `sort` table.
    pub fn update_sort(&self, sort: Sort) -> Result<Sort> {
        let res = self.inner.update_sort(sort)?;
        Ok(res)
    }

    /// Deletes a record from the `sort` table by its primary key `id`.
    pub fn delete_sort(&self, id: i32) -> Result<usize> {
        let res = self.inner.delete_sort(id)?;
        Ok(res)
    }
}
