// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::menu_item::MenuItem;
use super::schema;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

type JoinTuple = (CategoryRaw, Option<MenuItem>);

#[derive(
    Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset, Associations,
)]
#[diesel(table_name = schema::category)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(MenuItem, foreign_key = menu_item_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct CategoryRaw {
    pub id: i32,
    pub menu_item_id: i32,
    pub sequence_no: i32,
    pub is_visible: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub id: i32,
    pub menu_item_id: i32,
    pub sequence_no: i32,
    pub is_visible: i32,

    pub menu_item: Option<MenuItem>,
}

impl Into<CategoryRaw> for Category {
    fn into(self) -> CategoryRaw {
        CategoryRaw {
            id: self.id,
            menu_item_id: self.menu_item_id,
            sequence_no: self.sequence_no,
            is_visible: self.is_visible,
        }
    }
}

impl From<CategoryRaw> for Category {
    fn from(raw: CategoryRaw) -> Self {
        Category {
            id: raw.id,
            menu_item_id: raw.menu_item_id,
            sequence_no: raw.sequence_no,
            is_visible: raw.is_visible,
            menu_item: None,
        }
    }
}

impl Category {
    pub fn from_join(tuple: JoinTuple) -> Self {
        let mut result = Self::from(tuple.0);
        result.menu_item = tuple.1;
        result
    }
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::category)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewCategory {
    pub menu_item_id: i32,
    pub sequence_no: i32,
    pub is_visible: i32,
}

impl NewCategory {
    pub fn new(menu_item_id: i32, sequence_no: i32, is_visible: i32) -> Self {
        Self {
            menu_item_id,
            sequence_no,
            is_visible,
        }
    }
}
