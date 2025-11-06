// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::schema;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::history)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct History {
    pub id: i32,
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub history_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::history)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewHistory {
    pub id: i32,
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub history_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = schema::history_content)]
#[diesel(primary_key(history_id, content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct HistoryContent {
    pub history_id: i32,
    pub content_id: i32,
    pub sequence_no: i32,
}
