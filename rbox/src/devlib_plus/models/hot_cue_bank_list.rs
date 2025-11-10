// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::schema::{hotCueBankList, hotCueBankList_cue};
use crate::model_traits::Sequence;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = hotCueBankList)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct HotCueBankList {
    pub id: i32,
    pub seq: i32,
    pub name: Option<String>,
    pub image_id: Option<i32>,
    pub attribute: i32,
    pub hot_cue_bank_list_id_parent: Option<i32>,
}

impl Sequence for HotCueBankList {
    #[inline]
    fn seq(&self) -> i32 {
        self.seq
    }
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = hotCueBankList)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewHotCueBankList {
    pub seq: i32,
    pub name: Option<String>,
    pub image_id: Option<i32>,
    pub attribute: i32,
    pub hot_cue_bank_list_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = hotCueBankList_cue)]
#[diesel(primary_key(hot_cue_bank_list_id, cue_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct HotCueBankListCue {
    pub hot_cue_bank_list_id: i32,
    pub cue_id: i32,
    pub seq: i32,
}

impl HotCueBankListCue {
    pub fn delete_by_cue_id(conn: &mut SqliteConnection, cue_id: i32) -> QueryResult<usize> {
        let results =
            diesel::delete(hotCueBankList_cue::table.filter(hotCueBankList_cue::cue_id.eq(cue_id)))
                .execute(conn)?;
        // TODO: Update seq numbers
        Ok(results)
    }
    pub fn delete_by_cue_ids(conn: &mut SqliteConnection, cue_ids: &[i32]) -> QueryResult<usize> {
        let results = diesel::delete(
            hotCueBankList_cue::table.filter(hotCueBankList_cue::cue_id.eq_any(cue_ids)),
        )
        .execute(conn)?;
        // TODO: Update seq numbers
        Ok(results)
    }
}

impl Sequence for HotCueBankListCue {
    #[inline]
    fn seq(&self) -> i32 {
        self.seq
    }
}
