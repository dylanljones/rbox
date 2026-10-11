// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Directory naming for analysis and artwork files in rekordbox's shared storage.
//!
//! USB analysis directories are derived from the exported audio path using a
//! 32-bit hash. Local directories use a UUID split into two path components.
//! The public helpers return forward-slash paths with a leading `/PIONEER/`,
//! without a filename or trailing slash. They only construct strings and do not
//! inspect or create directories.
//!
//! Resolve USB paths beneath the USB volume root and local paths beneath the
//! configured analysis/share root. Remove the leading slash before joining to
//! a filesystem root so that the returned path does not replace that root.

use uuid::Uuid;

/// Compute the USB directory selector and reduced hash for an exported audio path.
///
/// `track_path` is relative to the USB volume root, with a leading slash, such as
/// `/Contents/Artist/Track.flac`. The text is processed exactly as supplied,
/// without changing case, separators, or Unicode normalization and without
/// appending a NUL terminator.
///
/// Starting at zero, the accumulator processes each UTF-16 code unit with two
/// multiply-add rounds: `hash * 0x5BC9 + unit`, then `hash * 0x93B5 + unit`.
/// Each round wraps modulo 2^32. Non-BMP characters contribute both surrogate
/// code units. The final accumulator is reduced modulo 200003 (`0x30D43`).
///
/// Returns `(p, h)`, where `h` is the reduced hash and `p` packs bits
/// 0, 2, 6, 7, 9, 13, and 16 of `h` into bits 0 through 6, respectively.
/// Thus `p` is in `0..=127` and `h` is in `0..200003`. The hash can collide;
/// this helper does not resolve collisions.
fn compute_usb_share_path_hash(track_path: &str) -> (u16, u32) {
    let mut hash = 0u32;
    for unit in track_path.encode_utf16() {
        let c = u32::from(unit);
        hash = hash
            .wrapping_mul(0x5bc9)
            .wrapping_add(c)
            .wrapping_mul(0x93b5)
            .wrapping_add(c);
    }
    let h = hash % 200003;
    let p = (h & 1)
        | ((h >> 1) & 2)
        | ((h >> 4) & 4)
        | ((h >> 4) & 8)
        | ((h >> 5) & 0x10)
        | ((h >> 8) & 0x20)
        | ((h >> 10) & 0x40);
    (p as u16, h)
}

/// Format the two USB directory components as `P{p:03X}/{h:08X}`.
///
/// The selector and reduced hash use uppercase, zero-padded hexadecimal.
/// The result has no leading or trailing slash. Input handling is described in
/// [`compute_usb_share_path_hash`].
fn compute_share_relpath_usb(track_path: &str) -> String {
    let (p, h) = compute_usb_share_path_hash(track_path);
    format!("P{p:03X}/{h:08X}")
}

/// Format a UUID as the two directory components used by local shared storage.
///
/// Parsing normalizes the UUID to lowercase hyphenated text. The first three
/// characters form the first directory and the remaining 33 form the second,
/// preserving the hyphens. For example, `0e80d4cc-06a6-4574-ba18-1c2301ea9924`
/// becomes `0e8/0d4cc-06a6-4574-ba18-1c2301ea9924`.
///
/// # Panics
///
/// Panics if `uuid` cannot be parsed as a UUID.
fn compute_share_relpath_local(uuid: &str) -> String {
    let uuid = Uuid::parse_str(uuid).expect("Invalid UUID");
    let text = uuid.hyphenated().to_string();
    format!("{}/{}", &text[..3], &text[3..])
}

/// Return the ANLZ directory for an audio file exported to a USB volume.
///
/// `track_path` must be the exported audio path relative to the volume root,
/// including its leading slash, for example `/Contents/Artist/Track.flac`.
/// It is hashed as supplied; this function does not normalize the input or
/// validate that it points to an exported file.
///
/// The result has the form `/PIONEER/USBANLZ/Pxxx/hhhhhhhh`, with uppercase
/// hexadecimal directory names. Append an analysis filename such as
/// `ANLZ0000.DAT`, `ANLZ0000.EXT`, or `ANLZ0000.2EX` as needed.
pub fn anlz_dir_usb(track_path: &str) -> String {
    let relpath = compute_share_relpath_usb(track_path);
    format!("/PIONEER/USBANLZ/{}", relpath)
}

/// Return the local ANLZ directory derived from a track's `djmdContent.UUID`.
///
/// The UUID is normalized to lowercase hyphenated text and split after three
/// characters beneath `/PIONEER/USBANLZ/`. Resolve the result relative to the
/// configured analysis/share root, then append the required ANLZ filename.
/// For existing tracks, the stored `AnalysisDataPath` remains authoritative.
///
/// # Panics
///
/// Panics if `uuid` cannot be parsed as a UUID.
pub fn anlz_dir_local(uuid: &str) -> String {
    let relpath = compute_share_relpath_local(uuid);
    format!("/PIONEER/USBANLZ/{}", relpath)
}

/// Return the local artwork directory derived from the supplied UUID.
///
/// The UUID is normalized to lowercase hyphenated text and split after three
/// characters beneath `/PIONEER/Artwork/`, using the same directory layout as
/// local analysis files. The result identifies a directory, not an image file;
/// the caller supplies the artwork UUID and chooses the filename separately.
///
/// # Panics
///
/// Panics if `uuid` cannot be parsed as a UUID.
pub fn artwork_dir_local(uuid: &str) -> String {
    let relpath = compute_share_relpath_local(uuid);
    format!("/PIONEER/Artwork/{}", relpath)
}
