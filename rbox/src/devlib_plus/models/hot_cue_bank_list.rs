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
#[diesel(table_name = schema::hotCueBankList)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct HotCueBankList {
    pub id: i32,
    pub sequence_no: i32,
    pub name: Option<String>,
    pub image_id: Option<i32>,
    pub attribute: i32,
    pub hot_cue_bank_list_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::hotCueBankList)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewHotCueBankList {
    pub sequence_no: i32,
    pub name: Option<String>,
    pub image_id: Option<i32>,
    pub attribute: i32,
    pub hot_cue_bank_list_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = schema::hotCueBankList_cue)]
#[diesel(primary_key(hot_cue_bank_list_id, cue_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct HotCueBankListCue {
    pub hot_cue_bank_list_id: i32,
    pub cue_id: i32,
    pub sequence_no: i32,
}
