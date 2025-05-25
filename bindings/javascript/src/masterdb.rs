// Author: Dylan Jones
// Date:   2025-05-04

use crate::anlz::JsAnlzPaths;
use crate::js_models::*;
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

    // -- AgentRegistry ----------------------------------------------------------------------------

    #[napi]
    pub fn get_agent_registry(&mut self) -> Vec<JsAgentRegistry> {
        let models = self
            .db
            .get_agent_registry()
            .expect("Could not fetch AgentRegistry");
        let objects: Vec<JsAgentRegistry> = models.into_iter().map(JsAgentRegistry::from).collect();
        objects
    }

    #[napi]
    pub fn get_agent_registry_by_id(&mut self, id: String) -> Option<JsAgentRegistry> {
        let model = self
            .db
            .get_agent_registry_by_id(id.as_str())
            .expect("Failed to fetch AgentRegistry");
        model.map(JsAgentRegistry::from)
    }

    #[napi]
    pub fn get_local_usn(&mut self) -> i32 {
        self.db.get_local_usn().expect("Failed to fetch Local USN")
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_agent_registry(&mut self) -> Vec<JsCloudAgentRegistry> {
        let models = self
            .db
            .get_cloud_agent_registry()
            .expect("Could not fetch CloudAgentRegistry");
        let objects: Vec<JsCloudAgentRegistry> =
            models.into_iter().map(JsCloudAgentRegistry::from).collect();
        objects
    }

    #[napi]
    pub fn get_cloud_agent_registry_by_id(&mut self, id: String) -> Option<JsCloudAgentRegistry> {
        let model = self
            .db
            .get_cloud_agent_registry_by_id(id.as_str())
            .expect("Failed to fetch CloudAgentRegistry");
        model.map(JsCloudAgentRegistry::from)
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    #[napi]
    pub fn get_content_active_censor(&mut self) -> Vec<JsContentActiveCensor> {
        let models = self
            .db
            .get_content_active_censor()
            .expect("Could not fetch ContenActiveCensor");
        let objects: Vec<JsContentActiveCensor> = models
            .into_iter()
            .map(JsContentActiveCensor::from)
            .collect();
        objects
    }

    #[napi]
    pub fn get_content_active_censor_by_id(&mut self, id: String) -> Option<JsContentActiveCensor> {
        let model = self
            .db
            .get_content_active_censor_by_id(id.as_str())
            .expect("Failed to fetch ContenActiveCensor");
        model.map(JsContentActiveCensor::from)
    }

    // -- ContentCue -------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_cue(&mut self) -> Vec<JsContentCue> {
        let models = self
            .db
            .get_content_cue()
            .expect("Could not fetch ContentCue");
        let objects: Vec<JsContentCue> = models.into_iter().map(JsContentCue::from).collect();
        objects
    }

    #[napi]
    pub fn get_content_cue_by_id(&mut self, id: String) -> Option<JsContentCue> {
        let model = self
            .db
            .get_content_cue_by_id(id.as_str())
            .expect("Failed to fetch ContentCue");
        model.map(JsContentCue::from)
    }

    // -- ContentFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_content_file(&mut self) -> Vec<JsContentFile> {
        let models = self
            .db
            .get_content_file()
            .expect("Could not fetch ContentFile");
        let objects: Vec<JsContentFile> = models.into_iter().map(JsContentFile::from).collect();
        objects
    }

    #[napi]
    pub fn get_content_file_by_id(&mut self, id: String) -> Option<JsContentFile> {
        let model = self
            .db
            .get_content_file_by_id(id.as_str())
            .expect("Failed to fetch ContentFile");
        model.map(JsContentFile::from)
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    #[napi]
    pub fn get_active_censor(&mut self) -> Vec<JsDjmdActiveCensor> {
        let models = self
            .db
            .get_active_censor()
            .expect("Could not fetch DjmdActiveCensor");
        let objects: Vec<JsDjmdActiveCensor> =
            models.into_iter().map(JsDjmdActiveCensor::from).collect();
        objects
    }

    #[napi]
    pub fn get_active_censor_by_id(&mut self, id: String) -> Option<JsDjmdActiveCensor> {
        let model = self
            .db
            .get_active_censor_by_id(id.as_str())
            .expect("Failed to fetch DjmdActiveCensor");
        model.map(JsDjmdActiveCensor::from)
    }

    // -- Album ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_album(&mut self) -> Vec<JsDjmdAlbum> {
        let models = self.db.get_album().expect("Could not fetch DjmdAlbum");
        let objects: Vec<JsDjmdAlbum> = models.into_iter().map(JsDjmdAlbum::from).collect();
        objects
    }

    #[napi]
    pub fn get_album_by_id(&mut self, id: String) -> Option<JsDjmdAlbum> {
        let model = self
            .db
            .get_album_by_id(id.as_str())
            .expect("Failed to fetch DjmdAlbum");
        model.map(JsDjmdAlbum::from)
    }

    #[napi]
    pub fn get_album_by_name(&mut self, name: String) -> Option<JsDjmdAlbum> {
        let model = self
            .db
            .get_album_by_name(name.as_str())
            .expect("Failed to fetch DjmdAlbum");
        model.map(JsDjmdAlbum::from)
    }

    #[napi]
    pub fn insert_album(
        &mut self,
        name: String,
        artist_id: Option<String>,
        image_path: Option<String>,
        compilation: Option<i32>,
    ) -> JsDjmdAlbum {
        let model = self
            .db
            .insert_album(name, artist_id, image_path, compilation)
            .expect("Failed to fetch DjmdAlbum");
        JsDjmdAlbum::from(model)
    }

    #[napi]
    pub fn update_album(&mut self, object: JsDjmdAlbum) -> JsDjmdAlbum {
        let mut model: DjmdAlbum = DjmdAlbum::from(object);
        let updated_model = self
            .db
            .update_album(&mut model)
            .expect("Failed to update DjmdAlbum");
        JsDjmdAlbum::from(updated_model)
    }

    #[napi]
    pub fn delete_album(&mut self, id: String) {
        self.db
            .delete_album(id.as_str())
            .expect("Failed to delete DjmdAlbum");
    }

    // -- Artist -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_artist(&mut self) -> Vec<JsDjmdArtist> {
        let models = self.db.get_artist().expect("Could not fetch DjmdArtist");
        let objects: Vec<JsDjmdArtist> = models.into_iter().map(JsDjmdArtist::from).collect();
        objects
    }

    #[napi]
    pub fn get_artist_by_id(&mut self, id: String) -> Option<JsDjmdArtist> {
        let model = self
            .db
            .get_artist_by_id(id.as_str())
            .expect("Failed to fetch DjmdArtist");
        model.map(JsDjmdArtist::from)
    }

    #[napi]
    pub fn get_artist_by_name(&mut self, name: String) -> Option<JsDjmdArtist> {
        let model = self
            .db
            .get_artist_by_name(name.as_str())
            .expect("Failed to fetch DjmdArtist");
        model.map(JsDjmdArtist::from)
    }

    #[napi]
    pub fn insert_artist(&mut self, name: String) -> JsDjmdArtist {
        let model = self
            .db
            .insert_artist(name)
            .expect("Failed to fetch DjmdArtist");
        JsDjmdArtist::from(model)
    }

    #[napi]
    pub fn update_artist(&mut self, object: JsDjmdArtist) -> JsDjmdArtist {
        let mut model: DjmdArtist = DjmdArtist::from(object);
        let updated_model = self
            .db
            .update_artist(&mut model)
            .expect("Failed to update DjmdArtist");
        JsDjmdArtist::from(updated_model)
    }

    #[napi]
    pub fn delete_artist(&mut self, id: String) {
        self.db
            .delete_artist(id.as_str())
            .expect("Failed to delete DjmdArtist");
    }

    // -- Category ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_category(&mut self) -> Vec<JsDjmdCategory> {
        let models = self
            .db
            .get_category()
            .expect("Could not fetch DjmdCategory");
        let objects: Vec<JsDjmdCategory> = models.into_iter().map(JsDjmdCategory::from).collect();
        objects
    }

    #[napi]
    pub fn get_category_by_id(&mut self, id: String) -> Option<JsDjmdCategory> {
        let model = self
            .db
            .get_category_by_id(id.as_str())
            .expect("Failed to fetch DjmdCategory");
        model.map(JsDjmdCategory::from)
    }

    // -- Color ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_color(&mut self) -> Vec<JsDjmdColor> {
        let models = self.db.get_color().expect("Could not fetch DjmdColor");
        let objects: Vec<JsDjmdColor> = models.into_iter().map(JsDjmdColor::from).collect();
        objects
    }

    #[napi]
    pub fn get_color_by_id(&mut self, id: String) -> Option<JsDjmdColor> {
        let model = self
            .db
            .get_color_by_id(id.as_str())
            .expect("Failed to fetch DjmdColor");
        model.map(JsDjmdColor::from)
    }

    // -- Content ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_content(&mut self) -> Vec<JsDjmdContent> {
        let models = self.db.get_content().expect("Could not fetch DjmdContent");
        let objects: Vec<JsDjmdContent> = models.into_iter().map(JsDjmdContent::from).collect();
        objects
    }

    #[napi]
    pub fn get_content_by_id(&mut self, id: String) -> Option<JsDjmdContent> {
        let model = self
            .db
            .get_content_by_id(id.as_str())
            .expect("Failed to fetch DjmdContent");
        model.map(JsDjmdContent::from)
    }

    #[napi]
    pub fn get_content_by_path(&mut self, id: String) -> Option<JsDjmdContent> {
        let model = self
            .db
            .get_content_by_path(id.as_str())
            .expect("Failed to fetch DjmdContent");
        model.map(JsDjmdContent::from)
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
    pub fn insert_content(&mut self, path: String) -> JsDjmdContent {
        let model = self
            .db
            .insert_content(path)
            .expect("Failed to insert DjmdContent");
        JsDjmdContent::from(model)
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

    // #[napi]
    // pub fn delete_content(&mut self, id: String) {
    //     self.db
    //         .delete_content(id.as_str())
    //         .expect("Failed to delete DjmdContent");
    // }

    // -- Cue --------------------------------------------------------------------------------------

    #[napi]
    pub fn get_cue(&mut self) -> Vec<JsDjmdCue> {
        let models = self.db.get_cue().expect("Could not fetch DjmdCue");
        let objects: Vec<JsDjmdCue> = models.into_iter().map(JsDjmdCue::from).collect();
        objects
    }

    #[napi]
    pub fn get_cue_by_id(&mut self, id: String) -> Option<JsDjmdCue> {
        let model = self
            .db
            .get_cue_by_id(id.as_str())
            .expect("Failed to fetch DjmdCue");
        model.map(JsDjmdCue::from)
    }

    // -- Device -----------------------------------------------------------------------------------

    #[napi]
    pub fn get_device(&mut self) -> Vec<JsDjmdDevice> {
        let models = self.db.get_device().expect("Could not fetch DjmdDevice");
        let objects: Vec<JsDjmdDevice> = models.into_iter().map(JsDjmdDevice::from).collect();
        objects
    }

    #[napi]
    pub fn get_device_by_id(&mut self, id: String) -> Option<JsDjmdDevice> {
        let model = self
            .db
            .get_device_by_id(id.as_str())
            .expect("Failed to fetch DjmdDevice");
        model.map(JsDjmdDevice::from)
    }

    // -- Genre ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_genre(&mut self) -> Vec<JsDjmdGenre> {
        let models = self.db.get_genre().expect("Could not fetch DjmdGenre");
        let objects: Vec<JsDjmdGenre> = models.into_iter().map(JsDjmdGenre::from).collect();
        objects
    }

    #[napi]
    pub fn get_genre_by_id(&mut self, id: String) -> Option<JsDjmdGenre> {
        let model = self
            .db
            .get_genre_by_id(id.as_str())
            .expect("Failed to fetch DjmdGenre");
        model.map(JsDjmdGenre::from)
    }

    #[napi]
    pub fn get_genre_by_name(&mut self, name: String) -> Option<JsDjmdGenre> {
        let model = self
            .db
            .get_genre_by_name(name.as_str())
            .expect("Failed to fetch DjmdGenre");
        model.map(JsDjmdGenre::from)
    }

    #[napi]
    pub fn insert_genre(&mut self, name: String) -> JsDjmdGenre {
        let model = self
            .db
            .insert_genre(name)
            .expect("Failed to fetch DjmdGenre");
        JsDjmdGenre::from(model)
    }

    #[napi]
    pub fn update_genre(&mut self, object: JsDjmdGenre) -> JsDjmdGenre {
        let mut model: DjmdGenre = DjmdGenre::from(object);
        let updated_model = self
            .db
            .update_genre(&mut model)
            .expect("Failed to update DjmdGenre");
        JsDjmdGenre::from(updated_model)
    }

    #[napi]
    pub fn delete_genre(&mut self, id: String) {
        self.db
            .delete_genre(id.as_str())
            .expect("Failed to delete DjmdGenre");
    }

    // -- History ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_history(&mut self) -> Vec<JsDjmdHistory> {
        let models = self.db.get_history().expect("Could not fetch DjmdGenre");
        let objects: Vec<JsDjmdHistory> = models.into_iter().map(JsDjmdHistory::from).collect();
        objects
    }

    #[napi]
    pub fn get_history_by_id(&mut self, id: String) -> Option<JsDjmdHistory> {
        let model = self
            .db
            .get_history_by_id(id.as_str())
            .expect("Failed to fetch DjmdGenre");
        model.map(JsDjmdHistory::from)
    }

    #[napi]
    pub fn get_history_songs(&mut self, id: String) -> Vec<JsDjmdSongHistory> {
        let models = self
            .db
            .get_history_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        let objects: Vec<JsDjmdSongHistory> =
            models.into_iter().map(JsDjmdSongHistory::from).collect();
        objects
    }

    #[napi]
    pub fn get_history_contents(&mut self, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_history_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        let objects: Vec<JsDjmdContent> = models.into_iter().map(JsDjmdContent::from).collect();
        objects
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    #[napi]
    pub fn get_hot_cue_banklist(&mut self) -> Vec<JsDjmdHotCueBanklist> {
        let models = self
            .db
            .get_hot_cue_banklist()
            .expect("Could not fetch DjmdHotCueBanklist");
        let objects: Vec<JsDjmdHotCueBanklist> =
            models.into_iter().map(JsDjmdHotCueBanklist::from).collect();
        objects
    }

    #[napi]
    pub fn get_hot_cue_banklist_children(&mut self, id: String) -> Vec<JsDjmdHotCueBanklist> {
        let models = self
            .db
            .get_hot_cue_banklist_children(id.as_str())
            .expect("Failed to fetch DjmdHotCueBanklist");
        let objects: Vec<JsDjmdHotCueBanklist> =
            models.into_iter().map(JsDjmdHotCueBanklist::from).collect();
        objects
    }

    #[napi]
    pub fn get_hot_cue_banklist_by_id(&mut self, id: String) -> Option<JsDjmdHotCueBanklist> {
        let model = self
            .db
            .get_hot_cue_banklist_by_id(id.as_str())
            .expect("Failed to fetch DjmdHotCueBanklist");
        model.map(JsDjmdHotCueBanklist::from)
    }

    #[napi]
    pub fn get_hot_cue_banklist_songs(&mut self, id: String) -> Vec<JsDjmdSongHotCueBanklist> {
        let models = self
            .db
            .get_hot_cue_banklist_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        let objects: Vec<JsDjmdSongHotCueBanklist> = models
            .into_iter()
            .map(JsDjmdSongHotCueBanklist::from)
            .collect();
        objects
    }

    #[napi]
    pub fn get_hot_cue_banklist_contents(&mut self, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_hot_cue_banklist_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        let objects: Vec<JsDjmdContent> = models.into_iter().map(JsDjmdContent::from).collect();
        objects
    }

    #[napi]
    pub fn get_hot_cue_banklist_cues(&mut self, id: String) -> Vec<JsHotCueBanklistCue> {
        let models = self
            .db
            .get_hot_cue_banklist_cues(id.as_str())
            .expect("Could not fetch HotCueBanklistCue");
        let objects: Vec<JsHotCueBanklistCue> =
            models.into_iter().map(JsHotCueBanklistCue::from).collect();
        objects
    }

    // -- Key --------------------------------------------------------------------------------------

    #[napi]
    pub fn get_key(&mut self) -> Vec<JsDjmdKey> {
        let models = self.db.get_key().expect("Could not fetch DjmdKey");
        let objects: Vec<JsDjmdKey> = models.into_iter().map(JsDjmdKey::from).collect();
        objects
    }

    #[napi]
    pub fn get_key_by_id(&mut self, id: String) -> Option<JsDjmdKey> {
        let model = self
            .db
            .get_key_by_id(id.as_str())
            .expect("Failed to fetch DjmdKey");
        model.map(JsDjmdKey::from)
    }

    #[napi]
    pub fn get_key_by_name(&mut self, name: String) -> Option<JsDjmdKey> {
        let model = self
            .db
            .get_key_by_name(name.as_str())
            .expect("Failed to fetch DjmdKey");
        model.map(JsDjmdKey::from)
    }

    #[napi]
    pub fn insert_key(&mut self, name: String) -> JsDjmdKey {
        let model = self.db.insert_key(name).expect("Failed to fetch DjmdKey");
        JsDjmdKey::from(model)
    }

    #[napi]
    pub fn update_key(&mut self, object: JsDjmdKey) -> JsDjmdKey {
        let mut model: DjmdKey = DjmdKey::from(object);
        let updated_model = self
            .db
            .update_key(&mut model)
            .expect("Failed to update DjmdKey");
        JsDjmdKey::from(updated_model)
    }

    #[napi]
    pub fn delete_key(&mut self, id: String) {
        self.db
            .delete_key(id.as_str())
            .expect("Failed to delete DjmdKey");
    }

    // -- Label ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_label(&mut self) -> Vec<JsDjmdLabel> {
        let models = self.db.get_label().expect("Could not fetch DjmdLabel");
        let objects: Vec<JsDjmdLabel> = models.into_iter().map(JsDjmdLabel::from).collect();
        objects
    }

    #[napi]
    pub fn get_label_by_id(&mut self, id: String) -> Option<JsDjmdLabel> {
        let model = self
            .db
            .get_label_by_id(id.as_str())
            .expect("Failed to fetch DjmdLabel");
        model.map(JsDjmdLabel::from)
    }

    #[napi]
    pub fn get_label_by_name(&mut self, name: String) -> Option<JsDjmdLabel> {
        let model = self
            .db
            .get_label_by_name(name.as_str())
            .expect("Failed to fetch DjmdLabel");
        model.map(JsDjmdLabel::from)
    }

    #[napi]
    pub fn insert_label(&mut self, name: String) -> JsDjmdLabel {
        let model = self
            .db
            .insert_label(name)
            .expect("Failed to fetch DjmdLabel");
        JsDjmdLabel::from(model)
    }

    #[napi]
    pub fn update_label(&mut self, object: JsDjmdLabel) -> JsDjmdLabel {
        let mut model: DjmdLabel = DjmdLabel::from(object);
        let updated_model = self
            .db
            .update_label(&mut model)
            .expect("Failed to update DjmdLabel");
        JsDjmdLabel::from(updated_model)
    }

    #[napi]
    pub fn delete_label(&mut self, id: String) {
        self.db
            .delete_label(id.as_str())
            .expect("Failed to delete DjmdLabel");
    }

    // -- MenuItems --------------------------------------------------------------------------------

    #[napi]
    pub fn get_menu_item(&mut self) -> Vec<JsDjmdMenuItems> {
        let models = self
            .db
            .get_menu_item()
            .expect("Could not fetch DjmdMenuItems");
        let objects: Vec<JsDjmdMenuItems> = models.into_iter().map(JsDjmdMenuItems::from).collect();
        objects
    }

    #[napi]
    pub fn get_menu_item_by_id(&mut self, id: String) -> Option<JsDjmdMenuItems> {
        let model = self
            .db
            .get_menu_item_by_id(id.as_str())
            .expect("Failed to fetch DjmdMenuItems");
        model.map(JsDjmdMenuItems::from)
    }

    // -- MixerParam -------------------------------------------------------------------------------

    #[napi]
    pub fn get_mixer_param(&mut self) -> Vec<JsDjmdMixerParam> {
        let models = self
            .db
            .get_mixer_param()
            .expect("Could not fetch DjmdMixerParam");
        let objects: Vec<JsDjmdMixerParam> =
            models.into_iter().map(JsDjmdMixerParam::from).collect();
        objects
    }

    #[napi]
    pub fn get_mixer_param_by_id(&mut self, id: String) -> Option<JsDjmdMixerParam> {
        let model = self
            .db
            .get_mixer_param_by_id(id.as_str())
            .expect("Failed to fetch DjmdMixerParam");
        model.map(JsDjmdMixerParam::from)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    #[napi]
    pub fn get_my_tag(&mut self) -> Vec<JsDjmdMyTag> {
        let models = self.db.get_my_tag().expect("Could not fetch DjmdMyTag");
        let objects: Vec<JsDjmdMyTag> = models.into_iter().map(JsDjmdMyTag::from).collect();
        objects
    }

    #[napi]
    pub fn get_my_tag_children(&mut self, id: String) -> Vec<JsDjmdMyTag> {
        let models = self
            .db
            .get_my_tag_children(id.as_str())
            .expect("Failed to fetch DjmdMyTag");
        let objects: Vec<JsDjmdMyTag> = models.into_iter().map(JsDjmdMyTag::from).collect();
        objects
    }

    #[napi]
    pub fn get_my_tag_by_id(&mut self, id: String) -> Option<JsDjmdMyTag> {
        let model = self
            .db
            .get_my_tag_by_id(id.as_str())
            .expect("Failed to fetch DjmdMyTag");
        model.map(JsDjmdMyTag::from)
    }

    #[napi]
    pub fn get_my_tag_songs(&mut self, id: String) -> Vec<JsDjmdSongMyTag> {
        let models = self
            .db
            .get_my_tag_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        let objects: Vec<JsDjmdSongMyTag> = models.into_iter().map(JsDjmdSongMyTag::from).collect();
        objects
    }

    #[napi]
    pub fn get_my_tag_contents(&mut self, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_my_tag_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        let objects: Vec<JsDjmdContent> = models.into_iter().map(JsDjmdContent::from).collect();
        objects
    }

    // -- Playlist ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_playlist(&mut self) -> Vec<JsDjmdPlaylist> {
        let models = self
            .db
            .get_playlist()
            .expect("Could not fetch DjmdPlaylist");
        let objects: Vec<JsDjmdPlaylist> = models.into_iter().map(JsDjmdPlaylist::from).collect();
        objects
    }

    #[napi]
    pub fn get_playlist_tree(&mut self) -> Vec<JsPlaylistTreeItem> {
        let models = self
            .db
            .get_playlist_tree()
            .expect("Could not fetch DjmdPlaylist");
        let objects: Vec<JsPlaylistTreeItem> = models
            .iter()
            .map(|item| JsPlaylistTreeItem::from(item.borrow().clone()))
            .collect();
        objects
    }

    #[napi]
    pub fn get_playlist_children(&mut self, id: String) -> Vec<JsDjmdPlaylist> {
        let models = self
            .db
            .get_playlist_children(id.as_str())
            .expect("Failed to fetch DjmdPlaylist");
        let objects: Vec<JsDjmdPlaylist> = models.into_iter().map(JsDjmdPlaylist::from).collect();
        objects
    }

    #[napi]
    pub fn get_playlist_by_id(&mut self, id: String) -> Option<JsDjmdPlaylist> {
        let model = self
            .db
            .get_playlist_by_id(id.as_str())
            .expect("Failed to fetch DjmdPlaylist");
        model.map(JsDjmdPlaylist::from)
    }

    #[napi]
    pub fn get_playlist_by_path(&mut self, path: Vec<String>) -> Option<JsDjmdPlaylist> {
        let path_str: Vec<&str> = path.iter().map(|item| item.as_str()).collect();
        let model = self
            .db
            .get_playlist_by_path(path_str)
            .expect("Failed to fetch DjmdPlaylist");
        model.map(JsDjmdPlaylist::from)
    }

    #[napi]
    pub fn get_playlist_songs(&mut self, id: String) -> Vec<JsDjmdSongPlaylist> {
        let models = self
            .db
            .get_playlist_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        let objects: Vec<JsDjmdSongPlaylist> =
            models.into_iter().map(JsDjmdSongPlaylist::from).collect();
        objects
    }

    #[napi]
    pub fn get_playlist_contents(&mut self, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_playlist_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        let objects: Vec<JsDjmdContent> = models.into_iter().map(JsDjmdContent::from).collect();
        objects
    }

    #[napi]
    pub fn get_playlist_song_by_id(&mut self, id: String) -> Vec<JsDjmdSongPlaylist> {
        let models = self
            .db
            .get_playlist_song_by_id(id.as_str())
            .expect("Could not fetch DjmdSongPlaylist");
        let objects: Vec<JsDjmdSongPlaylist> =
            models.into_iter().map(JsDjmdSongPlaylist::from).collect();
        objects
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
    ) -> JsDjmdPlaylist {
        let attr = PlaylistType::try_from(attribute).expect("Wrong Playlist attribute");
        let model = self
            .db
            .insert_playlist(name, attr, parent_id, seq, image_path, smart_list)
            .expect("Failed to insert DjmdPlaylist");
        JsDjmdPlaylist::from(model)
    }

    #[napi]
    pub fn rename_playlist(&mut self, id: String, name: String) -> JsDjmdPlaylist {
        let model = self
            .db
            .rename_playlist(&id, name)
            .expect("Failed to rename DjmdPlaylist");
        JsDjmdPlaylist::from(model)
    }

    #[napi]
    pub fn move_playlist(
        &mut self,
        id: String,
        seq: Option<i32>,
        parent_id: Option<String>,
    ) -> JsDjmdPlaylist {
        let model = self
            .db
            .move_playlist(&id, seq, parent_id)
            .expect("Failed to move DjmdPlaylist");
        JsDjmdPlaylist::from(model)
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
        playlist_id: String,
        content_id: String,
        seq: Option<i32>,
    ) -> JsDjmdSongPlaylist {
        let model = self
            .db
            .insert_playlist_song(&playlist_id, &content_id, seq)
            .expect("Failed to insert DjmdSongPlaylist");
        JsDjmdSongPlaylist::from(model)
    }

    #[napi]
    pub fn move_playlist_song(&mut self, id: String, seq: i32) -> JsDjmdSongPlaylist {
        let model = self
            .db
            .move_playlist_song(&id, seq)
            .expect("Failed to move DjmdSongPlaylist");
        JsDjmdSongPlaylist::from(model)
    }

    #[napi]
    pub fn delete_playlist_song(&mut self, id: String) {
        self.db
            .delete_playlist_song(id.as_str())
            .expect("Failed to delete DjmdSongPlaylist");
    }

    // -- Property ---------------------------------------------------------------------------------

    #[napi]
    pub fn get_property(&mut self) -> Vec<JsDjmdProperty> {
        let models = self
            .db
            .get_property()
            .expect("Could not fetch DjmdProperty");
        let objects: Vec<JsDjmdProperty> = models.into_iter().map(JsDjmdProperty::from).collect();
        objects
    }

    #[napi]
    pub fn get_property_by_id(&mut self, id: String) -> Option<JsDjmdProperty> {
        let model = self
            .db
            .get_property_by_id(id.as_str())
            .expect("Failed to fetch DjmdProperty");
        model.map(JsDjmdProperty::from)
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    #[napi]
    pub fn get_cloud_property(&mut self) -> Vec<JsDjmdCloudProperty> {
        let models = self
            .db
            .get_cloud_property()
            .expect("Could not fetch DjmdCloudProperty");
        let objects: Vec<JsDjmdCloudProperty> =
            models.into_iter().map(JsDjmdCloudProperty::from).collect();
        objects
    }

    #[napi]
    pub fn get_cloud_property_by_id(&mut self, id: String) -> Option<JsDjmdCloudProperty> {
        let model = self
            .db
            .get_cloud_property_by_id(id.as_str())
            .expect("Failed to fetch DjmdCloudProperty");
        model.map(JsDjmdCloudProperty::from)
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    #[napi]
    pub fn get_recommend_like(&mut self) -> Vec<JsDjmdRecommendLike> {
        let models = self
            .db
            .get_recommend_like()
            .expect("Could not fetch DjmdRecommendLike");
        let objects: Vec<JsDjmdRecommendLike> =
            models.into_iter().map(JsDjmdRecommendLike::from).collect();
        objects
    }

    #[napi]
    pub fn get_recommend_like_by_id(&mut self, id: String) -> Option<JsDjmdRecommendLike> {
        let model = self
            .db
            .get_recommend_like_by_id(id.as_str())
            .expect("Failed to fetch DjmdRecommendLike");
        model.map(JsDjmdRecommendLike::from)
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    #[napi]
    pub fn get_related_tracks(&mut self) -> Vec<JsDjmdRelatedTracks> {
        let models = self
            .db
            .get_related_tracks()
            .expect("Could not fetch DjmdRelatedTracks");
        let objects: Vec<JsDjmdRelatedTracks> =
            models.into_iter().map(JsDjmdRelatedTracks::from).collect();
        objects
    }

    #[napi]
    pub fn get_related_tracks_children(&mut self, id: String) -> Vec<JsDjmdRelatedTracks> {
        let models = self
            .db
            .get_related_tracks_children(id.as_str())
            .expect("Failed to fetch DjmdRelatedTracks");
        let objects: Vec<JsDjmdRelatedTracks> =
            models.into_iter().map(JsDjmdRelatedTracks::from).collect();
        objects
    }

    #[napi]
    pub fn get_related_tracks_by_id(&mut self, id: String) -> Option<JsDjmdRelatedTracks> {
        let model = self
            .db
            .get_related_tracks_by_id(id.as_str())
            .expect("Failed to fetch DjmdRelatedTracks");
        model.map(JsDjmdRelatedTracks::from)
    }

    #[napi]
    pub fn get_related_tracks_songs(&mut self, id: String) -> Vec<JsDjmdSongRelatedTracks> {
        let models = self
            .db
            .get_related_tracks_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        let objects: Vec<JsDjmdSongRelatedTracks> = models
            .into_iter()
            .map(JsDjmdSongRelatedTracks::from)
            .collect();
        objects
    }

    #[napi]
    pub fn get_related_tracks_contents(&mut self, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_related_tracks_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        let objects: Vec<JsDjmdContent> = models.into_iter().map(JsDjmdContent::from).collect();
        objects
    }

    // -- Sampler ----------------------------------------------------------------------------------

    #[napi]
    pub fn get_sampler(&mut self) -> Vec<JsDjmdSampler> {
        let models = self.db.get_sampler().expect("Could not fetch DjmdSampler");
        let objects: Vec<JsDjmdSampler> = models.into_iter().map(JsDjmdSampler::from).collect();
        objects
    }

    #[napi]
    pub fn get_sampler_children(&mut self, id: String) -> Vec<JsDjmdSampler> {
        let models = self
            .db
            .get_sampler_children(id.as_str())
            .expect("Failed to fetch DjmdSampler");
        let objects: Vec<JsDjmdSampler> = models.into_iter().map(JsDjmdSampler::from).collect();
        objects
    }

    #[napi]
    pub fn get_sampler_by_id(&mut self, id: String) -> Option<JsDjmdSampler> {
        let model = self
            .db
            .get_sampler_by_id(id.as_str())
            .expect("Failed to fetch DjmdSampler");
        model.map(JsDjmdSampler::from)
    }

    #[napi]
    pub fn get_sampler_songs(&mut self, id: String) -> Vec<JsDjmdSongSampler> {
        let models = self
            .db
            .get_sampler_songs(id.as_str())
            .expect("Could not fetch DjmdSongHistory");
        let objects: Vec<JsDjmdSongSampler> =
            models.into_iter().map(JsDjmdSongSampler::from).collect();
        objects
    }

    #[napi]
    pub fn get_sampler_contents(&mut self, id: String) -> Vec<JsDjmdContent> {
        let models = self
            .db
            .get_sampler_contents(id.as_str())
            .expect("Could not fetch DjmdContent");
        let objects: Vec<JsDjmdContent> = models.into_iter().map(JsDjmdContent::from).collect();
        objects
    }

    // -- SongTagList ------------------------------------------------------------------------------

    #[napi]
    pub fn get_song_tag_list(&mut self) -> Vec<JsDjmdSongTagList> {
        let models = self
            .db
            .get_song_tag_list()
            .expect("Could not fetch DjmdSongTagList");
        let objects: Vec<JsDjmdSongTagList> =
            models.into_iter().map(JsDjmdSongTagList::from).collect();
        objects
    }

    #[napi]
    pub fn get_song_tag_list_by_id(&mut self, id: String) -> Option<JsDjmdSongTagList> {
        let model = self
            .db
            .get_song_tag_list_by_id(id.as_str())
            .expect("Failed to fetch DjmdSongTagList");
        model.map(JsDjmdSongTagList::from)
    }

    // -- Sort -------------------------------------------------------------------------------------

    #[napi]
    pub fn get_sort(&mut self) -> Vec<JsDjmdSort> {
        let models = self.db.get_sort().expect("Could not fetch DjmdSort");
        let objects: Vec<JsDjmdSort> = models.into_iter().map(JsDjmdSort::from).collect();
        objects
    }

    #[napi]
    pub fn get_sort_by_id(&mut self, id: String) -> Option<JsDjmdSort> {
        let model = self
            .db
            .get_sort_by_id(id.as_str())
            .expect("Failed to fetch DjmdSort");
        model.map(JsDjmdSort::from)
    }

    // -- ImageFile --------------------------------------------------------------------------------

    #[napi]
    pub fn get_image_file(&mut self) -> Vec<JsImageFile> {
        let models = self.db.get_image_file().expect("Could not fetch ImageFile");
        let objects: Vec<JsImageFile> = models.into_iter().map(JsImageFile::from).collect();
        objects
    }

    #[napi]
    pub fn get_image_file_by_id(&mut self, id: String) -> Option<JsImageFile> {
        let model = self
            .db
            .get_image_file_by_id(id.as_str())
            .expect("Failed to fetch ImageFile");
        model.map(JsImageFile::from)
    }

    // -- SettingFile ------------------------------------------------------------------------------

    #[napi]
    pub fn get_setting_file(&mut self) -> Vec<JsSettingFile> {
        let models = self
            .db
            .get_setting_file()
            .expect("Could not fetch SettingFile");
        let objects: Vec<JsSettingFile> = models.into_iter().map(JsSettingFile::from).collect();
        objects
    }

    #[napi]
    pub fn get_setting_file_by_id(&mut self, id: String) -> Option<JsSettingFile> {
        let model = self
            .db
            .get_setting_file_by_id(id.as_str())
            .expect("Failed to fetch SettingFile");
        model.map(JsSettingFile::from)
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    #[napi]
    pub fn get_uuid_id_map(&mut self) -> Vec<JsUuidIDMap> {
        let models = self
            .db
            .get_uuid_id_map()
            .expect("Could not fetch UuidIDMap");
        let objects: Vec<JsUuidIDMap> = models.into_iter().map(JsUuidIDMap::from).collect();
        objects
    }

    #[napi]
    pub fn get_uuid_id_map_by_id(&mut self, id: String) -> Option<JsUuidIDMap> {
        let model = self
            .db
            .get_uuid_id_map_by_id(id.as_str())
            .expect("Failed to fetch UuidIDMap");
        model.map(JsUuidIDMap::from)
    }
}
