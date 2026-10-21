// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Runtime iterators used by the Python mapping derive macro.
//!
//! Available with the pyo3 feature. Generated implementations reference these
//! types directly, so model modules do not need to import them.

use pyo3::prelude::*;

/// String iterator used for Python bindings.
#[pyclass]
pub struct PyStrIter {
    pub inner: std::vec::IntoIter<String>,
}

impl PyStrIter {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            inner: items.into_iter(),
        }
    }
}

#[pymethods]
impl PyStrIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
        slf.inner.next()
    }
}

/// Object iterator used for Python bindings.
#[pyclass]
pub struct PyObjectIter {
    pub inner: std::vec::IntoIter<Py<PyAny>>,
}

impl PyObjectIter {
    pub fn new(items: Vec<Py<PyAny>>) -> Self {
        Self {
            inner: items.into_iter(),
        }
    }
}

#[pymethods]
impl PyObjectIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        slf.inner.next()
    }
}

/// Items iterator used for Python bindings.
#[pyclass]
pub struct PyItemsIter {
    pub inner: std::vec::IntoIter<(String, Py<PyAny>)>,
}

impl PyItemsIter {
    pub fn new(items: Vec<(String, Py<PyAny>)>) -> Self {
        Self {
            inner: items.into_iter(),
        }
    }
}

#[pymethods]
impl PyItemsIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(String, Py<PyAny>)> {
        slf.inner.next()
    }
}
