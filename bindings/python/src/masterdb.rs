// Author: Dylan Jones
// Date:   2025-05-01

use chrono::{DateTime, Utc};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyType};
use rbox::masterdb::{enums::PlaylistType, models::*, MasterDb};

use super::errors::DatabaseError;
use super::traits::IntoPy;

#[inline]
fn db_error<E: ToString>(e: E) -> PyErr {
    PyErr::new::<DatabaseError, _>(e.to_string())
}

#[derive(Clone)]
#[pyclass(name = "DjmdPlaylistTreeNode", unsendable, get_all, set_all)]
pub struct PyDjmdPlaylistTreeNode {
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
    pub children: Vec<PyDjmdPlaylistTreeNode>,
}

impl IntoPy<PyDjmdPlaylistTreeNode> for DjmdPlaylistTreeNode {
    fn into_py(self, py: Python) -> PyResult<PyDjmdPlaylistTreeNode> {
        let children = self
            .children
            .into_iter()
            .map(|c| c.borrow().clone().into_py(py).unwrap())
            .collect();
        let model = PyDjmdPlaylistTreeNode {
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
        };
        Ok(model)
    }
}

#[pyclass(unsendable)]
pub struct PyMasterDb {
    db: MasterDb,
}

#[pymethods]
impl PyMasterDb {
    #[new]
    pub fn new(path: &str) -> PyResult<Self> {
        let db = MasterDb::new(path).map_err(|e| db_error(e))?;
        Ok(PyMasterDb { db })
    }

    #[classmethod]
    fn open(_cls: &Bound<'_, PyType>) -> PyResult<Self> {
        let db = MasterDb::open().map_err(|e| db_error(e))?;
        Ok(PyMasterDb { db })
    }

    pub fn set_unsafe_writes(&mut self, unsafe_writes: bool) -> PyResult<()> {
        self.db.set_unsafe_writes(unsafe_writes);
        Ok(())
    }

    // -- AgentRegistry ----------------------------------------------------------------------------

    pub fn get_agent_registry(&mut self) -> PyResult<Vec<AgentRegistry>> {
        let models = self.db.get_agent_registry().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_agent_registry_by_id(&mut self, id: &str) -> PyResult<Option<AgentRegistry>> {
        let model = self
            .db
            .get_agent_registry_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_local_usn(&mut self) -> PyResult<i32> {
        let result = self.db.get_local_usn().map_err(|e| db_error(e))?;
        Ok(result)
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    pub fn get_cloud_agent_registry(&mut self) -> PyResult<Vec<CloudAgentRegistry>> {
        let models = self
            .db
            .get_cloud_agent_registry()
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_cloud_agent_registry_by_id(
        &mut self,
        id: &str,
    ) -> PyResult<Option<CloudAgentRegistry>> {
        let model = self
            .db
            .get_cloud_agent_registry_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    pub fn get_content_active_censor(&mut self) -> PyResult<Vec<ContentActiveCensor>> {
        let models = self
            .db
            .get_content_active_censor()
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_content_active_censor_by_id(
        &mut self,
        id: &str,
    ) -> PyResult<Option<ContentActiveCensor>> {
        let model = self
            .db
            .get_content_active_censor_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- ContentCue -------------------------------------------------------------------------------

    pub fn get_content_cue(&mut self) -> PyResult<Vec<ContentCue>> {
        let models = self.db.get_content_cue().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_content_cue_by_id(&mut self, id: &str) -> PyResult<Option<ContentCue>> {
        let model = self.db.get_content_cue_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- ContentFile ------------------------------------------------------------------------------

    pub fn get_content_file(&mut self) -> PyResult<Vec<ContentFile>> {
        let models = self.db.get_content_file().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_content_file_by_id(&mut self, id: &str) -> PyResult<Option<ContentFile>> {
        let model = self
            .db
            .get_content_file_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    pub fn get_active_censor(&mut self) -> PyResult<Vec<DjmdActiveCensor>> {
        let models = self.db.get_active_censor().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_active_censor_by_id(&mut self, id: &str) -> PyResult<Option<DjmdActiveCensor>> {
        let model = self
            .db
            .get_active_censor_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Album ------------------------------------------------------------------------------------

    pub fn get_album(&mut self) -> PyResult<Vec<DjmdAlbum>> {
        let models = self.db.get_album().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_album_by_id(&mut self, id: &str) -> PyResult<Option<DjmdAlbum>> {
        let model = self.db.get_album_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_album_by_name(&mut self, name: &str) -> PyResult<Option<DjmdAlbum>> {
        let model = self.db.get_album_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn create_album(
        &mut self,
        name: &str,
        artist_id: Option<&str>,
        image_path: Option<&str>,
        compilation: Option<i32>,
    ) -> PyResult<DjmdAlbum> {
        // Convert to str if it's Some
        let artist_id = artist_id.map(|s| s.to_string());
        let image_path = image_path.map(|s| s.to_string());
        let model = self
            .db
            .create_album(name.to_string(), artist_id, image_path, compilation)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn update_album(&mut self, mut item: PyRefMut<'_, DjmdAlbum>) -> PyResult<DjmdAlbum> {
        let model = self.db.update_album(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn delete_album(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_album(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- Artist -----------------------------------------------------------------------------------

    pub fn get_artist(&mut self) -> PyResult<Vec<DjmdArtist>> {
        let models = self.db.get_artist().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_artist_by_id(&mut self, id: &str) -> PyResult<Option<DjmdArtist>> {
        let model = self.db.get_artist_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_artist_by_name(&mut self, name: &str) -> PyResult<Option<DjmdArtist>> {
        let model = self.db.get_artist_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn create_artist(&mut self, name: &str) -> PyResult<DjmdArtist> {
        let model = self
            .db
            .create_artist(name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn update_artist(&mut self, mut item: PyRefMut<'_, DjmdArtist>) -> PyResult<DjmdArtist> {
        let model = self.db.update_artist(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn delete_artist(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_artist(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- Category ---------------------------------------------------------------------------------

    pub fn get_category(&mut self) -> PyResult<Vec<DjmdCategory>> {
        let models = self.db.get_category().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_category_by_id(&mut self, id: &str) -> PyResult<Option<DjmdCategory>> {
        let model = self.db.get_category_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Color ------------------------------------------------------------------------------------

    pub fn get_color(&mut self) -> PyResult<Vec<DjmdColor>> {
        let models = self.db.get_color().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_color_by_id(&mut self, id: &str) -> PyResult<Option<DjmdColor>> {
        let model = self.db.get_color_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Content ----------------------------------------------------------------------------------

    pub fn get_content(&mut self) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_content().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_content_by_id(&mut self, id: &str) -> PyResult<Option<DjmdContent>> {
        // Get the content by ID from the database
        let model = self.db.get_content_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_content_by_path(&mut self, path: &str) -> PyResult<Option<DjmdContent>> {
        let model = self.db.get_content_by_path(path).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_content_anlz_dir(&mut self, id: &str) -> PyResult<Option<String>> {
        let path = self.db.get_content_anlz_dir(id).map_err(|e| db_error(e))?;
        if path.is_none() {
            return Ok(None);
        }
        Ok(Some(
            path.unwrap().as_os_str().to_str().unwrap().to_string(),
        ))
    }

    pub fn get_content_anlz_paths(&mut self, py: Python, id: &str) -> PyResult<Py<PyDict>> {
        let paths = self
            .db
            .get_content_anlz_paths(id)
            .map_err(|e| db_error(e))?;
        if let Some(paths) = paths {
            let result = PyDict::new(py);
            result.set_item("DAT", paths.dat)?;
            if let Some(ext) = paths.ext {
                result.set_item("EXT", ext)?;
            }
            if let Some(ex2) = paths.ex2 {
                result.set_item("EX2", ex2)?;
            }
            Ok(result.into())
        } else {
            Ok(PyDict::new(py).into())
        }
    }

    pub fn create_content(&mut self, path: &str) -> PyResult<DjmdContent> {
        let model = self.db.create_content(path).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn update_content(&mut self, item: PyRef<'_, DjmdContent>) -> PyResult<()> {
        self.db.update_content(&item).map_err(|e| db_error(e))?;
        Ok(())
    }

    pub fn update_content_album(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_album(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    pub fn update_content_artist(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_artist(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    pub fn update_content_remixer(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_remixer(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    pub fn update_content_original_artist(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_original_artist(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    pub fn update_content_composer(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_composer(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    pub fn update_content_genre(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_genre(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    pub fn update_content_label(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_label(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    pub fn update_content_key(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_key(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    // pub fn delete_content(&mut self, id: &str) -> PyResult<()> {
    //     self.db
    //         .delete_content(id)
    //         .map_err(|e| db_error(e))?;
    //     Ok(())
    // }

    // -- Cue --------------------------------------------------------------------------------------

    pub fn get_cue(&mut self) -> PyResult<Vec<DjmdCue>> {
        let models = self.db.get_cue().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_cue_by_id(&mut self, id: &str) -> PyResult<Option<DjmdCue>> {
        let model = self.db.get_cue_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Device -----------------------------------------------------------------------------------

    pub fn get_device(&mut self) -> PyResult<Vec<DjmdDevice>> {
        let models = self.db.get_device().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_device_by_id(&mut self, id: &str) -> PyResult<Option<DjmdDevice>> {
        let model = self.db.get_device_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Genre ------------------------------------------------------------------------------------

    pub fn get_genre(&mut self) -> PyResult<Vec<DjmdGenre>> {
        let models = self.db.get_genre().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_genre_by_id(&mut self, id: &str) -> PyResult<Option<DjmdGenre>> {
        let model = self.db.get_genre_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_genre_by_name(&mut self, name: &str) -> PyResult<Option<DjmdGenre>> {
        let model = self.db.get_genre_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn create_genre(&mut self, name: &str) -> PyResult<DjmdGenre> {
        let model = self
            .db
            .create_genre(name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn update_genre(&mut self, mut item: PyRefMut<'_, DjmdGenre>) -> PyResult<DjmdGenre> {
        let model = self.db.update_genre(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn delete_genre(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_genre(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- History ----------------------------------------------------------------------------------

    pub fn get_history(&mut self) -> PyResult<Vec<DjmdHistory>> {
        let models = self.db.get_history().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_history_by_id(&mut self, id: &str) -> PyResult<Option<DjmdHistory>> {
        let model = self.db.get_history_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_history_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongHistory>> {
        let models = self.db.get_history_songs(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_history_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_history_contents(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    pub fn get_hot_cue_banklist(&mut self) -> PyResult<Vec<DjmdHotCueBanklist>> {
        let models = self.db.get_hot_cue_banklist().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_hot_cue_banklist_by_id(&mut self, id: &str) -> PyResult<Option<DjmdHotCueBanklist>> {
        let model = self
            .db
            .get_hot_cue_banklist_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_hot_cue_banklist_children(
        &mut self,
        parent_id: &str,
    ) -> PyResult<Vec<DjmdHotCueBanklist>> {
        let models = self
            .db
            .get_hot_cue_banklist_children(parent_id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_hot_cue_banklist_songs(
        &mut self,
        id: &str,
    ) -> PyResult<Vec<DjmdSongHotCueBanklist>> {
        let models = self
            .db
            .get_hot_cue_banklist_songs(id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_hot_cue_banklist_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self
            .db
            .get_hot_cue_banklist_contents(id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_hot_cue_banklist_cues(&mut self, id: &str) -> PyResult<Vec<HotCueBanklistCue>> {
        let models = self
            .db
            .get_hot_cue_banklist_cues(id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- Key --------------------------------------------------------------------------------------

    pub fn get_key(&mut self) -> PyResult<Vec<DjmdKey>> {
        let models = self.db.get_key().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_key_by_id(&mut self, id: &str) -> PyResult<Option<DjmdKey>> {
        let model = self.db.get_key_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_key_by_name(&mut self, name: &str) -> PyResult<Option<DjmdKey>> {
        let model = self.db.get_key_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn create_key(&mut self, name: &str) -> PyResult<DjmdKey> {
        let model = self
            .db
            .create_key(name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn update_key(&mut self, mut item: PyRefMut<'_, DjmdKey>) -> PyResult<DjmdKey> {
        let model = self.db.update_key(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn delete_key(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_key(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- Label ------------------------------------------------------------------------------------

    pub fn get_label(&mut self) -> PyResult<Vec<DjmdLabel>> {
        let models = self.db.get_label().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_label_by_id(&mut self, id: &str) -> PyResult<Option<DjmdLabel>> {
        let model = self.db.get_label_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_label_by_name(&mut self, name: &str) -> PyResult<Option<DjmdLabel>> {
        let model = self.db.get_label_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn create_label(&mut self, name: &str) -> PyResult<DjmdLabel> {
        let model = self
            .db
            .create_label(name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn update_label(&mut self, mut item: PyRefMut<'_, DjmdLabel>) -> PyResult<DjmdLabel> {
        let model = self.db.update_label(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn delete_label(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_label(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- MenuItems --------------------------------------------------------------------------------

    pub fn get_menu_item(&mut self) -> PyResult<Vec<DjmdMenuItems>> {
        let models = self.db.get_menu_item().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_menu_item_by_id(&mut self, id: &str) -> PyResult<Option<DjmdMenuItems>> {
        let model = self.db.get_menu_item_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- MixerParam -------------------------------------------------------------------------------

    pub fn get_mixer_param(&mut self) -> PyResult<Vec<DjmdMixerParam>> {
        let models = self.db.get_mixer_param().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_mixer_param_by_id(&mut self, id: &str) -> PyResult<Option<DjmdMixerParam>> {
        let model = self.db.get_mixer_param_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    pub fn get_my_tag(&mut self) -> PyResult<Vec<DjmdMyTag>> {
        let models = self.db.get_my_tag().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_my_tag_children(&mut self, parent_id: &str) -> PyResult<Vec<DjmdMyTag>> {
        let models = self
            .db
            .get_my_tag_children(parent_id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_my_tag_by_id(&mut self, id: &str) -> PyResult<Option<DjmdMyTag>> {
        let model = self.db.get_my_tag_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_my_tag_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongMyTag>> {
        let models = self.db.get_my_tag_songs(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_my_tag_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_my_tag_contents(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- Playlist ---------------------------------------------------------------------------------

    pub fn get_playlist(&mut self) -> PyResult<Vec<DjmdPlaylist>> {
        let models = self.db.get_playlist().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_playlist_tree(&mut self, py: Python) -> PyResult<Vec<PyDjmdPlaylistTreeNode>> {
        let models = self.db.get_playlist_tree().map_err(|e| db_error(e))?;
        let items = models
            .into_iter()
            .map(|m| m.borrow().clone().into_py(py).unwrap())
            .collect();
        Ok(items)
    }

    pub fn get_playlist_children(&mut self, parent_id: &str) -> PyResult<Vec<DjmdPlaylist>> {
        let models = self
            .db
            .get_playlist_children(parent_id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_playlist_by_id(&mut self, id: &str) -> PyResult<Option<DjmdPlaylist>> {
        let model = self.db.get_playlist_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_playlist_by_path(
        &mut self,
        path: Vec<Bound<'_, PyString>>,
    ) -> PyResult<Option<DjmdPlaylist>> {
        let rpath: Vec<String> = path.iter().map(|item| item.to_string()).collect();
        let rpath_str: Vec<&str> = rpath.iter().map(|item| item.as_str()).collect();
        let model = self
            .db
            .get_playlist_by_path(rpath_str)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_playlist_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongPlaylist>> {
        let models = self.db.get_playlist_songs(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_playlist_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_playlist_contents(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_playlist_song_by_id(&mut self, id: &str) -> PyResult<Option<DjmdSongPlaylist>> {
        let model = self
            .db
            .get_playlist_song_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn create_playlist_node(
        &mut self,
        name: &str,
        attribute: i32,
        parent_id: Option<&str>,
        seq: Option<i32>,
        image_path: Option<&str>,
        smart_list: Option<&str>,
    ) -> PyResult<DjmdPlaylist> {
        // Convert to str if it's Some
        let parent_id = parent_id.map(|s| s.to_string());
        let image_path = image_path.map(|s| s.to_string());
        let smart_list = smart_list.map(|s| s.to_string());
        let attr = PlaylistType::try_from(attribute).map_err(|e| db_error(e))?;
        let model = self
            .db
            .create_playlist_node(
                name.to_string(),
                attr,
                parent_id,
                seq,
                image_path,
                smart_list,
            )
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn rename_playlist(&mut self, id: &str, name: &str) -> PyResult<DjmdPlaylist> {
        let model = self
            .db
            .rename_playlist(&id.to_string(), name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn move_playlist(
        &mut self,
        id: &str,
        seq: Option<i32>,
        parent_id: Option<&str>,
    ) -> PyResult<DjmdPlaylist> {
        // Convert to str if it's Some
        let parent_id = parent_id.map(|s| s.to_string());
        let model = self
            .db
            .move_playlist(&id.to_string(), seq, parent_id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn delete_playlist(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_playlist(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    pub fn create_playlist_song(
        &mut self,
        playlist_id: &str,
        content_id: &str,
        seq: Option<i32>,
    ) -> PyResult<DjmdSongPlaylist> {
        let model = self
            .db
            .create_playlist_song(&playlist_id.to_string(), &content_id.to_string(), seq)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn move_playlist_song(&mut self, id: &str, seq: i32) -> PyResult<DjmdSongPlaylist> {
        let model = self
            .db
            .move_playlist_song(&id.to_string(), seq)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn delete_playlist_song(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_playlist_song(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- Property ---------------------------------------------------------------------------------

    pub fn get_property(&mut self) -> PyResult<Vec<DjmdProperty>> {
        let models = self.db.get_property().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_property_by_id(&mut self, id: &str) -> PyResult<Option<DjmdProperty>> {
        let model = self.db.get_property_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    pub fn get_cloud_property(&mut self) -> PyResult<Vec<DjmdCloudProperty>> {
        let models = self.db.get_cloud_property().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_cloud_property_by_id(&mut self, id: &str) -> PyResult<Option<DjmdCloudProperty>> {
        let model = self
            .db
            .get_cloud_property_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    pub fn get_recommend_like(&mut self) -> PyResult<Vec<DjmdRecommendLike>> {
        let models = self.db.get_recommend_like().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_recommend_like_by_id(&mut self, id: &str) -> PyResult<Option<DjmdRecommendLike>> {
        let model = self
            .db
            .get_recommend_like_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    pub fn get_related_tracks(&mut self) -> PyResult<Vec<DjmdRelatedTracks>> {
        let models = self.db.get_related_tracks().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_related_tracks_children(
        &mut self,
        parent_id: &str,
    ) -> PyResult<Vec<DjmdRelatedTracks>> {
        let models = self
            .db
            .get_related_tracks_children(parent_id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_related_tracks_by_id(&mut self, id: &str) -> PyResult<Option<DjmdRelatedTracks>> {
        let model = self
            .db
            .get_related_tracks_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_related_tracks_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongRelatedTracks>> {
        let models = self
            .db
            .get_related_tracks_songs(id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_related_tracks_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self
            .db
            .get_related_tracks_contents(id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- Sampler ----------------------------------------------------------------------------------

    pub fn get_sampler(&mut self) -> PyResult<Vec<DjmdSampler>> {
        let models = self.db.get_sampler().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_sampler_children(&mut self, parent_id: &str) -> PyResult<Vec<DjmdSampler>> {
        let models = self
            .db
            .get_sampler_children(parent_id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_sampler_by_id(&mut self, id: &str) -> PyResult<Option<DjmdSampler>> {
        let model = self.db.get_sampler_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    pub fn get_sampler_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongSampler>> {
        let models = self.db.get_sampler_songs(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_sampler_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_sampler_contents(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- SongTagList ------------------------------------------------------------------------------

    pub fn get_song_tag_list(&mut self) -> PyResult<Vec<DjmdSongTagList>> {
        let models = self.db.get_song_tag_list().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_song_tag_list_by_id(&mut self, id: &str) -> PyResult<Option<DjmdSongTagList>> {
        let model = self
            .db
            .get_song_tag_list_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Sort -------------------------------------------------------------------------------------

    pub fn get_sort(&mut self) -> PyResult<Vec<DjmdSort>> {
        let models = self.db.get_sort().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_sort_by_id(&mut self, id: &str) -> PyResult<Option<DjmdSort>> {
        let model = self.db.get_sort_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- ImageFile --------------------------------------------------------------------------------

    pub fn get_image_file(&mut self) -> PyResult<Vec<ImageFile>> {
        let models = self.db.get_image_file().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_image_file_by_id(&mut self, id: &str) -> PyResult<Option<ImageFile>> {
        let model = self.db.get_image_file_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- SettingFile ------------------------------------------------------------------------------

    pub fn get_setting_file(&mut self) -> PyResult<Vec<SettingFile>> {
        let models = self.db.get_setting_file().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_setting_file_by_id(&mut self, id: &str) -> PyResult<Option<SettingFile>> {
        let model = self
            .db
            .get_setting_file_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    pub fn get_uuid_id_map(&mut self) -> PyResult<Vec<UuidIDMap>> {
        let models = self.db.get_uuid_id_map().map_err(|e| db_error(e))?;
        Ok(models)
    }

    pub fn get_uuid_id_map_by_id(&mut self, id: &str) -> PyResult<Option<UuidIDMap>> {
        let model = self.db.get_uuid_id_map_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }
}
