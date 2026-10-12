// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Common handler types and database traits for enabled components.

#[cfg(feature = "anlz")]
pub use crate::Anlz;
#[cfg(feature = "master-db")]
pub use crate::MasterDb;
#[cfg(feature = "one-library")]
pub use crate::OneLibrary;
#[cfg(feature = "xml")]
pub use crate::RekordboxXml;
#[cfg(feature = "settings")]
pub use crate::Setting;
pub use crate::{Error, PathExt, RekordboxOptions, Result};
#[cfg(any(feature = "master-db", feature = "one-library"))]
pub use rbox_db::{
    Model, ModelDelete, ModelInsert, ModelList, ModelRelations, ModelTree, ModelUpdate,
};
