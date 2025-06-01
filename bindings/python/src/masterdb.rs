// Author: Dylan Jones
// Date:   2025-05-01

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyType};
use rbox::masterdb::{enums::PlaylistType, models::*, MasterDb};

use super::errors::DatabaseError;
use super::py_models::{FromPy, IntoPy, *};

#[pyclass(unsendable)]
pub struct PyMasterDb {
    db: MasterDb,
}

#[pymethods]
impl PyMasterDb {
    #[new]
    pub fn new(path: &str) -> PyResult<Self> {
        let db = MasterDb::new(path).map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(PyMasterDb { db })
    }

    #[classmethod]
    fn open(_cls: &Bound<'_, PyType>) -> PyResult<Self> {
        let db = MasterDb::open().map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(PyMasterDb { db })
    }

    pub fn set_unsafe_writes(&mut self, unsafe_writes: bool) -> PyResult<()> {
        self.db.set_unsafe_writes(unsafe_writes);
        Ok(())
    }

    // -- AgentRegistry ----------------------------------------------------------------------------

    pub fn get_agent_registry(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_agent_registry()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_agent_registry_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyAgentRegistry>> {
        let model = self
            .db
            .get_agent_registry_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_local_usn(&mut self, _py: Python) -> PyResult<i32> {
        let result = self
            .db
            .get_local_usn()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(result)
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    pub fn get_cloud_agent_registry(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_cloud_agent_registry()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_cloud_agent_registry_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyCloudAgentRegistry>> {
        let model = self
            .db
            .get_cloud_agent_registry_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    pub fn get_content_active_censor(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_content_active_censor()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_content_active_censor_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyContentActiveCensor>> {
        let model = self
            .db
            .get_content_active_censor_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- ContentCue -------------------------------------------------------------------------------

    pub fn get_content_cue(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_content_cue()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_content_cue_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyContentCue>> {
        let model = self
            .db
            .get_content_cue_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- ContentFile ------------------------------------------------------------------------------

    pub fn get_content_file(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_content_file()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_content_file_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyContentFile>> {
        let model = self
            .db
            .get_content_file_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    pub fn get_active_censor(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_active_censor()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_active_censor_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyDjmdActiveCensor>> {
        let model = self
            .db
            .get_active_censor_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- Album ------------------------------------------------------------------------------------

    pub fn get_album(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_album()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_album_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdAlbum>> {
        let model = self
            .db
            .get_album_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_album_by_name(&mut self, py: Python, name: &str) -> PyResult<Option<PyDjmdAlbum>> {
        let model = self
            .db
            .get_album_by_name(name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn insert_album(
        &mut self,
        py: Python,
        name: &str,
        artist_id: Option<&str>,
        image_path: Option<&str>,
        compilation: Option<i32>,
    ) -> PyResult<PyDjmdAlbum> {
        // Convert to str if it's Some
        let artist_id = artist_id.map(|s| s.to_string());
        let image_path = image_path.map(|s| s.to_string());
        let model = self
            .db
            .insert_album(name.to_string(), artist_id, image_path, compilation)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn update_album(
        &mut self,
        py: Python,
        py_model: PyRef<'_, PyDjmdAlbum>,
    ) -> PyResult<PyDjmdAlbum> {
        let mut model = DjmdAlbum::from_py(py, py_model);
        let item = self
            .db
            .update_album(&mut model)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        item.into_py(py)
    }

    pub fn delete_album(&mut self, id: &str) -> PyResult<()> {
        self.db
            .delete_album(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    // -- Artist -----------------------------------------------------------------------------------

    pub fn get_artist(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_artist()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_artist_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdArtist>> {
        let model = self
            .db
            .get_artist_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_artist_by_name(&mut self, py: Python, name: &str) -> PyResult<Option<PyDjmdArtist>> {
        let model = self
            .db
            .get_artist_by_name(name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn insert_artist(&mut self, py: Python, name: &str) -> PyResult<PyDjmdArtist> {
        let model = self
            .db
            .insert_artist(name.to_string())
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn update_artist(
        &mut self,
        py: Python,
        py_model: PyRef<'_, PyDjmdArtist>,
    ) -> PyResult<PyDjmdArtist> {
        let mut model = DjmdArtist::from_py(py, py_model);
        let item = self
            .db
            .update_artist(&mut model)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        item.into_py(py)
    }

    pub fn delete_artist(&mut self, id: &str) -> PyResult<()> {
        self.db
            .delete_artist(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    // -- Category ---------------------------------------------------------------------------------

    pub fn get_category(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_category()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_category_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdCategory>> {
        let model = self
            .db
            .get_category_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- Color ------------------------------------------------------------------------------------

    pub fn get_color(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_color()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_color_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdColor>> {
        let model = self
            .db
            .get_color_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- Content ----------------------------------------------------------------------------------

    pub fn get_content(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_content()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_content_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdContent>> {
        // Get the content by ID from the database
        let model = self
            .db
            .get_content_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_content_by_path(
        &mut self,
        py: Python,
        path: &str,
    ) -> PyResult<Option<PyDjmdContent>> {
        let model = self
            .db
            .get_content_by_path(path)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_content_anlz_dir(&mut self, id: &str) -> PyResult<Option<String>> {
        let path = self
            .db
            .get_content_anlz_dir(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
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
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
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

    pub fn insert_content(&mut self, py: Python, path: &str) -> PyResult<PyDjmdContent> {
        let model = self
            .db
            .insert_content(path)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn update_content(
        &mut self,
        py: Python,
        py_model: PyRef<'_, PyDjmdContent>,
    ) -> PyResult<()> {
        let model = DjmdContent::from_py(py, py_model);
        self.db
            .update_content(&model)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn update_content_album(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_album(id, name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn update_content_artist(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_artist(id, name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn update_content_remixer(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_remixer(id, name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn update_content_original_artist(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_original_artist(id, name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn update_content_composer(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_composer(id, name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn update_content_genre(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_genre(id, name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn update_content_label(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_label(id, name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn update_content_key(&mut self, id: &str, name: &str) -> PyResult<()> {
        self.db
            .update_content_key(id, name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    // pub fn delete_content(&mut self, id: &str) -> PyResult<()> {
    //     self.db
    //         .delete_content(id)
    //         .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
    //     Ok(())
    // }

    // -- Cue --------------------------------------------------------------------------------------

    pub fn get_cue(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_cue()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_cue_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdCue>> {
        let model = self
            .db
            .get_cue_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- Device -----------------------------------------------------------------------------------

    pub fn get_device(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_device()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_device_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdDevice>> {
        let model = self
            .db
            .get_device_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- Genre ------------------------------------------------------------------------------------

    pub fn get_genre(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_genre()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_genre_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdGenre>> {
        let model = self
            .db
            .get_genre_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_genre_by_name(&mut self, py: Python, name: &str) -> PyResult<Option<PyDjmdGenre>> {
        let model = self
            .db
            .get_genre_by_name(name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn insert_genre(&mut self, py: Python, name: &str) -> PyResult<PyDjmdGenre> {
        let model = self
            .db
            .insert_genre(name.to_string())
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn update_genre(
        &mut self,
        py: Python,
        py_model: PyRef<'_, PyDjmdGenre>,
    ) -> PyResult<PyDjmdGenre> {
        let mut model = DjmdGenre::from_py(py, py_model);
        let item = self
            .db
            .update_genre(&mut model)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        item.into_py(py)
    }

    pub fn delete_genre(&mut self, id: &str) -> PyResult<()> {
        self.db
            .delete_genre(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    // -- History ----------------------------------------------------------------------------------

    pub fn get_history(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_history()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_history_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdHistory>> {
        let model = self
            .db
            .get_history_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_history_songs(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_history_songs(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_history_contents(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_history_contents(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    pub fn get_hot_cue_banklist(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_hot_cue_banklist()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_hot_cue_banklist_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyDjmdHotCueBanklist>> {
        let model = self
            .db
            .get_hot_cue_banklist_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_hot_cue_banklist_children(
        &mut self,
        py: Python,
        parent_id: &str,
    ) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_hot_cue_banklist_children(parent_id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_hot_cue_banklist_songs(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_hot_cue_banklist_songs(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_hot_cue_banklist_contents(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_hot_cue_banklist_contents(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_hot_cue_banklist_cues(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_hot_cue_banklist_cues(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    // -- Key --------------------------------------------------------------------------------------

    pub fn get_key(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_key()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_key_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdKey>> {
        let model = self
            .db
            .get_key_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_key_by_name(&mut self, py: Python, name: &str) -> PyResult<Option<PyDjmdKey>> {
        let model = self
            .db
            .get_key_by_name(name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn insert_key(&mut self, py: Python, name: &str) -> PyResult<PyDjmdKey> {
        let model = self
            .db
            .insert_key(name.to_string())
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn update_key(
        &mut self,
        py: Python,
        py_model: PyRef<'_, PyDjmdKey>,
    ) -> PyResult<PyDjmdKey> {
        let mut model = DjmdKey::from_py(py, py_model);
        let item = self
            .db
            .update_key(&mut model)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        item.into_py(py)
    }

    pub fn delete_key(&mut self, id: &str) -> PyResult<()> {
        self.db
            .delete_key(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    // -- Label ------------------------------------------------------------------------------------

    pub fn get_label(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_label()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_label_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdLabel>> {
        let model = self
            .db
            .get_label_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_label_by_name(&mut self, py: Python, name: &str) -> PyResult<Option<PyDjmdLabel>> {
        let model = self
            .db
            .get_label_by_name(name)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn insert_label(&mut self, py: Python, name: &str) -> PyResult<PyDjmdLabel> {
        let model = self
            .db
            .insert_label(name.to_string())
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn update_label(
        &mut self,
        py: Python,
        py_model: PyRef<'_, PyDjmdLabel>,
    ) -> PyResult<PyDjmdLabel> {
        let mut model = DjmdLabel::from_py(py, py_model);
        let item = self
            .db
            .update_label(&mut model)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        item.into_py(py)
    }

    pub fn delete_label(&mut self, id: &str) -> PyResult<()> {
        self.db
            .delete_label(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    // -- MenuItems --------------------------------------------------------------------------------

    pub fn get_menu_item(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_menu_item()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_menu_item_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyDjmdMenuItems>> {
        let model = self
            .db
            .get_menu_item_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- MixerParam -------------------------------------------------------------------------------

    pub fn get_mixer_param(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_mixer_param()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_mixer_param_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyDjmdMixerParam>> {
        let model = self
            .db
            .get_mixer_param_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- MyTag ------------------------------------------------------------------------------------

    pub fn get_my_tag(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_my_tag()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_my_tag_children(&mut self, py: Python, parent_id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_my_tag_children(parent_id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_my_tag_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdMyTag>> {
        let model = self
            .db
            .get_my_tag_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_my_tag_songs(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_my_tag_songs(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_my_tag_contents(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_my_tag_contents(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    // -- Playlist ---------------------------------------------------------------------------------

    pub fn get_playlist(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_playlist()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_playlist_tree(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_playlist_tree()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        // let serializable_items: Vec<PlaylistTreeItem> =
        //     items.iter().map(|item| item.borrow().clone()).collect();
        let items = models
            .into_iter()
            .map(|m| m.borrow().clone().into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_playlist_children(&mut self, py: Python, parent_id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_playlist_children(parent_id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_playlist_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdPlaylist>> {
        let model = self
            .db
            .get_playlist_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_playlist_by_path(
        &mut self,
        py: Python,
        path: Vec<Bound<'_, PyString>>,
    ) -> PyResult<Option<PyDjmdPlaylist>> {
        let rpath: Vec<String> = path.iter().map(|item| item.to_string()).collect();
        let rpath_str: Vec<&str> = rpath.iter().map(|item| item.as_str()).collect();
        let model = self
            .db
            .get_playlist_by_path(rpath_str)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_playlist_songs(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_playlist_songs(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_playlist_contents(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_playlist_contents(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_playlist_song_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyDjmdSongPlaylist>> {
        let model = self
            .db
            .get_playlist_song_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn insert_playlist(
        &mut self,
        py: Python,
        name: &str,
        attribute: i32,
        parent_id: Option<&str>,
        seq: Option<i32>,
        image_path: Option<&str>,
        smart_list: Option<&str>,
    ) -> PyResult<PyDjmdPlaylist> {
        // Convert to str if it's Some
        let parent_id = parent_id.map(|s| s.to_string());
        let image_path = image_path.map(|s| s.to_string());
        let smart_list = smart_list.map(|s| s.to_string());
        let attr = PlaylistType::try_from(attribute)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let model = self
            .db
            .insert_playlist(
                name.to_string(),
                attr,
                parent_id,
                seq,
                image_path,
                smart_list,
            )
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn rename_playlist(
        &mut self,
        py: Python,
        id: &str,
        name: &str,
    ) -> PyResult<PyDjmdPlaylist> {
        let model = self
            .db
            .rename_playlist(&id.to_string(), name.to_string())
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn move_playlist(
        &mut self,
        py: Python,
        id: &str,
        seq: Option<i32>,
        parent_id: Option<&str>,
    ) -> PyResult<PyDjmdPlaylist> {
        // Convert to str if it's Some
        let parent_id = parent_id.map(|s| s.to_string());
        let model = self
            .db
            .move_playlist(&id.to_string(), seq, parent_id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn delete_playlist(&mut self, id: &str) -> PyResult<()> {
        self.db
            .delete_playlist(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn insert_playlist_song(
        &mut self,
        py: Python,
        playlist_id: &str,
        content_id: &str,
        seq: Option<i32>,
    ) -> PyResult<PyDjmdSongPlaylist> {
        let model = self
            .db
            .insert_playlist_song(&playlist_id.to_string(), &content_id.to_string(), seq)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn move_playlist_song(
        &mut self,
        py: Python,
        id: &str,
        seq: i32,
    ) -> PyResult<PyDjmdSongPlaylist> {
        let model = self
            .db
            .move_playlist_song(&id.to_string(), seq)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        model.into_py(py)
    }

    pub fn delete_playlist_song(&mut self, id: &str) -> PyResult<()> {
        self.db
            .delete_playlist_song(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(())
    }

    // -- Property ---------------------------------------------------------------------------------

    pub fn get_property(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_property()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_property_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdProperty>> {
        let model = self
            .db
            .get_property_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    pub fn get_cloud_property(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_cloud_property()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_cloud_property_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyDjmdCloudProperty>> {
        let model = self
            .db
            .get_cloud_property_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    pub fn get_recommend_like(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_recommend_like()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_recommend_like_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyDjmdRecommendLike>> {
        let model = self
            .db
            .get_recommend_like_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    pub fn get_related_tracks(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_related_tracks()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_related_tracks_children(
        &mut self,
        py: Python,
        parent_id: &str,
    ) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_related_tracks_children(parent_id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_related_tracks_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyDjmdRelatedTracks>> {
        let model = self
            .db
            .get_related_tracks_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_related_tracks_songs(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_related_tracks_songs(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_related_tracks_contents(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_related_tracks_contents(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    // -- Sampler ----------------------------------------------------------------------------------

    pub fn get_sampler(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_sampler()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_sampler_children(&mut self, py: Python, parent_id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_sampler_children(parent_id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_sampler_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdSampler>> {
        let model = self
            .db
            .get_sampler_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    pub fn get_sampler_songs(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_sampler_songs(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_sampler_contents(&mut self, py: Python, id: &str) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_sampler_contents(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    // -- SongTagList ------------------------------------------------------------------------------

    pub fn get_song_tag_list(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_song_tag_list()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_song_tag_list_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PyDjmdSongTagList>> {
        let model = self
            .db
            .get_song_tag_list_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- Sort -------------------------------------------------------------------------------------

    pub fn get_sort(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_sort()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_sort_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyDjmdSort>> {
        let model = self
            .db
            .get_sort_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- ImageFile --------------------------------------------------------------------------------

    pub fn get_image_file(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_image_file()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_image_file_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyImageFile>> {
        let model = self
            .db
            .get_image_file_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- SettingFile ------------------------------------------------------------------------------

    pub fn get_setting_file(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_setting_file()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_setting_file_by_id(
        &mut self,
        py: Python,
        id: &str,
    ) -> PyResult<Option<PySettingFile>> {
        let model = self
            .db
            .get_setting_file_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    pub fn get_uuid_id_map(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let models = self
            .db
            .get_uuid_id_map()
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        let items = models
            .into_iter()
            .map(|m| m.into_py(py).unwrap())
            .collect::<Vec<_>>();
        Ok(PyList::new(py, items)?.into())
    }

    pub fn get_uuid_id_map_by_id(&mut self, py: Python, id: &str) -> PyResult<Option<PyUuidIDMap>> {
        let model = self
            .db
            .get_uuid_id_map_by_id(id)
            .map_err(|e| PyErr::new::<DatabaseError, _>(e.to_string()))?;
        Ok(model.map(|m| m.into_py(py).unwrap()))
    }
}
