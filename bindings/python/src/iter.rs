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
