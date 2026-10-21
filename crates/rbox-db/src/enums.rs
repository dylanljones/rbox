// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared database enum codes and their Diesel integer conversions.

use diesel::sql_types::Integer;
use diesel::{AsExpression, FromSqlRow};

#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Implement Diesel integer conversion for a local `repr(i32)` enum.
///
/// The enum must implement `Copy` and `TryFrom<i32>` with a boxable error.
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

/// Stored Rekordbox codes for HistoryType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
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

diesel_int_enum!(HistoryType);

/// Stored Rekordbox codes for HotCueBanklistType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
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

diesel_int_enum!(HotCueBanklistType);

/// Stored Rekordbox codes for MyTagType.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
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

diesel_int_enum!(MyTagType);
