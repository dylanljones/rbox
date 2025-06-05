// Author: Dylan Jones
// Date:   2025-05-04

use crate::anlz::JsAnlzPaths;
use crate::js_models::IntoJs;
use crate::js_models::*;
use napi::Env;
use napi_derive::napi;
use rbox::masterdb::{models::*, MasterDb, PlaylistType};

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
    pub fn get_agent_registry(&mut self, env: Env) -> Vec<JsAgentRegistry> {
        let models = self
            .db
            .get_agent_registry()
            .expect("Could not fetch AgentRegistry");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_agent_registry_by_id(&mut self, env: Env, id: String) -> Option<JsAgentRegistry> {
        let model = self
            .db
            .get_agent_registry_by_id(id.as_str())
            .expect("Failed to fetch AgentRegistry");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_local_usn(&mut self) -> i32 {
        self.db.get_local_usn().expect("Failed to fetch Local USN")
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_agent_registry(&mut self, env: Env) -> Vec<JsCloudAgentRegistry> {
        let models = self
            .db
            .get_cloud_agent_registry()
            .expect("Could not fetch CloudAgentRegistry");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_cloud_agent_registry_by_id(
        &mut self,
        env: Env,
        id: String,
    ) -> Option<JsCloudAgentRegistry> {
        let model = self
            .db
            .get_cloud_agent_registry_by_id(id.as_str())
            .expect("Failed to fetch CloudAgentRegistry");
        model.map(|m| m.into_js(&env))
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    #[napi]
    pub fn get_content_active_censor(&mut self, env: Env) -> Vec<JsContentActiveCensor> {
        let models = self
            .db
            .get_content_active_censor()
            .expect("Could not fetch ContenActiveCensor");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_content_active_censor_by_id(
        &mut self,
        env: Env,
        id: String,
    ) -> Option<JsContentActiveCensor> {
        let model = self
            .db
            .get_content_active_censor_by_id(id.as_str())
            .expect("Failed to fetch ContenActiveCensor");
        model.map(|m| m.into_js(&env))
    }

    // -- ContentCue -------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_cue(&mut self, env: Env) -> Vec<JsContentCue> {
        let models = self
            .db
            .get_content_cue()
            .expect("Could not fetch ContentCue");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_content_cue_by_id(&mut self, env: Env, id: String) -> Option<JsContentCue> {
        let model = self
            .db
            .get_content_cue_by_id(id.as_str())
            .expect("Failed to fetch ContentCue");
        model.map(|m| m.into_js(&env))
    }

    // -- ContentFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_file(&mut self, env: Env) -> Vec<JsContentFile> {
        let models = self
            .db
            .get_content_file()
            .expect("Could not fetch ContentFile");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_content_file_by_id(&mut self, env: Env, id: String) -> Option<JsContentFile> {
        let model = self
            .db
            .get_content_file_by_id(id.as_str())
            .expect("Failed to fetch ContentFile");
        model.map(|m| m.into_js(&env))
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    #[napi]
    pub fn get_active_censor(&mut self, env: Env) -> Vec<JsDjmdActiveCensor> {
        let models = self
            .db
            .get_active_censor()
            .expect("Could not fetch DjmdActiveCensor");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_active_censor_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdActiveCensor> {
        let model = self
            .db
            .get_active_censor_by_id(id.as_str())
            .expect("Failed to fetch DjmdActiveCensor");
        model.map(|m| m.into_js(&env))
    }

    // -- Album ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_album(&mut self, env: Env) -> Vec<JsDjmdAlbum> {
        let models = self.db.get_album().expect("Could not fetch DjmdAlbum");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_album_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdAlbum> {
        let model = self
            .db
            .get_album_by_id(id.as_str())
            .expect("Failed to fetch DjmdAlbum");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_album_by_name(&mut self, env: Env, name: String) -> Option<JsDjmdAlbum> {
        let model = self
            .db
            .get_album_by_name(name.as_str())
            .expect("Failed to fetch DjmdAlbum");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn insert_album(
        &mut self,
        env: Env,
        name: String,
        artist_id: Option<String>,
        image_path: Option<String>,
        compilation: Option<i32>,
    ) -> JsDjmdAlbum {
        let model = self
            .db
            .insert_album(name, artist_id, image_path, compilation)
            .expect("Failed to fetch DjmdAlbum");
        model.into_js(&env)
    }

    #[napi]
    pub fn update_album(&mut self, env: Env, object: JsDjmdAlbum) -> JsDjmdAlbum {
        let mut model: DjmdAlbum = DjmdAlbum::from(object);
        let updated_model = self
            .db
            .update_album(&mut model)
            .expect("Failed to update DjmdAlbum");
        updated_model.into_js(&env)
    }

    #[napi]
    pub fn delete_album(&mut self, id: String) {
        self.db
            .delete_album(id.as_str())
            .expect("Failed to delete DjmdAlbum");
    }

    // -- Artist -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_artist(&mut self, env: Env) -> Vec<JsDjmdArtist> {
        let models = self.db.get_artist().expect("Could not fetch DjmdArtist");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_artist_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdArtist> {
        let model = self
            .db
            .get_artist_by_id(id.as_str())
            .expect("Failed to fetch DjmdArtist");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_artist_by_name(&mut self, env: Env, name: String) -> Option<JsDjmdArtist> {
        let model = self
            .db
            .get_artist_by_name(name.as_str())
            .expect("Failed to fetch DjmdArtist");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn insert_artist(&mut self, env: Env, name: String) -> JsDjmdArtist {
        let model = self
            .db
            .insert_artist(name)
            .expect("Failed to fetch DjmdArtist");
        model.into_js(&env)
    }

    #[napi]
    pub fn update_artist(&mut self, env: Env, object: JsDjmdArtist) -> JsDjmdArtist {
        let mut model: DjmdArtist = DjmdArtist::from(object);
        let updated_model = self
            .db
            .update_artist(&mut model)
            .expect("Failed to update DjmdArtist");
        updated_model.into_js(&env)
    }

    #[napi]
    pub fn delete_artist(&mut self, id: String) {
        self.db
            .delete_artist(id.as_str())
            .expect("Failed to delete DjmdArtist");
    }

    // -- Category ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_category(&mut self, env: Env) -> Vec<JsDjmdCategory> {
        let models = self
            .db
            .get_category()
            .expect("Could not fetch DjmdCategory");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_category_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdCategory> {
        let model = self
            .db
            .get_category_by_id(id.as_str())
            .expect("Failed to fetch DjmdCategory");
        model.map(|m| m.into_js(&env))
    }

    // -- Color ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_color(&mut self, env: Env) -> Vec<JsDjmdColor> {
        let models = self.db.get_color().expect("Could not fetch DjmdColor");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_color_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdColor> {
        let model = self
            .db
            .get_color_by_id(id.as_str())
            .expect("Failed to fetch DjmdColor");
        model.map(|m| m.into_js(&env))
    }

    // -- Content ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_content(&mut self, env: Env) -> Vec<JsDjmdContent> {
        let models = self.db.get_content().expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_content_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdContent> {
        let model = self
            .db
            .get_content_by_id(id.as_str())
            .expect("Failed to fetch DjmdContent");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_content_by_path(&mut self, env: Env, id: String) -> Option<JsDjmdContent> {
        let model = self
            .db
            .get_content_by_path(id.as_str())
            .expect("Failed to fetch DjmdContent");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_content_anlz_dir(&mut self, id: String) -> Option<String> {
        let dir = self
            .db
            .get_content_anlz_dir(id.as_str())
            .expect("Failed to get ANLZ directory");
        if let Some(dir) = dir {
            return Some(dir.as_os_str().to_str().unwrap().to_string());
        }
        None
    }

    #[napi]
    pub fn get_content_anlz_paths(&mut self, id: String) -> Option<JsAnlzPaths> {
        let paths = self
            .db
            .get_content_anlz_paths(id.as_str())
            .expect("Failed to get ANLZ directory");
        if let Some(paths) = paths {
            return Some(JsAnlzPaths::from(paths));
        }
        None
    }

    #[napi]
    pub fn insert_content(&mut self, env: Env, path: String) -> JsDjmdContent {
        let model = self
            .db
            .insert_content(path)
            .expect("Failed to insert DjmdContent");
        model.into_js(&env)
    }

    #[napi]
    pub fn update_content(&mut self, object: JsDjmdContent) -> i32 {
        let mut model: DjmdContent = DjmdContent::from(object);
        let result = self
            .db
            .update_content(&mut model)
            .expect("Failed to update DjmdContent");
        result as i32
    }

    #[napi]
    pub fn update_content_album(&mut self, id: String, name: String) -> i32 {
        let result = self
            .db
            .update_content_album(&id, &name)
            .expect("Failed to update content album");
        result as i32
    }

    #[napi]
    pub fn update_content_artist(&mut self, id: String, name: String) -> i32 {
        let result = self
            .db
            .update_content_artist(&id, &name)
            .expect("Failed to update content artist");
        result as i32
    }

    #[napi]
    pub fn update_content_remixer(&mut self, id: String, name: String) -> i32 {
        let result = self
            .db
            .update_content_remixer(&id, &name)
            .expect("Failed to update content remixer");
        result as i32
    }

    #[napi]
    pub fn update_content_original_artist(&mut self, id: String, name: String) -> i32 {
        let result = self
            .db
            .update_content_original_artist(&id, &name)
            .expect("Failed to update content original artist");
        result as i32
    }

    #[napi]
    pub fn update_content_composer(&mut self, id: String, name: String) -> i32 {
        let result = self
            .db
            .update_content_composer(&id, &name)
            .expect("Failed to update content composer");
        result as i32
    }

    #[napi]
    pub fn update_content_genre(&mut self, id: String, name: String) -> i32 {
        let result = self
            .db
            .update_content_genre(&id, &name)
            .expect("Failed to update content genre");
        result as i32
    }

    #[napi]
    pub fn update_content_label(&mut self, id: String, name: String) -> i32 {
        let result = self
            .db
            .update_content_label(&id, &name)
            .expect("Failed to update content label");
        result as i32
    }

    #[napi]
    pub fn update_content_key(&mut self, id: String, name: String) -> i32 {
        let result = self
            .db
            .update_content_key(&id, &name)
            .expect("Failed to update content key");
        result as i32
    }

    // #[napi]
    // pub fn delete_content(&mut self, id: String) {
    //     self.db
    //         .delete_content(id.as_str())
    //         .expect("Failed to delete DjmdContent");
    // }

    // -- Cue --------------------------------------------------------------------------------------

    #[napi]
    pub fn get_cue(&mut self, env: Env) -> Vec<JsDjmdCue> {
        let models = self.db.get_cue().expect("Could not fetch DjmdCue");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_cue_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdCue> {
        let model = self
            .db
            .get_cue_by_id(id.as_str())
            .expect("Failed to fetch DjmdCue");
        model.map(|m| m.into_js(&env))
    }

    // -- Device -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_device(&mut self, env: Env) -> Vec<JsDjmdDevice> {
        let models = self.db.get_device().expect("Could not fetch DjmdDevice");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_device_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdDevice> {
        let model = self
            .db
            .get_device_by_id(id.as_str())
            .expect("Failed to fetch DjmdDevice");
        model.map(|m| m.into_js(&env))
    }

    // -- Genre ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_genre(&mut self, env: Env) -> Vec<JsDjmdGenre> {
        let models = self.db.get_genre().expect("Could not fetch DjmdGenre");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_genre_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdGenre> {
        let model = self
            .db
            .get_genre_by_id(id.as_str())
            .expect("Failed to fetch DjmdGenre");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_genre_by_name(&mut self, env: Env, name: String) -> Option<JsDjmdGenre> {
        let model = self
            .db
            .get_genre_by_name(name.as_str())
            .expect("Failed to fetch DjmdGenre");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn insert_genre(&mut self, env: Env, name: String) -> JsDjmdGenre {
        let model = self
            .db
            .insert_genre(name)
            .expect("Failed to fetch DjmdGenre");
        model.into_js(&env)
    }

    #[napi]
    pub fn update_genre(&mut self, env: Env, object: JsDjmdGenre) -> JsDjmdGenre {
        let mut model: DjmdGenre = DjmdGenre::from(object);
        let updated_model = self
            .db
            .update_genre(&mut model)
            .expect("Failed to update DjmdGenre");
        updated_model.into_js(&env)
    }

    #[napi]
    pub fn delete_genre(&mut self, id: String) {
        self.db
            .delete_genre(id.as_str())
            .expect("Failed to delete DjmdGenre");
    }

    // -- History ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_history(&mut self, env: Env) -> Vec<JsDjmdHistory> {
        let models = self.db.get_history().expect("Could not fetch DjmdGenre");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_history_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdHistory> {
        let model = self
            .db
            .get_history_by_id(id.as_str())
            .expect("Failed to fetch DjmdGenre");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_history_songs(&mut self, env: Env, id: String) -> Vec<JsDjmdSongHistory> {
        let models = self
            .db
            .get_history_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_history_contents(&mut self, env: Env, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_history_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    #[napi]
    pub fn get_hot_cue_banklist(&mut self, env: Env) -> Vec<JsDjmdHotCueBanklist> {
        let models = self
            .db
            .get_hot_cue_banklist()
            .expect("Could not fetch DjmdHotCueBanklist");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_hot_cue_banklist_children(
        &mut self,
        env: Env,
        id: String,
    ) -> Vec<JsDjmdHotCueBanklist> {
        let models = self
            .db
            .get_hot_cue_banklist_children(id.as_str())
            .expect("Failed to fetch DjmdHotCueBanklist");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_hot_cue_banklist_by_id(
        &mut self,
        env: Env,
        id: String,
    ) -> Option<JsDjmdHotCueBanklist> {
        let model = self
            .db
            .get_hot_cue_banklist_by_id(id.as_str())
            .expect("Failed to fetch DjmdHotCueBanklist");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_hot_cue_banklist_songs(
        &mut self,
        env: Env,
        id: String,
    ) -> Vec<JsDjmdSongHotCueBanklist> {
        let models = self
            .db
            .get_hot_cue_banklist_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_hot_cue_banklist_contents(&mut self, env: Env, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_hot_cue_banklist_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_hot_cue_banklist_cues(&mut self, env: Env, id: String) -> Vec<JsHotCueBanklistCue> {
        let models = self
            .db
            .get_hot_cue_banklist_cues(id.as_str())
            .expect("Could not fetch HotCueBanklistCue");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    // -- Key --------------------------------------------------------------------------------------

    #[napi]
    pub fn get_key(&mut self, env: Env) -> Vec<JsDjmdKey> {
        let models = self.db.get_key().expect("Could not fetch DjmdKey");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_key_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdKey> {
        let model = self
            .db
            .get_key_by_id(id.as_str())
            .expect("Failed to fetch DjmdKey");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_key_by_name(&mut self, env: Env, name: String) -> Option<JsDjmdKey> {
        let model = self
            .db
            .get_key_by_name(name.as_str())
            .expect("Failed to fetch DjmdKey");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn insert_key(&mut self, env: Env, name: String) -> JsDjmdKey {
        let model = self.db.insert_key(name).expect("Failed to fetch DjmdKey");
        model.into_js(&env)
    }

    #[napi]
    pub fn update_key(&mut self, env: Env, object: JsDjmdKey) -> JsDjmdKey {
        let mut model: DjmdKey = DjmdKey::from(object);
        let updated_model = self
            .db
            .update_key(&mut model)
            .expect("Failed to update DjmdKey");
        updated_model.into_js(&env)
    }

    #[napi]
    pub fn delete_key(&mut self, id: String) {
        self.db
            .delete_key(id.as_str())
            .expect("Failed to delete DjmdKey");
    }

    // -- Label ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_label(&mut self, env: Env) -> Vec<JsDjmdLabel> {
        let models = self.db.get_label().expect("Could not fetch DjmdLabel");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_label_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdLabel> {
        let model = self
            .db
            .get_label_by_id(id.as_str())
            .expect("Failed to fetch DjmdLabel");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_label_by_name(&mut self, env: Env, name: String) -> Option<JsDjmdLabel> {
        let model = self
            .db
            .get_label_by_name(name.as_str())
            .expect("Failed to fetch DjmdLabel");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn insert_label(&mut self, env: Env, name: String) -> JsDjmdLabel {
        let model = self
            .db
            .insert_label(name)
            .expect("Failed to fetch DjmdLabel");
        model.into_js(&env)
    }

    #[napi]
    pub fn update_label(&mut self, env: Env, object: JsDjmdLabel) -> JsDjmdLabel {
        let mut model: DjmdLabel = DjmdLabel::from(object);
        let updated_model = self
            .db
            .update_label(&mut model)
            .expect("Failed to update DjmdLabel");
        updated_model.into_js(&env)
    }

    #[napi]
    pub fn delete_label(&mut self, id: String) {
        self.db
            .delete_label(id.as_str())
            .expect("Failed to delete DjmdLabel");
    }

    // -- MenuItems --------------------------------------------------------------------------------

    #[napi]
    pub fn get_menu_item(&mut self, env: Env) -> Vec<JsDjmdMenuItems> {
        let models = self
            .db
            .get_menu_item()
            .expect("Could not fetch DjmdMenuItems");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_menu_item_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdMenuItems> {
        let model = self
            .db
            .get_menu_item_by_id(id.as_str())
            .expect("Failed to fetch DjmdMenuItems");
        model.map(|m| m.into_js(&env))
    }

    // -- MixerParam -------------------------------------------------------------------------------

    #[napi]
    pub fn get_mixer_param(&mut self, env: Env) -> Vec<JsDjmdMixerParam> {
        let models = self
            .db
            .get_mixer_param()
            .expect("Could not fetch DjmdMixerParam");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_mixer_param_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdMixerParam> {
        let model = self
            .db
            .get_mixer_param_by_id(id.as_str())
            .expect("Failed to fetch DjmdMixerParam");
        model.map(|m| m.into_js(&env))
    }

    // -- MyTag ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_my_tag(&mut self, env: Env) -> Vec<JsDjmdMyTag> {
        let models = self.db.get_my_tag().expect("Could not fetch DjmdMyTag");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_my_tag_children(&mut self, env: Env, id: String) -> Vec<JsDjmdMyTag> {
        let models = self
            .db
            .get_my_tag_children(id.as_str())
            .expect("Failed to fetch DjmdMyTag");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_my_tag_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdMyTag> {
        let model = self
            .db
            .get_my_tag_by_id(id.as_str())
            .expect("Failed to fetch DjmdMyTag");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_my_tag_songs(&mut self, env: Env, id: String) -> Vec<JsDjmdSongMyTag> {
        let models = self
            .db
            .get_my_tag_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_my_tag_contents(&mut self, env: Env, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_my_tag_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    // -- Playlist ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_playlist(&mut self, env: Env) -> Vec<JsDjmdPlaylist> {
        let models = self
            .db
            .get_playlist()
            .expect("Could not fetch DjmdPlaylist");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_playlist_tree(&mut self, env: Env) -> Vec<JsDjmdPlaylistTreeItem> {
        let models = self
            .db
            .get_playlist_tree()
            .expect("Could not fetch DjmdPlaylist");
        models
            .into_iter()
            .map(|m| m.borrow().clone().into_js(&env))
            .collect()
    }

    #[napi]
    pub fn get_playlist_children(&mut self, env: Env, id: String) -> Vec<JsDjmdPlaylist> {
        let models = self
            .db
            .get_playlist_children(id.as_str())
            .expect("Failed to fetch DjmdPlaylist");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_playlist_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdPlaylist> {
        let model = self
            .db
            .get_playlist_by_id(id.as_str())
            .expect("Failed to fetch DjmdPlaylist");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_playlist_by_path(&mut self, env: Env, path: Vec<String>) -> Option<JsDjmdPlaylist> {
        let path_str: Vec<&str> = path.iter().map(|item| item.as_str()).collect();
        let model = self
            .db
            .get_playlist_by_path(path_str)
            .expect("Failed to fetch DjmdPlaylist");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_playlist_songs(&mut self, env: Env, id: String) -> Vec<JsDjmdSongPlaylist> {
        let models = self
            .db
            .get_playlist_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_playlist_contents(&mut self, env: Env, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_playlist_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_playlist_song_by_id(&mut self, env: Env, id: String) -> Vec<JsDjmdSongPlaylist> {
        let models = self
            .db
            .get_playlist_song_by_id(id.as_str())
            .expect("Could not fetch DjmdSongPlaylist");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn insert_playlist(
        &mut self,
        env: Env,
        name: String,
        attribute: i32,
        parent_id: Option<String>,
        seq: Option<i32>,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> JsDjmdPlaylist {
        let attr = PlaylistType::try_from(attribute).expect("Wrong Playlist attribute");
        let model = self
            .db
            .insert_playlist(name, attr, parent_id, seq, image_path, smart_list)
            .expect("Failed to insert DjmdPlaylist");
        model.into_js(&env)
    }

    #[napi]
    pub fn rename_playlist(&mut self, env: Env, id: String, name: String) -> JsDjmdPlaylist {
        let model = self
            .db
            .rename_playlist(&id, name)
            .expect("Failed to rename DjmdPlaylist");
        model.into_js(&env)
    }

    #[napi]
    pub fn move_playlist(
        &mut self,
        env: Env,
        id: String,
        seq: Option<i32>,
        parent_id: Option<String>,
    ) -> JsDjmdPlaylist {
        let model = self
            .db
            .move_playlist(&id, seq, parent_id)
            .expect("Failed to move DjmdPlaylist");
        model.into_js(&env)
    }

    #[napi]
    pub fn delete_playlist(&mut self, id: String) {
        self.db
            .delete_playlist(id.as_str())
            .expect("Failed to delete DjmdPlaylist");
    }

    #[napi]
    pub fn insert_playlist_song(
        &mut self,
        env: Env,
        playlist_id: String,
        content_id: String,
        seq: Option<i32>,
    ) -> JsDjmdSongPlaylist {
        let model = self
            .db
            .insert_playlist_song(&playlist_id, &content_id, seq)
            .expect("Failed to insert DjmdSongPlaylist");
        model.into_js(&env)
    }

    #[napi]
    pub fn move_playlist_song(&mut self, env: Env, id: String, seq: i32) -> JsDjmdSongPlaylist {
        let model = self
            .db
            .move_playlist_song(&id, seq)
            .expect("Failed to move DjmdSongPlaylist");
        model.into_js(&env)
    }

    #[napi]
    pub fn delete_playlist_song(&mut self, id: String) {
        self.db
            .delete_playlist_song(id.as_str())
            .expect("Failed to delete DjmdSongPlaylist");
    }

    // -- Property ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_property(&mut self, env: Env) -> Vec<JsDjmdProperty> {
        let models = self
            .db
            .get_property()
            .expect("Could not fetch DjmdProperty");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_property_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdProperty> {
        let model = self
            .db
            .get_property_by_id(id.as_str())
            .expect("Failed to fetch DjmdProperty");
        model.map(|m| m.into_js(&env))
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_property(&mut self, env: Env) -> Vec<JsDjmdCloudProperty> {
        let models = self
            .db
            .get_cloud_property()
            .expect("Could not fetch DjmdCloudProperty");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_cloud_property_by_id(
        &mut self,
        env: Env,
        id: String,
    ) -> Option<JsDjmdCloudProperty> {
        let model = self
            .db
            .get_cloud_property_by_id(id.as_str())
            .expect("Failed to fetch DjmdCloudProperty");
        model.map(|m| m.into_js(&env))
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    #[napi]
    pub fn get_recommend_like(&mut self, env: Env) -> Vec<JsDjmdRecommendLike> {
        let models = self
            .db
            .get_recommend_like()
            .expect("Could not fetch DjmdRecommendLike");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_recommend_like_by_id(
        &mut self,
        env: Env,
        id: String,
    ) -> Option<JsDjmdRecommendLike> {
        let model = self
            .db
            .get_recommend_like_by_id(id.as_str())
            .expect("Failed to fetch DjmdRecommendLike");
        model.map(|m| m.into_js(&env))
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    #[napi]
    pub fn get_related_tracks(&mut self, env: Env) -> Vec<JsDjmdRelatedTracks> {
        let models = self
            .db
            .get_related_tracks()
            .expect("Could not fetch DjmdRelatedTracks");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_related_tracks_children(
        &mut self,
        env: Env,
        id: String,
    ) -> Vec<JsDjmdRelatedTracks> {
        let models = self
            .db
            .get_related_tracks_children(id.as_str())
            .expect("Failed to fetch DjmdRelatedTracks");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_related_tracks_by_id(
        &mut self,
        env: Env,
        id: String,
    ) -> Option<JsDjmdRelatedTracks> {
        let model = self
            .db
            .get_related_tracks_by_id(id.as_str())
            .expect("Failed to fetch DjmdRelatedTracks");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_related_tracks_songs(
        &mut self,
        env: Env,
        id: String,
    ) -> Vec<JsDjmdSongRelatedTracks> {
        let models = self
            .db
            .get_related_tracks_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_related_tracks_contents(&mut self, env: Env, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_related_tracks_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    // -- Sampler ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_sampler(&mut self, env: Env) -> Vec<JsDjmdSampler> {
        let models = self.db.get_sampler().expect("Could not fetch DjmdSampler");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_sampler_children(&mut self, env: Env, id: String) -> Vec<JsDjmdSampler> {
        let models = self
            .db
            .get_sampler_children(id.as_str())
            .expect("Failed to fetch DjmdSampler");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_sampler_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdSampler> {
        let model = self
            .db
            .get_sampler_by_id(id.as_str())
            .expect("Failed to fetch DjmdSampler");
        model.map(|m| m.into_js(&env))
    }

    #[napi]
    pub fn get_sampler_songs(&mut self, env: Env, id: String) -> Vec<JsDjmdSongSampler> {
        let models = self
            .db
            .get_sampler_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_sampler_contents(&mut self, env: Env, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_sampler_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    // -- SongTagList ------------------------------------------------------------------------------

    #[napi]
    pub fn get_song_tag_list(&mut self, env: Env) -> Vec<JsDjmdSongTagList> {
        let models = self
            .db
            .get_song_tag_list()
            .expect("Could not fetch DjmdSongTagList");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_song_tag_list_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdSongTagList> {
        let model = self
            .db
            .get_song_tag_list_by_id(id.as_str())
            .expect("Failed to fetch DjmdSongTagList");
        model.map(|m| m.into_js(&env))
    }

    // -- Sort -------------------------------------------------------------------------------------

    #[napi]
    pub fn get_sort(&mut self, env: Env) -> Vec<JsDjmdSort> {
        let models = self.db.get_sort().expect("Could not fetch DjmdSort");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_sort_by_id(&mut self, env: Env, id: String) -> Option<JsDjmdSort> {
        let model = self
            .db
            .get_sort_by_id(id.as_str())
            .expect("Failed to fetch DjmdSort");
        model.map(|m| m.into_js(&env))
    }

    // -- ImageFile --------------------------------------------------------------------------------

    #[napi]
    pub fn get_image_file(&mut self, env: Env) -> Vec<JsImageFile> {
        let models = self.db.get_image_file().expect("Could not fetch ImageFile");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_image_file_by_id(&mut self, env: Env, id: String) -> Option<JsImageFile> {
        let model = self
            .db
            .get_image_file_by_id(id.as_str())
            .expect("Failed to fetch ImageFile");
        model.map(|m| m.into_js(&env))
    }

    // -- SettingFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_setting_file(&mut self, env: Env) -> Vec<JsSettingFile> {
        let models = self
            .db
            .get_setting_file()
            .expect("Could not fetch SettingFile");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_setting_file_by_id(&mut self, env: Env, id: String) -> Option<JsSettingFile> {
        let model = self
            .db
            .get_setting_file_by_id(id.as_str())
            .expect("Failed to fetch SettingFile");
        model.map(|m| m.into_js(&env))
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    #[napi]
    pub fn get_uuid_id_map(&mut self, env: Env) -> Vec<JsUuidIDMap> {
        let models = self
            .db
            .get_uuid_id_map()
            .expect("Could not fetch UuidIDMap");
        models.into_iter().map(|m| m.into_js(&env)).collect()
    }

    #[napi]
    pub fn get_uuid_id_map_by_id(&mut self, env: Env, id: String) -> Option<JsUuidIDMap> {
        let model = self
            .db
            .get_uuid_id_map_by_id(id.as_str())
            .expect("Failed to fetch UuidIDMap");
        model.map(|m| m.into_js(&env))
    }
}
