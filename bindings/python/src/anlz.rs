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

    pub fn get_beat_grid(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let beat_grid = self.anlz.get_beat_grid();
        if let Some(beat_grid) = beat_grid {
            let results = PyList::empty(py);
            for beat in beat_grid {
                let beat_dict = PyDict::new(py);
                beat_dict.set_item("beat_number", beat.beat_number)?;
                beat_dict.set_item("tempo", beat.tempo)?;
                beat_dict.set_item("time", beat.time)?;
                results.append(beat_dict)?;
            }
            return Ok(results.into());
        }
        Err(PyErr::new::<AnlzError, _>("No beat grid tag not found"))
    }

    pub fn get_extended_beat_grid(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let beat_grid = self.anlz.get_extended_beat_grid();
        if let Some(beat_grid) = beat_grid {
            let results = PyList::empty(py);
            for beat in beat_grid {
                let dict = PyDict::new(py);
                dict.set_item("beat_number", beat.beat_number)?;
                results.append(dict)?;
            }
            return Ok(results.into());
        }
        Err(PyErr::new::<AnlzError, _>(
            "No extended beat grid tag not found",
        ))
    }

    pub fn get_hot_cues(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let items = self.anlz.get_hot_cues();
        if let Some(items) = items {
            let results = PyList::empty(py);
            for item in items {
                let dict = PyDict::new(py);
                dict.set_item("hot_cue", item.hot_cue)?;
                dict.set_item("status", item.status)?;
                dict.set_item("order_first", item.order_first)?;
                dict.set_item("order_last", item.order_last)?;
                dict.set_item("cue_type", item.cue_type.to_int())?;
                dict.set_item("time", item.time)?;
                dict.set_item("loop_time", item.loop_time)?;
                results.append(dict)?;
            }
            return Ok(results.into());
        }
        Err(PyErr::new::<AnlzError, _>("No cue tag not found"))
    }

    pub fn get_memory_cues(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let items = self.anlz.get_memory_cues();
        if let Some(items) = items {
            let results = PyList::empty(py);
            for item in items {
                let dict = PyDict::new(py);
                dict.set_item("hot_cue", item.hot_cue)?;
                dict.set_item("status", item.status)?;
                dict.set_item("order_first", item.order_first)?;
                dict.set_item("order_last", item.order_last)?;
                dict.set_item("cue_type", item.cue_type.to_int())?;
                dict.set_item("time", item.time)?;
                dict.set_item("loop_time", item.loop_time)?;
                results.append(dict)?;
            }
            return Ok(results.into());
        }
        Err(PyErr::new::<AnlzError, _>("No cue tag not found"))
    }

    pub fn get_extended_hot_cues(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let items = self.anlz.get_extended_hot_cues();
        if let Some(items) = items {
            let results = PyList::empty(py);
            for item in items {
                let dict = PyDict::new(py);
                dict.set_item("hot_cue", item.hot_cue)?;
                dict.set_item("cue_type", item.cue_type.to_int())?;
                dict.set_item("time", item.time)?;
                dict.set_item("loop_time", item.loop_time)?;
                dict.set_item("color", item.color)?;
                dict.set_item("loop_numerator", item.loop_numerator)?;
                dict.set_item("loop_denominator", item.loop_denominator)?;
                dict.set_item("comment", item.comment.to_string())?;
                dict.set_item("hot_cue_color_index", item.hot_cue_color_index)?;
                dict.set_item("red", item.hot_cue_color_rgb.0)?;
                dict.set_item("green", item.hot_cue_color_rgb.1)?;
                dict.set_item("blue", item.hot_cue_color_rgb.1)?;
                results.append(dict)?;
            }
            return Ok(results.into());
        }
        Err(PyErr::new::<AnlzError, _>("No extended cue tag not found"))
    }

    pub fn get_extended_memory_cues(&mut self, py: Python) -> PyResult<Py<PyList>> {
        let items = self.anlz.get_extended_memory_cues();
        if let Some(items) = items {
            let results = PyList::empty(py);
            for item in items {
                let dict = PyDict::new(py);
                dict.set_item("hot_cue", item.hot_cue)?;
                dict.set_item("cue_type", item.cue_type.to_int())?;
                dict.set_item("time", item.time)?;
                dict.set_item("loop_time", item.loop_time)?;
                dict.set_item("color", item.color)?;
                dict.set_item("loop_numerator", item.loop_numerator)?;
                dict.set_item("loop_denominator", item.loop_denominator)?;
                dict.set_item("comment", item.comment.to_string())?;
                dict.set_item("hot_cue_color_index", item.hot_cue_color_index)?;
                dict.set_item("red", item.hot_cue_color_rgb.0)?;
                dict.set_item("green", item.hot_cue_color_rgb.1)?;
                dict.set_item("blue", item.hot_cue_color_rgb.1)?;
                results.append(dict)?;
            }
            return Ok(results.into());
        }
        Err(PyErr::new::<AnlzError, _>("No extended cue tag not found"))
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
