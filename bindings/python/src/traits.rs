// Author: Dylan Jones
// Date:   2025-07-01

use pyo3::{PyClass, PyRef, PyResult, Python};

pub trait IntoPy<T> {
    fn into_py(self, py: Python) -> PyResult<T>;
}

pub trait FromPy<T: PyClass> {
    fn from_py(py: Python, model: PyRef<'_, T>) -> Self;
}
