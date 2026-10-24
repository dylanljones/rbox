// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared-storage path helpers exposed to Python.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn validate_uuid(value: &str) -> PyResult<()> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|err| PyValueError::new_err(format!("Invalid UUID: {err}")))
}

/// Return the USB ANLZ directory derived from the exported audio path.
///
/// Supply the volume-relative path with a leading slash, for example
/// /Contents/Artist/Track.flac. Input is hashed without normalization.
/// Remove the result's leading slash before joining to the USB root, then
/// append an analysis filename such as ANLZ0000.DAT, ANLZ0000.EXT or ANLZ0000.2EX.
#[pyfunction]
pub fn anlz_dir_usb(track_path: &str) -> String {
    rbox::core::anlz_dir_usb(track_path)
}

/// Return the local ANLZ directory derived from a track's djmdContent.UUID.
///
/// The UUID is normalized to lowercase hyphenated text and split after three
/// characters beneath /PIONEER/USBANLZ/. Remove the leading slash before joining
/// to the analysis/share root, then append the analysis filename. Existing
/// AnalysisDataPath values remain authoritative. Invalid UUIDs raise ValueError.
#[pyfunction]
pub fn anlz_dir_local(uuid: &str) -> PyResult<String> {
    validate_uuid(uuid)?;
    Ok(rbox::core::anlz_dir_local(uuid))
}

/// Return the local artwork directory derived from an artwork UUID.
///
/// The UUID is normalized to lowercase hyphenated text and split after three
/// characters beneath /PIONEER/Artwork/. Remove the leading slash before joining
/// to the share root. The result has no filename or trailing slash.
/// Invalid UUIDs raise ValueError.
#[pyfunction]
pub fn artwork_dir_local(uuid: &str) -> PyResult<String> {
    validate_uuid(uuid)?;
    Ok(rbox::core::artwork_dir_local(uuid))
}

/// Return the local artwork path, ending in artwork.jpg, for an artwork UUID.
///
/// Uses the same UUID normalization and directory layout as artwork_dir_local.
/// Remove the leading slash before joining to the share root.
/// Invalid UUIDs raise ValueError.
#[pyfunction]
pub fn image_path_local(uuid: &str) -> PyResult<String> {
    validate_uuid(uuid)?;
    Ok(rbox::core::share::image_path_local(uuid))
}

/// Return the USB artwork directory for an unsigned 32-bit artwork ID.
///
/// The directory number is artwork_id // 20 + 1, padded to at least five
/// decimal digits: IDs 1..19 use /PIONEER/Artwork/00001 and 20..39 use 00002.
/// Zero also formats into 00001, although it normally denotes absent artwork.
/// The result has no filename or trailing slash. Remove its leading slash before
/// joining it to the USB volume root. Existing stored artwork paths remain authoritative.
#[pyfunction]
pub fn artwork_dir_usb(artwork_id: u32) -> String {
    rbox::core::artwork_dir_usb(artwork_id)
}
