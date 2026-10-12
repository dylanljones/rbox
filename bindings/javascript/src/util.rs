// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

#![allow(dead_code)]

use napi::Result;
use napi_derive::napi;
use rbox;

/// Translate component errors at the JavaScript boundary.
pub(crate) fn binding_error(error: impl std::fmt::Display) -> napi::Error {
    napi::Error::from_reason(error.to_string())
}

#[napi]
fn rekordbox_agent_app_dir() -> Result<String> {
    let path = rbox::core::rekordbox_agent_app_dir().map_err(crate::util::binding_error)?;
    Ok(path.to_str().unwrap().to_string())
}

#[napi]
fn options_json_path() -> Result<String> {
    let path = rbox::core::options_json_path().map_err(crate::util::binding_error)?;
    Ok(path.to_str().unwrap().to_string())
}

#[napi]
fn rekordbox_app_dir() -> Result<String> {
    let path = rbox::core::rekordbox_app_dir().map_err(crate::util::binding_error)?;
    Ok(path.to_str().unwrap().to_string())
}

#[napi]
fn is_rekordbox_running() -> bool {
    rbox::core::is_rekordbox_running()
}
