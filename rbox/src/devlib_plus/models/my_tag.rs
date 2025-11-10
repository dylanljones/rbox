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

/// Represents the `playlist` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `playlist` table in the Device Library Plus export database.
/// It stores information about playlists, including metadata such as sequence, name, image path,
/// attributes, and parent relationships.
///
/// # Referenced by
///
///
/// # References
///
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = schema::myTag)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct MyTag {
    pub id: i32,
    pub seq: i32,
    pub name: String,
    pub attribute: i32,
    pub my_tag_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::myTag)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewMyTag {
    pub seq: i32,
    pub name: String,
    pub attribute: i32,
    pub my_tag_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, Insertable)]
#[diesel(table_name = schema::myTag_content)]
#[diesel(primary_key(my_tag_id, content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct MyTagContent {
    pub my_tag_id: i32,
    pub content_id: i32,
}
