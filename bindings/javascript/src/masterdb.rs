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
    pub fn get_agent_registry<'a>(&mut self, env: &'a Env) -> Vec<JsAgentRegistry<'a>> {
        let models = self
            .db
            .get_agent_registry()
            .expect("Could not fetch AgentRegistry");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_agent_registry_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsAgentRegistry<'a>> {
        let model = self
            .db
            .get_agent_registry_by_id(id.as_str())
            .expect("Failed to fetch AgentRegistry");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_local_usn(&mut self) -> i32 {
        self.db.get_local_usn().expect("Failed to fetch Local USN")
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_agent_registry<'a>(&mut self, env: &'a Env) -> Vec<JsCloudAgentRegistry<'a>> {
        let models = self
            .db
            .get_cloud_agent_registry()
            .expect("Could not fetch CloudAgentRegistry");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_cloud_agent_registry_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsCloudAgentRegistry<'a>> {
        let model = self
            .db
            .get_cloud_agent_registry_by_id(id.as_str())
            .expect("Failed to fetch CloudAgentRegistry");
        model.map(|m| m.into_js(env))
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    #[napi]
    pub fn get_content_active_censor<'a>(
        &mut self,
        env: &'a Env,
    ) -> Vec<JsContentActiveCensor<'a>> {
        let models = self
            .db
            .get_content_active_censor()
            .expect("Could not fetch ContenActiveCensor");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_content_active_censor_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsContentActiveCensor<'a>> {
        let model = self
            .db
            .get_content_active_censor_by_id(id.as_str())
            .expect("Failed to fetch ContenActiveCensor");
        model.map(|m| m.into_js(env))
    }

    // -- ContentCue -------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_cue<'a>(&mut self, env: &'a Env) -> Vec<JsContentCue<'a>> {
        let models = self
            .db
            .get_content_cue()
            .expect("Could not fetch ContentCue");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_content_cue_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsContentCue<'a>> {
        let model = self
            .db
            .get_content_cue_by_id(id.as_str())
            .expect("Failed to fetch ContentCue");
        model.map(|m| m.into_js(env))
    }

    // -- ContentFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_file<'a>(&mut self, env: &'a Env) -> Vec<JsContentFile<'a>> {
        let models = self
            .db
            .get_content_file()
            .expect("Could not fetch ContentFile");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_content_file_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsContentFile<'a>> {
        let model = self
            .db
            .get_content_file_by_id(id.as_str())
            .expect("Failed to fetch ContentFile");
        model.map(|m| m.into_js(env))
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    #[napi]
    pub fn get_active_censor<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdActiveCensor<'a>> {
        let models = self
            .db
            .get_active_censor()
            .expect("Could not fetch DjmdActiveCensor");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_active_censor_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdActiveCensor<'a>> {
        let model = self
            .db
            .get_active_censor_by_id(id.as_str())
            .expect("Failed to fetch DjmdActiveCensor");
        model.map(|m| m.into_js(env))
    }

    // -- Album ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_album<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdAlbum<'a>> {
        let models = self.db.get_album().expect("Could not fetch DjmdAlbum");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_album_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdAlbum<'a>> {
        let model = self
            .db
            .get_album_by_id(id.as_str())
            .expect("Failed to fetch DjmdAlbum");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_album_by_name<'a>(&mut self, env: &'a Env, name: String) -> Option<JsDjmdAlbum<'a>> {
        let model = self
            .db
            .get_album_by_name(name.as_str())
            .expect("Failed to fetch DjmdAlbum");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn insert_album<'a>(
        &mut self,
        env: &'a Env,
        name: String,
        artist_id: Option<String>,
        image_path: Option<String>,
        compilation: Option<i32>,
    ) -> JsDjmdAlbum<'a> {
        let model = self
            .db
            .insert_album(name, artist_id, image_path, compilation)
            .expect("Failed to fetch DjmdAlbum");
        model.into_js(env)
    }

    #[napi]
    pub fn update_album<'a>(&mut self, env: &'a Env, object: JsDjmdAlbum) -> JsDjmdAlbum<'a> {
        let mut model: DjmdAlbum = DjmdAlbum::from(object);
        let updated_model = self
            .db
            .update_album(&mut model)
            .expect("Failed to update DjmdAlbum");
        updated_model.into_js(env)
    }

    #[napi]
    pub fn delete_album(&mut self, id: String) {
        self.db
            .delete_album(id.as_str())
            .expect("Failed to delete DjmdAlbum");
    }

    // -- Artist -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_artist<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdArtist<'a>> {
        let models = self.db.get_artist().expect("Could not fetch DjmdArtist");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_artist_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdArtist<'a>> {
        let model = self
            .db
            .get_artist_by_id(id.as_str())
            .expect("Failed to fetch DjmdArtist");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_artist_by_name<'a>(
        &mut self,
        env: &'a Env,
        name: String,
    ) -> Option<JsDjmdArtist<'a>> {
        let model = self
            .db
            .get_artist_by_name(name.as_str())
            .expect("Failed to fetch DjmdArtist");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn insert_artist<'a>(&mut self, env: &'a Env, name: String) -> JsDjmdArtist<'a> {
        let model = self
            .db
            .insert_artist(name)
            .expect("Failed to fetch DjmdArtist");
        model.into_js(env)
    }

    #[napi]
    pub fn update_artist<'a>(&mut self, env: &'a Env, object: JsDjmdArtist) -> JsDjmdArtist<'a> {
        let mut model: DjmdArtist = DjmdArtist::from(object);
        let updated_model = self
            .db
            .update_artist(&mut model)
            .expect("Failed to update DjmdArtist");
        updated_model.into_js(env)
    }

    #[napi]
    pub fn delete_artist(&mut self, id: String) {
        self.db
            .delete_artist(id.as_str())
            .expect("Failed to delete DjmdArtist");
    }

    // -- Category ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_category<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdCategory<'a>> {
        let models = self
            .db
            .get_category()
            .expect("Could not fetch DjmdCategory");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_category_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdCategory<'a>> {
        let model = self
            .db
            .get_category_by_id(id.as_str())
            .expect("Failed to fetch DjmdCategory");
        model.map(|m| m.into_js(env))
    }

    // -- Color ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_color<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdColor<'a>> {
        let models = self.db.get_color().expect("Could not fetch DjmdColor");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_color_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdColor<'a>> {
        let model = self
            .db
            .get_color_by_id(id.as_str())
            .expect("Failed to fetch DjmdColor");
        model.map(|m| m.into_js(env))
    }

    // -- Content ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_content<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdContent<'a>> {
        let models = self.db.get_content().expect("Could not fetch djmdContent");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_content_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdContent<'a>> {
        let model = self
            .db
            .get_content_by_id(id.as_str())
            .expect("Failed to fetch djmdContent");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_content_by_path<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdContent<'a>> {
        let model = self
            .db
            .get_content_by_path(id.as_str())
            .expect("Failed to fetch djmdContent");
        model.map(|m| m.into_js(env))
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
    pub fn insert_content<'a>(&mut self, env: &'a Env, path: String) -> JsDjmdContent<'a> {
        let model = self
            .db
            .insert_content(path)
            .expect("Failed to insert DjmdContent");
        model.into_js(env)
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
    pub fn get_cue<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdCue<'a>> {
        let models = self.db.get_cue().expect("Could not fetch DjmdCue");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_cue_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdCue<'a>> {
        let model = self
            .db
            .get_cue_by_id(id.as_str())
            .expect("Failed to fetch DjmdCue");
        model.map(|m| m.into_js(env))
    }

    // -- Device -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_device<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdDevice<'a>> {
        let models = self.db.get_device().expect("Could not fetch DjmdDevice");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_device_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdDevice<'a>> {
        let model = self
            .db
            .get_device_by_id(id.as_str())
            .expect("Failed to fetch DjmdDevice");
        model.map(|m| m.into_js(env))
    }

    // -- Genre ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_genre<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdGenre<'a>> {
        let models = self.db.get_genre().expect("Could not fetch DjmdGenre");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_genre_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdGenre<'a>> {
        let model = self
            .db
            .get_genre_by_id(id.as_str())
            .expect("Failed to fetch DjmdGenre");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_genre_by_name<'a>(&mut self, env: &'a Env, name: String) -> Option<JsDjmdGenre<'a>> {
        let model = self
            .db
            .get_genre_by_name(name.as_str())
            .expect("Failed to fetch DjmdGenre");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn insert_genre<'a>(&mut self, env: &'a Env, name: String) -> JsDjmdGenre<'a> {
        let model = self
            .db
            .insert_genre(name)
            .expect("Failed to fetch DjmdGenre");
        model.into_js(env)
    }

    #[napi]
    pub fn update_genre<'a>(&mut self, env: &'a Env, object: JsDjmdGenre) -> JsDjmdGenre<'a> {
        let mut model: DjmdGenre = DjmdGenre::from(object);
        let updated_model = self
            .db
            .update_genre(&mut model)
            .expect("Failed to update DjmdGenre");
        updated_model.into_js(env)
    }

    #[napi]
    pub fn delete_genre(&mut self, id: String) {
        self.db
            .delete_genre(id.as_str())
            .expect("Failed to delete DjmdGenre");
    }

    // -- History ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_history<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdHistory<'a>> {
        let models = self.db.get_history().expect("Could not fetch djmdGenre");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_history_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdHistory<'a>> {
        let model = self
            .db
            .get_history_by_id(id.as_str())
            .expect("Failed to fetch djmdGenre");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_history_songs<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdSongHistory<'a>> {
        let models = self
            .db
            .get_history_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_history_contents<'a>(&mut self, env: &'a Env, id: String) -> Vec<JsDjmdContent<'a>> {
        let models = self
            .db
            .get_history_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    #[napi]
    pub fn get_hot_cue_banklist<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdHotCueBanklist<'a>> {
        let models = self
            .db
            .get_hot_cue_banklist()
            .expect("Could not fetch DjmdHotCueBanklist");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_hot_cue_banklist_children<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdHotCueBanklist<'a>> {
        let models = self
            .db
            .get_hot_cue_banklist_children(id.as_str())
            .expect("Failed to fetch DjmdHotCueBanklist");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_hot_cue_banklist_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdHotCueBanklist<'a>> {
        let model = self
            .db
            .get_hot_cue_banklist_by_id(id.as_str())
            .expect("Failed to fetch DjmdHotCueBanklist");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_hot_cue_banklist_songs<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdSongHotCueBanklist<'a>> {
        let models = self
            .db
            .get_hot_cue_banklist_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_hot_cue_banklist_contents<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdContent<'a>> {
        let models = self
            .db
            .get_hot_cue_banklist_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_hot_cue_banklist_cues<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsHotCueBanklistCue<'a>> {
        let models = self
            .db
            .get_hot_cue_banklist_cues(id.as_str())
            .expect("Could not fetch HotCueBanklistCue");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    // -- Key --------------------------------------------------------------------------------------

    #[napi]
    pub fn get_key<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdKey<'a>> {
        let models = self.db.get_key().expect("Could not fetch DjmdKey");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_key_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdKey<'a>> {
        let model = self
            .db
            .get_key_by_id(id.as_str())
            .expect("Failed to fetch DjmdKey");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_key_by_name<'a>(&mut self, env: &'a Env, name: String) -> Option<JsDjmdKey<'a>> {
        let model = self
            .db
            .get_key_by_name(name.as_str())
            .expect("Failed to fetch DjmdKey");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn insert_key<'a>(&mut self, env: &'a Env, name: String) -> JsDjmdKey<'a> {
        let model = self.db.insert_key(name).expect("Failed to fetch DjmdKey");
        model.into_js(env)
    }

    #[napi]
    pub fn update_key<'a>(&mut self, env: &'a Env, object: JsDjmdKey) -> JsDjmdKey<'a> {
        let mut model: DjmdKey = DjmdKey::from(object);
        let updated_model = self
            .db
            .update_key(&mut model)
            .expect("Failed to update DjmdKey");
        updated_model.into_js(env)
    }

    #[napi]
    pub fn delete_key(&mut self, id: String) {
        self.db
            .delete_key(id.as_str())
            .expect("Failed to delete DjmdKey");
    }

    // -- Label ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_label<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdLabel<'a>> {
        let models = self.db.get_label().expect("Could not fetch DjmdLabel");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_label_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdLabel<'a>> {
        let model = self
            .db
            .get_label_by_id(id.as_str())
            .expect("Failed to fetch DjmdLabel");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_label_by_name<'a>(&mut self, env: &'a Env, name: String) -> Option<JsDjmdLabel<'a>> {
        let model = self
            .db
            .get_label_by_name(name.as_str())
            .expect("Failed to fetch DjmdLabel");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn insert_label<'a>(&mut self, env: &'a Env, name: String) -> JsDjmdLabel<'a> {
        let model = self
            .db
            .insert_label(name)
            .expect("Failed to fetch DjmdLabel");
        model.into_js(env)
    }

    #[napi]
    pub fn update_label<'a>(&mut self, env: &'a Env, object: JsDjmdLabel) -> JsDjmdLabel<'a> {
        let mut model: DjmdLabel = DjmdLabel::from(object);
        let updated_model = self
            .db
            .update_label(&mut model)
            .expect("Failed to update DjmdLabel");
        updated_model.into_js(env)
    }

    #[napi]
    pub fn delete_label(&mut self, id: String) {
        self.db
            .delete_label(id.as_str())
            .expect("Failed to delete DjmdLabel");
    }

    // -- MenuItems --------------------------------------------------------------------------------

    #[napi]
    pub fn get_menu_item<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdMenuItems<'a>> {
        let models = self
            .db
            .get_menu_item()
            .expect("Could not fetch DjmdMenuItems");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_menu_item_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdMenuItems<'a>> {
        let model = self
            .db
            .get_menu_item_by_id(id.as_str())
            .expect("Failed to fetch DjmdMenuItems");
        model.map(|m| m.into_js(env))
    }

    // -- MixerParam -------------------------------------------------------------------------------

    #[napi]
    pub fn get_mixer_param<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdMixerParam<'a>> {
        let models = self
            .db
            .get_mixer_param()
            .expect("Could not fetch djmdMixerParam");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_mixer_param_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdMixerParam<'a>> {
        let model = self
            .db
            .get_mixer_param_by_id(id.as_str())
            .expect("Failed to fetch djmdMixerParam");
        model.map(|m| m.into_js(env))
    }

    // -- MyTag ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_my_tag<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdMyTag<'a>> {
        let models = self.db.get_my_tag().expect("Could not fetch DjmdMyTag");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_my_tag_children<'a>(&mut self, env: &'a Env, id: String) -> Vec<JsDjmdMyTag<'a>> {
        let models = self
            .db
            .get_my_tag_children(id.as_str())
            .expect("Failed to fetch DjmdMyTag");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_my_tag_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdMyTag<'a>> {
        let model = self
            .db
            .get_my_tag_by_id(id.as_str())
            .expect("Failed to fetch DjmdMyTag");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_my_tag_songs<'a>(&mut self, env: &'a Env, id: String) -> Vec<JsDjmdSongMyTag<'a>> {
        let models = self
            .db
            .get_my_tag_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_my_tag_contents<'a>(&mut self, env: &'a Env, id: String) -> Vec<JsDjmdContent<'a>> {
        let models = self
            .db
            .get_my_tag_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    // -- Playlist ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_playlist<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdPlaylist<'a>> {
        let models = self
            .db
            .get_playlist()
            .expect("Could not fetch DjmdPlaylist");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_playlist_tree<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdPlaylistTreeItem<'a>> {
        let models = self
            .db
            .get_playlist_tree()
            .expect("Could not fetch DjmdPlaylist");
        models
            .into_iter()
            .map(|m| m.borrow().clone().into_js(env))
            .collect()
    }

    #[napi]
    pub fn get_playlist_children<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdPlaylist<'a>> {
        let models = self
            .db
            .get_playlist_children(id.as_str())
            .expect("Failed to fetch DjmdPlaylist");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_playlist_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdPlaylist<'a>> {
        let model = self
            .db
            .get_playlist_by_id(id.as_str())
            .expect("Failed to fetch DjmdPlaylist");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_playlist_by_path<'a>(
        &mut self,
        env: &'a Env,
        path: Vec<String>,
    ) -> Option<JsDjmdPlaylist<'a>> {
        let path_str: Vec<&str> = path.iter().map(|item| item.as_str()).collect();
        let model = self
            .db
            .get_playlist_by_path(path_str)
            .expect("Failed to fetch DjmdPlaylist");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_playlist_songs<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdSongPlaylist<'a>> {
        let models = self
            .db
            .get_playlist_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_playlist_contents<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdContent<'a>> {
        let models = self
            .db
            .get_playlist_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_playlist_song_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdSongPlaylist<'a>> {
        let models = self
            .db
            .get_playlist_song_by_id(id.as_str())
            .expect("Could not fetch DjmdSongPlaylist");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn insert_playlist<'a>(
        &mut self,
        env: &'a Env,
        name: String,
        attribute: i32,
        parent_id: Option<String>,
        seq: Option<i32>,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> JsDjmdPlaylist<'a> {
        let attr = PlaylistType::try_from(attribute).expect("Wrong Playlist attribute");
        let model = self
            .db
            .insert_playlist(name, attr, parent_id, seq, image_path, smart_list)
            .expect("Failed to insert DjmdPlaylist");
        model.into_js(env)
    }

    #[napi]
    pub fn rename_playlist<'a>(
        &mut self,
        env: &'a Env,
        id: String,
        name: String,
    ) -> JsDjmdPlaylist<'a> {
        let model = self
            .db
            .rename_playlist(&id, name)
            .expect("Failed to rename DjmdPlaylist");
        model.into_js(env)
    }

    #[napi]
    pub fn move_playlist<'a>(
        &mut self,
        env: &'a Env,
        id: String,
        seq: Option<i32>,
        parent_id: Option<String>,
    ) -> JsDjmdPlaylist<'a> {
        let model = self
            .db
            .move_playlist(&id, seq, parent_id)
            .expect("Failed to move DjmdPlaylist");
        model.into_js(env)
    }

    #[napi]
    pub fn delete_playlist(&mut self, id: String) {
        self.db
            .delete_playlist(id.as_str())
            .expect("Failed to delete DjmdPlaylist");
    }

    #[napi]
    pub fn insert_playlist_song<'a>(
        &mut self,
        env: &'a Env,
        playlist_id: String,
        content_id: String,
        seq: Option<i32>,
    ) -> JsDjmdSongPlaylist<'a> {
        let model = self
            .db
            .insert_playlist_song(&playlist_id, &content_id, seq)
            .expect("Failed to insert DjmdSongPlaylist");
        model.into_js(env)
    }

    #[napi]
    pub fn move_playlist_song<'a>(
        &mut self,
        env: &'a Env,
        id: String,
        seq: i32,
    ) -> JsDjmdSongPlaylist<'a> {
        let model = self
            .db
            .move_playlist_song(&id, seq)
            .expect("Failed to move DjmdSongPlaylist");
        model.into_js(env)
    }

    #[napi]
    pub fn delete_playlist_song(&mut self, id: String) {
        self.db
            .delete_playlist_song(id.as_str())
            .expect("Failed to delete DjmdSongPlaylist");
    }

    // -- Property ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_property<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdProperty<'a>> {
        let models = self
            .db
            .get_property()
            .expect("Could not fetch DjmdProperty");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_property_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdProperty<'a>> {
        let model = self
            .db
            .get_property_by_id(id.as_str())
            .expect("Failed to fetch DjmdProperty");
        model.map(|m| m.into_js(env))
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_property<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdCloudProperty<'a>> {
        let models = self
            .db
            .get_cloud_property()
            .expect("Could not fetch DjmdCloudProperty");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_cloud_property_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdCloudProperty<'a>> {
        let model = self
            .db
            .get_cloud_property_by_id(id.as_str())
            .expect("Failed to fetch DjmdCloudProperty");
        model.map(|m| m.into_js(env))
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    #[napi]
    pub fn get_recommend_like<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdRecommendLike<'a>> {
        let models = self
            .db
            .get_recommend_like()
            .expect("Could not fetch DjmdRecommendLike");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_recommend_like_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdRecommendLike<'a>> {
        let model = self
            .db
            .get_recommend_like_by_id(id.as_str())
            .expect("Failed to fetch DjmdRecommendLike");
        model.map(|m| m.into_js(env))
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    #[napi]
    pub fn get_related_tracks<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdRelatedTracks<'a>> {
        let models = self
            .db
            .get_related_tracks()
            .expect("Could not fetch DjmdRelatedTracks");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_related_tracks_children<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdRelatedTracks<'a>> {
        let models = self
            .db
            .get_related_tracks_children(id.as_str())
            .expect("Failed to fetch DjmdRelatedTracks");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_related_tracks_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdRelatedTracks<'a>> {
        let model = self
            .db
            .get_related_tracks_by_id(id.as_str())
            .expect("Failed to fetch DjmdRelatedTracks");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_related_tracks_songs<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdSongRelatedTracks<'a>> {
        let models = self
            .db
            .get_related_tracks_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_related_tracks_contents<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdContent<'a>> {
        let models = self
            .db
            .get_related_tracks_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    // -- Sampler ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_sampler<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdSampler<'a>> {
        let models = self.db.get_sampler().expect("Could not fetch DjmdSampler");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_sampler_children<'a>(&mut self, env: &'a Env, id: String) -> Vec<JsDjmdSampler<'a>> {
        let models = self
            .db
            .get_sampler_children(id.as_str())
            .expect("Failed to fetch DjmdSampler");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_sampler_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdSampler<'a>> {
        let model = self
            .db
            .get_sampler_by_id(id.as_str())
            .expect("Failed to fetch DjmdSampler");
        model.map(|m| m.into_js(env))
    }

    #[napi]
    pub fn get_sampler_songs<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Vec<JsDjmdSongSampler<'a>> {
        let models = self
            .db
            .get_sampler_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_sampler_contents<'a>(&mut self, env: &'a Env, id: String) -> Vec<JsDjmdContent<'a>> {
        let models = self
            .db
            .get_sampler_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    // -- SongTagList ------------------------------------------------------------------------------

    #[napi]
    pub fn get_song_tag_list<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdSongTagList<'a>> {
        let models = self
            .db
            .get_song_tag_list()
            .expect("Could not fetch DjmdSongTagList");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_song_tag_list_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsDjmdSongTagList<'a>> {
        let model = self
            .db
            .get_song_tag_list_by_id(id.as_str())
            .expect("Failed to fetch DjmdSongTagList");
        model.map(|m| m.into_js(env))
    }

    // -- Sort -------------------------------------------------------------------------------------

    #[napi]
    pub fn get_sort<'a>(&mut self, env: &'a Env) -> Vec<JsDjmdSort<'a>> {
        let models = self.db.get_sort().expect("Could not fetch DjmdSort");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_sort_by_id<'a>(&mut self, env: &'a Env, id: String) -> Option<JsDjmdSort<'a>> {
        let model = self
            .db
            .get_sort_by_id(id.as_str())
            .expect("Failed to fetch DjmdSort");
        model.map(|m| m.into_js(env))
    }

    // -- ImageFile --------------------------------------------------------------------------------

    #[napi]
    pub fn get_image_file<'a>(&mut self, env: &'a Env) -> Vec<JsImageFile<'a>> {
        let models = self.db.get_image_file().expect("Could not fetch ImageFile");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_image_file_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsImageFile<'a>> {
        let model = self
            .db
            .get_image_file_by_id(id.as_str())
            .expect("Failed to fetch ImageFile");
        model.map(|m| m.into_js(env))
    }

    // -- SettingFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_setting_file<'a>(&mut self, env: &'a Env) -> Vec<JsSettingFile<'a>> {
        let models = self
            .db
            .get_setting_file()
            .expect("Could not fetch SettingFile");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_setting_file_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsSettingFile<'a>> {
        let model = self
            .db
            .get_setting_file_by_id(id.as_str())
            .expect("Failed to fetch SettingFile");
        model.map(|m| m.into_js(env))
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    #[napi]
    pub fn get_uuid_id_map<'a>(&mut self, env: &'a Env) -> Vec<JsUuidIDMap<'a>> {
        let models = self
            .db
            .get_uuid_id_map()
            .expect("Could not fetch UuidIDMap");
        models.into_iter().map(|m| m.into_js(env)).collect()
    }

    #[napi]
    pub fn get_uuid_id_map_by_id<'a>(
        &mut self,
        env: &'a Env,
        id: String,
    ) -> Option<JsUuidIDMap<'a>> {
        let model = self
            .db
            .get_uuid_id_map_by_id(id.as_str())
            .expect("Failed to fetch UuidIDMap");
        model.map(|m| m.into_js(env))
    }
}
