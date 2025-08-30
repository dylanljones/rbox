// Author: Dylan Jones
// Date:   2025-05-04

use napi::Result;
use napi_derive::napi;
use rbox::masterdb::{models::*, MasterDb, PlaylistType};

use crate::anlz::JsAnlzPaths;
use crate::js_models::*;

#[napi(js_name = "MasterDb")]
pub struct JsMasterDb {
    db: MasterDb,
}

#[napi]
impl JsMasterDb {
    #[napi(constructor)]
    pub fn new(path: String) -> Self {
        JsMasterDb {
            db: MasterDb::new(path.as_str()).expect("Failed to open MasterDb"),
        }
    }

    #[napi(factory)]
    pub fn open() -> Self {
        JsMasterDb {
            db: MasterDb::open().expect("Failed to open MasterDb"),
        }
    }

    #[napi]
    pub fn set_unsafe_writes(&mut self, unsafe_writes: bool) {
        self.db.set_unsafe_writes(unsafe_writes);
    }

    // -- AgentRegistry ----------------------------------------------------------------------------

    #[napi]
    pub fn get_agent_registry(&mut self) -> Result<Vec<JsAgentRegistry>> {
        let models = self.db.get_agent_registry()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_agent_registry_by_id(&mut self, id: String) -> Result<Option<JsAgentRegistry>> {
        let model = self.db.get_agent_registry_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_local_usn(&mut self) -> Result<i32> {
        Ok(self.db.get_local_usn()?)
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_agent_registry(&mut self) -> Result<Vec<JsCloudAgentRegistry>> {
        let models = self.db.get_cloud_agent_registry()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_cloud_agent_registry_by_id(
        &mut self,
        id: String,
    ) -> Result<Option<JsCloudAgentRegistry>> {
        let model = self.db.get_cloud_agent_registry_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    #[napi]
    pub fn get_content_active_censor(&mut self) -> Result<Vec<JsContentActiveCensor>> {
        let models = self.db.get_content_active_censor()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_content_active_censor_by_id(
        &mut self,
        id: String,
    ) -> Result<Option<JsContentActiveCensor>> {
        let model = self.db.get_content_active_censor_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- ContentCue -------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_cue(&mut self) -> Result<Vec<JsContentCue>> {
        let models = self.db.get_content_cue()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_content_cue_by_id(&mut self, id: String) -> Result<Option<JsContentCue>> {
        let model = self.db.get_content_cue_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- ContentFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_file(&mut self) -> Result<Vec<JsContentFile>> {
        let models = self.db.get_content_file()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_content_file_by_id(&mut self, id: String) -> Result<Option<JsContentFile>> {
        let model = self.db.get_content_file_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    #[napi]
    pub fn get_active_censor(&mut self) -> Result<Vec<JsDjmdActiveCensor>> {
        let models = self.db.get_active_censor()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_active_censor_by_id(&mut self, id: String) -> Result<Option<JsDjmdActiveCensor>> {
        let model = self.db.get_active_censor_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- Album ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_album(&mut self) -> Result<Vec<JsDjmdAlbum>> {
        let models = self.db.get_album()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_album_by_id(&mut self, id: String) -> Result<Option<JsDjmdAlbum>> {
        let model = self.db.get_album_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_album_by_name(&mut self, name: String) -> Result<Option<JsDjmdAlbum>> {
        let model = self.db.get_album_by_name(name.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn insert_album(
        &mut self,
        name: String,
        artist_id: Option<String>,
        image_path: Option<String>,
        compilation: Option<i32>,
    ) -> Result<JsDjmdAlbum> {
        let model = self
            .db
            .insert_album(name, artist_id, image_path, compilation)?;
        Ok(model.into())
    }

    #[napi]
    pub fn update_album(&mut self, object: JsDjmdAlbum) -> Result<JsDjmdAlbum> {
        let mut model: DjmdAlbum = DjmdAlbum::from(object);
        let updated_model = self.db.update_album(&mut model)?;
        Ok(updated_model.into())
    }

    #[napi]
    pub fn delete_album(&mut self, id: String) -> Result<()> {
        self.db.delete_album(id.as_str())?;
        Ok(())
    }

    // -- Artist -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_artist(&mut self) -> Result<Vec<JsDjmdArtist>> {
        let models = self.db.get_artist()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_artist_by_id(&mut self, id: String) -> Result<Option<JsDjmdArtist>> {
        let model = self.db.get_artist_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_artist_by_name(&mut self, name: String) -> Result<Option<JsDjmdArtist>> {
        let model = self.db.get_artist_by_name(name.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn insert_artist(&mut self, name: String) -> Result<JsDjmdArtist> {
        let model = self.db.insert_artist(name)?;
        Ok(model.into())
    }

    #[napi]
    pub fn update_artist(&mut self, object: JsDjmdArtist) -> Result<JsDjmdArtist> {
        let mut model: DjmdArtist = DjmdArtist::from(object);
        let updated_model = self.db.update_artist(&mut model)?;
        Ok(updated_model.into())
    }

    #[napi]
    pub fn delete_artist(&mut self, id: String) -> Result<()> {
        self.db.delete_artist(id.as_str())?;
        Ok(())
    }

    // -- Category ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_category(&mut self) -> Result<Vec<JsDjmdCategory>> {
        let models = self.db.get_category()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_category_by_id(&mut self, id: String) -> Result<Option<JsDjmdCategory>> {
        let model = self.db.get_category_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- Color ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_color(&mut self) -> Result<Vec<JsDjmdColor>> {
        let models = self.db.get_color()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_color_by_id(&mut self, id: String) -> Result<Option<JsDjmdColor>> {
        let model = self.db.get_color_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- Content ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_content(&mut self) -> Result<Vec<JsDjmdContent>> {
        let models = self.db.get_content()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_content_by_id(&mut self, id: String) -> Result<Option<JsDjmdContent>> {
        let model = self.db.get_content_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_content_by_path(&mut self, id: String) -> Result<Option<JsDjmdContent>> {
        let model = self.db.get_content_by_path(id.as_str())?;
        Ok(model.map(|m| m.into()))
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
    pub fn insert_content(&mut self, path: String) -> Result<JsDjmdContent> {
        let model = self.db.insert_content(path)?;
        Ok(model.into())
    }

    #[napi]
    pub fn update_content(&mut self, object: JsDjmdContent) -> Result<i32> {
        let mut model: DjmdContent = DjmdContent::from(object);
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
    pub fn get_cue(&mut self) -> Result<Vec<JsDjmdCue>> {
        let models = self.db.get_cue()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_cue_by_id(&mut self, id: String) -> Result<Option<JsDjmdCue>> {
        let model = self.db.get_cue_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- Device -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_device(&mut self) -> Result<Vec<JsDjmdDevice>> {
        let models = self.db.get_device()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_device_by_id(&mut self, id: String) -> Result<Option<JsDjmdDevice>> {
        let model = self.db.get_device_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- Genre ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_genre(&mut self) -> Result<Vec<JsDjmdGenre>> {
        let models = self.db.get_genre()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_genre_by_id(&mut self, id: String) -> Result<Option<JsDjmdGenre>> {
        let model = self.db.get_genre_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_genre_by_name(&mut self, name: String) -> Result<Option<JsDjmdGenre>> {
        let model = self.db.get_genre_by_name(name.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn insert_genre(&mut self, name: String) -> Result<JsDjmdGenre> {
        let model = self.db.insert_genre(name)?;
        Ok(model.into())
    }

    #[napi]
    pub fn update_genre(&mut self, object: JsDjmdGenre) -> Result<JsDjmdGenre> {
        let mut model: DjmdGenre = DjmdGenre::from(object);
        let updated_model = self.db.update_genre(&mut model)?;
        Ok(updated_model.into())
    }

    #[napi]
    pub fn delete_genre(&mut self, id: String) -> Result<()> {
        self.db.delete_genre(id.as_str())?;
        Ok(())
    }

    // -- History ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_history(&mut self) -> Result<Vec<JsDjmdHistory>> {
        let models = self.db.get_history()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_history_by_id(&mut self, id: String) -> Result<Option<JsDjmdHistory>> {
        let model = self.db.get_history_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_history_songs(&mut self, id: String) -> Result<Vec<JsDjmdSongHistory>> {
        let models = self.db.get_history_songs(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_history_contents(&mut self, id: String) -> Result<Vec<JsDjmdContent>> {
        let models = self.db.get_history_contents(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    #[napi]
    pub fn get_hot_cue_banklist(&mut self) -> Result<Vec<JsDjmdHotCueBanklist>> {
        let models = self.db.get_hot_cue_banklist()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_hot_cue_banklist_children(
        &mut self,
        id: String,
    ) -> Result<Vec<JsDjmdHotCueBanklist>> {
        let models = self.db.get_hot_cue_banklist_children(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_hot_cue_banklist_by_id(
        &mut self,
        id: String,
    ) -> Result<Option<JsDjmdHotCueBanklist>> {
        let model = self.db.get_hot_cue_banklist_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_hot_cue_banklist_songs(
        &mut self,
        id: String,
    ) -> Result<Vec<JsDjmdSongHotCueBanklist>> {
        let models = self.db.get_hot_cue_banklist_songs(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_hot_cue_banklist_contents(&mut self, id: String) -> Result<Vec<JsDjmdContent>> {
        let models = self.db.get_hot_cue_banklist_contents(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_hot_cue_banklist_cues(&mut self, id: String) -> Result<Vec<JsHotCueBanklistCue>> {
        let models = self.db.get_hot_cue_banklist_cues(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    // -- Key --------------------------------------------------------------------------------------

    #[napi]
    pub fn get_key(&mut self) -> Result<Vec<JsDjmdKey>> {
        let models = self.db.get_key()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_key_by_id(&mut self, id: String) -> Result<Option<JsDjmdKey>> {
        let model = self.db.get_key_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_key_by_name(&mut self, name: String) -> Result<Option<JsDjmdKey>> {
        let model = self.db.get_key_by_name(name.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn insert_key(&mut self, name: String) -> Result<JsDjmdKey> {
        let model = self.db.insert_key(name)?;
        Ok(model.into())
    }

    #[napi]
    pub fn update_key(&mut self, object: JsDjmdKey) -> Result<JsDjmdKey> {
        let mut model: DjmdKey = DjmdKey::from(object);
        let updated_model = self.db.update_key(&mut model)?;
        Ok(updated_model.into())
    }

    #[napi]
    pub fn delete_key(&mut self, id: String) -> Result<()> {
        self.db.delete_key(id.as_str())?;
        Ok(())
    }

    // -- Label ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_label(&mut self) -> Result<Vec<JsDjmdLabel>> {
        let models = self.db.get_label()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_label_by_id(&mut self, id: String) -> Result<Option<JsDjmdLabel>> {
        let model = self.db.get_label_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_label_by_name(&mut self, name: String) -> Result<Option<JsDjmdLabel>> {
        let model = self.db.get_label_by_name(name.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn insert_label(&mut self, name: String) -> Result<JsDjmdLabel> {
        let model = self.db.insert_label(name)?;
        Ok(model.into())
    }

    #[napi]
    pub fn update_label(&mut self, object: JsDjmdLabel) -> Result<JsDjmdLabel> {
        let mut model: DjmdLabel = DjmdLabel::from(object);
        let updated_model = self.db.update_label(&mut model)?;
        Ok(updated_model.into())
    }

    #[napi]
    pub fn delete_label(&mut self, id: String) -> Result<()> {
        self.db.delete_label(id.as_str())?;
        Ok(())
    }

    // -- MenuItems --------------------------------------------------------------------------------

    #[napi]
    pub fn get_menu_item(&mut self) -> Result<Vec<JsDjmdMenuItems>> {
        let models = self.db.get_menu_item()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_menu_item_by_id(&mut self, id: String) -> Result<Option<JsDjmdMenuItems>> {
        let model = self.db.get_menu_item_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- MixerParam -------------------------------------------------------------------------------

    #[napi]
    pub fn get_mixer_param(&mut self) -> Result<Vec<JsDjmdMixerParam>> {
        let models = self.db.get_mixer_param()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_mixer_param_by_id(&mut self, id: String) -> Result<Option<JsDjmdMixerParam>> {
        let model = self.db.get_mixer_param_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- MyTag ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_my_tag(&mut self) -> Result<Vec<JsDjmdMyTag>> {
        let models = self.db.get_my_tag()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_my_tag_children(&mut self, id: String) -> Result<Vec<JsDjmdMyTag>> {
        let models = self.db.get_my_tag_children(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_my_tag_by_id(&mut self, id: String) -> Result<Option<JsDjmdMyTag>> {
        let model = self.db.get_my_tag_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_my_tag_songs(&mut self, id: String) -> Result<Vec<JsDjmdSongMyTag>> {
        let models = self.db.get_my_tag_songs(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_my_tag_contents(&mut self, id: String) -> Result<Vec<JsDjmdContent>> {
        let models = self.db.get_my_tag_contents(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    // -- Playlist ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_playlist(&mut self) -> Result<Vec<JsDjmdPlaylist>> {
        let models = self.db.get_playlist()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
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
    pub fn get_playlist_children(&mut self, id: String) -> Result<Vec<JsDjmdPlaylist>> {
        let models = self.db.get_playlist_children(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_playlist_by_id(&mut self, id: String) -> Result<Option<JsDjmdPlaylist>> {
        let model = self.db.get_playlist_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_playlist_by_path(&mut self, path: Vec<String>) -> Result<Option<JsDjmdPlaylist>> {
        let path_str: Vec<&str> = path.iter().map(|item| item.as_str()).collect();
        let model = self.db.get_playlist_by_path(path_str)?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_playlist_songs(&mut self, id: String) -> Result<Vec<JsDjmdSongPlaylist>> {
        let models = self.db.get_playlist_songs(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_playlist_contents(&mut self, id: String) -> Result<Vec<JsDjmdContent>> {
        let models = self.db.get_playlist_contents(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_playlist_song_by_id(&mut self, id: String) -> Result<Vec<JsDjmdSongPlaylist>> {
        let models = self.db.get_playlist_song_by_id(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn insert_playlist(
        &mut self,
        name: String,
        attribute: i32,
        parent_id: Option<String>,
        seq: Option<i32>,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> Result<JsDjmdPlaylist> {
        let attr = PlaylistType::try_from(attribute).expect("Wrong Playlist attribute");
        let model = self
            .db
            .insert_playlist(name, attr, parent_id, seq, image_path, smart_list)?;
        Ok(model.into())
    }

    #[napi]
    pub fn rename_playlist(&mut self, id: String, name: String) -> Result<JsDjmdPlaylist> {
        let model = self.db.rename_playlist(&id, name)?;
        Ok(model.into())
    }

    #[napi]
    pub fn move_playlist(
        &mut self,
        id: String,
        seq: Option<i32>,
        parent_id: Option<String>,
    ) -> Result<JsDjmdPlaylist> {
        let model = self.db.move_playlist(&id, seq, parent_id)?;
        Ok(model.into())
    }

    #[napi]
    pub fn delete_playlist(&mut self, id: String) -> Result<()> {
        self.db.delete_playlist(id.as_str())?;
        Ok(())
    }

    #[napi]
    pub fn insert_playlist_song(
        &mut self,
        playlist_id: String,
        content_id: String,
        seq: Option<i32>,
    ) -> Result<JsDjmdSongPlaylist> {
        let model = self
            .db
            .insert_playlist_song(&playlist_id, &content_id, seq)?;
        Ok(model.into())
    }

    #[napi]
    pub fn move_playlist_song(&mut self, id: String, seq: i32) -> Result<JsDjmdSongPlaylist> {
        let model = self.db.move_playlist_song(&id, seq)?;
        Ok(model.into())
    }

    #[napi]
    pub fn delete_playlist_song(&mut self, id: String) -> Result<()> {
        self.db.delete_playlist_song(id.as_str())?;
        Ok(())
    }

    // -- Property ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_property(&mut self) -> Result<Vec<JsDjmdProperty>> {
        let models = self.db.get_property()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_property_by_id(&mut self, id: String) -> Result<Option<JsDjmdProperty>> {
        let model = self.db.get_property_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_property(&mut self) -> Result<Vec<JsDjmdCloudProperty>> {
        let models = self.db.get_cloud_property()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_cloud_property_by_id(&mut self, id: String) -> Result<Option<JsDjmdCloudProperty>> {
        let model = self.db.get_cloud_property_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    #[napi]
    pub fn get_recommend_like(&mut self) -> Result<Vec<JsDjmdRecommendLike>> {
        let models = self.db.get_recommend_like()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_recommend_like_by_id(&mut self, id: String) -> Result<Option<JsDjmdRecommendLike>> {
        let model = self.db.get_recommend_like_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    #[napi]
    pub fn get_related_tracks(&mut self) -> Result<Vec<JsDjmdRelatedTracks>> {
        let models = self.db.get_related_tracks()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_related_tracks_children(&mut self, id: String) -> Result<Vec<JsDjmdRelatedTracks>> {
        let models = self.db.get_related_tracks_children(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_related_tracks_by_id(&mut self, id: String) -> Result<Option<JsDjmdRelatedTracks>> {
        let model = self.db.get_related_tracks_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_related_tracks_songs(&mut self, id: String) -> Result<Vec<JsDjmdSongRelatedTracks>> {
        let models = self.db.get_related_tracks_songs(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_related_tracks_contents(&mut self, id: String) -> Result<Vec<JsDjmdContent>> {
        let models = self.db.get_related_tracks_contents(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    // -- Sampler ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_sampler(&mut self) -> Result<Vec<JsDjmdSampler>> {
        let models = self.db.get_sampler()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_sampler_children(&mut self, id: String) -> Result<Vec<JsDjmdSampler>> {
        let models = self.db.get_sampler_children(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_sampler_by_id(&mut self, id: String) -> Result<Option<JsDjmdSampler>> {
        let model = self.db.get_sampler_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    #[napi]
    pub fn get_sampler_songs(&mut self, id: String) -> Result<Vec<JsDjmdSongSampler>> {
        let models = self.db.get_sampler_songs(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_sampler_contents(&mut self, id: String) -> Result<Vec<JsDjmdContent>> {
        let models = self.db.get_sampler_contents(id.as_str())?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    // -- SongTagList ------------------------------------------------------------------------------

    #[napi]
    pub fn get_song_tag_list(&mut self) -> Result<Vec<JsDjmdSongTagList>> {
        let models = self.db.get_song_tag_list()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_song_tag_list_by_id(&mut self, id: String) -> Result<Option<JsDjmdSongTagList>> {
        let model = self.db.get_song_tag_list_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- Sort -------------------------------------------------------------------------------------

    #[napi]
    pub fn get_sort(&mut self) -> Result<Vec<JsDjmdSort>> {
        let models = self.db.get_sort()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_sort_by_id(&mut self, id: String) -> Result<Option<JsDjmdSort>> {
        let model = self.db.get_sort_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- ImageFile --------------------------------------------------------------------------------

    #[napi]
    pub fn get_image_file(&mut self) -> Result<Vec<JsImageFile>> {
        let models = self.db.get_image_file()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_image_file_by_id(&mut self, id: String) -> Result<Option<JsImageFile>> {
        let model = self.db.get_image_file_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- SettingFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_setting_file(&mut self) -> Result<Vec<JsSettingFile>> {
        let models = self.db.get_setting_file()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_setting_file_by_id(&mut self, id: String) -> Result<Option<JsSettingFile>> {
        let model = self.db.get_setting_file_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    #[napi]
    pub fn get_uuid_id_map(&mut self) -> Result<Vec<JsUuidIDMap>> {
        let models = self.db.get_uuid_id_map()?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    #[napi]
    pub fn get_uuid_id_map_by_id(&mut self, id: String) -> Result<Option<JsUuidIDMap>> {
        let model = self.db.get_uuid_id_map_by_id(id.as_str())?;
        Ok(model.map(|m| m.into()))
    }
}
