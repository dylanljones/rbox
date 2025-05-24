// Author: Dylan Jones
// Date:   2025-05-13

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// Normalize a path, removing things like `.` and `..`.
///
/// CAUTION: This does not resolve symlinks (unlike
/// [`std::fs::canonicalize`]). This may cause incorrect or surprising
/// behavior at times. This should be used carefully. Unfortunately,
/// [`std::fs::canonicalize`] can be hard to use correctly, since it can often
/// fail, or on Windows returns annoying device paths. This is a problem Cargo
/// needs to improve on.
pub fn normalize<P: AsRef<OsStr>>(path: P) -> PathBuf {
    let path = Path::new(&path);
    let mut components = path.components().peekable();
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

/// Same as [`normalize_path`] except that if
/// `Component::Prefix`/`Component::RootDir` is encountered,
/// or if the path points outside of current dir, returns `None`.
fn try_normalize<P: AsRef<OsStr>>(path: P) -> Option<PathBuf> {
    let path = Path::new(&path);
    let mut ret = PathBuf::new();

    for component in path.components() {
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

/// Normalize a path with a given seperator.
///
/// Same as [`normalize_path`] except it replaces the seperators with the given one.
pub fn normalize_sep<P: AsRef<OsStr>>(path: P, seq: &str) -> PathBuf {
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

/// Return `true` if the path is normalized.
///
/// # Quirk
///
/// If the path does not start with `./` but contains `./` in the middle,
/// then this function might returns `true`.
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

/// Extension trait to add `normalize_path` to std's [`Path`].
pub trait NormalizePath {
    /// Normalize a path without performing I/O.
    ///
    /// All redundant separator and up-level references are collapsed.
    ///
    /// However, this does not resolve links.
    fn normalize(&self) -> PathBuf;

    /// Same as [`NormalizePath::normalize`] except that if
    /// `Component::Prefix`/`Component::RootDir` is encountered,
    /// or if the path points outside of current dir, returns `None`.
    fn try_normalize(&self) -> Option<PathBuf>;

    /// Normalize a path with a given seperator without performing I/O.
    ///
    /// All redundant separator and up-level references are collapsed.
    /// The given seperator is used to normalize the path.
    fn normalize_sep(&self, seq: &str) -> PathBuf;

    /// Return `true` if the path is normalized.
    ///
    /// # Quirk
    ///
    /// If the path does not start with `./` but contains `./` in the middle,
    /// then this function might returns `true`.
    fn is_normalized(&self) -> bool;
}

impl NormalizePath for Path {
    fn normalize(&self) -> PathBuf {
        normalize(self)
    }

    fn try_normalize(&self) -> Option<PathBuf> {
        try_normalize(self)
    }

    fn normalize_sep(&self, seq: &str) -> PathBuf {
        normalize_sep(self, seq)
    }

    fn is_normalized(&self) -> bool {
        is_normalized(self)
    }
}
