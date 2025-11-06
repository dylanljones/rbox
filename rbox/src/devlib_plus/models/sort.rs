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
#[diesel(table_name = schema::sort)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Sort {
    pub id: i32,
    pub menu_item_id: i32,
    pub sequence_no: i32,
    pub is_visible: i32,
    pub is_selected_as_sub_column: i32,
}

#[derive(Debug, Clone, PartialEq, Insertable)]
#[diesel(table_name = schema::sort)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewSort {
    pub id: i32,
    pub menu_item_id: i32,
    pub sequence_no: i32,
    pub is_visible: i32,
    pub is_selected_as_sub_column: i32,
}
