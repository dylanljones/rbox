// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Check that mapping derives resolve their runtime helpers without imports.
#![cfg(feature = "pyo3")]

use pyo3::prelude::*;
use rbox_derives::PyMutableMapping;

#[pyclass]
#[derive(PyMutableMapping)]
struct Mapping {
    pub name: String,
}

#[test]
fn mapping_iterator_signatures_resolve_without_helper_imports() {
    // Type-check all generated iterator methods without starting Python.
    fn check(py: Python<'_>, mapping: &Mapping) -> PyResult<()> {
        let _: Py<rbox_core::python::PyStrIter> = mapping.__iter__(py)?;
        let _: Py<rbox_core::python::PyStrIter> = mapping.keys(py)?;
        let _: Py<rbox_core::python::PyObjectIter> = mapping.values(py)?;
        let _: Py<rbox_core::python::PyItemsIter> = mapping.items(py)?;
        Ok(())
    }
    let _ = check;
}
