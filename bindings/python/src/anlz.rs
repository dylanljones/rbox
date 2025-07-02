// Author: Dylan Jones
// Date:   2025-05-10

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rbox::anlz::{Anlz, AnlzTag};

use super::errors::AnlzError;

#[pyclass(unsendable)]
pub struct PyAnlz {
    anlz: Anlz,
}

#[pymethods]
impl PyAnlz {
    #[new]
    pub fn new(path: &str) -> PyResult<Self> {
        let anlz = Anlz::load(path).map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(PyAnlz { anlz })
    }

    pub fn dump_copy(&mut self, path: &str) -> PyResult<()> {
        self.anlz
            .dump_copy(path)
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn dump(&mut self) -> PyResult<()> {
        self.anlz
            .dump()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn contains(&mut self, tag: &str) -> PyResult<bool> {
        let tag_type = AnlzTag::from(tag.to_string());
        let contains = self.anlz.contains(tag_type);
        Ok(contains)
    }

    pub fn get_tags(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let tags = self
            .anlz
            .get_tags()
            .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        let py_tags = PyList::empty(py);
        for tag in tags {
            py_tags
                .append(tag.to_string())
                .map_err(|e| PyErr::new::<AnlzError, _>(e.to_string()))?;
        }
        Ok(py_tags.into())
    }

    pub fn get_path(&mut self) -> PyResult<Option<String>> {
        let path = self.anlz.get_path();
        if let Some(path) = path {
            return Ok(Some(path));
        }
        Err(PyErr::new::<AnlzError, _>("No path tag not found"))
    }

    pub fn set_path(&mut self, path: &str) -> PyResult<()> {
        self.anlz.set_path(path)?;
        Ok(())
    }
}
