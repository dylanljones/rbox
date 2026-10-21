// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Read-only library and program discovery bindings.

use pyo3::prelude::*;
use pyo3::types::PyType;
use rbox::core::locate::{self, LocationSource};
use std::path::PathBuf;

fn discovery_error(error: rbox::core::error::Error) -> PyErr {
    crate::errors::Error::new_err(error.to_string())
}

/// Selected library and its availability at discovery time.
#[derive(Clone)]
#[pyclass(from_py_object, name = "LibraryLocation", get_all)]
pub struct PyLibraryLocation {
    pub db_path: PathBuf,
    pub analysis_root: PathBuf,
    pub settings_root: PathBuf,
    pub source: LocationSource,
    pub available: bool,
}

impl From<locate::LibraryLocation> for PyLibraryLocation {
    fn from(value: locate::LibraryLocation) -> Self {
        Self {
            db_path: value.db_path,
            analysis_root: value.analysis_root,
            settings_root: value.settings_root,
            source: value.source,
            available: value.available,
        }
    }
}
impl From<PyLibraryLocation> for locate::LibraryLocation {
    fn from(value: PyLibraryLocation) -> Self {
        Self {
            db_path: value.db_path,
            analysis_root: value.analysis_root,
            settings_root: value.settings_root,
            source: value.source,
            available: value.available,
        }
    }
}

/// A discovered executable and version parsed from its directory name.
#[derive(Clone)]
#[pyclass(from_py_object, name = "ProgramLocation", get_all)]
pub struct PyProgramLocation {
    pub executable: PathBuf,
    pub version: Vec<u32>,
}
impl From<locate::ProgramLocation> for PyProgramLocation {
    fn from(value: locate::ProgramLocation) -> Self {
        Self {
            executable: value.executable,
            version: value.version,
        }
    }
}

/// Custom discovery inputs. Construction does not access the filesystem.
#[pyclass(name = "LookupPaths")]
pub struct PyLookupPaths {
    inner: locate::LookupPaths,
}
#[pymethods]
impl PyLookupPaths {
    #[new]
    fn new(
        settings_file: PathBuf,
        options_file: PathBuf,
        default_library_dirs: Vec<PathBuf>,
    ) -> Self {
        Self {
            inner: locate::LookupPaths {
                settings_file,
                options_file,
                default_library_dirs,
            },
        }
    }
    #[classmethod]
    fn under(_cls: &Bound<'_, PyType>, base: PathBuf) -> Self {
        Self {
            inner: locate::LookupPaths::under(base),
        }
    }
    #[classmethod]
    fn installed(_cls: &Bound<'_, PyType>) -> PyResult<Self> {
        Ok(Self {
            inner: locate::LookupPaths::installed().map_err(discovery_error)?,
        })
    }
    fn locate(&self) -> PyResult<PyLibraryLocation> {
        self.inner.locate().map(Into::into).map_err(discovery_error)
    }
    #[getter]
    fn settings_file(&self) -> PathBuf {
        self.inner.settings_file.clone()
    }
    #[setter]
    fn set_settings_file(&mut self, value: PathBuf) {
        self.inner.settings_file = value;
    }
    #[getter]
    fn options_file(&self) -> PathBuf {
        self.inner.options_file.clone()
    }
    #[setter]
    fn set_options_file(&mut self, value: PathBuf) {
        self.inner.options_file = value;
    }
    #[getter]
    fn default_library_dirs(&self) -> Vec<PathBuf> {
        self.inner.default_library_dirs.clone()
    }
    #[setter]
    fn set_default_library_dirs(&mut self, value: Vec<PathBuf>) {
        self.inner.default_library_dirs = value;
    }
}

/// Locate the selected library, including unavailable configured paths.
#[pyfunction]
pub fn locate_library() -> PyResult<PyLibraryLocation> {
    locate::locate_library()
        .map(Into::into)
        .map_err(discovery_error)
}

/// Discover installed Rekordbox programs, newest version first.
#[pyfunction]
pub fn find_programs() -> PyResult<Vec<PyProgramLocation>> {
    locate::find_programs()
        .map(|values| values.into_iter().map(Into::into).collect())
        .map_err(discovery_error)
}

/// Discover Rekordbox programs beneath explicit roots without consulting the environment.
#[pyfunction]
pub fn find_programs_in(roots: Vec<PathBuf>) -> PyResult<Vec<PyProgramLocation>> {
    locate::find_programs_in(roots)
        .map(|values| values.into_iter().map(Into::into).collect())
        .map_err(discovery_error)
}

/// Return the platform application-data base directory without checking existence.
#[pyfunction]
pub fn app_dir() -> PyResult<PathBuf> {
    locate::app_dir().map_err(discovery_error)
}

/// Return an existing database in the platform default locations.
#[pyfunction]
pub fn default_master_db_path() -> PyResult<PathBuf> {
    locate::default_master_db_path().map_err(discovery_error)
}

/// Return the selected database path, requiring an existing file.
#[pyfunction]
pub fn master_db_path() -> PyResult<PathBuf> {
    locate::master_db_path().map_err(discovery_error)
}
