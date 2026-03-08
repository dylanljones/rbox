// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use std::convert::TryFrom;

use diesel::backend::Backend;
use diesel::deserialize::{FromSql, Result as DResult};
use diesel::serialize::{IsNull, Output, Result as SResult, ToSql};
use diesel::sql_types::Integer;
use diesel::sqlite::Sqlite;
use diesel::{AsExpression, FromSqlRow};

#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Generate Diesel impls for a `#[repr(i32)]` enum that implements `TryFrom<i32>`
#[macro_export]
macro_rules! diesel_int_enum {
    ($name:ident) => {
        impl<B> FromSql<Integer, B> for $name
        where
            B: Backend,
            i32: FromSql<Integer, B>,
        {
            fn from_sql(bytes: B::RawValue<'_>) -> DResult<Self> {
                let v = i32::from_sql(bytes)?;
                Ok(<$name>::try_from(v)?)
            }
        }

        impl ToSql<Integer, Sqlite> for $name
        where
            i32: ToSql<Integer, Sqlite>,
        {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> SResult {
                out.set_value((*self) as i32);
                Ok(IsNull::No)
            }
        }
    };
}

#[cfg(any(feature = "master-db", feature = "one-library"))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
#[cfg_attr(feature = "pyo3", pyclass(eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum FileType {
    #[default]
    MP3 = 0,
    MP3_2 = 1,
    MP4 = 3,
    ALAC = 4,
    FLAC = 5,
    M4A = 6,
    WAV = 11,
    AIFF = 12,
}

impl TryFrom<i32> for FileType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::MP3),
            1 => Ok(Self::MP3_2),
            3 => Ok(Self::MP4),
            4 => Ok(Self::ALAC),
            5 => Ok(Self::FLAC),
            6 => Ok(Self::M4A),
            11 => Ok(Self::WAV),
            12 => Ok(Self::AIFF),
            _ => Err("Invalid value for FileType".into()),
        }
    }
}

impl FileType {
    pub fn try_from_extension(ext: &str) -> std::result::Result<Self, String> {
        match ext.to_lowercase().as_str() {
            "mp3" => Ok(Self::MP3),
            "mp4" => Ok(Self::MP4),
            "m4a" => Ok(Self::ALAC),
            "flac" => Ok(Self::FLAC),
            "wav" => Ok(Self::WAV),
            "aiff" => Ok(Self::AIFF),
            "aif" => Ok(Self::AIFF),
            &_ => Err(format!("Unknown file type '{}'", ext)),
        }
    }
}

diesel_int_enum!(FileType);

#[cfg(any(feature = "master-db"))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
#[cfg_attr(feature = "pyo3", pyclass(eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum Analyzed {
    #[default]
    NotAnalyzed = 0,
    Standard = 105,
    Advanced = 121,
    Locked = 233,
}

impl TryFrom<i32> for Analyzed {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Analyzed::NotAnalyzed),
            105 => Ok(Analyzed::Standard),
            121 => Ok(Analyzed::Advanced),
            233 => Ok(Analyzed::Locked),
            _ => Err("Invalid value for Analyzed".into()),
        }
    }
}

diesel_int_enum!(Analyzed);

#[cfg(any(feature = "master-db", feature = "one-library"))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
#[cfg_attr(feature = "pyo3", pyclass(eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum HistoryType {
    #[default]
    List = 0,
    Folder = 1,
}

impl TryFrom<i32> for HistoryType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::List),
            1 => Ok(Self::Folder),
            _ => Err(format!("Invalid value for HistoryType: {}", value)),
        }
    }
}

diesel_int_enum!(HistoryType);

#[cfg(any(feature = "master-db", feature = "one-library"))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
#[cfg_attr(feature = "pyo3", pyclass(eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum HotCueBanklistType {
    #[default]
    List = 0,
    Folder = 1,
}

impl TryFrom<i32> for HotCueBanklistType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::List),
            1 => Ok(Self::Folder),
            _ => Err(format!("Invalid value for HotCueBanklistType: {}", value)),
        }
    }
}

diesel_int_enum!(HotCueBanklistType);

#[cfg(any(feature = "master-db", feature = "one-library"))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
#[cfg_attr(feature = "pyo3", pyclass(eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum MyTagType {
    #[default]
    List = 0,
    Folder = 1,
}

impl TryFrom<i32> for MyTagType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::List),
            1 => Ok(Self::Folder),
            _ => Err("Invalid value for MyTagType".into()),
        }
    }
}

diesel_int_enum!(MyTagType);

#[cfg(any(feature = "master-db", feature = "one-library"))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
#[cfg_attr(feature = "pyo3", pyclass(eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum PlaylistType {
    #[default]
    List = 0,
    Folder = 1,
    SmartList = 4,
    CloudLibrarySync = -128,
}

impl TryFrom<i32> for PlaylistType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::List),
            1 => Ok(Self::Folder),
            4 => Ok(Self::SmartList),
            -128 => Ok(Self::CloudLibrarySync),
            _ => Err(format!("Invalid value for PlaylistType: {}", value)),
        }
    }
}

diesel_int_enum!(PlaylistType);

#[cfg(any(feature = "master-db", feature = "one-library"))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
#[cfg_attr(feature = "pyo3", pyclass(eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum RelatedTracksType {
    #[default]
    List = 0,
    Folder = 1,
}

impl TryFrom<i32> for RelatedTracksType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::List),
            1 => Ok(Self::Folder),
            _ => Err(format!("Invalid value for RelatedTracksType: {}", value)),
        }
    }
}

diesel_int_enum!(RelatedTracksType);

#[cfg(any(feature = "master-db"))]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
#[cfg_attr(feature = "pyo3", pyclass(eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum SamplerType {
    #[default]
    List = 0,
    Folder = 1,
}

impl TryFrom<i32> for SamplerType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::List),
            1 => Ok(Self::Folder),
            _ => Err(format!("Invalid value for SamplerType: {}", value)),
        }
    }
}

diesel_int_enum!(SamplerType);
