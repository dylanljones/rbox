// Author: Dylan Jones
// Date:   2025-07-01

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::PyClass;
use std::collections::HashMap;

use super::iter::{PyItemsIter, PyObjectIter, PyStrIter};

pub trait IntoPy<T> {
    fn into_py(self, py: Python) -> PyResult<T>;
}

pub trait FromPy<T: PyClass> {
    fn from_py(py: Python, model: PyRef<'_, T>) -> Self;
}

pub trait MutableMapping {
    fn field_names() -> Vec<String>;

    fn __len__(&self) -> usize {
        Self::field_names().len()
    }

    fn __contains__(&self, key: &str) -> bool {
        Self::field_names().iter().any(|k| k == key)
    }

    fn __getitem__(&self, py: Python, key: &str) -> PyResult<PyObject>;
    fn __setitem__(&mut self, py: Python, key: &str, value: PyObject) -> PyResult<()>;

    fn __iter__(&self, py: Python) -> PyResult<Py<PyStrIter>> {
        let iter = PyStrIter::new(Self::field_names());
        Py::new(py, iter)
    }

    fn keys(&self, py: Python) -> PyResult<Py<PyStrIter>> {
        self.__iter__(py)
    }

    fn values(&self, py: Python) -> PyResult<Py<PyObjectIter>> {
        let mut values: Vec<PyObject> = Vec::new();
        for key in Self::field_names() {
            let value = self.__getitem__(py, &key)?;
            values.push(value);
        }
        let iter = PyObjectIter::new(values);
        Py::new(py, iter)
    }

    fn items(&self, py: Python) -> PyResult<Py<PyItemsIter>> {
        let mut values: Vec<(String, PyObject)> = Vec::new();
        for key in Self::field_names() {
            let value = self.__getitem__(py, &key)?;
            values.push((key, value));
        }
        let iter = PyItemsIter::new(values);
        Py::new(py, iter)
    }

    fn get(&self, py: Python, key: &str, default: PyObject) -> PyResult<PyObject> {
        let res = self.__getitem__(py, key);
        if let Ok(res) = res {
            Ok(res)
        } else {
            Ok(default)
        }
    }

    fn update(&mut self, py: Python, data: HashMap<String, PyObject>) -> PyResult<()> {
        for (key, py_value) in data.into_iter() {
            let _ = self.__setitem__(py, &key, py_value);
        }
        Ok(())
    }

    fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for key in Self::field_names() {
            let value = self.__getitem__(py, &key)?;
            dict.set_item(key, value)?;
        }
        Ok(dict.into())
    }
}
