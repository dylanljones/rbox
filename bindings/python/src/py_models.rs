// Author: Dylan Jones
// Date:   2025-06-01

#![allow(non_snake_case)]

use chrono::{DateTime, Utc};
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyList};
use pyo3::PyClass;
use rbox::masterdb::models::*;

pub trait IntoPy<T> {
    fn into_py(self, py: Python) -> PyResult<T>;
}

pub trait FromPy<T: PyClass> {
    fn from_py(py: Python, model: PyRef<'_, T>) -> Self;
}

#[pyclass(name = "AgentRegistry", unsendable, get_all, set_all)]
pub struct PyAgentRegistry {
    pub registry_id: String,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub id_1: Option<String>,
    pub id_2: Option<String>,
    pub int_1: Option<i32>,
    pub int_2: Option<i32>,
    pub str_1: Option<String>,
    pub str_2: Option<String>,
    pub date_1: Option<String>,
    pub date_2: Option<String>,
    pub text_1: Option<String>,
    pub text_2: Option<String>,
}

impl IntoPy<PyAgentRegistry> for AgentRegistry {
    fn into_py(self, py: Python) -> PyResult<PyAgentRegistry> {
        let model = PyAgentRegistry {
            registry_id: self.registry_id,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            id_1: self.id_1,
            id_2: self.id_2,
            int_1: self.int_1,
            int_2: self.int_2,
            str_1: self.str_1,
            str_2: self.str_2,
            date_1: self.date_1,
            date_2: self.date_2,
            text_1: self.text_1,
            text_2: self.text_2,
        };
        Ok(model)
    }
}

impl FromPy<PyAgentRegistry> for AgentRegistry {
    fn from_py(py: Python, model: PyRef<'_, PyAgentRegistry>) -> Self {
        Self {
            registry_id: model.registry_id.clone(),
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            id_1: model.id_1.clone(),
            id_2: model.id_2.clone(),
            int_1: model.int_1,
            int_2: model.int_2,
            str_1: model.str_1.clone(),
            str_2: model.str_2.clone(),
            date_1: model.date_1.clone(),
            date_2: model.date_2.clone(),
            text_1: model.text_1.clone(),
            text_2: model.text_2.clone(),
        }
    }
}

#[pyclass(name = "CloudAgentRegistry", unsendable, get_all, set_all)]
pub struct PyCloudAgentRegistry {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub int_1: Option<i32>,
    pub int_2: Option<i32>,
    pub str_1: Option<String>,
    pub str_2: Option<String>,
    pub date_1: Option<String>,
    pub date_2: Option<String>,
    pub text_1: Option<String>,
    pub text_2: Option<String>,
}

impl IntoPy<PyCloudAgentRegistry> for CloudAgentRegistry {
    fn into_py(self, py: Python) -> PyResult<PyCloudAgentRegistry> {
        let model = PyCloudAgentRegistry {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            int_1: self.int_1,
            int_2: self.int_2,
            str_1: self.str_1,
            str_2: self.str_2,
            date_1: self.date_1,
            date_2: self.date_2,
            text_1: self.text_1,
            text_2: self.text_2,
        };
        Ok(model)
    }
}

impl FromPy<PyCloudAgentRegistry> for CloudAgentRegistry {
    fn from_py(py: Python, model: PyRef<'_, PyCloudAgentRegistry>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            int_1: model.int_1,
            int_2: model.int_2,
            str_1: model.str_1.clone(),
            str_2: model.str_2.clone(),
            date_1: model.date_1.clone(),
            date_2: model.date_2.clone(),
            text_1: model.text_1.clone(),
            text_2: model.text_2.clone(),
        }
    }
}

#[pyclass(name = "ContentActiveCensor", unsendable, get_all, set_all)]
pub struct PyContentActiveCensor {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ContentID: Option<String>,
    pub ActiveCensors: Option<String>,
    pub rb_activecensor_count: Option<i32>,
}

impl IntoPy<PyContentActiveCensor> for ContentActiveCensor {
    fn into_py(self, py: Python) -> PyResult<PyContentActiveCensor> {
        let model = PyContentActiveCensor {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ContentID: self.ContentID,
            ActiveCensors: self.ActiveCensors,
            rb_activecensor_count: self.rb_activecensor_count,
        };
        Ok(model)
    }
}

impl FromPy<PyContentActiveCensor> for ContentActiveCensor {
    fn from_py(py: Python, model: PyRef<'_, PyContentActiveCensor>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ContentID: model.ContentID.clone(),
            ActiveCensors: model.ActiveCensors.clone(),
            rb_activecensor_count: model.rb_activecensor_count,
        }
    }
}

#[pyclass(name = "ContentCue", unsendable, get_all, set_all)]
pub struct PyContentCue {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ContentID: Option<String>,
    pub Cues: Option<String>,
    pub rb_cue_count: Option<i32>,
}

impl IntoPy<PyContentCue> for ContentCue {
    fn into_py(self, py: Python) -> PyResult<PyContentCue> {
        let model = PyContentCue {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ContentID: self.ContentID,
            Cues: self.Cues,
            rb_cue_count: self.rb_cue_count,
        };
        Ok(model)
    }
}

impl FromPy<PyContentCue> for ContentCue {
    fn from_py(py: Python, model: PyRef<'_, PyContentCue>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ContentID: model.ContentID.clone(),
            Cues: model.Cues.clone(),
            rb_cue_count: model.rb_cue_count,
        }
    }
}

#[pyclass(name = "ContentFile", unsendable, get_all, set_all)]
pub struct PyContentFile {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ContentID: Option<String>,
    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_local_file_status: Option<i32>,
    pub rb_in_progress: Option<i32>,
    pub rb_process_type: Option<i32>,
    pub rb_temp_path: Option<String>,
    pub rb_priority: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

impl IntoPy<PyContentFile> for ContentFile {
    fn into_py(self, py: Python) -> PyResult<PyContentFile> {
        let model = PyContentFile {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ContentID: self.ContentID,
            Path: self.Path,
            Hash: self.Hash,
            Size: self.Size,
            rb_local_path: self.rb_local_path,
            rb_insync_hash: self.rb_insync_hash,
            rb_insync_local_usn: self.rb_insync_local_usn,
            rb_file_hash_dirty: self.rb_file_hash_dirty,
            rb_local_file_status: self.rb_local_file_status,
            rb_in_progress: self.rb_in_progress,
            rb_process_type: self.rb_process_type,
            rb_temp_path: self.rb_temp_path,
            rb_priority: self.rb_priority,
            rb_file_size_dirty: self.rb_file_size_dirty,
        };
        Ok(model)
    }
}

impl FromPy<PyContentFile> for ContentFile {
    fn from_py(py: Python, model: PyRef<'_, PyContentFile>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ContentID: model.ContentID.clone(),
            Path: model.Path.clone(),
            Hash: model.Hash.clone(),
            Size: model.Size.clone(),
            rb_local_path: model.rb_local_path.clone(),
            rb_insync_hash: model.rb_insync_hash.clone(),
            rb_insync_local_usn: model.rb_insync_local_usn,
            rb_file_hash_dirty: model.rb_file_hash_dirty,
            rb_local_file_status: model.rb_local_file_status,
            rb_in_progress: model.rb_in_progress,
            rb_process_type: model.rb_process_type,
            rb_temp_path: model.rb_temp_path.clone(),
            rb_priority: model.rb_priority,
            rb_file_size_dirty: model.rb_file_size_dirty,
        }
    }
}

#[pyclass(name = "DjmdActiveCensor", unsendable, get_all, set_all)]
pub struct PyDjmdActiveCensor {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ContentID: Option<String>,
    pub InMsec: Option<i32>,
    pub OutMsec: Option<i32>,
    pub Info: Option<i32>,
    pub ParameterList: Option<String>,
    pub ContentUUID: Option<String>,
}

impl IntoPy<PyDjmdActiveCensor> for DjmdActiveCensor {
    fn into_py(self, py: Python) -> PyResult<PyDjmdActiveCensor> {
        let model = PyDjmdActiveCensor {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ContentID: self.ContentID,
            InMsec: self.InMsec,
            OutMsec: self.OutMsec,
            Info: self.Info,
            ParameterList: self.ParameterList,
            ContentUUID: self.ContentUUID,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdActiveCensor> for DjmdActiveCensor {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdActiveCensor>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ContentID: model.ContentID.clone(),
            InMsec: model.InMsec,
            OutMsec: model.OutMsec,
            Info: model.Info,
            ParameterList: model.ParameterList.clone(),
            ContentUUID: model.ContentUUID.clone(),
        }
    }
}

#[pyclass(name = "DjmdAlbum", unsendable, get_all, set_all)]
pub struct PyDjmdAlbum {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Name: Option<String>,
    pub AlbumArtistID: Option<String>,
    pub ImagePath: Option<String>,
    pub Compilation: Option<i32>,
    pub SearchStr: Option<String>,
}

impl IntoPy<PyDjmdAlbum> for DjmdAlbum {
    fn into_py(self, py: Python) -> PyResult<PyDjmdAlbum> {
        let model = PyDjmdAlbum {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Name: self.Name,
            AlbumArtistID: self.AlbumArtistID,
            ImagePath: self.ImagePath,
            Compilation: self.Compilation,
            SearchStr: self.SearchStr,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdAlbum> for DjmdAlbum {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdAlbum>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Name: model.Name.clone(),
            AlbumArtistID: model.AlbumArtistID.clone(),
            ImagePath: model.ImagePath.clone(),
            Compilation: model.Compilation.clone(),
            SearchStr: model.SearchStr.clone(),
        }
    }
}

#[pyclass(name = "DjmdArtist", unsendable, get_all, set_all)]
pub struct PyDjmdArtist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Name: Option<String>,
    pub SearchStr: Option<String>,
}

impl IntoPy<PyDjmdArtist> for DjmdArtist {
    fn into_py(self, py: Python) -> PyResult<PyDjmdArtist> {
        let model = PyDjmdArtist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Name: self.Name,
            SearchStr: self.SearchStr,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdArtist> for DjmdArtist {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdArtist>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Name: model.Name.clone(),
            SearchStr: model.SearchStr.clone(),
        }
    }
}

#[pyclass(name = "DjmdCategory", unsendable, get_all, set_all)]
pub struct PyDjmdCategory {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub MenuItemID: Option<String>,
    pub Seq: Option<i32>,
    pub Disable: Option<i32>,
    pub InfoOrder: Option<i32>,
}

impl IntoPy<PyDjmdCategory> for DjmdCategory {
    fn into_py(self, py: Python) -> PyResult<PyDjmdCategory> {
        let model = PyDjmdCategory {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            MenuItemID: self.MenuItemID,
            Seq: self.Seq,
            Disable: self.Disable,
            InfoOrder: self.InfoOrder,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdCategory> for DjmdCategory {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdCategory>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            MenuItemID: model.MenuItemID.clone(),
            Seq: model.Seq,
            Disable: model.Disable,
            InfoOrder: model.InfoOrder,
        }
    }
}

#[pyclass(name = "DjmdColor", unsendable, get_all, set_all)]
pub struct PyDjmdColor {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ColorCode: Option<String>,
    pub SortKey: Option<i32>,
    pub Commnt: Option<String>,
}

impl IntoPy<PyDjmdColor> for DjmdColor {
    fn into_py(self, py: Python) -> PyResult<PyDjmdColor> {
        let model = PyDjmdColor {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ColorCode: self.ColorCode,
            SortKey: self.SortKey,
            Commnt: self.Commnt,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdColor> for DjmdColor {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdColor>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ColorCode: model.ColorCode.clone(),
            SortKey: model.SortKey,
            Commnt: model.Commnt.clone(),
        }
    }
}

#[pyclass(name = "DjmdContent", unsendable, get_all, set_all)]
pub struct PyDjmdContent {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub FolderPath: Option<String>,
    pub FileNameL: Option<String>,
    pub FileNameS: Option<String>,
    pub Title: Option<String>,
    pub ArtistID: Option<String>,
    pub AlbumID: Option<String>,
    pub GenreID: Option<String>,
    pub BPM: Option<i32>,
    pub Length: Option<i32>,
    pub TrackNo: Option<i32>,
    pub BitRate: Option<i32>,
    pub BitDepth: Option<i32>,
    pub Commnt: Option<String>,
    pub FileType: Option<i32>,
    pub Rating: Option<i32>,
    pub ReleaseYear: Option<i32>,
    pub RemixerID: Option<String>,
    pub LabelID: Option<String>,
    pub OrgArtistID: Option<String>,
    pub KeyID: Option<String>,
    pub StockDate: Option<String>,
    pub ColorID: Option<String>,
    pub DJPlayCount: Option<i32>,
    pub ImagePath: Option<String>,
    pub MasterDBID: Option<String>,
    pub MasterSongID: Option<String>,
    pub AnalysisDataPath: Option<String>,
    pub SearchStr: Option<String>,
    pub FileSize: Option<i32>,
    pub DiscNo: Option<i32>,
    pub ComposerID: Option<String>,
    pub Subtitle: Option<String>,
    pub SampleRate: Option<i32>,
    pub DisableQuantize: Option<i32>,
    pub Analysed: Option<i32>,
    pub ReleaseDate: Option<String>,
    pub DateCreated: Option<String>,
    pub ContentLink: Option<i32>,
    pub Tag: Option<String>,
    pub ModifiedByRBM: Option<String>,
    pub HotCueAutoLoad: Option<String>,
    pub DeliveryControl: Option<String>,
    pub DeliveryComment: Option<String>,
    pub CueUpdated: Option<String>,
    pub AnalysisUpdated: Option<String>,
    pub TrackInfoUpdated: Option<String>,
    pub Lyricist: Option<String>,
    pub ISRC: Option<String>,
    pub SamplerTrackInfo: Option<i32>,
    pub SamplerPlayOffset: Option<i32>,
    pub SamplerGain: Option<f64>,
    pub VideoAssociate: Option<String>,
    pub LyricStatus: Option<i32>,
    pub ServiceID: Option<i32>,
    pub OrgFolderPath: Option<String>,
    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub ExtInfo: Option<String>,
    pub rb_file_id: Option<String>,
    pub DeviceID: Option<String>,
    pub rb_LocalFolderPath: Option<String>,
    pub SrcID: Option<String>,
    pub SrcTitle: Option<String>,
    pub SrcArtistName: Option<String>,
    pub SrcAlbumName: Option<String>,
    pub SrcLength: Option<i32>,
}

impl IntoPy<PyDjmdContent> for DjmdContent {
    fn into_py(self, py: Python) -> PyResult<PyDjmdContent> {
        let model = PyDjmdContent {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            FolderPath: self.FolderPath,
            FileNameL: self.FileNameL,
            FileNameS: self.FileNameS,
            Title: self.Title,
            ArtistID: self.ArtistID,
            AlbumID: self.AlbumID,
            GenreID: self.GenreID,
            BPM: self.BPM,
            Length: self.Length,
            TrackNo: self.TrackNo,
            BitRate: self.BitRate,
            BitDepth: self.BitDepth,
            Commnt: self.Commnt,
            FileType: self.FileType,
            Rating: self.Rating,
            ReleaseYear: self.ReleaseYear,
            RemixerID: self.RemixerID,
            LabelID: self.LabelID,
            OrgArtistID: self.OrgArtistID,
            KeyID: self.KeyID,
            StockDate: self.StockDate,
            ColorID: self.ColorID,
            DJPlayCount: self.DJPlayCount,
            ImagePath: self.ImagePath,
            MasterDBID: self.MasterDBID,
            MasterSongID: self.MasterSongID,
            AnalysisDataPath: self.AnalysisDataPath,
            SearchStr: self.SearchStr,
            FileSize: self.FileSize,
            DiscNo: self.DiscNo,
            ComposerID: self.ComposerID,
            Subtitle: self.Subtitle,
            SampleRate: self.SampleRate,
            DisableQuantize: self.DisableQuantize,
            Analysed: self.Analysed,
            ReleaseDate: self.ReleaseDate,
            DateCreated: self.DateCreated,
            ContentLink: self.ContentLink,
            Tag: self.Tag,
            ModifiedByRBM: self.ModifiedByRBM,
            HotCueAutoLoad: self.HotCueAutoLoad,
            DeliveryControl: self.DeliveryControl,
            DeliveryComment: self.DeliveryComment,
            CueUpdated: self.CueUpdated,
            AnalysisUpdated: self.AnalysisUpdated,
            TrackInfoUpdated: self.TrackInfoUpdated,
            Lyricist: self.Lyricist,
            ISRC: self.ISRC,
            SamplerTrackInfo: self.SamplerTrackInfo,
            SamplerPlayOffset: self.SamplerPlayOffset,
            SamplerGain: self.SamplerGain,
            VideoAssociate: self.VideoAssociate,
            LyricStatus: self.LyricStatus,
            ServiceID: self.ServiceID,
            OrgFolderPath: self.OrgFolderPath,
            Reserved1: self.Reserved1,
            Reserved2: self.Reserved2,
            Reserved3: self.Reserved3,
            Reserved4: self.Reserved4,
            ExtInfo: self.ExtInfo,
            rb_file_id: self.rb_file_id,
            DeviceID: self.DeviceID,
            rb_LocalFolderPath: self.rb_LocalFolderPath,
            SrcID: self.SrcID,
            SrcTitle: self.SrcTitle,
            SrcArtistName: self.SrcArtistName,
            SrcAlbumName: self.SrcAlbumName,
            SrcLength: self.SrcLength,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdContent> for DjmdContent {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdContent>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            FolderPath: model.FolderPath.clone(),
            FileNameL: model.FileNameL.clone(),
            FileNameS: model.FileNameS.clone(),
            Title: model.Title.clone(),
            ArtistID: model.ArtistID.clone(),
            AlbumID: model.AlbumID.clone(),
            GenreID: model.GenreID.clone(),
            BPM: model.BPM,
            Length: model.Length,
            TrackNo: model.TrackNo,
            BitRate: model.BitRate,
            BitDepth: model.BitDepth,
            Commnt: model.Commnt.clone(),
            FileType: model.FileType,
            Rating: model.Rating,
            ReleaseYear: model.ReleaseYear,
            RemixerID: model.RemixerID.clone(),
            LabelID: model.LabelID.clone(),
            OrgArtistID: model.OrgArtistID.clone(),
            KeyID: model.KeyID.clone(),
            StockDate: model.StockDate.clone(),
            ColorID: model.ColorID.clone(),
            DJPlayCount: model.DJPlayCount,
            ImagePath: model.ImagePath.clone(),
            MasterDBID: model.MasterDBID.clone(),
            MasterSongID: model.MasterSongID.clone(),
            AnalysisDataPath: model.AnalysisDataPath.clone(),
            SearchStr: model.SearchStr.clone(),
            FileSize: model.FileSize,
            DiscNo: model.DiscNo,
            ComposerID: model.ComposerID.clone(),
            Subtitle: model.Subtitle.clone(),
            SampleRate: model.SampleRate,
            DisableQuantize: model.DisableQuantize,
            Analysed: model.Analysed,
            ReleaseDate: model.ReleaseDate.clone(),
            DateCreated: model.DateCreated.clone(),
            ContentLink: model.ContentLink,
            Tag: model.Tag.clone(),
            ModifiedByRBM: model.ModifiedByRBM.clone(),
            HotCueAutoLoad: model.HotCueAutoLoad.clone(),
            DeliveryControl: model.DeliveryControl.clone(),
            DeliveryComment: model.DeliveryComment.clone(),
            CueUpdated: model.CueUpdated.clone(),
            AnalysisUpdated: model.AnalysisUpdated.clone(),
            TrackInfoUpdated: model.TrackInfoUpdated.clone(),
            Lyricist: model.Lyricist.clone(),
            ISRC: model.ISRC.clone(),
            SamplerTrackInfo: model.SamplerTrackInfo,
            SamplerPlayOffset: model.SamplerPlayOffset,
            SamplerGain: model.SamplerGain,
            VideoAssociate: model.VideoAssociate.clone(),
            LyricStatus: model.LyricStatus,
            ServiceID: model.ServiceID,
            OrgFolderPath: model.OrgFolderPath.clone(),
            Reserved1: model.Reserved1.clone(),
            Reserved2: model.Reserved2.clone(),
            Reserved3: model.Reserved3.clone(),
            Reserved4: model.Reserved4.clone(),
            ExtInfo: model.ExtInfo.clone(),
            rb_file_id: model.rb_file_id.clone(),
            DeviceID: model.DeviceID.clone(),
            rb_LocalFolderPath: model.rb_LocalFolderPath.clone(),
            SrcID: model.SrcID.clone(),
            SrcTitle: model.SrcTitle.clone(),
            SrcArtistName: model.SrcArtistName.clone(),
            SrcAlbumName: model.SrcAlbumName.clone(),
            SrcLength: model.SrcLength,
        }
    }
}

#[pyclass(name = "DjmdCue", unsendable, get_all, set_all)]
pub struct PyDjmdCue {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ContentID: Option<String>,
    pub InMsec: Option<i32>,
    pub InFrame: Option<i32>,
    pub InMpegFrame: Option<i32>,
    pub InMpegAbs: Option<i32>,
    pub OutMsec: Option<i32>,
    pub OutFrame: Option<i32>,
    pub OutMpegFrame: Option<i32>,
    pub OutMpegAbs: Option<i32>,
    pub Kind: Option<i32>,
    pub Color: Option<i32>,
    pub ColorTableIndex: Option<i32>,
    pub ActiveLoop: Option<i32>,
    pub Comment: Option<String>,
    pub BeatLoopSize: Option<i32>,
    pub CueMicrosec: Option<i32>,
    pub InPointSeekInfo: Option<String>,
    pub OutPointSeekInfo: Option<String>,
    pub ContentUUID: Option<String>,
}

impl IntoPy<PyDjmdCue> for DjmdCue {
    fn into_py(self, py: Python) -> PyResult<PyDjmdCue> {
        let model = PyDjmdCue {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ContentID: self.ContentID,
            InMsec: self.InMsec,
            InFrame: self.InFrame,
            InMpegFrame: self.InMpegFrame,
            InMpegAbs: self.InMpegAbs,
            OutMsec: self.OutMsec,
            OutFrame: self.OutFrame,
            OutMpegFrame: self.OutMpegFrame,
            OutMpegAbs: self.OutMpegAbs,
            Kind: self.Kind,
            Color: self.Color,
            ColorTableIndex: self.ColorTableIndex,
            ActiveLoop: self.ActiveLoop,
            Comment: self.Comment,
            BeatLoopSize: self.BeatLoopSize,
            CueMicrosec: self.CueMicrosec,
            InPointSeekInfo: self.InPointSeekInfo,
            OutPointSeekInfo: self.OutPointSeekInfo,
            ContentUUID: self.ContentUUID,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdCue> for DjmdCue {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdCue>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ContentID: model.ContentID.clone(),
            InMsec: model.InMsec,
            InFrame: model.InFrame,
            InMpegFrame: model.InMpegFrame,
            InMpegAbs: model.InMpegAbs,
            OutMsec: model.OutMsec,
            OutFrame: model.OutFrame,
            OutMpegFrame: model.OutMpegFrame,
            OutMpegAbs: model.OutMpegAbs,
            Kind: model.Kind,
            Color: model.Color,
            ColorTableIndex: model.ColorTableIndex,
            ActiveLoop: model.ActiveLoop,
            Comment: model.Comment.clone(),
            BeatLoopSize: model.BeatLoopSize,
            CueMicrosec: model.CueMicrosec,
            InPointSeekInfo: model.InPointSeekInfo.clone(),
            OutPointSeekInfo: model.OutPointSeekInfo.clone(),
            ContentUUID: model.ContentUUID.clone(),
        }
    }
}

#[pyclass(name = "DjmdDevice", unsendable, get_all, set_all)]
pub struct PyDjmdDevice {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub MasterDBID: Option<String>,
    pub Name: Option<String>,
}

impl IntoPy<PyDjmdDevice> for DjmdDevice {
    fn into_py(self, py: Python) -> PyResult<PyDjmdDevice> {
        let model = PyDjmdDevice {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            MasterDBID: self.MasterDBID,
            Name: self.Name,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdDevice> for DjmdDevice {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdDevice>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            MasterDBID: model.MasterDBID.clone(),
            Name: model.Name.clone(),
        }
    }
}

#[pyclass(name = "DjmdGenre", unsendable, get_all, set_all)]
pub struct PyDjmdGenre {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Name: Option<String>,
}

impl IntoPy<PyDjmdGenre> for DjmdGenre {
    fn into_py(self, py: Python) -> PyResult<PyDjmdGenre> {
        let model = PyDjmdGenre {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Name: self.Name,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdGenre> for DjmdGenre {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdGenre>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Name: model.Name.clone(),
        }
    }
}

#[pyclass(name = "DjmdHistory", unsendable, get_all, set_all)]
pub struct PyDjmdHistory {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub DateCreated: Option<String>,
}

impl IntoPy<PyDjmdHistory> for DjmdHistory {
    fn into_py(self, py: Python) -> PyResult<PyDjmdHistory> {
        let model = PyDjmdHistory {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Seq: self.Seq,
            Name: self.Name,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
            DateCreated: self.DateCreated,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdHistory> for DjmdHistory {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdHistory>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Seq: model.Seq,
            Name: model.Name.clone(),
            Attribute: model.Attribute,
            ParentID: model.ParentID.clone(),
            DateCreated: model.DateCreated.clone(),
        }
    }
}

#[pyclass(name = "DjmdSongHistory", unsendable, get_all, set_all)]
pub struct PyDjmdSongHistory {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub HistoryID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoPy<PyDjmdSongHistory> for DjmdSongHistory {
    fn into_py(self, py: Python) -> PyResult<PyDjmdSongHistory> {
        let model = PyDjmdSongHistory {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            HistoryID: self.HistoryID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdSongHistory> for DjmdSongHistory {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdSongHistory>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            HistoryID: model.HistoryID.clone(),
            ContentID: model.ContentID.clone(),
            TrackNo: model.TrackNo,
        }
    }
}

#[pyclass(name = "DjmdHotCueBanklist", unsendable, get_all, set_all)]
pub struct PyDjmdHotCueBanklist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

impl IntoPy<PyDjmdHotCueBanklist> for DjmdHotCueBanklist {
    fn into_py(self, py: Python) -> PyResult<PyDjmdHotCueBanklist> {
        let model = PyDjmdHotCueBanklist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Seq: self.Seq,
            Name: self.Name,
            ImagePath: self.ImagePath,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdHotCueBanklist> for DjmdHotCueBanklist {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdHotCueBanklist>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Seq: model.Seq,
            Name: model.Name.clone(),
            ImagePath: model.ImagePath.clone(),
            Attribute: model.Attribute,
            ParentID: model.ParentID.clone(),
        }
    }
}

#[pyclass(name = "DjmdSongHotCueBanklist", unsendable, get_all, set_all)]
pub struct PyDjmdSongHotCueBanklist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
    pub CueID: Option<String>,
    pub InMsec: Option<i32>,
    pub InFrame: Option<i32>,
    pub InMpegFrame: Option<i32>,
    pub InMpegAbs: Option<i32>,
    pub OutMsec: Option<i32>,
    pub OutFrame: Option<i32>,
    pub OutMpegFrame: Option<i32>,
    pub OutMpegAbs: Option<i32>,
    pub Color: Option<i32>,
    pub ColorTableIndex: Option<i32>,
    pub ActiveLoop: Option<i32>,
    pub Comment: Option<String>,
    pub BeatLoopSize: Option<i32>,
    pub CueMicrosec: Option<i32>,
    pub InPointSeekInfo: Option<String>,
    pub OutPointSeekInfo: Option<String>,
    pub HotCueBanklistUUID: Option<String>,
}

impl IntoPy<PyDjmdSongHotCueBanklist> for DjmdSongHotCueBanklist {
    fn into_py(self, py: Python) -> PyResult<PyDjmdSongHotCueBanklist> {
        let model = PyDjmdSongHotCueBanklist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
            CueID: self.CueID,
            InMsec: self.InMsec,
            InFrame: self.InFrame,
            InMpegFrame: self.InMpegFrame,
            InMpegAbs: self.InMpegAbs,
            OutMsec: self.OutMsec,
            OutFrame: self.OutFrame,
            OutMpegFrame: self.OutMpegFrame,
            OutMpegAbs: self.OutMpegAbs,
            Color: self.Color,
            ColorTableIndex: self.ColorTableIndex,
            ActiveLoop: self.ActiveLoop,
            Comment: self.Comment,
            BeatLoopSize: self.BeatLoopSize,
            CueMicrosec: self.CueMicrosec,
            InPointSeekInfo: self.InPointSeekInfo,
            OutPointSeekInfo: self.OutPointSeekInfo,
            HotCueBanklistUUID: self.HotCueBanklistUUID,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdSongHotCueBanklist> for DjmdSongHotCueBanklist {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdSongHotCueBanklist>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ContentID: model.ContentID.clone(),
            TrackNo: model.TrackNo,
            CueID: model.CueID.clone(),
            InMsec: model.InMsec,
            InFrame: model.InFrame,
            InMpegFrame: model.InMpegFrame,
            InMpegAbs: model.InMpegAbs,
            OutMsec: model.OutMsec,
            OutFrame: model.OutFrame,
            OutMpegFrame: model.OutMpegFrame,
            OutMpegAbs: model.OutMpegAbs,
            Color: model.Color,
            ColorTableIndex: model.ColorTableIndex,
            ActiveLoop: model.ActiveLoop,
            Comment: model.Comment.clone(),
            BeatLoopSize: model.BeatLoopSize,
            CueMicrosec: model.CueMicrosec,
            InPointSeekInfo: model.InPointSeekInfo.clone(),
            OutPointSeekInfo: model.OutPointSeekInfo.clone(),
            HotCueBanklistUUID: model.HotCueBanklistUUID.clone(),
        }
    }
}

#[pyclass(name = "HotCueBanklistCue", unsendable, get_all, set_all)]
pub struct PyHotCueBanklistCue {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub HotCueBanklistID: Option<String>,
    pub Cues: Option<String>,
    pub rb_cue_count: Option<i32>,
}

impl IntoPy<PyHotCueBanklistCue> for HotCueBanklistCue {
    fn into_py(self, py: Python) -> PyResult<PyHotCueBanklistCue> {
        let model = PyHotCueBanklistCue {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            HotCueBanklistID: self.HotCueBanklistID,
            Cues: self.Cues,
            rb_cue_count: self.rb_cue_count,
        };
        Ok(model)
    }
}

impl FromPy<PyHotCueBanklistCue> for HotCueBanklistCue {
    fn from_py(py: Python, model: PyRef<'_, PyHotCueBanklistCue>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            HotCueBanklistID: model.HotCueBanklistID.clone(),
            Cues: model.Cues.clone(),
            rb_cue_count: model.rb_cue_count,
        }
    }
}

#[pyclass(name = "DjmdKey", unsendable, get_all, set_all)]
pub struct PyDjmdKey {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ScaleName: Option<String>,
    pub Seq: Option<i32>,
}

impl IntoPy<PyDjmdKey> for DjmdKey {
    fn into_py(self, py: Python) -> PyResult<PyDjmdKey> {
        let model = PyDjmdKey {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ScaleName: self.ScaleName,
            Seq: self.Seq,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdKey> for DjmdKey {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdKey>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ScaleName: model.ScaleName.clone(),
            Seq: model.Seq,
        }
    }
}

#[pyclass(name = "DjmdLabel", unsendable, get_all, set_all)]
pub struct PyDjmdLabel {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Name: Option<String>,
}

impl IntoPy<PyDjmdLabel> for DjmdLabel {
    fn into_py(self, py: Python) -> PyResult<PyDjmdLabel> {
        let model = PyDjmdLabel {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Name: self.Name,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdLabel> for DjmdLabel {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdLabel>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Name: model.Name.clone(),
        }
    }
}

#[pyclass(name = "DjmdMenuItems", unsendable, get_all, set_all)]
pub struct PyDjmdMenuItems {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Class: Option<i32>,
    pub Name: Option<String>,
}

impl IntoPy<PyDjmdMenuItems> for DjmdMenuItems {
    fn into_py(self, py: Python) -> PyResult<PyDjmdMenuItems> {
        let model = PyDjmdMenuItems {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Class: self.Class,
            Name: self.Name,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdMenuItems> for DjmdMenuItems {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdMenuItems>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Class: model.Class,
            Name: model.Name.clone(),
        }
    }
}

#[pyclass(name = "DjmdMixerParam", unsendable, get_all, set_all)]
pub struct PyDjmdMixerParam {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ContentID: Option<String>,
    pub GainHigh: Option<i32>,
    pub GainLow: Option<i32>,
    pub PeakHigh: Option<i32>,
    pub PeakLow: Option<i32>,
}

impl IntoPy<PyDjmdMixerParam> for DjmdMixerParam {
    fn into_py(self, py: Python) -> PyResult<PyDjmdMixerParam> {
        let model = PyDjmdMixerParam {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ContentID: self.ContentID,
            GainHigh: self.GainHigh,
            GainLow: self.GainLow,
            PeakHigh: self.PeakHigh,
            PeakLow: self.PeakLow,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdMixerParam> for DjmdMixerParam {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdMixerParam>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ContentID: model.ContentID.clone(),
            GainHigh: model.GainHigh,
            GainLow: model.GainLow,
            PeakHigh: model.PeakHigh,
            PeakLow: model.PeakLow,
        }
    }
}

#[pyclass(name = "DjmdMyTag", unsendable, get_all, set_all)]
pub struct PyDjmdMyTag {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

impl IntoPy<PyDjmdMyTag> for DjmdMyTag {
    fn into_py(self, py: Python) -> PyResult<PyDjmdMyTag> {
        let model = PyDjmdMyTag {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Seq: self.Seq,
            Name: self.Name,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdMyTag> for DjmdMyTag {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdMyTag>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Seq: model.Seq,
            Name: model.Name.clone(),
            Attribute: model.Attribute,
            ParentID: model.ParentID.clone(),
        }
    }
}

#[pyclass(name = "DjmdSongMyTag", unsendable, get_all, set_all)]
pub struct PyDjmdSongMyTag {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub MyTagID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoPy<PyDjmdSongMyTag> for DjmdSongMyTag {
    fn into_py(self, py: Python) -> PyResult<PyDjmdSongMyTag> {
        let model = PyDjmdSongMyTag {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            MyTagID: self.MyTagID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdSongMyTag> for DjmdSongMyTag {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdSongMyTag>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            MyTagID: model.MyTagID.clone(),
            ContentID: model.ContentID.clone(),
            TrackNo: model.TrackNo,
        }
    }
}

#[pyclass(name = "DjmdPlaylist", unsendable, get_all, set_all)]
pub struct PyDjmdPlaylist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub SmartList: Option<String>,
}

impl IntoPy<PyDjmdPlaylist> for DjmdPlaylist {
    fn into_py(self, py: Python) -> PyResult<PyDjmdPlaylist> {
        let model = PyDjmdPlaylist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Seq: self.Seq,
            Name: self.Name,
            ImagePath: self.ImagePath,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
            SmartList: self.SmartList,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdPlaylist> for DjmdPlaylist {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdPlaylist>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Seq: model.Seq,
            Name: model.Name.clone(),
            ImagePath: model.ImagePath.clone(),
            Attribute: model.Attribute,
            ParentID: model.ParentID.clone(),
            SmartList: model.SmartList.clone(),
        }
    }
}

#[pyclass(name = "PlaylistTreeItem", unsendable, get_all, set_all)]
pub struct PyPlaylistTreeItem {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub SmartList: Option<String>,

    pub Children: Py<PyList>,
}

impl IntoPy<PyPlaylistTreeItem> for PlaylistTreeItem {
    fn into_py(self, py: Python) -> PyResult<PyPlaylistTreeItem> {
        let py_list = PyList::empty(py);
        for child in self.Children {
            let py_child = child.borrow().clone().into_py(py)?;
            let _ = py_list.append(py_child);
        }

        let model = PyPlaylistTreeItem {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Seq: self.Seq,
            Name: self.Name,
            ImagePath: self.ImagePath,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
            SmartList: self.SmartList,
            Children: py_list.into(),
        };

        Ok(model)
    }
}

#[pyclass(name = "DjmdSongPlaylist", unsendable, get_all, set_all)]
pub struct PyDjmdSongPlaylist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub PlaylistID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoPy<PyDjmdSongPlaylist> for DjmdSongPlaylist {
    fn into_py(self, py: Python) -> PyResult<PyDjmdSongPlaylist> {
        let model = PyDjmdSongPlaylist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            PlaylistID: self.PlaylistID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdSongPlaylist> for DjmdSongPlaylist {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdSongPlaylist>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            PlaylistID: model.PlaylistID.clone(),
            ContentID: model.ContentID.clone(),
            TrackNo: model.TrackNo,
        }
    }
}

#[pyclass(name = "DjmdProperty", unsendable, get_all, set_all)]
pub struct PyDjmdProperty {
    pub DBID: String,
    pub DBVersion: Option<String>,
    pub BaseDBDrive: Option<String>,
    pub CurrentDBDrive: Option<String>,
    pub DeviceID: Option<String>,
    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub Reserved5: Option<String>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
}

impl IntoPy<PyDjmdProperty> for DjmdProperty {
    fn into_py(self, py: Python) -> PyResult<PyDjmdProperty> {
        let model = PyDjmdProperty {
            DBID: self.DBID,
            DBVersion: self.DBVersion,
            BaseDBDrive: self.BaseDBDrive,
            CurrentDBDrive: self.CurrentDBDrive,
            DeviceID: self.DeviceID,
            Reserved1: self.Reserved1,
            Reserved2: self.Reserved2,
            Reserved3: self.Reserved3,
            Reserved4: self.Reserved4,
            Reserved5: self.Reserved5,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdProperty> for DjmdProperty {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdProperty>) -> Self {
        Self {
            DBID: model.DBID.clone(),
            DBVersion: model.DBVersion.clone(),
            BaseDBDrive: model.BaseDBDrive.clone(),
            CurrentDBDrive: model.CurrentDBDrive.clone(),
            DeviceID: model.DeviceID.clone(),
            Reserved1: model.Reserved1.clone(),
            Reserved2: model.Reserved2.clone(),
            Reserved3: model.Reserved3.clone(),
            Reserved4: model.Reserved4.clone(),
            Reserved5: model.Reserved5.clone(),
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
        }
    }
}

#[pyclass(name = "DjmdCloudProperty", unsendable, get_all, set_all)]
pub struct PyDjmdCloudProperty {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub Reserved5: Option<String>,
}

impl IntoPy<PyDjmdCloudProperty> for DjmdCloudProperty {
    fn into_py(self, py: Python) -> PyResult<PyDjmdCloudProperty> {
        let model = PyDjmdCloudProperty {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Reserved1: self.Reserved1,
            Reserved2: self.Reserved2,
            Reserved3: self.Reserved3,
            Reserved4: self.Reserved4,
            Reserved5: self.Reserved5,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdCloudProperty> for DjmdCloudProperty {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdCloudProperty>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Reserved1: model.Reserved1.clone(),
            Reserved2: model.Reserved2.clone(),
            Reserved3: model.Reserved3.clone(),
            Reserved4: model.Reserved4.clone(),
            Reserved5: model.Reserved5.clone(),
        }
    }
}

#[pyclass(name = "DjmdRecommendLike", unsendable, get_all, set_all)]
pub struct PyDjmdRecommendLike {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ContentID1: Option<String>,
    pub ContentID2: Option<String>,
    pub LikeRate: Option<i32>,
    pub DataCreatedH: Option<i32>,
    pub DataCreatedL: Option<i32>,
}

impl IntoPy<PyDjmdRecommendLike> for DjmdRecommendLike {
    fn into_py(self, py: Python) -> PyResult<PyDjmdRecommendLike> {
        let model = PyDjmdRecommendLike {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ContentID1: self.ContentID1,
            ContentID2: self.ContentID2,
            LikeRate: self.LikeRate,
            DataCreatedH: self.DataCreatedH,
            DataCreatedL: self.DataCreatedL,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdRecommendLike> for DjmdRecommendLike {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdRecommendLike>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ContentID1: model.ContentID1.clone(),
            ContentID2: model.ContentID2.clone(),
            LikeRate: model.LikeRate,
            DataCreatedH: model.DataCreatedH,
            DataCreatedL: model.DataCreatedL,
        }
    }
}

#[pyclass(name = "DjmdRelatedTracks", unsendable, get_all, set_all)]
pub struct PyDjmdRelatedTracks {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub Criteria: Option<String>,
}

impl IntoPy<PyDjmdRelatedTracks> for DjmdRelatedTracks {
    fn into_py(self, py: Python) -> PyResult<PyDjmdRelatedTracks> {
        let model = PyDjmdRelatedTracks {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Seq: self.Seq,
            Name: self.Name,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
            Criteria: self.Criteria,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdRelatedTracks> for DjmdRelatedTracks {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdRelatedTracks>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Seq: model.Seq,
            Name: model.Name.clone(),
            Attribute: model.Attribute,
            ParentID: model.ParentID.clone(),
            Criteria: model.Criteria.clone(),
        }
    }
}

#[pyclass(name = "DjmdSongRelatedTracks", unsendable, get_all, set_all)]
pub struct PyDjmdSongRelatedTracks {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub RelatedTracksID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoPy<PyDjmdSongRelatedTracks> for DjmdSongRelatedTracks {
    fn into_py(self, py: Python) -> PyResult<PyDjmdSongRelatedTracks> {
        let model = PyDjmdSongRelatedTracks {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            RelatedTracksID: self.RelatedTracksID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdSongRelatedTracks> for DjmdSongRelatedTracks {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdSongRelatedTracks>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            RelatedTracksID: model.RelatedTracksID.clone(),
            ContentID: model.ContentID.clone(),
            TrackNo: model.TrackNo,
        }
    }
}

#[pyclass(name = "DjmdSampler", unsendable, get_all, set_all)]
pub struct PyDjmdSampler {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

impl IntoPy<PyDjmdSampler> for DjmdSampler {
    fn into_py(self, py: Python) -> PyResult<PyDjmdSampler> {
        let model = PyDjmdSampler {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Seq: self.Seq,
            Name: self.Name,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdSampler> for DjmdSampler {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdSampler>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Seq: model.Seq,
            Name: model.Name.clone(),
            Attribute: model.Attribute,
            ParentID: model.ParentID.clone(),
        }
    }
}

#[pyclass(name = "DjmdSongSampler", unsendable, get_all, set_all)]
pub struct PyDjmdSongSampler {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub SamplerID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoPy<PyDjmdSongSampler> for DjmdSongSampler {
    fn into_py(self, py: Python) -> PyResult<PyDjmdSongSampler> {
        let model = PyDjmdSongSampler {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            SamplerID: self.SamplerID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdSongSampler> for DjmdSongSampler {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdSongSampler>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            SamplerID: model.SamplerID.clone(),
            ContentID: model.ContentID.clone(),
            TrackNo: model.TrackNo,
        }
    }
}

#[pyclass(name = "DjmdSongTagList", unsendable, get_all, set_all)]
pub struct PyDjmdSongTagList {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoPy<PyDjmdSongTagList> for DjmdSongTagList {
    fn into_py(self, py: Python) -> PyResult<PyDjmdSongTagList> {
        let model = PyDjmdSongTagList {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdSongTagList> for DjmdSongTagList {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdSongTagList>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            ContentID: model.ContentID.clone(),
            TrackNo: model.TrackNo,
        }
    }
}

#[pyclass(name = "DjmdSort", unsendable, get_all, set_all)]
pub struct PyDjmdSort {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub MenuItemID: Option<String>,
    pub Seq: Option<i32>,
    pub Disable: Option<i32>,
}

impl IntoPy<PyDjmdSort> for DjmdSort {
    fn into_py(self, py: Python) -> PyResult<PyDjmdSort> {
        let model = PyDjmdSort {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            MenuItemID: self.MenuItemID,
            Seq: self.Seq,
            Disable: self.Disable,
        };
        Ok(model)
    }
}

impl FromPy<PyDjmdSort> for DjmdSort {
    fn from_py(py: Python, model: PyRef<'_, PyDjmdSort>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            MenuItemID: model.MenuItemID.clone(),
            Seq: model.Seq,
            Disable: model.Disable,
        }
    }
}

#[pyclass(name = "ImageFile", unsendable, get_all, set_all)]
pub struct PyImageFile {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub TableName: Option<String>,
    pub TargetUUID: Option<String>,
    pub TargetID: Option<String>,
    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_local_file_status: Option<i32>,
    pub rb_in_progress: Option<i32>,
    pub rb_process_type: Option<i32>,
    pub rb_temp_path: Option<String>,
    pub rb_priority: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

impl IntoPy<PyImageFile> for ImageFile {
    fn into_py(self, py: Python) -> PyResult<PyImageFile> {
        let model = PyImageFile {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            TableName: self.TableName,
            TargetUUID: self.TargetUUID,
            TargetID: self.TargetID,
            Path: self.Path,
            Hash: self.Hash,
            Size: self.Size,
            rb_local_path: self.rb_local_path,
            rb_insync_hash: self.rb_insync_hash,
            rb_insync_local_usn: self.rb_insync_local_usn,
            rb_file_hash_dirty: self.rb_file_hash_dirty,
            rb_local_file_status: self.rb_local_file_status,
            rb_in_progress: self.rb_in_progress,
            rb_process_type: self.rb_process_type,
            rb_temp_path: self.rb_temp_path,
            rb_priority: self.rb_priority,
            rb_file_size_dirty: self.rb_file_size_dirty,
        };
        Ok(model)
    }
}

impl FromPy<PyImageFile> for ImageFile {
    fn from_py(py: Python, model: PyRef<'_, PyImageFile>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            TableName: model.TableName.clone(),
            TargetUUID: model.TargetUUID.clone(),
            TargetID: model.TargetID.clone(),
            Path: model.Path.clone(),
            Hash: model.Hash.clone(),
            Size: model.Size,
            rb_local_path: model.rb_local_path.clone(),
            rb_insync_hash: model.rb_insync_hash.clone(),
            rb_insync_local_usn: model.rb_insync_local_usn,
            rb_file_hash_dirty: model.rb_file_hash_dirty,
            rb_local_file_status: model.rb_local_file_status,
            rb_in_progress: model.rb_in_progress,
            rb_process_type: model.rb_process_type,
            rb_temp_path: model.rb_temp_path.clone(),
            rb_priority: model.rb_priority,
            rb_file_size_dirty: model.rb_file_size_dirty,
        }
    }
}

#[pyclass(name = "SettingFile", unsendable, get_all, set_all)]
pub struct PySettingFile {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

impl IntoPy<PySettingFile> for SettingFile {
    fn into_py(self, py: Python) -> PyResult<PySettingFile> {
        let model = PySettingFile {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            Path: self.Path,
            Hash: self.Hash,
            Size: self.Size,
            rb_local_path: self.rb_local_path,
            rb_insync_hash: self.rb_insync_hash,
            rb_insync_local_usn: self.rb_insync_local_usn,
            rb_file_hash_dirty: self.rb_file_hash_dirty,
            rb_file_size_dirty: self.rb_file_size_dirty,
        };
        Ok(model)
    }
}

impl FromPy<PySettingFile> for SettingFile {
    fn from_py(py: Python, model: PyRef<'_, PySettingFile>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            Path: model.Path.clone(),
            Hash: model.Hash.clone(),
            Size: model.Size,
            rb_local_path: model.rb_local_path.clone(),
            rb_insync_hash: model.rb_insync_hash.clone(),
            rb_insync_local_usn: model.rb_insync_local_usn,
            rb_file_hash_dirty: model.rb_file_hash_dirty,
            rb_file_size_dirty: model.rb_file_size_dirty,
        }
    }
}

#[pyclass(name = "UuidIDMap", unsendable, get_all, set_all)]
pub struct PyUuidIDMap {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Py<PyDateTime>,
    pub updated_at: Py<PyDateTime>,
    pub TableName: Option<String>,
    pub TargetUUID: Option<String>,
    pub CurrentID: Option<String>,
}

impl IntoPy<PyUuidIDMap> for UuidIDMap {
    fn into_py(self, py: Python) -> PyResult<PyUuidIDMap> {
        let model = PyUuidIDMap {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: self.created_at.into_pyobject(py)?.into(),
            updated_at: self.updated_at.into_pyobject(py)?.into(),
            TableName: self.TableName,
            TargetUUID: self.TargetUUID,
            CurrentID: self.CurrentID,
        };
        Ok(model)
    }
}

impl FromPy<PyUuidIDMap> for UuidIDMap {
    fn from_py(py: Python, model: PyRef<'_, PyUuidIDMap>) -> Self {
        Self {
            ID: model.ID.clone(),
            UUID: model.UUID.clone(),
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at.extract::<DateTime<Utc>>(py).unwrap(),
            updated_at: model.updated_at.extract::<DateTime<Utc>>(py).unwrap(),
            TableName: model.TableName.clone(),
            TargetUUID: model.TargetUUID.clone(),
            CurrentID: model.CurrentID.clone(),
        }
    }
}
