// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Path normalization and manipulation utilities.
//!
//! This module provides functionality for normalizing and converting file system paths without
//! performing any I/O operations. It includes utilities to handle path components, remove
//! redundant separators, collapse up-level references (like `.` and `..`), and produce the
//! forward-slash paths stored internally by Rekordbox.
//!
//! The main functionality is exposed through the [`PathExt`] trait, which extends
//! the standard `Path` type with normalization methods.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use rbox::pathlib::PathExt;
//!
//! let path = Path::new("./some/path/../with/./redundant/components");
//! let normalized = path.normalize();
//! assert_eq!(normalized.to_str().unwrap(), "some/with/redundant/components");
//!
//! // Normalize with a specific separator
//! let windows_path = path.normalize_sep("\\");
//! assert_eq!(windows_path.to_str().unwrap(), "some\\with\\redundant\\components");
//!
//! let rekordbox_path = Path::new(r"some\artist\..\track.mp3").normalize_unix();
//! assert_eq!(rekordbox_path.to_str().unwrap(), "some/track.mp3");
//! ```
//!
//! Unlike `std::fs::canonicalize`, these functions do not resolve symlinks and do not
//! require the paths to actually exist on the filesystem.

use std::path::{Component, Path, PathBuf};

/// Normalize a path, removing things like `.` and `..`.
///
/// CAUTION: This does not resolve symlinks (unlike
/// [`std::fs::canonicalize`]). This may cause incorrect or surprising
/// behavior at times. This should be used carefully. Unfortunately,
/// [`std::fs::canonicalize`] can be hard to use correctly, since it can often
/// fail, or on Windows returns annoying device paths. This is a problem Cargo
/// needs to improve on.
pub fn normalize<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut components = path.as_ref().components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(Component::RootDir);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if ret.ends_with(Component::ParentDir) {
                    ret.push(Component::ParentDir);
                } else {
                    let popped = ret.pop();
                    if !popped && !ret.has_root() {
                        ret.push(Component::ParentDir);
                    }
                }
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

/// Same as [`normalize`] except that if
/// `Component::Prefix`/`Component::RootDir` is encountered,
/// or if the path points outside of current dir, returns `None`.
pub fn try_normalize<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let mut ret = PathBuf::new();
    for component in path.as_ref().components() {
        match component {
            Component::Prefix(..) | Component::RootDir => return None,
            Component::CurDir => {}
            Component::ParentDir => {
                if !ret.pop() {
                    return None;
                }
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }

    Some(ret)
}

/// Normalize a path with a given separator.
///
/// Same as [`normalize`] except it replaces the separators with the given one.
pub fn normalize_sep<P: AsRef<Path>>(path: P, seq: &str) -> PathBuf {
    let path = normalize(path);
    let s = path
        .into_os_string()
        .into_string()
        .expect("Failed to convert Path to String");
    if s.contains("\\") {
        return PathBuf::from(s.replace("\\", seq));
    } else if s.contains("/") {
        return PathBuf::from(s.replace("/", seq));
    }
    PathBuf::from(s).to_path_buf()
}

/// Convert path separators to forward slashes without otherwise normalizing the path.
///
/// This preserves `.`, `..`, repeated separators, roots, and Windows drive prefixes. It also
/// preserves non-Unicode path data on Unix and Windows.
pub fn to_unix<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();

    #[cfg(unix)]
    {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};

        let bytes = path
            .as_os_str()
            .as_bytes()
            .iter()
            .map(|byte| if *byte == b'\\' { b'/' } else { *byte })
            .collect();
        return PathBuf::from(std::ffi::OsString::from_vec(bytes));
    }

    #[cfg(windows)]
    {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};

        let wide = path
            .as_os_str()
            .encode_wide()
            .map(|unit| {
                if unit == b'\\' as u16 {
                    b'/' as u16
                } else {
                    unit
                }
            })
            .collect::<Vec<_>>();
        return PathBuf::from(std::ffi::OsString::from_wide(&wide));
    }

    #[cfg(not(any(unix, windows)))]
    {
        PathBuf::from(path.to_string_lossy().replace('\\', "/"))
    }
}

/// Normalize a path without I/O and convert its separators to forward slashes.
///
/// Separator conversion happens before normalization so Windows-style input is handled
/// consistently on every platform. The returned path is suitable for Rekordbox database and
/// analysis-file fields.
pub fn normalize_unix<P: AsRef<Path>>(path: P) -> PathBuf {
    let unix = to_unix(path);
    to_unix(normalize(unix))
}

/// Return `true` if the path is normalized.
///
/// # Quirk
///
/// If the path does not start with `./` but contains `./` in the middle,
/// then this function might return `true`.
fn is_normalized(path: &Path) -> bool {
    for component in path.components() {
        match component {
            Component::CurDir | Component::ParentDir => {
                return false;
            }
            _ => continue,
        }
    }
    true
}

/// Extension trait for normalizing paths and converting them to Rekordbox's path format.
///
/// Implementors only need to provide [`AsRef<Path>`]. All methods operate on that borrowed path,
/// avoiding an intermediate copy.
pub trait PathExt: AsRef<Path> {
    /// Normalize a path without performing I/O.
    ///
    /// All redundant separator and up-level references are collapsed.
    ///
    /// However, this does not resolve links.
    fn normalize(&self) -> PathBuf {
        normalize(self.as_ref())
    }

    /// Normalize the path and return it as a Unicode string.
    ///
    /// Returns `None` if the normalized path contains non-Unicode data.
    fn normalize_string(&self) -> Option<String> {
        normalize(self.as_ref()).into_os_string().into_string().ok()
    }

    /// Same as [`PathExt::normalize`] except that if
    /// `Component::Prefix`/`Component::RootDir` is encountered,
    /// or if the path points outside of current dir, returns `None`.
    fn try_normalize(&self) -> Option<PathBuf> {
        try_normalize(self.as_ref())
    }

    /// Normalize a path with a given separator without performing I/O.
    ///
    /// All redundant separator and up-level references are collapsed.
    /// The given separator is used to normalize the path.
    fn normalize_sep(&self, seq: &str) -> PathBuf {
        normalize_sep(self.as_ref(), seq)
    }

    /// Convert path separators to forward slashes without normalizing path components.
    ///
    /// This is useful when a path must use Rekordbox's internal separator convention while
    /// preserving its existing `.`, `..`, and repeated separators.
    fn to_unix(&self) -> PathBuf {
        to_unix(self.as_ref())
    }

    /// Convert path separators to forward slashes and return the path as a Unicode string.
    ///
    /// Path components are not normalized. Returns `None` if the path contains non-Unicode data.
    fn to_unix_string(&self) -> Option<String> {
        Some(self.as_ref().to_str()?.replace('\\', "/"))
    }

    /// Normalize the path without I/O and convert its separators to forward slashes.
    ///
    /// This is the preferred conversion for paths written to Rekordbox databases and ANLZ files.
    fn normalize_unix(&self) -> PathBuf {
        normalize_unix(self.as_ref())
    }

    /// Normalize the path, convert its separators to forward slashes, and return a Unicode string.
    ///
    /// Returns `None` if the normalized path contains non-Unicode data.
    fn normalize_unix_string(&self) -> Option<String> {
        normalize_unix(self.as_ref())
            .into_os_string()
            .into_string()
            .ok()
    }

    /// Return `true` if the path is normalized.
    ///
    /// # Quirk
    ///
    /// If the path does not start with `./` but contains `./` in the middle,
    /// then this function might return `true`.
    fn is_normalized(&self) -> bool {
        is_normalized(self.as_ref())
    }
}

/// Backwards-compatible name for [`PathExt`].
#[deprecated(since = "0.7.0", note = "use `PathExt` instead")]
pub use PathExt as NormalizePath;

/// Implement `PathExt` for `Path` so that all methods can be called directly on `Path` instances.
impl PathExt for Path {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_separators_without_normalizing_components() {
        let path = Path::new(r"Music\Artist\.\Album\..\Track.mp3");

        assert_eq!(
            path.to_unix(),
            PathBuf::from("Music/Artist/./Album/../Track.mp3")
        );
    }

    #[test]
    fn normalizes_windows_style_input_to_a_rekordbox_path() {
        let path = Path::new(r"Music\Artist\.\Album\..\Track.mp3");

        assert_eq!(
            path.normalize_unix(),
            PathBuf::from("Music/Artist/Track.mp3")
        );
    }

    #[test]
    fn converts_paths_to_strings() {
        let path = Path::new(r"Music\Artist\.\Album\..\Track.mp3");

        assert_eq!(
            path.normalize_string(),
            path.normalize().to_str().map(str::to_owned)
        );
        assert_eq!(
            path.to_unix_string().as_deref(),
            Some("Music/Artist/./Album/../Track.mp3")
        );
        assert_eq!(
            path.normalize_unix_string().as_deref(),
            Some("Music/Artist/Track.mp3")
        );
    }

    #[cfg(unix)]
    #[test]
    fn string_conversions_reject_non_unicode_paths() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(OsString::from_vec(vec![b'M', 0xff]));

        assert_eq!(path.normalize_string(), None);
        assert_eq!(path.to_unix_string(), None);
        assert_eq!(path.normalize_unix_string(), None);
    }

    #[test]
    fn preserves_unix_roots_and_forward_slashes() {
        let path = Path::new("/Contents/Artist/../Track.flac");

        assert_eq!(path.to_unix(), path);
        assert_eq!(path.normalize_unix(), PathBuf::from("/Contents/Track.flac"));
    }
}
