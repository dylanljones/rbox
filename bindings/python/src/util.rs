// Author: Dylan Jones
// Date:   2025-05-15

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde::{Deserialize, Serialize};

pub fn model_to_pydict<T: Serialize>(py: Python, model: T) -> PyResult<Bound<'_, PyDict>> {
    // Convert to JSON string and then parse it back to a Python dict
    let json_str = serde_json::to_string(&model).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Serialization failed: {}", e))
    })?;

    let json_module = py.import("json")?;
    let loads = json_module.getattr("loads")?;
    let py_obj = loads.call1((json_str,))?;

    // Use extract to get the owned Bound<PyDict>
    py_obj.extract()
}

pub fn models_to_pylist<T: Serialize>(py: Python, models: Vec<T>) -> PyResult<Py<PyList>> {
    let result = PyList::empty(py);
    for model in models {
        let dict = model_to_pydict(py, model)?;
        result.append(dict)?;
    }
    Ok(result.into())
}

pub fn pydict_to_model<T: for<'de> Deserialize<'de>>(
    py: Python,
    dict: &Bound<'_, PyDict>,
) -> PyResult<T> {
    // Convert Python dict to a JSON string first
    let json_module = py.import("json")?;
    let dumps = json_module.getattr("dumps")?;
    let json_str: String = dumps.call1((dict,))?.extract()?;

    // Use serde to deserialize the JSON into the generic struct type
    let model: T = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to deserialize: {}", e))
    })?;

    Ok(model)
}
