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
    /// Open a Rekordbox database specified by path.
    #[new]
    pub fn new(path: &str) -> PyResult<Self> {
        let db = MasterDb::new(path).map_err(|e| db_error(e))?;
        Ok(PyMasterDb { db })
    }

    /// Open the default Rekordbox `master.db` database.
    #[classmethod]
    fn open(_cls: &Bound<'_, PyType>) -> PyResult<Self> {
        let db = MasterDb::open().map_err(|e| db_error(e))?;
        Ok(PyMasterDb { db })
    }

    /// Sets the unsafe writes flag for the database.
    pub fn set_unsafe_writes(&mut self, unsafe_writes: bool) -> PyResult<()> {
        self.db.set_unsafe_writes(unsafe_writes);
        Ok(())
    }

    /// Returns the path to the configured share directory.
    pub fn share_directory(&self) -> Option<String> {
        self.db
            .share_directory()
            .map(|p| p.to_string_lossy().to_string())
    }

    /// Returns the path to the configured playlist XML file.
    pub fn playlist_xml_path(&self) -> Option<String> {
        self.db
            .playlist_xml_path()
            .map(|p| p.to_string_lossy().to_string())
    }

    // -- AgentRegistry ----------------------------------------------------------------------------

    /// Retrieves all entries from the [`AgentRegistry`] table in the database.
    pub fn get_agent_registries(&mut self) -> PyResult<Vec<AgentRegistry>> {
        let models = self.db.get_agent_registries().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves an `agentRegistry` entry by its unique identifier.
    pub fn get_agent_registry_by_id(&mut self, id: &str) -> PyResult<Option<AgentRegistry>> {
        let model = self
            .db
            .get_agent_registry_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves the local update sequence number (USN) from the `agentRegistry` table.
    pub fn get_local_usn(&mut self) -> PyResult<i32> {
        let result = self.db.get_local_usn().map_err(|e| db_error(e))?;
        Ok(result)
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    /// Retrieves all entries from the `cloudAgentRegistry` table in the database.
    pub fn get_cloud_agent_registries(&mut self) -> PyResult<Vec<CloudAgentRegistry>> {
        let models = self
            .db
            .get_cloud_agent_registries()
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `cloudAgentRegistry` entry by its unique identifier.
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

    /// Retrieves all entries from the `contentActiveCensor` table in the database.
    pub fn get_content_active_censors(&mut self) -> PyResult<Vec<ContentActiveCensor>> {
        let models = self
            .db
            .get_content_active_censors()
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `contentActiveCensor` entry by its unique identifier.
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

    /// Retrieves all entries from the `contentCue` table in the database.
    pub fn get_content_cues(&mut self) -> PyResult<Vec<ContentCue>> {
        let models = self.db.get_content_cues().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `contentCue` entry by its unique identifier.
    pub fn get_content_cue_by_id(&mut self, id: &str) -> PyResult<Option<ContentCue>> {
        let model = self.db.get_content_cue_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- ContentFile ------------------------------------------------------------------------------

    /// Retrieves all entries from the `contentFile` table in the database.
    pub fn get_content_files(&mut self) -> PyResult<Vec<ContentFile>> {
        let models = self.db.get_content_files().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `contentFile` entry by its unique identifier.
    pub fn get_content_file_by_id(&mut self, id: &str) -> PyResult<Option<ContentFile>> {
        let model = self
            .db
            .get_content_file_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdActiveCensor` table in the database.
    pub fn get_active_censors(&mut self) -> PyResult<Vec<DjmdActiveCensor>> {
        let models = self.db.get_active_censors().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdActiveCensor` entry by its unique identifier.
    pub fn get_active_censor_by_id(&mut self, id: &str) -> PyResult<Option<DjmdActiveCensor>> {
        let model = self
            .db
            .get_active_censor_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Album ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdAlbum` table in the database.
    pub fn get_albums(&mut self) -> PyResult<Vec<DjmdAlbum>> {
        let models = self.db.get_albums().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdAlbum` entry by its unique identifier.
    pub fn get_album_by_id(&mut self, id: &str) -> PyResult<Option<DjmdAlbum>> {
        let model = self.db.get_album_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves a `djmdAlbum` entry by its name.
    pub fn get_album_by_name(&mut self, name: &str) -> PyResult<Option<DjmdAlbum>> {
        let model = self.db.get_album_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Creates a new album and inserts it into the `djmdAlbum` table in the database.
    #[pyo3(signature = (name, artist_id=None, image_path=None, compilation=None))]
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

    /// Updates an existing `djmdAlbum` entry in the database.
    pub fn update_album(&mut self, mut item: PyRefMut<'_, DjmdAlbum>) -> PyResult<DjmdAlbum> {
        let model = self.db.update_album(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Deletes an album entry from the `djmdAlbum` table in the database.
    pub fn delete_album(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_album(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- Artist -----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdArtist` table in the database.
    pub fn get_artists(&mut self) -> PyResult<Vec<DjmdArtist>> {
        let models = self.db.get_artists().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdArtist` entry by its unique identifier.
    pub fn get_artist_by_id(&mut self, id: &str) -> PyResult<Option<DjmdArtist>> {
        let model = self.db.get_artist_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves a `djmdArtist` entry by its name.
    pub fn get_artist_by_name(&mut self, name: &str) -> PyResult<Option<DjmdArtist>> {
        let model = self.db.get_artist_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Creates a new artist and inserts it into the `djmdArtist` table in the database.
    pub fn create_artist(&mut self, name: &str) -> PyResult<DjmdArtist> {
        let model = self
            .db
            .create_artist(name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Updates an existing `djmdArtist` entry in the database.
    pub fn update_artist(&mut self, mut item: PyRefMut<'_, DjmdArtist>) -> PyResult<DjmdArtist> {
        let model = self.db.update_artist(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Deletes an artist entry from the `djmdArtist` table in the database.
    pub fn delete_artist(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_artist(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- Category ---------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdCategory` table in the database.
    pub fn get_categories(&mut self) -> PyResult<Vec<DjmdCategory>> {
        let models = self.db.get_categories().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdCategory` entry by its unique identifier.
    pub fn get_category_by_id(&mut self, id: &str) -> PyResult<Option<DjmdCategory>> {
        let model = self.db.get_category_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Color ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdColor` table in the database.
    pub fn get_colors(&mut self) -> PyResult<Vec<DjmdColor>> {
        let models = self.db.get_colors().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdColor` entry by its unique identifier.
    pub fn get_color_by_id(&mut self, id: &str) -> PyResult<Option<DjmdColor>> {
        let model = self.db.get_color_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Content ----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdContent` table in the database.
    pub fn get_contents(&mut self) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_contents().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdContent` entry by its unique identifier.
    pub fn get_content_by_id(&mut self, id: &str) -> PyResult<Option<DjmdContent>> {
        // Get the content by ID from the database
        let model = self.db.get_content_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves a `djmdContent` entry by its folder path.
    pub fn get_content_by_path(&mut self, path: &str) -> PyResult<Option<DjmdContent>> {
        let model = self.db.get_content_by_path(path).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Returns the path to the directory containing the ANLZxxxx.xxx files for a given content ID.
    pub fn get_content_anlz_dir(&mut self, id: &str) -> PyResult<Option<String>> {
        let path = self.db.get_content_anlz_dir(id).map_err(|e| db_error(e))?;
        if path.is_none() {
            return Ok(None);
        }
        Ok(Some(
            path.unwrap().as_os_str().to_str().unwrap().to_string(),
        ))
    }

    /// Returns a struct containing the paths to ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 files.
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

    /// Creates a new content entry and inserts it into the `djmdContent` table.
    pub fn create_content(&mut self, path: &str) -> PyResult<DjmdContent> {
        let model = self.db.create_content(path).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Updates an existing `djmdContent` entry in the database.
    pub fn update_content(&mut self, item: PyRef<'_, DjmdContent>) -> PyResult<()> {
        self.db.update_content(&item).map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Update the content album field.
    pub fn update_content_album(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_album(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Update the content artist name.
    pub fn update_content_artist(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_artist(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Update the content remixer name.
    pub fn update_content_remixer(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_remixer(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Update the content original artist name.
    pub fn update_content_original_artist(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_original_artist(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Update the content composer name.
    pub fn update_content_composer(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_composer(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Update the content genre name.
    pub fn update_content_genre(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_genre(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Update the content label name.
    pub fn update_content_label(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_label(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Update the content key name.
    pub fn update_content_key(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_key(id, name)
            .map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Updates the file path of a content entry in the database.
    pub fn update_content_path(&mut self, content_id: &str, path: &str) -> PyResult<usize> {
        let result = self
            .db
            .update_content_path(content_id, path)
            .map_err(|e| db_error(e))?;
        Ok(result)
    }

    // -- Cue --------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdCue` table.
    pub fn get_cues(&mut self) -> PyResult<Vec<DjmdCue>> {
        let models = self.db.get_cues().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdCue` entry by its unique identifier.
    pub fn get_cue_by_id(&mut self, id: &str) -> PyResult<Option<DjmdCue>> {
        let model = self.db.get_cue_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Device -----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdDevice` table.
    pub fn get_devices(&mut self) -> PyResult<Vec<DjmdDevice>> {
        let models = self.db.get_devices().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdDevice` entry by its unique identifier.
    pub fn get_device_by_id(&mut self, id: &str) -> PyResult<Option<DjmdDevice>> {
        let model = self.db.get_device_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Genre ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdGenre` table in the database.
    pub fn get_genres(&mut self) -> PyResult<Vec<DjmdGenre>> {
        let models = self.db.get_genres().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdGenre` entry by its unique identifier.
    pub fn get_genre_by_id(&mut self, id: &str) -> PyResult<Option<DjmdGenre>> {
        let model = self.db.get_genre_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves a `djmdGenre` entry by its name.
    pub fn get_genre_by_name(&mut self, name: &str) -> PyResult<Option<DjmdGenre>> {
        let model = self.db.get_genre_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Creates a new genre and inserts it into the `djmdGenre` table in the database.
    pub fn create_genre(&mut self, name: &str) -> PyResult<DjmdGenre> {
        let model = self
            .db
            .create_genre(name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Updates an existing `djmdGenre` entry in the database.
    pub fn update_genre(&mut self, mut item: PyRefMut<'_, DjmdGenre>) -> PyResult<DjmdGenre> {
        let model = self.db.update_genre(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Deletes a genre entry from the `djmdGenre` table in the database.
    pub fn delete_genre(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_genre(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- History ----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdHistory` table in the database.
    pub fn get_histories(&mut self) -> PyResult<Vec<DjmdHistory>> {
        let models = self.db.get_histories().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdHistory` entry by its unique identifier.
    pub fn get_history_by_id(&mut self, id: &str) -> PyResult<Option<DjmdHistory>> {
        let model = self.db.get_history_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves all song history entries for a given history ID, ordered by track number.
    pub fn get_history_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongHistory>> {
        let models = self.db.get_history_songs(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves all content entries referenced by the song history for a given history ID.
    pub fn get_history_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_history_contents(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdHotCueBanklist` table in the database.
    pub fn get_hot_cue_banklists(&mut self) -> PyResult<Vec<DjmdHotCueBanklist>> {
        let models = self.db.get_hot_cue_banklists().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdHotCueBanklist` entry by its unique identifier.
    pub fn get_hot_cue_banklist_by_id(&mut self, id: &str) -> PyResult<Option<DjmdHotCueBanklist>> {
        let model = self
            .db
            .get_hot_cue_banklist_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves all child `djmdHotCueBanklist` entries for a given parent ID, ordered by sequence.
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

    /// Retrieves all `djmdSongHotCueBanklist` entries for a given hot cue banklist ID, ordered by track number.
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

    /// Retrieves all `djmdContent` entries referenced by the song hot cue banklist for a given hot cue banklist ID.
    pub fn get_hot_cue_banklist_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self
            .db
            .get_hot_cue_banklist_contents(id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves all `hotCueBanklistCue` entries for a given hot cue banklist ID.
    pub fn get_hot_cue_banklist_cues(&mut self, id: &str) -> PyResult<Vec<HotCueBanklistCue>> {
        let models = self
            .db
            .get_hot_cue_banklist_cues(id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- Key --------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdKey` table in the database.
    pub fn get_keys(&mut self) -> PyResult<Vec<DjmdKey>> {
        let models = self.db.get_keys().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdKey` entry by its unique identifier.
    pub fn get_key_by_id(&mut self, id: &str) -> PyResult<Option<DjmdKey>> {
        let model = self.db.get_key_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves a `djmdKey` entry by its scale name.
    pub fn get_key_by_name(&mut self, name: &str) -> PyResult<Option<DjmdKey>> {
        let model = self.db.get_key_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Creates a new key and inserts it into the `djmdKey` table in the database.
    pub fn create_key(&mut self, name: &str) -> PyResult<DjmdKey> {
        let model = self
            .db
            .create_key(name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Updates an existing `djmdKey` entry in the database.
    pub fn update_key(&mut self, mut item: PyRefMut<'_, DjmdKey>) -> PyResult<DjmdKey> {
        let model = self.db.update_key(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Deletes a key entry from the `djmdKey` table in the database.
    pub fn delete_key(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_key(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- Label ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdLabel` table in the database.
    pub fn get_labels(&mut self) -> PyResult<Vec<DjmdLabel>> {
        let models = self.db.get_labels().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdLabel` entry by its unique identifier.
    pub fn get_label_by_id(&mut self, id: &str) -> PyResult<Option<DjmdLabel>> {
        let model = self.db.get_label_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves a `djmdLabel` entry by its name.
    pub fn get_label_by_name(&mut self, name: &str) -> PyResult<Option<DjmdLabel>> {
        let model = self.db.get_label_by_name(name).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Creates a new label and inserts it into the `djmdLabel` table in the database.
    pub fn create_label(&mut self, name: &str) -> PyResult<DjmdLabel> {
        let model = self
            .db
            .create_label(name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Updates an existing `djmdLabel` entry in the database.
    pub fn update_label(&mut self, mut item: PyRefMut<'_, DjmdLabel>) -> PyResult<DjmdLabel> {
        let model = self.db.update_label(&mut item).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Deletes a label entry from the `djmdLabel` table in the database.
    pub fn delete_label(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_label(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- MenuItems --------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdMenuItems` table in the database.
    pub fn get_menu_items(&mut self) -> PyResult<Vec<DjmdMenuItems>> {
        let models = self.db.get_menu_items().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdMenuItems` entry by its unique identifier.
    pub fn get_menu_item_by_id(&mut self, id: &str) -> PyResult<Option<DjmdMenuItems>> {
        let model = self.db.get_menu_item_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- MixerParam -------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdMixerParam` table in the database.
    pub fn get_mixer_params(&mut self) -> PyResult<Vec<DjmdMixerParam>> {
        let models = self.db.get_mixer_params().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdMixerParam` entry by its unique identifier.
    pub fn get_mixer_param_by_id(&mut self, id: &str) -> PyResult<Option<DjmdMixerParam>> {
        let model = self.db.get_mixer_param_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdMyTag` table in the database.
    pub fn get_my_tags(&mut self) -> PyResult<Vec<DjmdMyTag>> {
        let models = self.db.get_my_tags().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves all child `djmdMyTag` entries for a given parent ID, ordered by sequence.
    pub fn get_my_tag_children(&mut self, parent_id: &str) -> PyResult<Vec<DjmdMyTag>> {
        let models = self
            .db
            .get_my_tag_children(parent_id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdMyTag` entry by its unique identifier.
    pub fn get_my_tag_by_id(&mut self, id: &str) -> PyResult<Option<DjmdMyTag>> {
        let model = self.db.get_my_tag_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves all `djmdSongMyTag` entries for a given tag ID, ordered by track number.
    pub fn get_my_tag_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongMyTag>> {
        let models = self.db.get_my_tag_songs(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given tag ID.
    pub fn get_my_tag_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_my_tag_contents(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- Playlist ---------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdPlaylist` table in the database.
    pub fn get_playlists(&mut self) -> PyResult<Vec<DjmdPlaylist>> {
        let models = self.db.get_playlists().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Returns a sorted tree of playlists as tree nodes.
    pub fn get_playlist_tree(&mut self, py: Python) -> PyResult<Vec<PyDjmdPlaylistTreeNode>> {
        let models = self.db.get_playlist_tree().map_err(|e| db_error(e))?;
        let items = models
            .into_iter()
            .map(|m| m.borrow().clone().into_py(py).unwrap())
            .collect();
        Ok(items)
    }

    /// Retrieves all child `djmdPlaylist` entries for a given parent ID, ordered by sequence.
    pub fn get_playlist_children(&mut self, parent_id: &str) -> PyResult<Vec<DjmdPlaylist>> {
        let models = self
            .db
            .get_playlist_children(parent_id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdPlaylist` entry by its unique identifier.
    pub fn get_playlist_by_id(&mut self, id: &str) -> PyResult<Option<DjmdPlaylist>> {
        let model = self.db.get_playlist_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves a `djmdPlaylist` entry by its hierarchical path.
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

    /// Retrieves all `djmdSongPlaylist` entries for a given playlist ID, ordered by track number.
    pub fn get_playlist_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongPlaylist>> {
        let models = self.db.get_playlist_songs(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given playlist ID.
    pub fn get_playlist_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_playlist_contents(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdSongPlaylist` entry by its unique identifier.
    pub fn get_playlist_song_by_id(&mut self, id: &str) -> PyResult<Option<DjmdSongPlaylist>> {
        let model = self
            .db
            .get_playlist_song_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Creates a new playlist and inserts it into the `djmdPlaylist` table in the database.
    #[pyo3(signature = (name, attribute, parent_id=None, seq=None, image_path=None, smart_list=None))]
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

    /// Renames an existing playlist.
    pub fn rename_playlist(&mut self, id: &str, name: &str) -> PyResult<DjmdPlaylist> {
        let model = self
            .db
            .rename_playlist(&id.to_string(), name.to_string())
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Moves a playlist to a new parent folder or changes its sequence within the same parent.
    #[pyo3(signature = (id, seq=None, parent_id=None))]
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

    /// Deletes a playlist entry from the `djmdPlaylist` table in the database.
    pub fn delete_playlist(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_playlist(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    /// Creates a new playlist song and inserts it into the `djmdSongPlaylist` table.
    #[pyo3(signature = (playlist_id, content_id, seq=None))]
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

    /// Moves a song within a playlist to a new sequence position.
    pub fn move_playlist_song(&mut self, id: &str, seq: i32) -> PyResult<DjmdSongPlaylist> {
        let model = self
            .db
            .move_playlist_song(&id.to_string(), seq)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Deletes a song entry from a playlist.
    pub fn delete_playlist_song(&mut self, id: &str) -> PyResult<()> {
        self.db.delete_playlist_song(id).map_err(|e| db_error(e))?;
        Ok(())
    }

    // -- Property ---------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdProperty` table in the database.
    pub fn get_properties(&mut self) -> PyResult<Vec<DjmdProperty>> {
        let models = self.db.get_properties().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdProperty` entry by its unique identifier.
    pub fn get_property_by_id(&mut self, id: &str) -> PyResult<Option<DjmdProperty>> {
        let model = self.db.get_property_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdCloudProperty` table in the database.
    pub fn get_cloud_properties(&mut self) -> PyResult<Vec<DjmdCloudProperty>> {
        let models = self.db.get_cloud_properties().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdCloudProperty` entry by its unique identifier.
    pub fn get_cloud_property_by_id(&mut self, id: &str) -> PyResult<Option<DjmdCloudProperty>> {
        let model = self
            .db
            .get_cloud_property_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdRecommendLike` table in the database.
    pub fn get_recommend_likes(&mut self) -> PyResult<Vec<DjmdRecommendLike>> {
        let models = self.db.get_recommend_likes().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdRecommendLike` entry by its unique identifier.
    pub fn get_recommend_like_by_id(&mut self, id: &str) -> PyResult<Option<DjmdRecommendLike>> {
        let model = self
            .db
            .get_recommend_like_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdRelatedTracks` table in the database.
    pub fn get_related_tracks(&mut self) -> PyResult<Vec<DjmdRelatedTracks>> {
        let models = self.db.get_related_tracks().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves all child `djmdRelatedTracks` entries for a given parent ID, ordered by sequence.
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

    /// Retrieves a `djmdRelatedTracks` entry by its unique identifier.
    pub fn get_related_tracks_by_id(&mut self, id: &str) -> PyResult<Option<DjmdRelatedTracks>> {
        let model = self
            .db
            .get_related_tracks_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves all `djmdSongRelatedTracks` entries for a given related tracks ID, ordered by track number.
    pub fn get_related_tracks_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongRelatedTracks>> {
        let models = self
            .db
            .get_related_tracks_songs(id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given related tracks ID.
    pub fn get_related_tracks_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self
            .db
            .get_related_tracks_contents(id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- Sampler ----------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdSampler` table in the database.
    pub fn get_samplers(&mut self) -> PyResult<Vec<DjmdSampler>> {
        let models = self.db.get_samplers().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves all child `djmdSampler` entries for a given parent ID, ordered by sequence.
    pub fn get_sampler_children(&mut self, parent_id: &str) -> PyResult<Vec<DjmdSampler>> {
        let models = self
            .db
            .get_sampler_children(parent_id)
            .map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdSampler` entry by its unique identifier.
    pub fn get_sampler_by_id(&mut self, id: &str) -> PyResult<Option<DjmdSampler>> {
        let model = self.db.get_sampler_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    /// Retrieves all `djmdSongSampler` entries for a given sampler ID, ordered by track number.
    pub fn get_sampler_songs(&mut self, id: &str) -> PyResult<Vec<DjmdSongSampler>> {
        let models = self.db.get_sampler_songs(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves all `djmdContent` entries referenced by the songs for a given sampler ID.
    pub fn get_sampler_contents(&mut self, id: &str) -> PyResult<Vec<DjmdContent>> {
        let models = self.db.get_sampler_contents(id).map_err(|e| db_error(e))?;
        Ok(models)
    }

    // -- SongTagList ------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdSongTagList` table in the database.
    pub fn get_song_tag_lists(&mut self) -> PyResult<Vec<DjmdSongTagList>> {
        let models = self.db.get_song_tag_lists().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdSongTagList` entry by its unique identifier.
    pub fn get_song_tag_list_by_id(&mut self, id: &str) -> PyResult<Option<DjmdSongTagList>> {
        let model = self
            .db
            .get_song_tag_list_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- Sort -------------------------------------------------------------------------------------

    /// Retrieves all entries from the `djmdSort` table in the database, ordered by sequence.
    pub fn get_sorts(&mut self) -> PyResult<Vec<DjmdSort>> {
        let models = self.db.get_sorts().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `djmdSort` entry by its unique identifier.
    pub fn get_sort_by_id(&mut self, id: &str) -> PyResult<Option<DjmdSort>> {
        let model = self.db.get_sort_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- ImageFile --------------------------------------------------------------------------------

    /// Retrieves all entries from the `imageFile` table in the database.
    pub fn get_image_files(&mut self) -> PyResult<Vec<ImageFile>> {
        let models = self.db.get_image_files().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves an `imageFile` entry by its unique identifier.
    pub fn get_image_file_by_id(&mut self, id: &str) -> PyResult<Option<ImageFile>> {
        let model = self.db.get_image_file_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- SettingFile ------------------------------------------------------------------------------

    /// Retrieves all entries from the `settingFile` table in the database.
    pub fn get_setting_files(&mut self) -> PyResult<Vec<SettingFile>> {
        let models = self.db.get_setting_files().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `settingFile` entry by its unique identifier.
    pub fn get_setting_file_by_id(&mut self, id: &str) -> PyResult<Option<SettingFile>> {
        let model = self
            .db
            .get_setting_file_by_id(id)
            .map_err(|e| db_error(e))?;
        Ok(model)
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    /// Retrieves all entries from the `uuidIDMap` table in the database.
    pub fn get_uuid_id_maps(&mut self) -> PyResult<Vec<UuidIDMap>> {
        let models = self.db.get_uuid_id_maps().map_err(|e| db_error(e))?;
        Ok(models)
    }

    /// Retrieves a `uuidIDMap` entry by its unique identifier.
    pub fn get_uuid_id_map_by_id(&mut self, id: &str) -> PyResult<Option<UuidIDMap>> {
        let model = self.db.get_uuid_id_map_by_id(id).map_err(|e| db_error(e))?;
        Ok(model)
    }
}
