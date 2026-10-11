// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! One Library enum types and re-exports shared with other database formats.
//!
//! Playlist codes retain the values accepted by the existing reader. Creation
//! remains restricted to lists and folders by the playlist model.

use diesel::sql_types::Integer;
use diesel::{AsExpression, FromSqlRow};
use rbox_db::diesel_int_enum;

pub use rbox_core::FileType;
pub use rbox_db::enums::{HistoryType, HotCueBanklistType, MyTagType};

#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Stored Rekordbox codes for PlaylistType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
#[cfg_attr(feature = "pyo3", pyclass(name = "OneLibraryPlaylistType", eq, eq_int))]
#[cfg_attr(feature = "napi", napi(js_name = "OneLibraryPlaylistType"))]
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
