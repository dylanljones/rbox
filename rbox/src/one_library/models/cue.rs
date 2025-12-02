// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::hot_cue_bank_list::HotCueBankListCue;
use super::schema::cue;
use crate::model_traits::{Model, ModelDelete, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `cue` table in the Rekordbox One Library database.
///
/// This struct maps to the `cue` table in the One Library export database.
/// It stores information about cue points for tracks, including their timing, type, and
/// additional metadata.
///
/// # Notes
/// Rekordbox internally represents time in “frames”, each being 1/150th of a second (6.666ms).
/// The InFrame and OutFrame values use this unit of time. However, when a track is encoded with
/// variable bit-rate (VBR) or average bit-rate (ABR), the InMpegFrame and OutMpegFrame values are
/// filled out to assist with correct seeking. Despite the names, these values are not the frame
/// indices within the MPEG file, but instead use an alternative timing scheme that is typically
/// around 1/75th of a second (13.333ms) per frame, i.e. about half the granularity of normal frames.
///
/// # Referenced by
/// * [`HotCueBankListCue`] via `cue_id` foreign key.
///
/// # References
/// * [`Content`] via `content_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = cue)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Cue {
    /// The unique identifier of the cue.
    pub id: i32,
    /// A foreign key referencing the [`Content`].
    pub content_id: i32,
    /// The type of the cue.
    ///
    /// 0 if a memory cue, otherwise the number of Hot Cue
    pub kind: Option<i32>,
    /// An optional integer representing the index of the color in the color table.
    pub color_table_index: Option<i32>,
    /// A comment for the cue
    pub cue_comment: Option<String>,
    /// A flag whether the cue is part of an active loop.
    pub is_active_loop: Option<i32>,
    /// Unknown
    pub beat_loop_numerator: Option<i32>,
    /// Unknonw
    pub beat_loop_denominator: Option<i32>,
    /// The cue's start time in microseconds.
    pub in_usec: Option<i32>,
    /// The cue's end time in microseconds.
    pub out_usec: Option<i32>,
    /// The cue's start frame.
    ///
    /// One frame is 1/150th of a second
    pub in150_frame_per_sec: Option<i32>,
    /// The cue's end frame or -1 if not a loop.
    ///
    /// One frame is 1/150th of a second
    pub out150_frame_per_sec: Option<i32>,
    /// The cue's start MPEG frame or 0 if not a VBR/ABR MPEG file (see note).
    pub in_mpeg_frame_number: Option<i32>,
    /// The cue's end MPEG frame or 0 if not a loop or not a VBR/ABR MPEG file (see note)
    pub out_mpeg_frame_number: Option<i32>,
    /// The cue's start MPEG absolute frame or 0 if not a VBR/ABR MPEG file.
    pub in_mpeg_abs: Option<i32>,
    /// The cue's end MPEG absolute frame or 0 if not a loop or not a VBR/ABR MPEG file
    pub out_mpeg_abs: Option<i32>,
    /// The cue's start frame position in the decoding block.
    pub in_decoding_start_frame_position: Option<i32>,
    /// The cue's end frame position in the decoding block.
    pub out_decoding_start_frame_position: Option<i32>,
    /// The cue's start file offset in the decoding block.
    pub in_file_offset_in_block: Option<i32>,
    /// The cue's end file offset in the decoding block.
    pub out_file_offset_in_block: Option<i32>,
    /// The cue's start sample position in the decoding block.
    pub in_number_of_sample_in_block: Option<i32>,
    /// The cue's end sample position in the decoding block.
    pub out_number_of_sample_in_block: Option<i32>,
}

impl Model for Cue {
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

impl ModelUpdate for Cue {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(cue::table.find(self.id))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Cue {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let mut result = diesel::delete(cue::table.find(id)).execute(conn)?;
        result += HotCueBankListCue::delete_by_cue_id(conn, *id)?;
        Ok(result)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let mut result = diesel::delete(cue::table.filter(cue::id.eq_any(&ids))).execute(conn)?;
        let ids: Vec<i32> = ids.iter().map(|&id| *id).collect();
        result += HotCueBankListCue::delete_by_cue_ids(conn, &ids)?;
        Ok(result)
    }
}

impl Cue {
    /// Deletes all record from the `cue` table referencing a `content` record.
    pub fn delete_by_content_id(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        let deleted_ids: Vec<i32> = diesel::delete(cue::table.filter(cue::content_id.eq(id)))
            .returning(cue::id)
            .get_results(conn)?;
        let mut result = deleted_ids.len();
        result += HotCueBankListCue::delete_by_cue_ids(conn, &deleted_ids)?;
        Ok(result)
    }

    /// Deletes all record from the `cue` table referencing multiple `content` records.
    pub fn delete_by_content_ids(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        let deleted_ids: Vec<i32> = diesel::delete(cue::table.filter(cue::content_id.eq_any(ids)))
            .returning(cue::id)
            .get_results(conn)?;
        let mut result = deleted_ids.len();
        result += HotCueBankListCue::delete_by_cue_ids(conn, &deleted_ids)?;
        Ok(result)
    }
}
