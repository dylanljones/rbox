// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Master database enum codes and re-exports of shared Rekordbox enums.

use diesel::sql_types::Integer;
use diesel::{AsExpression, FromSqlRow};

use rbox_db::diesel_int_enum;

pub use rbox_core::FileType;
pub use rbox_db::enums::{HistoryType, HotCueBanklistType, MyTagType};

#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

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

/// Stored Rekordbox codes for PlaylistType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
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

diesel_int_enum!(PlaylistType);

/// Stored Rekordbox codes for RelatedTracksType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
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

diesel_int_enum!(RelatedTracksType);

/// Stored Rekordbox codes for SamplerType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
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

diesel_int_enum!(SamplerType);
