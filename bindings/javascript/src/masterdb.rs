// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use chrono::{DateTime, Utc};
use napi::Result;
use napi_derive::napi;
use rbox::masterdb::enums::PlaylistType;
use rbox::masterdb::{models::*, MasterDb};

use super::anlz::JsAnlzPaths;
use super::options::JsRekordboxOptions;

#[napi(object, js_name = "DjmdPlaylistTreeNode")]
pub struct JsDjmdPlaylistTreeNode {
    pub id: String,
    pub uuid: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub seq: i32,
    pub name: String,
    pub image_path: Option<String>,
    pub attribute: PlaylistType,
    pub parent_id: String,
    pub smart_list: Option<String>,
    pub children: Vec<JsDjmdPlaylistTreeNode>,
}

impl Into<JsDjmdPlaylistTreeNode> for DjmdPlaylistTreeNode {
    fn into(self) -> JsDjmdPlaylistTreeNode {
        let children = self
            .children
            .into_iter()
            .map(|c| c.borrow().clone().into())
            .collect();
        JsDjmdPlaylistTreeNode {
            id: self.id,
            uuid: self.uuid,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            seq: self.seq,
            name: self.name,
            image_path: self.image_path,
            attribute: self.attribute,
            parent_id: self.parent_id,
            smart_list: self.smart_list,
            children,
        }
    }
}

#[napi(js_name = "MasterDb")]
pub struct JsMasterDb {
    db: MasterDb,
}

#[napi]
impl JsMasterDb {
    /// Open a Rekordbox database specified by path.
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        Ok(JsMasterDb {
            db: MasterDb::new(path.as_str()).map_err(crate::util::binding_error)?,
        })
    }

    /// Open the Rekordbox database specified by the `RekordboxOptions`.
    #[napi(factory)]
    pub fn from_options(options: &JsRekordboxOptions) -> Result<Self> {
        Ok(JsMasterDb {
            db: MasterDb::from_options(&options.inner).map_err(crate::util::binding_error)?,
        })
    }

    /// Open the default Rekordbox `master.db` database.
    #[napi(factory)]
    pub fn open() -> Result<Self> {
        Ok(JsMasterDb {
            db: MasterDb::open().map_err(crate::util::binding_error)?,
        })
    }

    /// Sets the unsafe writes flag for the database.
    #[napi]
    pub fn set_unsafe_writes(&mut self, unsafe_writes: bool) {
        self.db.set_unsafe_writes(unsafe_writes);
    }
    /// Returns the path to the configured share directory.
    #[napi]
    pub fn share_directory(&self) -> Option<String> {
        self.db
            .share_directory()
            .map(|p| p.to_string_lossy().to_string())
    }

    /// Returns the path to the configured playlist XML file.
    #[napi]
    pub fn playlist_xml_path(&self) -> Option<String> {
        self.db
            .playlist_xml_path()
            .map(|p| p.to_string_lossy().to_string())
    }

    // -- AgentRegistry ----------------------------------------------------------------------------

    /// Retrieves all entries from the [`AgentRegistry`] table in the database.
    #[napi]
    pub fn get_agent_registries(&mut self) -> Result<Vec<AgentRegistry>> {
        let models = self
            .db
            .get_agent_registries()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves an `agentRegistry` entry by its unique identifier.
    #[napi]
    pub fn get_agent_registry_by_id(&mut self, id: String) -> Result<Option<AgentRegistry>> {
        let model = self
            .db
            .get_agent_registry_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves the local update sequence number (USN) from the `agentRegistry` table.
    #[napi]
    pub fn get_local_usn(&mut self) -> Result<i32> {
        Ok(self
            .db
            .get_local_usn()
            .map_err(crate::util::binding_error)?)
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    /// Retrieves all entries from the `cloudAgentRegistry` table in the database.
    #[napi]
    pub fn get_cloud_agent_registries(&mut self) -> Result<Vec<CloudAgentRegistry>> {
        let models = self
            .db
            .get_cloud_agent_registries()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `cloudAgentRegistry` entry by its unique identifier.
    #[napi]
    pub fn get_cloud_agent_registry_by_id(
        &mut self,
        id: String,
    ) -> Result<Option<CloudAgentRegistry>> {
        let model = self
            .db
            .get_cloud_agent_registry_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    /// Retrieves all entries from the `contentActiveCensor` table in the database.
    #[napi]
    pub fn get_content_active_censors(&mut self) -> Result<Vec<ContentActiveCensor>> {
        let models = self
            .db
            .get_content_active_censors()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `contentActiveCensor` entry by its unique identifier.
    #[napi]
    pub fn get_content_active_censor_by_id(
        &mut self,
        id: String,
    ) -> Result<Option<ContentActiveCensor>> {
        let model = self
            .db
            .get_content_active_censor_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- ContentCue -------------------------------------------------------------------------------

    /// Retrieves all entries from the `contentCue` table in the database.
    #[napi]
    pub fn get_content_cues(&mut self) -> Result<Vec<ContentCue>> {
        let models = self
            .db
            .get_content_cues()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `contentCue` entry by its unique identifier.
    #[napi]
    pub fn get_content_cue_by_id(&mut self, id: String) -> Result<Option<ContentCue>> {
        let model = self
            .db
            .get_content_cue_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- ContentFile ------------------------------------------------------------------------------

    /// Retrieves all entries from the `contentFile` table in the database.
    #[napi]
    pub fn get_content_files(&mut self) -> Result<Vec<ContentFile>> {
        let models = self
            .db
            .get_content_files()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `contentFile` entry by its unique identifier.
    #[napi]
    pub fn get_content_file_by_id(&mut self, id: String) -> Result<Option<ContentFile>> {
        let model = self
            .db
            .get_content_file_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdActiveCensor` table in the database.
    #[napi]
    pub fn get_active_censors(&mut self) -> Result<Vec<DjmdActiveCensor>> {
        let models = self
            .db
            .get_active_censors()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdActiveCensor` entry by its unique identifier.
    #[napi]
    pub fn get_active_censor_by_id(&mut self, id: String) -> Result<Option<DjmdActiveCensor>> {
        let model = self
            .db
            .get_active_censor_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- Album ------------------------------------------------------------------------------------

    /// Load a detached album snapshot with all supported direct relationships.
    /// Related rows are shallow snapshots; changes require explicit model updates.
    #[napi]
    pub fn get_album_with_relations(&self, id: String) -> Result<Option<DjmdAlbumView>> {
        self.db
            .get_album_with_relations(&id)
            .map_err(crate::util::binding_error)
    }

    /// Load all albums with direct relationships using batched queries.
    /// Missing relationship targets remain absent; query errors propagate.
    #[napi]
    pub fn get_albums_with_relations(&self) -> Result<Vec<DjmdAlbumView>> {
        self.db
            .get_albums_with_relations()
            .map_err(crate::util::binding_error)
    }

    /// Retrieves all entries from the `djmdAlbum` table in the database.
    #[napi]
    pub fn get_albums(&mut self) -> Result<Vec<DjmdAlbum>> {
        let models = self.db.get_albums().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdAlbum` entry by its unique identifier.
    #[napi]
    pub fn get_album_by_id(&mut self, id: String) -> Result<Option<DjmdAlbum>> {
        let model = self
            .db
            .get_album_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves a `djmdAlbum` entry by its name.
    #[napi]
    pub fn get_album_by_name(&mut self, name: String) -> Result<Option<DjmdAlbum>> {
        let model = self
            .db
            .get_album_by_name(name.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Inserts a new album into the `djmdAlbum` table in the database.
    #[napi]
    pub fn insert_album(&mut self, item: NewDjmdAlbum) -> Result<DjmdAlbum> {
        let model = self
            .db
            .insert_album(item)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Creates a new album and inserts it into the `djmdAlbum` table in the database.
    #[napi]
    pub fn create_album(
        &mut self,
        name: String,
        artist_id: Option<String>,
        image_path: Option<String>,
        compilation: Option<i32>,
    ) -> Result<DjmdAlbum> {
        let model = self
            .db
            .create_album(name, artist_id, image_path, compilation)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Updates an existing `djmdAlbum` entry in the database.
    #[napi]
    pub fn update_album(&mut self, mut model: DjmdAlbum) -> Result<DjmdAlbum> {
        let updated_model = self
            .db
            .update_album(&mut model)
            .map_err(crate::util::binding_error)?;
        Ok(updated_model)
    }

    /// Deletes an album entry from the `djmdAlbum` table in the database.
    #[napi]
    pub fn delete_album(&mut self, id: String) -> Result<()> {
        self.db
            .delete_album(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(())
    }

    // -- Artist -----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdArtist` table in the database.
    #[napi]
    pub fn get_artists(&mut self) -> Result<Vec<DjmdArtist>> {
        let models = self.db.get_artists().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdArtist` entry by its unique identifier.
    #[napi]
    pub fn get_artist_by_id(&mut self, id: String) -> Result<Option<DjmdArtist>> {
        let model = self
            .db
            .get_artist_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves a `djmdArtist` entry by its name.
    #[napi]
    pub fn get_artist_by_name(&mut self, name: String) -> Result<Option<DjmdArtist>> {
        let model = self
            .db
            .get_artist_by_name(name.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Inserts a new artist into the `djmdArtist` table in the database.
    #[napi]
    pub fn insert_artist(&mut self, item: NewDjmdArtist) -> Result<DjmdArtist> {
        let model = self
            .db
            .insert_artist(item)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Creates a new artist and inserts it into the `djmdArtist` table in the database.
    #[napi]
    pub fn create_artist(&mut self, name: String) -> Result<DjmdArtist> {
        let model = self
            .db
            .create_artist(name)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Updates an existing `djmdArtist` entry in the database.
    #[napi]
    pub fn update_artist(&mut self, mut model: DjmdArtist) -> Result<DjmdArtist> {
        let updated_model = self
            .db
            .update_artist(&mut model)
            .map_err(crate::util::binding_error)?;
        Ok(updated_model)
    }

    /// Deletes an artist entry from the `djmdArtist` table in the database.
    #[napi]
    pub fn delete_artist(&mut self, id: String) -> Result<()> {
        self.db
            .delete_artist(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(())
    }

    // -- Category ---------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdCategory` table in the database.
    #[napi]
    pub fn get_categories(&mut self) -> Result<Vec<DjmdCategory>> {
        let models = self
            .db
            .get_categories()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdCategory` entry by its unique identifier.
    #[napi]
    pub fn get_category_by_id(&mut self, id: String) -> Result<Option<DjmdCategory>> {
        let model = self
            .db
            .get_category_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- Color ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdColor` table in the database.
    #[napi]
    pub fn get_colors(&mut self) -> Result<Vec<DjmdColor>> {
        let models = self.db.get_colors().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdColor` entry by its unique identifier.
    #[napi]
    pub fn get_color_by_id(&mut self, id: String) -> Result<Option<DjmdColor>> {
        let model = self
            .db
            .get_color_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- Content ----------------------------------------------------------------------------------

    /// Load a detached content snapshot with all supported direct relationships.
    /// Related rows are shallow snapshots; changes require explicit model updates.
    #[napi]
    pub fn get_content_with_relations(&self, id: String) -> Result<Option<DjmdContentView>> {
        self.db
            .get_content_with_relations(&id)
            .map_err(crate::util::binding_error)
    }

    /// Load all contents with direct relationships using batched queries.
    /// Missing relationship targets remain absent; query errors propagate.
    #[napi]
    pub fn get_contents_with_relations(&self) -> Result<Vec<DjmdContentView>> {
        self.db
            .get_contents_with_relations()
            .map_err(crate::util::binding_error)
    }

    /// Retrieves all entries from the `djmdContent` table in the database.
    #[napi]
    pub fn get_contents(&mut self) -> Result<Vec<DjmdContent>> {
        let models = self.db.get_contents().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdContent` entry by its unique identifier.
    #[napi]
    pub fn get_content_by_id(&mut self, id: String) -> Result<Option<DjmdContent>> {
        let model = self
            .db
            .get_content_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves a `djmdContent` entry by its unique identifier.
    #[napi]
    pub fn get_contents_by_ids(&mut self, ids: Vec<String>) -> Result<Vec<DjmdContent>> {
        let ids: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let models = self
            .db
            .get_contents_by_ids(&ids)
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdContent` entry by its folder path.
    #[napi]
    pub fn get_content_by_path(&mut self, id: String) -> Result<Option<DjmdContent>> {
        let model = self
            .db
            .get_content_by_path(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Returns the path to the directory containing the ANLZxxxx.xxx files for a given content ID.
    #[napi]
    pub fn get_content_anlz_dir(&mut self, id: String) -> Result<Option<String>> {
        let dir = self
            .db
            .get_content_anlz_dir(id.as_str())
            .map_err(crate::util::binding_error)?;
        if let Some(dir) = dir {
            return Ok(Some(dir.as_os_str().to_str().unwrap().to_string()));
        }
        Ok(None)
    }

    /// Returns a struct containing the paths to ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 files.
    #[napi]
    pub fn get_content_anlz_paths(&mut self, id: String) -> Result<Option<JsAnlzPaths>> {
        let paths = self
            .db
            .get_content_anlz_paths(id.as_str())
            .map_err(crate::util::binding_error)?;
        if let Some(paths) = paths {
            return Ok(Some(JsAnlzPaths::from(paths)));
        }
        Ok(None)
    }

    /// Creates a new content entry and inserts it into the `djmdContent` table.
    #[napi]
    pub fn create_content(&mut self, path: String) -> Result<DjmdContent> {
        let model = self
            .db
            .create_content(path)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Inserts a configured content model into the `djmdContent` table.
    #[napi]
    pub fn add_content(&mut self, content: NewDjmdContent) -> Result<DjmdContent> {
        self.db
            .add_content(content)
            .map_err(crate::util::binding_error)
    }

    /// Updates an existing `djmdContent` entry in the database.
    #[napi]
    pub fn update_content(&mut self, mut model: DjmdContent) -> Result<i32> {
        let result = self
            .db
            .update_content(&mut model)
            .map_err(crate::util::binding_error)?;
        Ok(result as i32)
    }

    /// Update the content album field.
    #[napi]
    pub fn update_content_album(&mut self, id: String, name: String) -> Result<i32> {
        let result = self
            .db
            .update_content_album(&id, &name)
            .map_err(crate::util::binding_error)?;
        Ok(result as i32)
    }

    /// Update the content artist name.
    #[napi]
    pub fn update_content_artist(&mut self, id: String, name: String) -> Result<i32> {
        let result = self
            .db
            .update_content_artist(&id, &name)
            .map_err(crate::util::binding_error)?;
        Ok(result as i32)
    }

    /// Update the content remixer name.
    #[napi]
    pub fn update_content_remixer(&mut self, id: String, name: String) -> Result<i32> {
        let result = self
            .db
            .update_content_remixer(&id, &name)
            .map_err(crate::util::binding_error)?;
        Ok(result as i32)
    }

    /// Update the content original artist name.
    #[napi]
    pub fn update_content_original_artist(&mut self, id: String, name: String) -> Result<i32> {
        let result = self
            .db
            .update_content_original_artist(&id, &name)
            .map_err(crate::util::binding_error)?;
        Ok(result as i32)
    }

    /// Update the content composer name.
    #[napi]
    pub fn update_content_composer(&mut self, id: String, name: String) -> Result<i32> {
        let result = self
            .db
            .update_content_composer(&id, &name)
            .map_err(crate::util::binding_error)?;
        Ok(result as i32)
    }

    /// Update the content genre name.
    #[napi]
    pub fn update_content_genre(&mut self, id: String, name: String) -> Result<i32> {
        let result = self
            .db
            .update_content_genre(&id, &name)
            .map_err(crate::util::binding_error)?;
        Ok(result as i32)
    }

    /// Update the content label name.
    #[napi]
    pub fn update_content_label(&mut self, id: String, name: String) -> Result<i32> {
        let result = self
            .db
            .update_content_label(&id, &name)
            .map_err(crate::util::binding_error)?;
        Ok(result as i32)
    }

    /// Update the content key name.
    #[napi]
    pub fn update_content_key(&mut self, id: String, name: String) -> Result<i32> {
        let result = self
            .db
            .update_content_key(&id, &name)
            .map_err(crate::util::binding_error)?;
        Ok(result as i32)
    }

    /// Updates the file path of a content entry in the database.
    #[napi]
    pub fn update_content_path(&mut self, content_id: String, path: String) -> Result<u32> {
        let result = self
            .db
            .update_content_path(&content_id, &path)
            .map_err(crate::util::binding_error)?;
        Ok(result as u32)
    }

    // -- Cue --------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdCue` table.
    #[napi]
    pub fn get_cues(&mut self) -> Result<Vec<DjmdCue>> {
        let models = self.db.get_cues().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdCue` entry by its unique identifier.
    #[napi]
    pub fn get_cue_by_id(&mut self, id: String) -> Result<Option<DjmdCue>> {
        let model = self
            .db
            .get_cue_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- Device -----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdDevice` table.
    #[napi]
    pub fn get_devices(&mut self) -> Result<Vec<DjmdDevice>> {
        let models = self.db.get_devices().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdDevice` entry by its unique identifier.
    #[napi]
    pub fn get_device_by_id(&mut self, id: String) -> Result<Option<DjmdDevice>> {
        let model = self
            .db
            .get_device_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- Genre ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdGenre` table in the database.
    #[napi]
    pub fn get_genres(&mut self) -> Result<Vec<DjmdGenre>> {
        let models = self.db.get_genres().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdGenre` entry by its unique identifier.
    #[napi]
    pub fn get_genre_by_id(&mut self, id: String) -> Result<Option<DjmdGenre>> {
        let model = self
            .db
            .get_genre_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves a `djmdGenre` entry by its name.
    #[napi]
    pub fn get_genre_by_name(&mut self, name: String) -> Result<Option<DjmdGenre>> {
        let model = self
            .db
            .get_genre_by_name(name.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Inserts a new genre into the `djmdGenre` table in the database.
    #[napi]
    pub fn insert_genre(&mut self, genre: NewDjmdGenre) -> Result<DjmdGenre> {
        let model = self
            .db
            .insert_genre(genre)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Creates a new genre and inserts it into the `djmdGenre` table in the database.
    #[napi]
    pub fn create_genre(&mut self, name: String) -> Result<DjmdGenre> {
        let model = self
            .db
            .create_genre(name)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Updates an existing `djmdGenre` entry in the database.
    #[napi]
    pub fn update_genre(&mut self, mut model: DjmdGenre) -> Result<DjmdGenre> {
        let updated_model = self
            .db
            .update_genre(&mut model)
            .map_err(crate::util::binding_error)?;
        Ok(updated_model)
    }

    /// Deletes a genre entry from the `djmdGenre` table in the database.
    #[napi]
    pub fn delete_genre(&mut self, id: String) -> Result<()> {
        self.db
            .delete_genre(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(())
    }

    // -- History ----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdHistory` table in the database.
    #[napi]
    pub fn get_histories(&mut self) -> Result<Vec<DjmdHistory>> {
        let models = self
            .db
            .get_histories()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdHistory` entry by its unique identifier.
    #[napi]
    pub fn get_history_by_id(&mut self, id: String) -> Result<Option<DjmdHistory>> {
        let model = self
            .db
            .get_history_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves all song history entries for a given history ID, ordered by track number.
    #[napi]
    pub fn get_history_songs(&mut self, id: String) -> Result<Vec<DjmdSongHistory>> {
        let models = self
            .db
            .get_history_songs(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all content entries referenced by the song history for a given history ID.
    #[napi]
    pub fn get_history_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self
            .db
            .get_history_contents(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdHotCueBanklist` table in the database.
    #[napi]
    pub fn get_hot_cue_banklists(&mut self) -> Result<Vec<DjmdHotCueBanklist>> {
        let models = self
            .db
            .get_hot_cue_banklists()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdHotCueBanklist` entry by its unique identifier.
    #[napi]
    pub fn get_hot_cue_banklist_children(&mut self, id: String) -> Result<Vec<DjmdHotCueBanklist>> {
        let models = self
            .db
            .get_hot_cue_banklist_children(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all child `djmdHotCueBanklist` entries for a given parent ID, ordered by sequence.
    #[napi]
    pub fn get_hot_cue_banklist_by_id(&mut self, id: String) -> Result<Option<DjmdHotCueBanklist>> {
        let model = self
            .db
            .get_hot_cue_banklist_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves all `djmdSongHotCueBanklist` entries for a given hot cue banklist ID, ordered by track number.
    #[napi]
    pub fn get_hot_cue_banklist_songs(
        &mut self,
        id: String,
    ) -> Result<Vec<DjmdSongHotCueBanklist>> {
        let models = self
            .db
            .get_hot_cue_banklist_songs(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all `djmdContent` entries referenced by the song hot cue banklist for a given hot cue banklist ID.
    #[napi]
    pub fn get_hot_cue_banklist_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self
            .db
            .get_hot_cue_banklist_contents(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all `hotCueBanklistCue` entries for a given hot cue banklist ID.
    #[napi]
    pub fn get_hot_cue_banklist_cues(&mut self, id: String) -> Result<Vec<HotCueBanklistCue>> {
        let models = self
            .db
            .get_hot_cue_banklist_cues(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    // -- Key --------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdKey` table in the database.
    #[napi]
    pub fn get_keys(&mut self) -> Result<Vec<DjmdKey>> {
        let models = self.db.get_keys().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdKey` entry by its unique identifier.
    #[napi]
    pub fn get_key_by_id(&mut self, id: String) -> Result<Option<DjmdKey>> {
        let model = self
            .db
            .get_key_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves a `djmdKey` entry by its scale name.
    #[napi]
    pub fn get_key_by_name(&mut self, name: String) -> Result<Option<DjmdKey>> {
        let model = self
            .db
            .get_key_by_name(name.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Inserts a new key into the `djmdKey` table in the database.
    #[napi]
    pub fn insert_key(&mut self, key: NewDjmdKey) -> Result<DjmdKey> {
        let model = self
            .db
            .insert_key(key)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Creates a new key and inserts it into the `djmdKey` table in the database.
    #[napi]
    pub fn create_key(&mut self, name: String) -> Result<DjmdKey> {
        let model = self
            .db
            .create_key(name)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Updates an existing `djmdKey` entry in the database.
    #[napi]
    pub fn update_key(&mut self, mut model: DjmdKey) -> Result<DjmdKey> {
        let updated_model = self
            .db
            .update_key(&mut model)
            .map_err(crate::util::binding_error)?;
        Ok(updated_model)
    }

    /// Deletes a key entry from the `djmdKey` table in the database.
    #[napi]
    pub fn delete_key(&mut self, id: String) -> Result<()> {
        self.db
            .delete_key(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(())
    }

    // -- Label ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdLabel` table in the database.
    #[napi]
    pub fn get_labels(&mut self) -> Result<Vec<DjmdLabel>> {
        let models = self.db.get_labels().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdLabel` entry by its unique identifier.
    #[napi]
    pub fn get_label_by_id(&mut self, id: String) -> Result<Option<DjmdLabel>> {
        let model = self
            .db
            .get_label_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves a `djmdLabel` entry by its name.
    #[napi]
    pub fn get_label_by_name(&mut self, name: String) -> Result<Option<DjmdLabel>> {
        let model = self
            .db
            .get_label_by_name(name.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Inserts a new label into the `djmdLabel` table in the database.
    pub fn insert_label(&mut self, label: NewDjmdLabel) -> Result<DjmdLabel> {
        let model = self
            .db
            .insert_label(label)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Creates a new label and inserts it into the `djmdLabel` table in the database.
    #[napi]
    pub fn create_label(&mut self, name: String) -> Result<DjmdLabel> {
        let model = self
            .db
            .create_label(name)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Updates an existing `djmdLabel` entry in the database.
    #[napi]
    pub fn update_label(&mut self, mut model: DjmdLabel) -> Result<DjmdLabel> {
        let updated_model = self
            .db
            .update_label(&mut model)
            .map_err(crate::util::binding_error)?;
        Ok(updated_model)
    }

    /// Deletes a label entry from the `djmdLabel` table in the database.
    #[napi]
    pub fn delete_label(&mut self, id: String) -> Result<()> {
        self.db
            .delete_label(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(())
    }

    // -- MenuItems --------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdMenuItems` table in the database.
    #[napi]
    pub fn get_menu_items(&mut self) -> Result<Vec<DjmdMenuItems>> {
        let models = self
            .db
            .get_menu_items()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdMenuItems` entry by its unique identifier.
    #[napi]
    pub fn get_menu_item_by_id(&mut self, id: String) -> Result<Option<DjmdMenuItems>> {
        let model = self
            .db
            .get_menu_item_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- MixerParam -------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdMixerParam` table in the database.
    #[napi]
    pub fn get_mixer_params(&mut self) -> Result<Vec<DjmdMixerParam>> {
        let models = self
            .db
            .get_mixer_params()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdMixerParam` entry by its unique identifier.
    #[napi]
    pub fn get_mixer_param_by_id(&mut self, id: String) -> Result<Option<DjmdMixerParam>> {
        let model = self
            .db
            .get_mixer_param_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdMyTag` table in the database.
    #[napi]
    pub fn get_my_tags(&mut self) -> Result<Vec<DjmdMyTag>> {
        let models = self.db.get_my_tags().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all child `djmdMyTag` entries for a given parent ID, ordered by sequence.
    #[napi]
    pub fn get_my_tag_children(&mut self, id: String) -> Result<Vec<DjmdMyTag>> {
        let models = self
            .db
            .get_my_tag_children(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdMyTag` entry by its unique identifier.
    #[napi]
    pub fn get_my_tag_by_id(&mut self, id: String) -> Result<Option<DjmdMyTag>> {
        let model = self
            .db
            .get_my_tag_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves all `djmdSongMyTag` entries for a given tag ID, ordered by track number.
    #[napi]
    pub fn get_my_tag_songs(&mut self, id: String) -> Result<Vec<DjmdSongMyTag>> {
        let models = self
            .db
            .get_my_tag_songs(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given tag ID.
    #[napi]
    pub fn get_my_tag_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self
            .db
            .get_my_tag_contents(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    // -- Playlist ---------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdPlaylist` table in the database.
    #[napi]
    pub fn get_playlists(&mut self) -> Result<Vec<DjmdPlaylist>> {
        let models = self
            .db
            .get_playlists()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Returns a sorted tree of playlists as tree nodes.
    #[napi]
    pub fn get_playlist_tree(&mut self) -> Result<Vec<JsDjmdPlaylistTreeNode>> {
        let models = self
            .db
            .get_playlist_tree()
            .map_err(crate::util::binding_error)?;
        Ok(models
            .into_iter()
            .map(|m| m.borrow().clone().into())
            .collect())
    }

    /// Retrieves all child `djmdPlaylist` entries for a given parent ID, ordered by sequence.
    #[napi]
    pub fn get_playlist_children(&mut self, id: String) -> Result<Vec<DjmdPlaylist>> {
        let models = self
            .db
            .get_playlist_children(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdPlaylist` entry by its unique identifier.
    #[napi]
    pub fn get_playlist_by_id(&mut self, id: String) -> Result<Option<DjmdPlaylist>> {
        let model = self
            .db
            .get_playlist_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves a `djmdPlaylist` entry by its hierarchical path.
    #[napi]
    pub fn get_playlist_by_path(&mut self, path: Vec<String>) -> Result<Option<DjmdPlaylist>> {
        let path_str: Vec<&str> = path.iter().map(|item| item.as_str()).collect();
        let model = self
            .db
            .get_playlist_by_path(path_str)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves all `djmdSongPlaylist` entries for a given playlist ID, ordered by track number.
    #[napi]
    pub fn get_playlist_songs(&mut self, id: String) -> Result<Vec<DjmdSongPlaylist>> {
        let models = self
            .db
            .get_playlist_songs(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given playlist ID.
    #[napi]
    pub fn get_playlist_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self
            .db
            .get_playlist_contents(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdSongPlaylist` entry by its unique identifier.
    #[napi]
    pub fn get_playlist_song_by_id(&mut self, id: String) -> Result<Option<DjmdSongPlaylist>> {
        let models = self
            .db
            .get_playlist_song_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Inserts a new playlist into the `djmdPlaylist` table in the database.
    pub fn insert_playlist(&mut self, playlist: NewDjmdPlaylist) -> Result<DjmdPlaylist> {
        let model = self
            .db
            .insert_playlist(playlist)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Creates a new playlist and inserts it into the `djmdPlaylist` table in the database.
    #[napi]
    pub fn create_playlist_node(
        &mut self,
        name: String,
        attribute: i32,
        parent_id: Option<String>,
        seq: Option<i32>,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> Result<DjmdPlaylist> {
        let attr = PlaylistType::try_from(attribute).expect("Wrong Playlist attribute");
        let model = self
            .db
            .create_playlist_node(name, attr, parent_id, seq, image_path, smart_list)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Creates a new regular playlist in the `djmdPlaylist` table.
    #[napi]
    pub fn create_playlist(
        &mut self,
        name: String,
        parent_id: Option<String>,
        seq: Option<i32>,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> Result<DjmdPlaylist> {
        let model = self
            .db
            .create_playlist(name, parent_id, seq, image_path, smart_list)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Creates a new playlist folder in the `djmdPlaylist` table.
    #[napi]
    pub fn create_playlist_folder(
        &mut self,
        name: String,
        parent_id: Option<String>,
        seq: Option<i32>,
    ) -> Result<DjmdPlaylist> {
        let model = self
            .db
            .create_playlist_folder(name, parent_id, seq)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Renames an existing playlist.
    #[napi]
    pub fn rename_playlist(&mut self, id: String, name: String) -> Result<DjmdPlaylist> {
        let model = self
            .db
            .rename_playlist(&id, name)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Moves a playlist to a new parent folder or changes its sequence within the same parent.
    #[napi]
    pub fn move_playlist(
        &mut self,
        id: String,
        seq: Option<i32>,
        parent_id: Option<String>,
    ) -> Result<DjmdPlaylist> {
        let model = self
            .db
            .move_playlist(&id, seq, parent_id)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Moves multiple playlists to a new parent folder or changes its sequence within the same parent.
    #[napi]
    pub fn move_playlists(
        &mut self,
        ids: Vec<String>,
        parent_id: String,
        start_seq: Option<i32>,
    ) -> Result<Vec<DjmdPlaylist>> {
        let ids: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let models = self
            .db
            .move_playlists(ids, &parent_id, start_seq)
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Deletes a playlist entry from the `djmdPlaylist` table in the database.
    #[napi]
    pub fn delete_playlist(&mut self, id: String) -> Result<()> {
        self.db
            .delete_playlist(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(())
    }

    /// Inserts a new `djmdSongPlaylist` song into a playlist.
    #[napi]
    pub fn insert_playlist_song(&mut self, item: NewDjmdSongPlaylist) -> Result<DjmdSongPlaylist> {
        let model = self
            .db
            .insert_playlist_song(item)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Creates a new playlist song and inserts it into the `djmdSongPlaylist` table.
    #[napi]
    pub fn create_playlist_song(
        &mut self,
        playlist_id: String,
        content_id: String,
        seq: Option<i32>,
    ) -> Result<DjmdSongPlaylist> {
        let model = self
            .db
            .create_playlist_song(&playlist_id, &content_id, seq)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Moves a song within a playlist to a new sequence position.
    #[napi]
    pub fn move_playlist_song(&mut self, id: String, seq: i32) -> Result<DjmdSongPlaylist> {
        let model = self
            .db
            .move_playlist_song(&id, seq)
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Moves multiple songs within a playlist to a new sequence position.
    #[napi]
    pub fn move_playlist_songs(
        &mut self,
        ids: Vec<String>,
        start_seq: i32,
    ) -> Result<Vec<DjmdSongPlaylist>> {
        let ids: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let models = self
            .db
            .move_playlist_songs(ids, start_seq)
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Deletes a song entry from a playlist.
    #[napi]
    pub fn delete_playlist_song(&mut self, id: String) -> Result<()> {
        self.db
            .delete_playlist_song(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(())
    }

    /// Deletes multiple songs from a playlist.
    pub fn delete_playlist_songs(&mut self, ids: Vec<String>) -> Result<usize> {
        let ids: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let result = self
            .db
            .delete_playlist_songs(ids)
            .map_err(crate::util::binding_error)?;
        Ok(result)
    }

    // -- Property ---------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdProperty` table in the database.
    #[napi]
    pub fn get_properties(&mut self) -> Result<Vec<DjmdProperty>> {
        let models = self
            .db
            .get_properties()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdProperty` entry by its unique identifier.
    #[napi]
    pub fn get_property_by_id(&mut self, id: String) -> Result<Option<DjmdProperty>> {
        let model = self
            .db
            .get_property_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdCloudProperty` table in the database.
    #[napi]
    pub fn get_cloud_properties(&mut self) -> Result<Vec<DjmdCloudProperty>> {
        let models = self
            .db
            .get_cloud_properties()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdCloudProperty` entry by its unique identifier.
    #[napi]
    pub fn get_cloud_property_by_id(&mut self, id: String) -> Result<Option<DjmdCloudProperty>> {
        let model = self
            .db
            .get_cloud_property_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdRecommendLike` table in the database.
    #[napi]
    pub fn get_recommend_likes(&mut self) -> Result<Vec<DjmdRecommendLike>> {
        let models = self
            .db
            .get_recommend_likes()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdRecommendLike` entry by its unique identifier.
    #[napi]
    pub fn get_recommend_like_by_id(&mut self, id: String) -> Result<Option<DjmdRecommendLike>> {
        let model = self
            .db
            .get_recommend_like_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdRelatedTracks` table in the database.
    #[napi]
    pub fn get_related_tracks(&mut self) -> Result<Vec<DjmdRelatedTracks>> {
        let models = self
            .db
            .get_related_tracks()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all child `djmdRelatedTracks` entries for a given parent ID, ordered by sequence.
    #[napi]
    pub fn get_related_tracks_children(&mut self, id: String) -> Result<Vec<DjmdRelatedTracks>> {
        let models = self
            .db
            .get_related_tracks_children(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdRelatedTracks` entry by its unique identifier.
    #[napi]
    pub fn get_related_tracks_by_id(&mut self, id: String) -> Result<Option<DjmdRelatedTracks>> {
        let model = self
            .db
            .get_related_tracks_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves all `djmdSongRelatedTracks` entries for a given related tracks ID, ordered by track number.
    #[napi]
    pub fn get_related_tracks_songs(&mut self, id: String) -> Result<Vec<DjmdSongRelatedTracks>> {
        let models = self
            .db
            .get_related_tracks_songs(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given related tracks ID.
    #[napi]
    pub fn get_related_tracks_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self
            .db
            .get_related_tracks_contents(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    // -- Sampler ----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdSampler` table in the database.
    #[napi]
    pub fn get_samplers(&mut self) -> Result<Vec<DjmdSampler>> {
        let models = self.db.get_samplers().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all child `djmdSampler` entries for a given parent ID, ordered by sequence.
    #[napi]
    pub fn get_sampler_children(&mut self, id: String) -> Result<Vec<DjmdSampler>> {
        let models = self
            .db
            .get_sampler_children(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdSampler` entry by its unique identifier.
    #[napi]
    pub fn get_sampler_by_id(&mut self, id: String) -> Result<Option<DjmdSampler>> {
        let model = self
            .db
            .get_sampler_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    /// Retrieves all `djmdSongSampler` entries for a given sampler ID, ordered by track number.
    #[napi]
    pub fn get_sampler_songs(&mut self, id: String) -> Result<Vec<DjmdSongSampler>> {
        let models = self
            .db
            .get_sampler_songs(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given sampler ID.
    #[napi]
    pub fn get_sampler_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self
            .db
            .get_sampler_contents(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    // -- SongTagList ------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdSongTagList` table in the database.
    #[napi]
    pub fn get_song_tag_lists(&mut self) -> Result<Vec<DjmdSongTagList>> {
        let models = self
            .db
            .get_song_tag_lists()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdSongTagList` entry by its unique identifier.
    #[napi]
    pub fn get_song_tag_list_by_id(&mut self, id: String) -> Result<Option<DjmdSongTagList>> {
        let model = self
            .db
            .get_song_tag_list_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- Sort -------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdSort` table in the database, ordered by sequence.
    #[napi]
    pub fn get_sorts(&mut self) -> Result<Vec<DjmdSort>> {
        let models = self.db.get_sorts().map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `djmdSort` entry by its unique identifier.
    #[napi]
    pub fn get_sort_by_id(&mut self, id: String) -> Result<Option<DjmdSort>> {
        let model = self
            .db
            .get_sort_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- ImageFile --------------------------------------------------------------------------------

    /// Retrieves all entries from the `imageFile` table in the database.
    #[napi]
    pub fn get_image_files(&mut self) -> Result<Vec<ImageFile>> {
        let models = self
            .db
            .get_image_files()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves an `imageFile` entry by its unique identifier.
    #[napi]
    pub fn get_image_file_by_id(&mut self, id: String) -> Result<Option<ImageFile>> {
        let model = self
            .db
            .get_image_file_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- SettingFile ------------------------------------------------------------------------------

    /// Retrieves all entries from the `settingFile` table in the database.
    #[napi]
    pub fn get_setting_files(&mut self) -> Result<Vec<SettingFile>> {
        let models = self
            .db
            .get_setting_files()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `settingFile` entry by its unique identifier.
    #[napi]
    pub fn get_setting_file_by_id(&mut self, id: String) -> Result<Option<SettingFile>> {
        let model = self
            .db
            .get_setting_file_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    /// Retrieves all entries from the `uuidIDMap` table in the database.
    #[napi]
    pub fn get_uuid_id_maps(&mut self) -> Result<Vec<UuidIDMap>> {
        let models = self
            .db
            .get_uuid_id_maps()
            .map_err(crate::util::binding_error)?;
        Ok(models)
    }

    /// Retrieves a `uuidIDMap` entry by its unique identifier.
    #[napi]
    pub fn get_uuid_id_map_by_id(&mut self, id: String) -> Result<Option<UuidIDMap>> {
        let model = self
            .db
            .get_uuid_id_map_by_id(id.as_str())
            .map_err(crate::util::binding_error)?;
        Ok(model)
    }
}
