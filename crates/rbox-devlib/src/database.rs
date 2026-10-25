// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Lifecycle of a Device Library's standard and extended DeviceSQL files.

use std::fs::{self, File};
use std::io::{self, Read, Seek};
use std::path::Path;

use crate::devicesql::{CreateOptions, DeviceSqlConnection, PageSaveStats, SaveStats};
use crate::{DatabaseKind, Error, Result};

const STANDARD_FILE: &str = "export.pdb";
const EXTENDED_FILE: &str = "exportExt.pdb";

/// Metadata for initializing both files of a new Device Library.
///
/// Creation uses the engine's populated profiles. No metadata or rows are copied
/// from an existing export. Text is validated when creating the library.
#[derive(Debug, Clone)]
pub struct LibraryCreateOptions {
    /// Export date, conventionally `YYYY-MM-DD`; not calendar-validated.
    pub date: String,
    /// Device name stored in the standard database's export properties.
    pub device_name: String,
    /// Source library identifier stored in the extended database's My Tag properties.
    pub my_tag_master_db_id: u32,
}

impl LibraryCreateOptions {
    /// Set creation metadata. Encoding validation happens in [`DeviceLibrary::create`].
    pub fn new(
        date: impl Into<String>,
        device_name: impl Into<String>,
        my_tag_master_db_id: u32,
    ) -> Self {
        Self {
            date: date.into(),
            device_name: device_name.into(),
            my_tag_master_db_id,
        }
    }

    /// Generate engine creation options for the standard file.
    pub fn standard_options(&self) -> Result<CreateOptions> {
        CreateOptions::standard(&self.date, &self.device_name)
    }

    /// Generate engine creation options for the extended file.
    pub fn extended_options(&self) -> Result<CreateOptions> {
        CreateOptions::extended(self.my_tag_master_db_id)
    }
}

/// Per-file results of a successful save. An absent extended file stays absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LibrarySaveStats<T = SaveStats> {
    /// Standard-file write statistics, including zero-write saves.
    pub standard: T,
    /// Extended-file statistics, or `None` for a standard-only library.
    pub extended: Option<T>,
}

/// A Device Library backed by `export.pdb` and optionally `exportExt.pdb`.
///
/// Opening reads file headers, leaving pages and models lazy. Mutations through
/// the engine accessors stay in memory until explicitly saved. Dropping the
/// handler does not save. The application must exclude external writers.
///
/// Writes are performed in standard-then-extended order, not atomically across
/// the pair. [`Error::LibraryWrite`] reports the failing file and whether the
/// standard-file operation completed. Backups and recovery belong to the caller.
/// This handler does not copy audio, artwork, or analysis files.
///
/// ```no_run
/// use rbox_devlib::{DeviceLibrary, LibraryCreateOptions};
/// # fn main() -> rbox_devlib::Result<()> {
/// let options = LibraryCreateOptions::new("2026-10-25", "My USB", 1);
/// let mut library = DeviceLibrary::create("output/PIONEER/rekordbox", &options)?;
/// let stats = library.save()?; // No edits: no writes.
/// assert_eq!(stats.standard.bytes_written, 0);
/// library.save_as("copy/PIONEER/rekordbox")?;
/// # Ok(())
/// # }
/// ```
pub struct DeviceLibrary {
    standard: DeviceSqlConnection<File>,
    extended: Option<DeviceSqlConnection<File>>,
}

impl DeviceLibrary {
    /// Open the directory containing `export.pdb` and its optional extended file.
    ///
    /// Pass the database directory, usually `PIONEER/rekordbox`, not the USB root.
    /// Only a missing extended directory entry is ignored. An unreadable or
    /// malformed extended header returns an error; page validation remains lazy.
    pub fn new(directory: impl AsRef<Path>) -> Result<Self> {
        let directory = directory.as_ref();
        let extended = directory.join(EXTENDED_FILE);
        let extended = match fs::symlink_metadata(&extended) {
            Ok(_) => Some(extended.as_path()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        Self::from_paths(directory.join(STANDARD_FILE), extended)
    }

    /// Open explicitly named files. `None` opens only the standard file.
    ///
    /// An explicitly supplied extended path must exist. Files must be distinct;
    /// canonical path aliases are rejected. The caller must also avoid hard-link
    /// aliases. The binary header cannot reliably identify the file namespace.
    pub fn from_paths(standard: impl AsRef<Path>, extended: Option<&Path>) -> Result<Self> {
        let standard_path = standard.as_ref().canonicalize()?;
        let extended_path = extended.map(Path::canonicalize).transpose()?;
        if extended_path.as_ref() == Some(&standard_path) {
            return Err(Error::AliasedDatabaseFiles(standard_path));
        }
        Ok(Self {
            standard: DeviceSqlConnection::open(standard_path, DatabaseKind::Standard)?,
            extended: extended_path
                .map(|path| DeviceSqlConnection::open(path, DatabaseKind::Extended))
                .transpose()?,
        })
    }

    /// Create and open both files using the engine's populated initialization profiles.
    ///
    /// Both images are initialized in memory before creating directories or
    /// writing files. Neither destination may exist. Publication is sequential;
    /// a failure can leave the standard file behind, as reported by
    /// [`Error::LibraryWrite`]. A later reopen failure leaves both files in place.
    pub fn create(directory: impl AsRef<Path>, options: &LibraryCreateOptions) -> Result<Self> {
        let mut standard = DeviceSqlConnection::create_with_options(
            DatabaseKind::Standard,
            &options.standard_options()?,
        )?;
        let mut extended = DeviceSqlConnection::create_with_options(
            DatabaseKind::Extended,
            &options.extended_options()?,
        )?;
        write_pair(
            directory.as_ref(),
            &mut standard,
            Some(&mut extended),
            "create",
        )?;
        Self::new(directory)
    }

    /// Whether this handler has an extended database connection.
    pub fn has_extended(&self) -> bool {
        self.extended.is_some()
    }

    /// Inspect the standard engine's header, resource usage, and read statistics.
    pub fn standard(&self) -> &DeviceSqlConnection<File> {
        &self.standard
    }

    /// Access standard-file reads and binary transactions directly.
    ///
    /// Direct engine edits do not enforce model or cross-file relationships.
    pub fn standard_mut(&mut self) -> &mut DeviceSqlConnection<File> {
        &mut self.standard
    }

    /// Inspect the extended engine, or report that this library lacks the file.
    pub fn extended(&self) -> Result<&DeviceSqlConnection<File>> {
        self.extended.as_ref().ok_or(Error::MissingExtendedDatabase)
    }

    /// Access extended-file reads and binary transactions directly.
    ///
    /// Direct engine edits do not enforce model or cross-file relationships.
    pub fn extended_mut(&mut self) -> Result<&mut DeviceSqlConnection<File>> {
        self.extended.as_mut().ok_or(Error::MissingExtendedDatabase)
    }

    /// Replace each changed source file using the engine's atomic file save.
    ///
    /// Unchanged files perform no writes. Each replacement is individually
    /// atomic; the two replacements are not atomic as a pair.
    pub fn save(&mut self) -> Result<LibrarySaveStats> {
        self.save_with("save", DeviceSqlConnection::save)
    }

    /// Write only changed/new pages to each source file, then sync it.
    ///
    /// This is non-atomic even within one file. After a failed direct write,
    /// follow [`DeviceSqlConnection::save_pages`]'s recovery contract: discard
    /// the affected connection and reopen after restoring or repairing its file.
    pub fn save_pages(&mut self) -> Result<LibrarySaveStats<PageSaveStats>> {
        self.save_with("save_pages", DeviceSqlConnection::save_pages)
    }

    /// Copy the current database images into a directory without overwriting.
    ///
    /// Both destination names must be absent, even for a standard-only library.
    /// A missing extended database remains missing. Sources and pending changes
    /// are retained: this does not switch the handler to the destination files.
    /// Partial publication is reported by [`Error::LibraryWrite`].
    pub fn save_as(&mut self, directory: impl AsRef<Path>) -> Result<()> {
        write_pair(
            directory.as_ref(),
            &mut self.standard,
            self.extended.as_mut(),
            "save_as",
        )
    }

    fn save_with<T>(
        &mut self,
        operation: &'static str,
        mut save: impl FnMut(&mut DeviceSqlConnection<File>) -> Result<T>,
    ) -> Result<LibrarySaveStats<T>> {
        let standard = save(&mut self.standard)
            .map_err(|source| write_error(operation, DatabaseKind::Standard, false, source))?;
        let extended = self
            .extended
            .as_mut()
            .map(|connection| {
                save(connection)
                    .map_err(|source| write_error(operation, DatabaseKind::Extended, true, source))
            })
            .transpose()?;
        Ok(LibrarySaveStats { standard, extended })
    }
}

fn write_error(
    operation: &'static str,
    database: DatabaseKind,
    standard_completed: bool,
    source: Error,
) -> Error {
    Error::LibraryWrite {
        operation,
        database,
        standard_completed,
        source: Box::new(source),
    }
}

fn write_pair<R: Read + Seek>(
    directory: &Path,
    standard: &mut DeviceSqlConnection<R>,
    extended: Option<&mut DeviceSqlConnection<R>>,
    operation: &'static str,
) -> Result<()> {
    // Preflight both names before publishing anything. Engine save_as also
    // enforces no-clobber publication if a destination appears after this check.
    for (kind, name) in [
        (DatabaseKind::Standard, STANDARD_FILE),
        (DatabaseKind::Extended, EXTENDED_FILE),
    ] {
        let result = match fs::symlink_metadata(directory.join(name)) {
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "destination already exists",
            )),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        };
        result.map_err(|error| write_error(operation, kind, false, error.into()))?;
    }
    fs::create_dir_all(directory)?;
    standard
        .save_as(directory.join(STANDARD_FILE))
        .map_err(|source| write_error(operation, DatabaseKind::Standard, false, source))?;
    if let Some(extended) = extended {
        extended
            .save_as(directory.join(EXTENDED_FILE))
            .map_err(|source| write_error(operation, DatabaseKind::Extended, true, source))?;
    }
    Ok(())
}
