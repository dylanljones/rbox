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
#[diesel(table_name = schema::cue)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Cue {
    pub id: i32,
    pub content_id: i32,
    pub kind: Option<i32>,
    pub color_table_index: Option<i32>,
    pub cue_comment: Option<String>,
    pub is_active_loop: Option<i32>,
    pub beat_loop_numerator: Option<i32>,
    pub beat_loop_denominator: Option<i32>,
    pub in_usec: Option<i32>,
    pub out_usec: Option<i32>,
    pub in150_frame_per_sec: Option<i32>,
    pub out150_frame_per_sec: Option<i32>,
    pub in_mpeg_frame_number: Option<i32>,
    pub out_mpeg_frame_number: Option<i32>,
    pub in_mpeg_abs: Option<i32>,
    pub out_mpeg_abs: Option<i32>,
    pub in_decoding_start_frame_position: Option<i32>,
    pub out_decoding_start_frame_position: Option<i32>,
    pub in_file_offset_in_block: Option<i32>,
    pub out_file_offset_in_block: Option<i32>,
    pub in_number_of_sample_in_block: Option<i32>,
    pub out_number_of_sample_in_block: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::cue)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewCue {
    pub content_id: i32,
    pub kind: Option<i32>,
    pub color_table_index: Option<i32>,
    pub cue_comment: Option<String>,
    pub is_active_loop: Option<i32>,
    pub beat_loop_numerator: Option<i32>,
    pub beat_loop_denominator: Option<i32>,
    pub in_usec: Option<i32>,
    pub out_usec: Option<i32>,
    pub in150_frame_per_sec: Option<i32>,
    pub out150_frame_per_sec: Option<i32>,
    pub in_mpeg_frame_number: Option<i32>,
    pub out_mpeg_frame_number: Option<i32>,
    pub in_mpeg_abs: Option<i32>,
    pub out_mpeg_abs: Option<i32>,
    pub in_decoding_start_frame_position: Option<i32>,
    pub out_decoding_start_frame_position: Option<i32>,
    pub in_file_offset_in_block: Option<i32>,
    pub out_file_offset_in_block: Option<i32>,
    pub in_number_of_sample_in_block: Option<i32>,
    pub out_number_of_sample_in_block: Option<i32>,
}
