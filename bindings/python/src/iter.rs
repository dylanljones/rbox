// Author: Dylan Jones
// Date:   2025-07-06

use pyo3::prelude::*;

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

#[pyclass]
pub struct PyObjectIter {
    pub inner: std::vec::IntoIter<PyObject>,
}

impl PyObjectIter {
    pub fn new(items: Vec<PyObject>) -> Self {
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

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
        slf.inner.next()
    }
}

#[pyclass]
pub struct PyItemsIter {
    pub inner: std::vec::IntoIter<(String, PyObject)>,
}

impl PyItemsIter {
    pub fn new(items: Vec<(String, PyObject)>) -> Self {
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

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(String, PyObject)> {
        slf.inner.next()
    }
}
