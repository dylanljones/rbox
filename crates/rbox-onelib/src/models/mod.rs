// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! SQLite row models for the OneLibrary export database.

mod album;
mod artist;
mod category;
mod color;
mod content;
mod cue;
mod genre;
mod history;
mod hot_cue_bank_list;
mod image;
mod key;
mod label;
mod menu_item;
mod my_tag;
mod playlist;
mod property;
mod recommend_like;
mod sort;

pub use album::{Album, AlbumView, NewAlbum};
pub use artist::{Artist, Composer, Lyricist, NewArtist, OriginalArtist, Remixer};
pub use category::{Category, CategoryView, NewCategory};
pub use color::{Color, NewColor};
pub use content::{Content, ContentView, NewContent};
pub use cue::Cue;
pub use genre::{Genre, NewGenre};
pub use history::{History, HistoryContent, NewHistory};
pub use hot_cue_bank_list::{HotCueBankList, HotCueBankListCue};
pub use image::{Image, NewImage};
pub use key::{Key, NewKey};
pub use label::{Label, NewLabel};
pub use menu_item::{MenuItem, NewMenuItem};
pub use my_tag::{MyTag, MyTagContent, NewMyTag};
pub use playlist::{NewPlaylist, Playlist, PlaylistContent, PlaylistTreeNode};
pub use property::Property;
pub use recommend_like::RecommendedLike;
pub use sort::{NewSort, Sort};

use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use diesel::backend::Backend;
use diesel::deserialize::{FromSql, Result as DResult};
use diesel::serialize::{IsNull, Output, Result as SResult, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};

use crate::error::{Error, Result};

pub type Date = DateTime<Utc>;

const DATEFMT: &str = "%Y-%m-%d";

/// Parse a One Library date as midnight UTC, ignoring surrounding whitespace.
///
/// Invalid dates retain the original input and Chrono's parse error.
pub fn parse_datetime(s: &str) -> Result<Date> {
    let naive_date =
        NaiveDate::parse_from_str(s.trim(), DATEFMT).map_err(|source| Error::InvalidDateTime {
            value: s.to_owned(),
            source,
        })?;
    let naive_dt = naive_date.and_time(NaiveTime::MIN);
    Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc))
}

/// Format a chrono DateTime<Tz> into a Devicelib Plus date string.
pub fn format_datetime<Tz: TimeZone>(dt: &DateTime<Tz>) -> String {
    dt.with_timezone(&Utc).format(DATEFMT).to_string()
}

/// Wrapper for NaiveDate for diesel serialization/deserialization
#[derive(Debug, FromSqlRow, AsExpression)]
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
        Ok(DateString(parse_datetime(&s)?))
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
pub trait OneLibraryDateString {
    fn from_date<Tz: TimeZone>(dt: DateTime<Tz>) -> Self;
    fn into_date(self) -> Result<Date>;
}

/// Implement DevlibPlusDateString for String
impl OneLibraryDateString for String {
    fn from_date<Tz: TimeZone>(dt: DateTime<Tz>) -> Self {
        format_datetime(&dt)
    }

    fn into_date(self) -> Result<Date> {
        parse_datetime(&self)
    }
}
