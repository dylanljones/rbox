// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared-storage path helpers exposed to JavaScript.

use napi::{Error, Result, Status};
use napi_derive::napi;

fn validate_uuid(value: &str) -> Result<()> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|err| Error::new(Status::InvalidArg, format!("Invalid UUID: {err}")))
}

/// Return the USB ANLZ directory derived from the exported audio path.
///
/// Supply the volume-relative path with a leading slash, for example
/// /Contents/Artist/Track.flac. Input is hashed without normalization.
/// Remove the result's leading slash before joining to the USB root, then
/// append an analysis filename such as ANLZ0000.DAT, ANLZ0000.EXT or ANLZ0000.2EX.
#[napi(strict)]
pub fn anlz_dir_usb(track_path: String) -> String {
    rbox::core::anlz_dir_usb(&track_path)
}

/// Return the local ANLZ directory derived from a track's djmdContent.UUID.
///
/// The UUID is normalized to lowercase hyphenated text and split after three
/// characters beneath /PIONEER/USBANLZ/. Remove the leading slash before joining
/// to the analysis/share root, then append the analysis filename. Existing
/// AnalysisDataPath values remain authoritative. Invalid UUIDs throw InvalidArg.
#[napi(strict)]
pub fn anlz_dir_local(uuid: String) -> Result<String> {
    validate_uuid(&uuid)?;
    Ok(rbox::core::anlz_dir_local(&uuid))
}

/// Return the local artwork directory derived from an artwork UUID.
///
/// The UUID is normalized to lowercase hyphenated text and split after three
/// characters beneath /PIONEER/Artwork/. Remove the leading slash before joining
/// to the share root. The result has no filename or trailing slash.
/// Invalid UUIDs throw InvalidArg.
#[napi(strict)]
pub fn artwork_dir_local(uuid: String) -> Result<String> {
    validate_uuid(&uuid)?;
    Ok(rbox::core::artwork_dir_local(&uuid))
}

/// Return the local artwork path, ending in artwork.jpg, for an artwork UUID.
///
/// Uses the same UUID normalization and directory layout as artworkDirLocal.
/// Remove the leading slash before joining to the share root.
/// Invalid UUIDs throw InvalidArg.
#[napi(strict)]
pub fn image_path_local(uuid: String) -> Result<String> {
    validate_uuid(&uuid)?;
    Ok(rbox::core::share::image_path_local(&uuid))
}

/// Return the USB artwork directory for an integer artwork ID in 0..=4294967295.
///
/// The directory number is Math.floor(artworkId / 20) + 1, padded to at least five
/// decimal digits: IDs 1..19 use /PIONEER/Artwork/00001 and 20..39 use 00002.
/// Zero also formats into 00001, although it normally denotes absent artwork.
/// The result has no filename or trailing slash. Remove its leading slash before
/// joining it to the USB volume root. Existing stored artwork paths remain authoritative.
#[napi(strict)]
pub fn artwork_dir_usb(artwork_id: f64) -> Result<String> {
    // N-API's direct u32 conversion truncates/wraps numbers. Reject invalid IDs
    // before conversion so the two language bindings accept the same ID range.
    if !artwork_id.is_finite()
        || artwork_id.fract() != 0.0
        || !(0.0..=f64::from(u32::MAX)).contains(&artwork_id)
    {
        return Err(Error::new(
            Status::InvalidArg,
            "artworkId must be an integer in 0..=4294967295",
        ));
    }
    Ok(rbox::core::artwork_dir_usb(artwork_id as u32))
}
