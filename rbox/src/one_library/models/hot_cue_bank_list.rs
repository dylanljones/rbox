// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::schema::{hotCueBankList, hotCueBankList_cue};
use crate::model_traits::Model;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `hotCueBankList` table in the Rekordbox One Library database.
///
/// This struct maps to the `hotCueBankList` table in the One Library export database.
///
/// # References
/// * [`HotCueBankList`] via `parent_id` foreign key.
/// * [`Image`] via `image_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = hotCueBankList)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct HotCueBankList {
    /// The unique identifier of the hot cue bank list.
    pub id: i32,
    /// The sequence/order number of the hot cue bank list (1-based index).
    pub seq: i32,
    /// The name of the hot cue bank list.
    pub name: Option<String>,
    /// An optional foreign key referencing an [`Image`].
    pub image_id: Option<i32>,
    /// The hot cue bank list type, either list (`0`) or a folder (`1`).
    pub attribute: i32,
    /// The ID of the parent [`HotCueBankList`], `0` for top-level records.
    pub parent_id: Option<i32>,
}

impl Model for HotCueBankList {
    type Id = i32;

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }
}

/// Represents the `hotCueBankList_cue` table in the Rekordbox One Library database.
///
/// This struct maps to the `hotCueBankList_cue` table in the One Library export database.
///
/// # References
/// * [`HotCueBankList`] via `hot_cue_bank_list_id` foreign key.
/// * [`Cue`] via `cue_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable)]
#[diesel(table_name = hotCueBankList_cue)]
#[diesel(primary_key(hot_cue_bank_list_id, cue_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct HotCueBankListCue {
    /// The unique identifier of the hot cue bank list.
    pub hot_cue_bank_list_id: i32,
    /// The unique identifier of the cue.
    pub cue_id: i32,
    /// The sequence/order number of the hot cue bank list cue (1-based index).
    pub seq: i32,
}

impl Model for HotCueBankListCue {
    type Id = (i32, i32);

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        Self::query().find(*id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(*id))).get_result(conn)
    }
}

impl HotCueBankListCue {
    pub fn delete_by_cue_id(conn: &mut SqliteConnection, cue_id: i32) -> QueryResult<usize> {
        let results =
            diesel::delete(hotCueBankList_cue::table.filter(hotCueBankList_cue::cue_id.eq(cue_id)))
                .execute(conn)?;
        // TODO: Update seq numbers
        Ok(results)
    }

    pub fn delete_by_cue_ids(
        conn: &mut SqliteConnection,
        cue_ids: &Vec<i32>,
    ) -> QueryResult<usize> {
        let results = diesel::delete(
            hotCueBankList_cue::table.filter(hotCueBankList_cue::cue_id.eq_any(cue_ids)),
        )
        .execute(conn)?;
        // TODO: Update seq numbers
        Ok(results)
    }
}
