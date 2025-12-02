// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::djmd_content::DjmdContent;
use super::schema::djmdMixerParam;
use super::{Date, DateString};
use crate::model_traits::Model;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdMixerParam` table in the Rekordbox database.
///
/// This struct maps to the `djmdMixerParam` table in the SQLite database used by Rekordbox.
/// It stores information about mixer parameters, including gain and peak values for high and low frequencies.
///
/// Each of the two gain values are represented by a 32-bit floating point number packed into a
/// pair of 16-bit integers. The floating point value represents the linear gain factor, which can
/// be converted into decibels (dB) by calculating 20.0 * math.log10(f) where f is the gain factor.
///
/// The auto-gain value is the one shown in the grid edit panel. The peak value does not appear to
/// be displayed anywhere in the program, and is most likely used internally for
/// limiting and/or waveform scaling.
///
/// # References
/// * [`DjmdContent`] via `content_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdMixerParam)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdMixerParam {
    /// A unique identifier for the entry.
    pub id: String,
    /// A unique universal identifier for the entry.
    pub uuid: String,
    /// An integer representing the data status in Rekordbox.
    pub rb_data_status: i32,
    /// An integer representing the local data status in Rekordbox.
    pub rb_local_data_status: i32,
    /// An integer indicating whether the entry is locally deleted.
    pub rb_local_deleted: i32,
    /// An integer indicating whether the entry is locally synced.
    pub rb_local_synced: i32,
    /// An optional integer representing the update sequence number.
    pub usn: Option<i32>,
    /// An optional integer representing the local update sequence number.
    pub rb_local_usn: Option<i32>,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    /// The ID of the associated [`DjmdContent`].
    pub content_id: String,
    /// The upper 16 bits of a IEEE754 single-precision floating point number representing the gain.
    ///
    /// Use `get_gain_db` and `set_gain_db` to access and modify the gain in decibels (dB).
    pub gain_high: i32,
    /// The lower 16 bits of a IEEE754 single-precision floating point number representing the gain.
    ///
    /// Use `get_gain_db` and `set_gain_db` to access and modify the gain in decibels (dB).
    pub gain_low: i32,
    /// The upper 16 bits of a IEEE754 single-precision floating point number representing the peak.
    ///
    /// Use `get_peak_db` and `set_peak_db` to access and modify the peak in decibels (dB).
    pub peak_high: i32,
    /// The lower 16 bits of a IEEE754 single-precision floating point number representing the peak.
    ///
    /// Use `get_peak_db` and `set_peak_db` to access and modify the peak in decibels (dB).
    pub peak_low: i32,
}

impl Model for DjmdMixerParam {
    type Id = str;

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

impl DjmdMixerParam {
    /// Returns the gain from the lower/upper 16 bits of a IEEE754 single-precision float.
    fn get_db(low: u16, high: u16) -> f64 {
        let bits = ((high as u32) << 16) | (low as u32);
        let factor: f32 = f32::from_bits(bits);
        let factor_f64 = factor as f64;
        if factor_f64 <= 0.0 {
            f64::NEG_INFINITY
        } else {
            20.0 * factor_f64.log10()
        }
    }

    /// Returns the lower/upper 16 bits of a IEEE754 single-precision float from a gain value.
    fn get_lower_upper_bits(value: f64) -> (u16, u16) {
        if value.is_infinite() && value.is_sign_negative() {
            return (0, 0);
        }
        let factor = 10f64.powf(value / 20.0);
        let bits = (factor as f32).to_bits();
        let low = (bits & 0xFFFF) as u16;
        let high = (bits >> 16) as u16;
        (low, high)
    }

    /// Returns the gain in decibels (dB) of the mixer parameter.
    pub fn get_gain_db(&self) -> f64 {
        Self::get_db(self.gain_low as u16, self.gain_high as u16)
    }

    /// Sets the gain in decibels (dB) of the mixer parameter.
    pub fn set_gain_db(&mut self, value: f64) {
        let (low, high) = Self::get_lower_upper_bits(value);
        self.gain_low = low as i32;
        self.gain_high = high as i32;
    }

    /// Returns the peak in decibels (dB) of the mixer parameter.
    pub fn get_peak_db(&self) -> f64 {
        Self::get_db(self.peak_low as u16, self.peak_high as u16)
    }

    /// Sets the peak in decibels (dB) of the mixer parameter.
    pub fn set_peak_db(&mut self, value: f64) {
        let (low, high) = Self::get_lower_upper_bits(value);
        self.peak_low = low as i32;
        self.peak_high = high as i32;
    }
}
