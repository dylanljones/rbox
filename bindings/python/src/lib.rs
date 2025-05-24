// Author: Dylan Jones
// Date:   2025-05-01

mod anlz;
mod errors;
mod masterdb;
mod util;
mod xml;

use pyo3::prelude::*;

#[pyfunction]
fn is_rekordbox_running() -> PyResult<bool> {
    Ok(rbox::util::is_rekordbox_running())
}

#[pymodule]
fn _rbox(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<masterdb::PyMasterDb>()?;
    m.add_class::<anlz::PyAnlz>()?;
    m.add_class::<xml::PyRekordboxXml>()?;
    let _ = m.add_function(wrap_pyfunction!(is_rekordbox_running, m)?);
    m.add("Error", m.py().get_type::<errors::Error>())?;
    m.add("DatabaseError", m.py().get_type::<errors::DatabaseError>())?;
    m.add("AnlzError", m.py().get_type::<errors::AnlzError>())?;
    m.add("XmlError", m.py().get_type::<errors::XmlError>())?;
    Ok(())
}
