// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Enum codes shared by Rekordbox formats; optional Diesel support uses the `diesel` feature.
//! Raw binary fields retain unknown codes independently of these known-value enums.

#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Implement Diesel integer conversion for a local `repr(i32)` enum.
///
/// The enum must implement `Copy` and `TryFrom<i32>` with a boxable error.
#[cfg(feature = "diesel")]
#[macro_export]
macro_rules! diesel_int_enum {
    ($name:ident) => {
        impl<B> diesel::deserialize::FromSql<diesel::sql_types::Integer, B> for $name
        where
            B: diesel::backend::Backend,
            i32: diesel::deserialize::FromSql<diesel::sql_types::Integer, B>,
        {
            fn from_sql(bytes: B::RawValue<'_>) -> diesel::deserialize::Result<Self> {
                let value =
                    <i32 as diesel::deserialize::FromSql<diesel::sql_types::Integer, B>>::from_sql(
                        bytes,
                    )?;
                Ok(Self::try_from(value)?)
            }
        }
        impl diesel::serialize::ToSql<diesel::sql_types::Integer, diesel::sqlite::Sqlite>
            for $name
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
            ) -> diesel::serialize::Result {
                out.set_value(*self as i32);
                Ok(diesel::serialize::IsNull::No)
            }
        }
    };
}

/// Known Rekordbox file type codes; use raw integers to retain unknown values.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "diesel", derive(diesel::FromSqlRow, diesel::AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Integer))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum FileType {
    /// Content that rekordbox could not read; this is a known on-disk code.
    #[default]
    INVALID = 0,
    MP3 = 1,
    MP4 = 3,
    M4A = 4,
    FLAC = 5,
    ALAC = 6,
    WAV = 11,
    AIFF = 12,
    VIDEO = 16,
    StreamApple = 26,
}

impl TryFrom<i32> for FileType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::INVALID),
            1 => Ok(Self::MP3),
            3 => Ok(Self::MP4),
            4 => Ok(Self::M4A),
            5 => Ok(Self::FLAC),
            6 => Ok(Self::ALAC),
            11 => Ok(Self::WAV),
            12 => Ok(Self::AIFF),
            16 => Ok(Self::VIDEO),
            26 => Ok(Self::StreamApple),
            _ => Err("Invalid value for FileType".into()),
        }
    }
}

impl Into<i32> for FileType {
    fn into(self) -> i32 {
        self as i32
    }
}

impl FileType {
    /// AAC audio uses the same database code as M4A.
    pub const AAC: Self = Self::M4A;
    /// Alternate extension spelling for AIFF.
    pub const AIF: Self = Self::AIFF;

    /// Infer a file type from an extension without its leading dot.
    ///
    /// Matching is case-insensitive. M4A defaults to AAC (code 4); distinguishing
    /// ALAC (code 6) in the same container requires inspecting the audio codec.
    pub fn try_from_extension(ext: &str) -> std::result::Result<Self, String> {
        match ext.to_lowercase().as_str() {
            "mp3" => Ok(Self::MP3),
            "mp4" => Ok(Self::MP4),
            "m4a" | "aac" => Ok(Self::M4A),
            "alac" => Ok(Self::ALAC),
            "flac" => Ok(Self::FLAC),
            "wav" => Ok(Self::WAV),
            "aiff" | "aif" => Ok(Self::AIFF),
            "avi" | "m4v" | "mov" | "mpg" => Ok(Self::VIDEO),
            &_ => Err(format!("Unknown file type '{}'", ext)),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl FileType {
    #[classattr]
    #[pyo3(name = "AAC")]
    fn aac_alias() -> Self {
        Self::AAC
    }

    #[classattr]
    #[pyo3(name = "AIF")]
    fn aif_alias() -> Self {
        Self::AIF
    }
}

/// Stored Rekordbox codes for HistoryType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "diesel", derive(diesel::FromSqlRow, diesel::AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Integer))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, eq, eq_int))]
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

impl Into<i32> for HistoryType {
    fn into(self) -> i32 {
        self as i32
    }
}

#[cfg(feature = "diesel")]
diesel_int_enum!(HistoryType);

/// Stored Rekordbox codes for HotCueBanklistType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "diesel", derive(diesel::FromSqlRow, diesel::AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Integer))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, eq, eq_int))]
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

impl Into<i32> for HotCueBanklistType {
    fn into(self) -> i32 {
        self as i32
    }
}

#[cfg(feature = "diesel")]
diesel_int_enum!(HotCueBanklistType);

/// Stored Rekordbox codes for MyTagType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "diesel", derive(diesel::FromSqlRow, diesel::AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Integer))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, eq, eq_int))]
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

impl Into<i32> for MyTagType {
    fn into(self) -> i32 {
        self as i32
    }
}

#[cfg(feature = "diesel")]
diesel_int_enum!(MyTagType);

/// Shared master database and OneLibrary playlist codes. DeviceSQL only uses the list/folder distinction;
/// these variants do not imply support for exporting smart or cloud playlists.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "diesel", derive(diesel::FromSqlRow, diesel::AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Integer))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, eq, eq_int))]
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

impl Into<i32> for PlaylistType {
    fn into(self) -> i32 {
        self as i32
    }
}

#[cfg(feature = "diesel")]
diesel_int_enum!(PlaylistType);

#[cfg(feature = "diesel")]
diesel_int_enum!(FileType);

/// Stored Rekordbox codes for Analyzed.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, eq, eq_int))]
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

impl Into<i32> for Analyzed {
    fn into(self) -> i32 {
        self as i32
    }
}

/// Stored Rekordbox codes for RelatedTracksType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "diesel", derive(diesel::FromSqlRow, diesel::AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Integer))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum RelatedTracksType {
    #[default]
    List = 11,
    Folder = 10,
}

impl TryFrom<i32> for RelatedTracksType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            11 => Ok(Self::List),
            10 => Ok(Self::Folder),
            _ => Err(format!("Invalid value for RelatedTracksType: {}", value)),
        }
    }
}

impl Into<i32> for RelatedTracksType {
    fn into(self) -> i32 {
        self as i32
    }
}

#[cfg(feature = "diesel")]
diesel_int_enum!(RelatedTracksType);

/// Stored Rekordbox codes for SamplerType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "diesel", derive(diesel::FromSqlRow, diesel::AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Integer))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum SamplerType {
    #[default]
    List = 2,
    Folder = 3,
    All = 5,
    Capture = 6,
}

impl TryFrom<i32> for SamplerType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            2 => Ok(Self::List),
            3 => Ok(Self::Folder),
            5 => Ok(Self::All),
            6 => Ok(Self::Capture),
            _ => Err(format!("Invalid value for SamplerType: {}", value)),
        }
    }
}

impl Into<i32> for SamplerType {
    fn into(self) -> i32 {
        self as i32
    }
}

#[cfg(feature = "diesel")]
diesel_int_enum!(SamplerType);
