// Author: Dylan Jones
// Date:   2025-05-01

//! # Rekordbox master.db database models
//!
//! This module defines the data models that map to the tables in the main Rekordbox SQLite database.

mod agent_registry;
mod content_active_censor;
mod content_cue;
mod content_file;
mod djmd_active_sensor;
mod djmd_album;
mod djmd_artist;
mod djmd_category;
mod djmd_color;
mod djmd_content;
mod djmd_cue;
mod djmd_device;
mod djmd_genre;
mod djmd_history;
mod djmd_hot_cue_bank_list;
mod djmd_key;
mod djmd_label;
mod djmd_menu_items;
mod djmd_mixer_param;
mod djmd_my_tag;
mod djmd_playlist;
mod djmd_property;
mod djmd_recommend_like;
mod djmd_related_tracks;
mod djmd_sampler;
mod djmd_song_tag_list;
mod djmd_sort;
mod image_file;
mod setting_file;
mod uuid_map;

use chrono::{DateTime, TimeZone, Utc};
use diesel::backend::Backend;
use diesel::deserialize::{FromSql, Result as DResult};
use diesel::serialize::{IsNull, Output, Result as SResult, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use rand::rngs::ThreadRng;
use rand::RngCore;

use super::schema;
use crate::error::{Error, Result};

pub use agent_registry::{AgentRegistry, CloudAgentRegistry};
pub use content_active_censor::ContentActiveCensor;
pub use content_cue::ContentCue;
pub use content_file::ContentFile;
pub use djmd_active_sensor::DjmdActiveCensor;
pub use djmd_album::{DjmdAlbum, NewDjmdAlbum};
pub use djmd_artist::{DjmdArtist, NewDjmdArtist};
pub use djmd_category::DjmdCategory;
pub use djmd_color::DjmdColor;
pub use djmd_content::{DjmdContent, NewDjmdContent};
pub use djmd_cue::DjmdCue;
pub use djmd_device::DjmdDevice;
pub use djmd_genre::{DjmdGenre, NewDjmdGenre};
pub use djmd_history::{DjmdHistory, DjmdSongHistory, NewDjmdHistory, NewDjmdSongHistory};
pub use djmd_hot_cue_bank_list::{DjmdHotCueBanklist, DjmdSongHotCueBanklist, HotCueBanklistCue};
pub use djmd_key::{DjmdKey, NewDjmdKey};
pub use djmd_label::{DjmdLabel, NewDjmdLabel};
pub use djmd_menu_items::DjmdMenuItems;
pub use djmd_mixer_param::DjmdMixerParam;
pub use djmd_my_tag::{DjmdMyTag, DjmdSongMyTag, NewDjmdMyTag, NewDjmdSongMyTag};
pub use djmd_playlist::{
    DjmdPlaylist, DjmdPlaylistTreeNode, DjmdSongPlaylist, NewDjmdPlaylist, NewDjmdSongPlaylist,
};
pub use djmd_property::{DjmdCloudProperty, DjmdProperty};
pub use djmd_recommend_like::DjmdRecommendLike;
pub use djmd_related_tracks::{
    DjmdRelatedTracks, DjmdSongRelatedTracks, NewDjmdRelatedTracks, NewDjmdSongRelatedTracks,
};
pub use djmd_sampler::{DjmdSampler, DjmdSongSampler, NewDjmdSampler, NewDjmdSongSampler};
pub use djmd_song_tag_list::DjmdSongTagList;
pub use djmd_sort::DjmdSort;
pub use image_file::ImageFile;
pub use setting_file::SettingFile;
pub use uuid_map::UuidIDMap;

pub type Date = DateTime<Utc>;

const DATEFMT: &str = "%Y-%m-%d %H:%M:%S%.3f %:z";

/// Parse a Rekordbox database date string into a chrono DateTime<Utc>
pub fn parse_datetime(s: &str) -> Result<Date> {
    // If the string contains additional info in brackets (...), remove it
    let idx = s.find('(');
    let s = if let Some(idx) = idx { &s[..idx] } else { s };
    DateTime::parse_from_str(s, DATEFMT)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::Error(format!("Failed to parse datetime: {}", e)))
}

/// Format a chrono DateTime<Tz> into a Rekordbox date string
pub fn format_datetime<Tz: TimeZone>(dt: &DateTime<Tz>) -> String {
    dt.with_timezone(&Utc).format(DATEFMT).to_string()
}

/// Wrapper for DateTime<Utc> for diesel serialization/deserialization
#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub struct DateString(pub Date);

impl From<DateString> for Date {
    fn from(s: DateString) -> Self {
        s.0
    }
}

impl From<Date> for DateString {
    fn from(dt: Date) -> Self {
        DateString(dt)
    }
}

impl<B> FromSql<Text, B> for DateString
where
    B: Backend,
    String: FromSql<Text, B>,
{
    fn from_sql(bytes: B::RawValue<'_>) -> DResult<Self> {
        let s = <String as FromSql<Text, B>>::from_sql(bytes)?;
        Ok(parse_datetime(&s).map(DateString).map_err(|_| {
            let msg = format!("Invalid datetime string: {}", s);
            diesel::result::Error::DeserializationError(msg.into())
        })?)
    }
}

impl ToSql<Text, diesel::sqlite::Sqlite> for DateString
where
    str: ToSql<Text, diesel::sqlite::Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> SResult {
        let s = format_datetime(&self.0); // Format the DateTime<Utc> as a string
        out.set_value(s); // Set the value in the output buffer
        Ok(IsNull::No)
    }
}

/// Trait for converting between Rekordbox date strings and chrono DateTime
pub trait MasterDbDateString {
    fn from_datetime<Tz: TimeZone>(dt: DateTime<Tz>) -> Self;
    fn into_datetime(self) -> Result<Date>;
}

/// Implement RekordboxDateString for String
impl MasterDbDateString for String {
    fn from_datetime<Tz: TimeZone>(dt: DateTime<Tz>) -> Self {
        format_datetime(&dt)
    }

    fn into_datetime(self) -> Result<Date> {
        parse_datetime(&self)
    }
}

/// A generator for creating random unique IDs for the master.db database models.
struct RandomIdGenerator {
    /// Use 28-bit IDs instead of 32-bit IDs.
    is_28_bit: bool,
}

impl RandomIdGenerator {
    /// Creates a new `RandomIdGenerator`.
    ///
    /// # Arguments
    /// * `is_28_bit` - If true, generates 28-bit IDs; otherwise, generates 32-bit IDs.
    pub fn new(is_28_bit: bool) -> Self {
        Self { is_28_bit }
    }

    /// Generates a new random unique ID as a string, returning None if the ID is invalid.
    fn generate_id(&self, rng: &mut ThreadRng, mut buf: [u8; 4]) -> Option<String> {
        rng.fill_bytes(&mut buf);
        let mut id = ((buf[0] as u32) << 24)
            + ((buf[1] as u32) << 16)
            + ((buf[2] as u32) << 8)
            + (buf[3] as u32);
        if self.is_28_bit {
            id >>= 4;
        }
        if id < 100 {
            return None;
        }
        Some(id.to_string())
    }

    /// Tries to generate a new random unique ID as a string in `attempts` attempts
    pub fn try_generate(&mut self, attempts: usize) -> Result<String> {
        let mut rng = rand::rng();
        let buf = [0u8; 4];
        for _ in 0..attempts {
            let id = self.generate_id(&mut rng, buf);
            if let Some(id) = id {
                return Ok(id);
            }
        }
        Err(Error::Error(format!(
            "Failed to generate a unique ID in {attempts} attempts"
        )))
    }
}

impl Iterator for RandomIdGenerator {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.try_generate(1_000_000))
    }
}
