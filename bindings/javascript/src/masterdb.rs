// Author: Dylan Jones
// Date:   2025-05-04

use chrono::{DateTime, Utc};
use napi::Result;
use napi_derive::napi;
use rbox::masterdb::{models::*, MasterDb, PlaylistType};

use super::anlz::JsAnlzPaths;

#[napi(object, js_name = "DjmdPlaylistTreeItem")]
pub struct JsDjmdPlaylistTreeItem {
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
    pub attribute: i32,
    pub parent_id: String,
    pub smart_list: Option<String>,
    pub children: Vec<JsDjmdPlaylistTreeItem>,
}

impl Into<JsDjmdPlaylistTreeItem> for DjmdPlaylistTreeItem {
    fn into(self) -> JsDjmdPlaylistTreeItem {
        let children = self
            .children
            .into_iter()
            .map(|c| c.borrow().clone().into())
            .collect();
        JsDjmdPlaylistTreeItem {
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
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        Ok(JsMasterDb {
            db: MasterDb::new(path.as_str())?,
        })
    }

    #[napi(factory)]
    pub fn open() -> Result<Self> {
        Ok(JsMasterDb {
            db: MasterDb::open()?,
        })
    }

    #[napi]
    pub fn set_unsafe_writes(&mut self, unsafe_writes: bool) {
        self.db.set_unsafe_writes(unsafe_writes);
    }

    // -- AgentRegistry ----------------------------------------------------------------------------

    #[napi]
    pub fn get_agent_registry(&mut self) -> Result<Vec<AgentRegistry>> {
        let models = self.db.get_agent_registry()?;
        Ok(models)
    }

    #[napi]
    pub fn get_agent_registry_by_id(&mut self, id: String) -> Result<Option<AgentRegistry>> {
        let model = self.db.get_agent_registry_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_local_usn(&mut self) -> Result<i32> {
        Ok(self.db.get_local_usn()?)
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_agent_registry(&mut self) -> Result<Vec<CloudAgentRegistry>> {
        let models = self.db.get_cloud_agent_registry()?;
        Ok(models)
    }

    #[napi]
    pub fn get_cloud_agent_registry_by_id(
        &mut self,
        id: String,
    ) -> Result<Option<CloudAgentRegistry>> {
        let model = self.db.get_cloud_agent_registry_by_id(id.as_str())?;
        Ok(model)
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    #[napi]
    pub fn get_content_active_censor(&mut self) -> Result<Vec<ContentActiveCensor>> {
        let models = self.db.get_content_active_censor()?;
        Ok(models)
    }

    #[napi]
    pub fn get_content_active_censor_by_id(
        &mut self,
        id: String,
    ) -> Result<Option<ContentActiveCensor>> {
        let model = self.db.get_content_active_censor_by_id(id.as_str())?;
        Ok(model)
    }

    // -- ContentCue -------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_cue(&mut self) -> Result<Vec<ContentCue>> {
        let models = self.db.get_content_cue()?;
        Ok(models)
    }

    #[napi]
    pub fn get_content_cue_by_id(&mut self, id: String) -> Result<Option<ContentCue>> {
        let model = self.db.get_content_cue_by_id(id.as_str())?;
        Ok(model)
    }

    // -- ContentFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_file(&mut self) -> Result<Vec<ContentFile>> {
        let models = self.db.get_content_file()?;
        Ok(models)
    }

    #[napi]
    pub fn get_content_file_by_id(&mut self, id: String) -> Result<Option<ContentFile>> {
        let model = self.db.get_content_file_by_id(id.as_str())?;
        Ok(model)
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    #[napi]
    pub fn get_active_censor(&mut self) -> Result<Vec<DjmdActiveCensor>> {
        let models = self.db.get_active_censor()?;
        Ok(models)
    }

    #[napi]
    pub fn get_active_censor_by_id(&mut self, id: String) -> Result<Option<DjmdActiveCensor>> {
        let model = self.db.get_active_censor_by_id(id.as_str())?;
        Ok(model)
    }

    // -- Album ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_album(&mut self) -> Result<Vec<DjmdAlbum>> {
        let models = self.db.get_album()?;
        Ok(models)
    }

    #[napi]
    pub fn get_album_by_id(&mut self, id: String) -> Result<Option<DjmdAlbum>> {
        let model = self.db.get_album_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_album_by_name(&mut self, name: String) -> Result<Option<DjmdAlbum>> {
        let model = self.db.get_album_by_name(name.as_str())?;
        Ok(model)
    }

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
            .create_album(name, artist_id, image_path, compilation)?;
        Ok(model)
    }

    #[napi]
    pub fn update_album(&mut self, mut model: DjmdAlbum) -> Result<DjmdAlbum> {
        let updated_model = self.db.update_album(&mut model)?;
        Ok(updated_model)
    }

    #[napi]
    pub fn delete_album(&mut self, id: String) -> Result<()> {
        self.db.delete_album(id.as_str())?;
        Ok(())
    }

    // -- Artist -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_artist(&mut self) -> Result<Vec<DjmdArtist>> {
        let models = self.db.get_artist()?;
        Ok(models)
    }

    #[napi]
    pub fn get_artist_by_id(&mut self, id: String) -> Result<Option<DjmdArtist>> {
        let model = self.db.get_artist_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_artist_by_name(&mut self, name: String) -> Result<Option<DjmdArtist>> {
        let model = self.db.get_artist_by_name(name.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn create_artist(&mut self, name: String) -> Result<DjmdArtist> {
        let model = self.db.create_artist(name)?;
        Ok(model)
    }

    #[napi]
    pub fn update_artist(&mut self, mut model: DjmdArtist) -> Result<DjmdArtist> {
        let updated_model = self.db.update_artist(&mut model)?;
        Ok(updated_model)
    }

    #[napi]
    pub fn delete_artist(&mut self, id: String) -> Result<()> {
        self.db.delete_artist(id.as_str())?;
        Ok(())
    }

    // -- Category ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_category(&mut self) -> Result<Vec<DjmdCategory>> {
        let models = self.db.get_category()?;
        Ok(models)
    }

    #[napi]
    pub fn get_category_by_id(&mut self, id: String) -> Result<Option<DjmdCategory>> {
        let model = self.db.get_category_by_id(id.as_str())?;
        Ok(model)
    }

    // -- Color ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_color(&mut self) -> Result<Vec<DjmdColor>> {
        let models = self.db.get_color()?;
        Ok(models)
    }

    #[napi]
    pub fn get_color_by_id(&mut self, id: String) -> Result<Option<DjmdColor>> {
        let model = self.db.get_color_by_id(id.as_str())?;
        Ok(model)
    }

    // -- Content ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_content(&mut self) -> Result<Vec<DjmdContent>> {
        let models = self.db.get_content()?;
        Ok(models)
    }

    #[napi]
    pub fn get_content_by_id(&mut self, id: String) -> Result<Option<DjmdContent>> {
        let model = self.db.get_content_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_content_by_path(&mut self, id: String) -> Result<Option<DjmdContent>> {
        let model = self.db.get_content_by_path(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_content_anlz_dir(&mut self, id: String) -> Result<Option<String>> {
        let dir = self.db.get_content_anlz_dir(id.as_str())?;
        if let Some(dir) = dir {
            return Ok(Some(dir.as_os_str().to_str().unwrap().to_string()));
        }
        Ok(None)
    }

    #[napi]
    pub fn get_content_anlz_paths(&mut self, id: String) -> Result<Option<JsAnlzPaths>> {
        let paths = self.db.get_content_anlz_paths(id.as_str())?;
        if let Some(paths) = paths {
            return Ok(Some(JsAnlzPaths::from(paths)));
        }
        Ok(None)
    }

    #[napi]
    pub fn create_content(&mut self, path: String) -> Result<DjmdContent> {
        let model = self.db.create_content(path)?;
        Ok(model)
    }

    #[napi]
    pub fn update_content(&mut self, mut model: DjmdContent) -> Result<i32> {
        let result = self.db.update_content(&mut model)?;
        Ok(result as i32)
    }

    #[napi]
    pub fn update_content_album(&mut self, id: String, name: String) -> Result<i32> {
        let result = self.db.update_content_album(&id, &name)?;
        Ok(result as i32)
    }

    #[napi]
    pub fn update_content_artist(&mut self, id: String, name: String) -> Result<i32> {
        let result = self.db.update_content_artist(&id, &name)?;
        Ok(result as i32)
    }

    #[napi]
    pub fn update_content_remixer(&mut self, id: String, name: String) -> Result<i32> {
        let result = self.db.update_content_remixer(&id, &name)?;
        Ok(result as i32)
    }

    #[napi]
    pub fn update_content_original_artist(&mut self, id: String, name: String) -> Result<i32> {
        let result = self.db.update_content_original_artist(&id, &name)?;
        Ok(result as i32)
    }

    #[napi]
    pub fn update_content_composer(&mut self, id: String, name: String) -> Result<i32> {
        let result = self.db.update_content_composer(&id, &name)?;
        Ok(result as i32)
    }

    #[napi]
    pub fn update_content_genre(&mut self, id: String, name: String) -> Result<i32> {
        let result = self.db.update_content_genre(&id, &name)?;
        Ok(result as i32)
    }

    #[napi]
    pub fn update_content_label(&mut self, id: String, name: String) -> Result<i32> {
        let result = self.db.update_content_label(&id, &name)?;
        Ok(result as i32)
    }

    #[napi]
    pub fn update_content_key(&mut self, id: String, name: String) -> Result<i32> {
        let result = self.db.update_content_key(&id, &name)?;
        Ok(result as i32)
    }

    // #[napi]
    // pub fn delete_content(&mut self, id: String) {
    //     self.db
    //         .delete_content(id.as_str())
    //         .expect("Failed to delete DjmdContent");
    // }

    // -- Cue --------------------------------------------------------------------------------------

    #[napi]
    pub fn get_cue(&mut self) -> Result<Vec<DjmdCue>> {
        let models = self.db.get_cue()?;
        Ok(models)
    }

    #[napi]
    pub fn get_cue_by_id(&mut self, id: String) -> Result<Option<DjmdCue>> {
        let model = self.db.get_cue_by_id(id.as_str())?;
        Ok(model)
    }

    // -- Device -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_device(&mut self) -> Result<Vec<DjmdDevice>> {
        let models = self.db.get_device()?;
        Ok(models)
    }

    #[napi]
    pub fn get_device_by_id(&mut self, id: String) -> Result<Option<DjmdDevice>> {
        let model = self.db.get_device_by_id(id.as_str())?;
        Ok(model)
    }

    // -- Genre ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_genre(&mut self) -> Result<Vec<DjmdGenre>> {
        let models = self.db.get_genre()?;
        Ok(models)
    }

    #[napi]
    pub fn get_genre_by_id(&mut self, id: String) -> Result<Option<DjmdGenre>> {
        let model = self.db.get_genre_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_genre_by_name(&mut self, name: String) -> Result<Option<DjmdGenre>> {
        let model = self.db.get_genre_by_name(name.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn create_genre(&mut self, name: String) -> Result<DjmdGenre> {
        let model = self.db.create_genre(name)?;
        Ok(model)
    }

    #[napi]
    pub fn update_genre(&mut self, mut model: DjmdGenre) -> Result<DjmdGenre> {
        let updated_model = self.db.update_genre(&mut model)?;
        Ok(updated_model)
    }

    #[napi]
    pub fn delete_genre(&mut self, id: String) -> Result<()> {
        self.db.delete_genre(id.as_str())?;
        Ok(())
    }

    // -- History ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_history(&mut self) -> Result<Vec<DjmdHistory>> {
        let models = self.db.get_history()?;
        Ok(models)
    }

    #[napi]
    pub fn get_history_by_id(&mut self, id: String) -> Result<Option<DjmdHistory>> {
        let model = self.db.get_history_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_history_songs(&mut self, id: String) -> Result<Vec<DjmdSongHistory>> {
        let models = self.db.get_history_songs(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_history_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self.db.get_history_contents(id.as_str())?;
        Ok(models)
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    #[napi]
    pub fn get_hot_cue_banklist(&mut self) -> Result<Vec<DjmdHotCueBanklist>> {
        let models = self.db.get_hot_cue_banklist()?;
        Ok(models)
    }

    #[napi]
    pub fn get_hot_cue_banklist_children(&mut self, id: String) -> Result<Vec<DjmdHotCueBanklist>> {
        let models = self.db.get_hot_cue_banklist_children(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_hot_cue_banklist_by_id(&mut self, id: String) -> Result<Option<DjmdHotCueBanklist>> {
        let model = self.db.get_hot_cue_banklist_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_hot_cue_banklist_songs(
        &mut self,
        id: String,
    ) -> Result<Vec<DjmdSongHotCueBanklist>> {
        let models = self.db.get_hot_cue_banklist_songs(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_hot_cue_banklist_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self.db.get_hot_cue_banklist_contents(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_hot_cue_banklist_cues(&mut self, id: String) -> Result<Vec<HotCueBanklistCue>> {
        let models = self.db.get_hot_cue_banklist_cues(id.as_str())?;
        Ok(models)
    }

    // -- Key --------------------------------------------------------------------------------------

    #[napi]
    pub fn get_key(&mut self) -> Result<Vec<DjmdKey>> {
        let models = self.db.get_key()?;
        Ok(models)
    }

    #[napi]
    pub fn get_key_by_id(&mut self, id: String) -> Result<Option<DjmdKey>> {
        let model = self.db.get_key_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_key_by_name(&mut self, name: String) -> Result<Option<DjmdKey>> {
        let model = self.db.get_key_by_name(name.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn create_key(&mut self, name: String) -> Result<DjmdKey> {
        let model = self.db.create_key(name)?;
        Ok(model)
    }

    #[napi]
    pub fn update_key(&mut self, mut model: DjmdKey) -> Result<DjmdKey> {
        let updated_model = self.db.update_key(&mut model)?;
        Ok(updated_model)
    }

    #[napi]
    pub fn delete_key(&mut self, id: String) -> Result<()> {
        self.db.delete_key(id.as_str())?;
        Ok(())
    }

    // -- Label ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_label(&mut self) -> Result<Vec<DjmdLabel>> {
        let models = self.db.get_label()?;
        Ok(models)
    }

    #[napi]
    pub fn get_label_by_id(&mut self, id: String) -> Result<Option<DjmdLabel>> {
        let model = self.db.get_label_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_label_by_name(&mut self, name: String) -> Result<Option<DjmdLabel>> {
        let model = self.db.get_label_by_name(name.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn create_label(&mut self, name: String) -> Result<DjmdLabel> {
        let model = self.db.create_label(name)?;
        Ok(model)
    }

    #[napi]
    pub fn update_label(&mut self, mut model: DjmdLabel) -> Result<DjmdLabel> {
        let updated_model = self.db.update_label(&mut model)?;
        Ok(updated_model)
    }

    #[napi]
    pub fn delete_label(&mut self, id: String) -> Result<()> {
        self.db.delete_label(id.as_str())?;
        Ok(())
    }

    // -- MenuItems --------------------------------------------------------------------------------

    #[napi]
    pub fn get_menu_item(&mut self) -> Result<Vec<DjmdMenuItems>> {
        let models = self.db.get_menu_item()?;
        Ok(models)
    }

    #[napi]
    pub fn get_menu_item_by_id(&mut self, id: String) -> Result<Option<DjmdMenuItems>> {
        let model = self.db.get_menu_item_by_id(id.as_str())?;
        Ok(model)
    }

    // -- MixerParam -------------------------------------------------------------------------------

    #[napi]
    pub fn get_mixer_param(&mut self) -> Result<Vec<DjmdMixerParam>> {
        let models = self.db.get_mixer_param()?;
        Ok(models)
    }

    #[napi]
    pub fn get_mixer_param_by_id(&mut self, id: String) -> Result<Option<DjmdMixerParam>> {
        let model = self.db.get_mixer_param_by_id(id.as_str())?;
        Ok(model)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_my_tag(&mut self) -> Result<Vec<DjmdMyTag>> {
        let models = self.db.get_my_tag()?;
        Ok(models)
    }

    #[napi]
    pub fn get_my_tag_children(&mut self, id: String) -> Result<Vec<DjmdMyTag>> {
        let models = self.db.get_my_tag_children(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_my_tag_by_id(&mut self, id: String) -> Result<Option<DjmdMyTag>> {
        let model = self.db.get_my_tag_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_my_tag_songs(&mut self, id: String) -> Result<Vec<DjmdSongMyTag>> {
        let models = self.db.get_my_tag_songs(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_my_tag_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self.db.get_my_tag_contents(id.as_str())?;
        Ok(models)
    }

    // -- Playlist ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_playlist(&mut self) -> Result<Vec<DjmdPlaylist>> {
        let models = self.db.get_playlist()?;
        Ok(models)
    }

    #[napi]
    pub fn get_playlist_tree(&mut self) -> Result<Vec<JsDjmdPlaylistTreeItem>> {
        let models = self.db.get_playlist_tree()?;
        Ok(models
            .into_iter()
            .map(|m| m.borrow().clone().into())
            .collect())
    }

    #[napi]
    pub fn get_playlist_children(&mut self, id: String) -> Result<Vec<DjmdPlaylist>> {
        let models = self.db.get_playlist_children(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_playlist_by_id(&mut self, id: String) -> Result<Option<DjmdPlaylist>> {
        let model = self.db.get_playlist_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_playlist_by_path(&mut self, path: Vec<String>) -> Result<Option<DjmdPlaylist>> {
        let path_str: Vec<&str> = path.iter().map(|item| item.as_str()).collect();
        let model = self.db.get_playlist_by_path(path_str)?;
        Ok(model)
    }

    #[napi]
    pub fn get_playlist_songs(&mut self, id: String) -> Result<Vec<DjmdSongPlaylist>> {
        let models = self.db.get_playlist_songs(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_playlist_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self.db.get_playlist_contents(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_playlist_song_by_id(&mut self, id: String) -> Result<Option<DjmdSongPlaylist>> {
        let models = self.db.get_playlist_song_by_id(id.as_str())?;
        Ok(models)
    }

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
            .create_playlist_node(name, attr, parent_id, seq, image_path, smart_list)?;
        Ok(model)
    }

    #[napi]
    pub fn rename_playlist(&mut self, id: String, name: String) -> Result<DjmdPlaylist> {
        let model = self.db.rename_playlist(&id, name)?;
        Ok(model)
    }

    #[napi]
    pub fn move_playlist(
        &mut self,
        id: String,
        seq: Option<i32>,
        parent_id: Option<String>,
    ) -> Result<DjmdPlaylist> {
        let model = self.db.move_playlist(&id, seq, parent_id)?;
        Ok(model)
    }

    #[napi]
    pub fn delete_playlist(&mut self, id: String) -> Result<()> {
        self.db.delete_playlist(id.as_str())?;
        Ok(())
    }

    #[napi]
    pub fn create_playlist_song(
        &mut self,
        playlist_id: String,
        content_id: String,
        seq: Option<i32>,
    ) -> Result<DjmdSongPlaylist> {
        let model = self
            .db
            .create_playlist_song(&playlist_id, &content_id, seq)?;
        Ok(model)
    }

    #[napi]
    pub fn move_playlist_song(&mut self, id: String, seq: i32) -> Result<DjmdSongPlaylist> {
        let model = self.db.move_playlist_song(&id, seq)?;
        Ok(model)
    }

    #[napi]
    pub fn delete_playlist_song(&mut self, id: String) -> Result<()> {
        self.db.delete_playlist_song(id.as_str())?;
        Ok(())
    }

    // -- Property ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_property(&mut self) -> Result<Vec<DjmdProperty>> {
        let models = self.db.get_property()?;
        Ok(models)
    }

    #[napi]
    pub fn get_property_by_id(&mut self, id: String) -> Result<Option<DjmdProperty>> {
        let model = self.db.get_property_by_id(id.as_str())?;
        Ok(model)
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_property(&mut self) -> Result<Vec<DjmdCloudProperty>> {
        let models = self.db.get_cloud_property()?;
        Ok(models)
    }

    #[napi]
    pub fn get_cloud_property_by_id(&mut self, id: String) -> Result<Option<DjmdCloudProperty>> {
        let model = self.db.get_cloud_property_by_id(id.as_str())?;
        Ok(model)
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    #[napi]
    pub fn get_recommend_like(&mut self) -> Result<Vec<DjmdRecommendLike>> {
        let models = self.db.get_recommend_like()?;
        Ok(models)
    }

    #[napi]
    pub fn get_recommend_like_by_id(&mut self, id: String) -> Result<Option<DjmdRecommendLike>> {
        let model = self.db.get_recommend_like_by_id(id.as_str())?;
        Ok(model)
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    #[napi]
    pub fn get_related_tracks(&mut self) -> Result<Vec<DjmdRelatedTracks>> {
        let models = self.db.get_related_tracks()?;
        Ok(models)
    }

    #[napi]
    pub fn get_related_tracks_children(&mut self, id: String) -> Result<Vec<DjmdRelatedTracks>> {
        let models = self.db.get_related_tracks_children(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_related_tracks_by_id(&mut self, id: String) -> Result<Option<DjmdRelatedTracks>> {
        let model = self.db.get_related_tracks_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_related_tracks_songs(&mut self, id: String) -> Result<Vec<DjmdSongRelatedTracks>> {
        let models = self.db.get_related_tracks_songs(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_related_tracks_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self.db.get_related_tracks_contents(id.as_str())?;
        Ok(models)
    }

    // -- Sampler ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_sampler(&mut self) -> Result<Vec<DjmdSampler>> {
        let models = self.db.get_sampler()?;
        Ok(models)
    }

    #[napi]
    pub fn get_sampler_children(&mut self, id: String) -> Result<Vec<DjmdSampler>> {
        let models = self.db.get_sampler_children(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_sampler_by_id(&mut self, id: String) -> Result<Option<DjmdSampler>> {
        let model = self.db.get_sampler_by_id(id.as_str())?;
        Ok(model)
    }

    #[napi]
    pub fn get_sampler_songs(&mut self, id: String) -> Result<Vec<DjmdSongSampler>> {
        let models = self.db.get_sampler_songs(id.as_str())?;
        Ok(models)
    }

    #[napi]
    pub fn get_sampler_contents(&mut self, id: String) -> Result<Vec<DjmdContent>> {
        let models = self.db.get_sampler_contents(id.as_str())?;
        Ok(models)
    }

    // -- SongTagList ------------------------------------------------------------------------------

    #[napi]
    pub fn get_song_tag_list(&mut self) -> Result<Vec<DjmdSongTagList>> {
        let models = self.db.get_song_tag_list()?;
        Ok(models)
    }

    #[napi]
    pub fn get_song_tag_list_by_id(&mut self, id: String) -> Result<Option<DjmdSongTagList>> {
        let model = self.db.get_song_tag_list_by_id(id.as_str())?;
        Ok(model)
    }

    // -- Sort -------------------------------------------------------------------------------------

    #[napi]
    pub fn get_sort(&mut self) -> Result<Vec<DjmdSort>> {
        let models = self.db.get_sort()?;
        Ok(models)
    }

    #[napi]
    pub fn get_sort_by_id(&mut self, id: String) -> Result<Option<DjmdSort>> {
        let model = self.db.get_sort_by_id(id.as_str())?;
        Ok(model)
    }

    // -- ImageFile --------------------------------------------------------------------------------

    #[napi]
    pub fn get_image_file(&mut self) -> Result<Vec<ImageFile>> {
        let models = self.db.get_image_file()?;
        Ok(models)
    }

    #[napi]
    pub fn get_image_file_by_id(&mut self, id: String) -> Result<Option<ImageFile>> {
        let model = self.db.get_image_file_by_id(id.as_str())?;
        Ok(model)
    }

    // -- SettingFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_setting_file(&mut self) -> Result<Vec<SettingFile>> {
        let models = self.db.get_setting_file()?;
        Ok(models)
    }

    #[napi]
    pub fn get_setting_file_by_id(&mut self, id: String) -> Result<Option<SettingFile>> {
        let model = self.db.get_setting_file_by_id(id.as_str())?;
        Ok(model)
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    #[napi]
    pub fn get_uuid_id_map(&mut self) -> Result<Vec<UuidIDMap>> {
        let models = self.db.get_uuid_id_map()?;
        Ok(models)
    }

    #[napi]
    pub fn get_uuid_id_map_by_id(&mut self, id: String) -> Result<Option<UuidIDMap>> {
        let model = self.db.get_uuid_id_map_by_id(id.as_str())?;
        Ok(model)
    }
}
