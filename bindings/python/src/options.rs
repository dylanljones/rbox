// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use pyo3::prelude::*;
use pyo3::types::PyType;
use rbox::RekordboxOptions;

use super::errors::OptionsError;

#[inline]
fn db_error<E: ToString>(e: E) -> PyErr {
    PyErr::new::<OptionsError, _>(e.to_string())
}

#[derive(Clone)]
#[pyclass(name = "RekordboxOptions", unsendable)]
pub struct PyRekordboxOptions {
    pub(crate) inner: RekordboxOptions,
}

#[pymethods]
impl PyRekordboxOptions {
    /// Creates a new `RekordboxOptions` instance.
    #[new]
    pub fn new(db_path: &str, analysis_root: &str, settings_root: &str) -> PyResult<Self> {
        let inner =
            RekordboxOptions::new(db_path, analysis_root, settings_root).map_err(db_error)?;
        Ok(PyRekordboxOptions { inner })
    }

    /// Loads Rekordbox options from a JSON file.
    #[classmethod]
    pub fn load(_cls: &Bound<'_, PyType>, path: &str) -> PyResult<Self> {
        let inner = RekordboxOptions::load(path).map_err(db_error)?;
        Ok(PyRekordboxOptions { inner })
    }

    /// Opens the Rekordbox options file from the default location.
    #[classmethod]
    pub fn open(_cls: &Bound<'_, PyType>) -> PyResult<Self> {
        let inner = RekordboxOptions::open().map_err(db_error)?;
        Ok(PyRekordboxOptions { inner })
    }

    /// Returns the path to the Rekordbox database.
    #[getter]
    pub fn db_path(&self) -> String {
        self.inner.db_path.to_string_lossy().to_string()
    }

    /// Returns the path to the analysis data directory.
    #[getter]
    pub fn analysis_root(&self) -> String {
        self.inner.analysis_root.to_string_lossy().to_string()
    }

    /// Returns the path to the settings directory.
    #[getter]
    pub fn settings_root(&self) -> String {
        self.inner.analysis_root.to_string_lossy().to_string()
    }
}
