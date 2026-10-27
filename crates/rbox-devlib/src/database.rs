// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Device Library lifecycle and model operations over its standard and extended files.

use std::fs::{self, File};
use std::io::{self, Read, Seek};
use std::path::Path;

use rbox_core::{PathExt, PlaylistType, Track};

use crate::models::playlist::DevPlaylistTreeNode;

use crate::devicesql::{CreateOptions, DeviceSqlConnection, PageSaveStats, SaveStats};
use crate::models::{
    checked_number, DevAlbum, DevAlbumView, DevArtist, DevArtwork, DevBrowseCategory,
    DevBrowseColumn, DevBrowseSort, DevColor, DevContent, DevContentView, DevDatabaseProperty,
    DevExportProperty, DevGenre, DevHistory, DevHistoryContent, DevKey, DevLabel, DevMyTag,
    DevMyTagContent, DevPlaylist, DevPlaylistContent, DevRecommendedLike, NewDevAlbum,
    NewDevArtist, NewDevArtwork, NewDevContent, NewDevGenre, NewDevHistory, NewDevHistoryContent,
    NewDevKey, NewDevLabel, NewDevMyTag, NewDevMyTagContent, NewDevPlaylist, NewDevPlaylistContent,
    NewDevRecommendedLike,
};
use crate::{DatabaseKind, Error, Result};

const STANDARD_FILE: &str = "export.pdb";
const EXTENDED_FILE: &str = "exportExt.pdb";

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
    pub my_tag_master_db_id: i64,
}

impl LibraryCreateOptions {
    /// Set creation metadata. Encoding validation happens in [`DeviceLibrary::create`].
    pub fn new(
        date: impl Into<String>,
        device_name: impl Into<String>,
        my_tag_master_db_id: i64,
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
        CreateOptions::extended(checked_number(
            self.my_tag_master_db_id,
            "my_tag_master_db_id",
        )?)
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
/// the models or engine accessors stay in memory until explicitly saved. Dropping the
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

    // -- Album ------------------------------------------------------------------------------------

    /// Load all albums in physical order without resolving related rows.
    pub fn get_albums(&mut self) -> Result<Vec<DevAlbum>> {
        DevAlbum::all(&mut self.standard)
    }

    /// Find an album by export ID; returns `None` when absent.
    pub fn get_album_by_id(&mut self, id: i64) -> Result<Option<DevAlbum>> {
        DevAlbum::find(&mut self.standard, &id)
    }

    /// Find the first exact, case-sensitive name match in physical order.
    ///
    /// Albums with different artists can share a name; this does not filter by artist.
    pub fn get_album_by_name(&mut self, name: &str) -> Result<Option<DevAlbum>> {
        DevAlbum::find_by_name(&mut self.standard, name)
    }

    /// Load an album and its album artist. Missing albums return `None`.
    ///
    /// Missing or unassigned artists remain `None` in the detached view.
    pub fn get_album_with_relations(&mut self, id: i64) -> Result<Option<DevAlbumView>> {
        DevAlbum::find_with_relations(&mut self.standard, &id)
    }

    /// Load all albums in physical order and resolve shared album artists in a batch.
    pub fn get_albums_with_relations(&mut self) -> Result<Vec<DevAlbumView>> {
        DevAlbum::all_with_relations(&mut self.standard)
    }

    /// Allocate an album ID and insert it, enforcing unique `(name, artist_id)` pairs.
    ///
    /// Changes remain in memory until explicitly saved.
    pub fn insert_album(&mut self, album: NewDevAlbum) -> Result<DevAlbum> {
        album.insert(&mut self.standard)
    }

    /// Create an album with an automatically allocated export ID and insert it in memory.
    ///
    /// `artist_id` is the album artist; `None` or zero means unassigned.
    pub fn create_album(
        &mut self,
        name: impl Into<String>,
        artist_id: Option<i64>,
    ) -> Result<DevAlbum> {
        let artist_id = artist_id.unwrap_or(0);
        self.insert_album(NewDevAlbum::new(name).artist_id(artist_id))
    }

    /// Persist edited album fields in memory and return a refreshed snapshot.
    /// Preserves unedited binary data and enforces the model's uniqueness checks.
    pub fn update_album(&mut self, album: DevAlbum) -> Result<DevAlbum> {
        album.update(&mut self.standard)
    }

    /// Delete an album and clear referencing contents' album IDs atomically in memory.
    /// Returns the total affected rows, or zero for a missing album. Saving is explicit.
    pub fn delete_album(&mut self, id: i64) -> Result<usize> {
        DevAlbum::delete(&mut self.standard, &id)
    }

    // -- Artist -----------------------------------------------------------------------------------

    /// Load all artists in physical order without resolving related rows.
    pub fn get_artists(&mut self) -> Result<Vec<DevArtist>> {
        DevArtist::all(&mut self.standard)
    }

    /// Find an artist by export ID; returns `None` when absent.
    pub fn get_artist_by_id(&mut self, id: i64) -> Result<Option<DevArtist>> {
        DevArtist::find(&mut self.standard, &id)
    }

    /// Find the first exact, case-sensitive name match; returns `None` when absent.
    pub fn get_artist_by_name(&mut self, name: &str) -> Result<Option<DevArtist>> {
        DevArtist::find_by_name(&mut self.standard, name)
    }

    /// Allocate an artist ID and insert it, enforcing unique, case-sensitive names.
    ///
    /// Changes remain in memory until explicitly saved.
    pub fn insert_artist(&mut self, artist: NewDevArtist) -> Result<DevArtist> {
        artist.insert(&mut self.standard)
    }

    /// Create an artist with an automatically allocated export ID and insert it in memory.
    pub fn create_artist(&mut self, name: impl Into<String>) -> Result<DevArtist> {
        self.insert_artist(NewDevArtist::new(name))
    }

    /// Persist an edited artist name in memory and return a refreshed snapshot.
    ///
    /// Preserves unedited binary data and enforces name uniqueness; saving is explicit.
    pub fn update_artist(&mut self, artist: DevArtist) -> Result<DevArtist> {
        artist.update(&mut self.standard)
    }

    /// Delete an artist and clear its album and content references atomically in memory.
    ///
    /// Clears album artist, main artist, composer, original artist, and remixer IDs.
    /// Returns the total affected rows, or zero for a missing artist. Saving is explicit.
    pub fn delete_artist(&mut self, id: i64) -> Result<usize> {
        DevArtist::delete(&mut self.standard, &id)
    }

    // -- Artwork ----------------------------------------------------------------------------------

    /// Load all artwork references in physical order without resolving related rows.
    pub fn get_artworks(&mut self) -> Result<Vec<DevArtwork>> {
        DevArtwork::all(&mut self.standard)
    }

    /// Find an artwork reference by export ID; returns `None` when absent.
    pub fn get_artwork_by_id(&mut self, id: i64) -> Result<Option<DevArtwork>> {
        DevArtwork::find(&mut self.standard, &id)
    }

    /// Find the first exact, case-sensitive path match; returns `None` when absent.
    ///
    /// Paths are compared as stored, without normalization or filesystem access.
    pub fn get_artwork_by_path(&mut self, path: &str) -> Result<Option<DevArtwork>> {
        DevArtwork::find_by_path(&mut self.standard, path)
    }

    /// Allocate an artwork ID and insert a reference to a nonempty image path.
    ///
    /// Duplicate paths are allowed. Changes remain in memory; no image file is copied.
    pub fn insert_artwork(&mut self, artwork: NewDevArtwork) -> Result<DevArtwork> {
        artwork.insert(&mut self.standard)
    }

    /// Create an artwork reference with an automatically allocated ID and insert it in memory.
    pub fn create_artwork(&mut self, path: impl Into<String>) -> Result<DevArtwork> {
        self.insert_artwork(NewDevArtwork::new(path))
    }

    /// Persist an edited artwork path in memory and return a refreshed snapshot.
    ///
    /// Saving is explicit; this does not copy or rename image files.
    pub fn update_artwork(&mut self, artwork: DevArtwork) -> Result<DevArtwork> {
        artwork.update(&mut self.standard)
    }

    /// Delete an artwork reference and clear matching content artwork IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero when absent. Image files are retained;
    /// saving database changes remains explicit.
    pub fn delete_artwork(&mut self, id: i64) -> Result<usize> {
        DevArtwork::delete(&mut self.standard, &id)
    }

    // -- Browse -----------------------------------------------------------------------------------

    /// Load all predefined browse labels in physical order without resolving settings.
    pub fn get_browse_columns(&mut self) -> Result<Vec<DevBrowseColumn>> {
        DevBrowseColumn::all(&mut self.standard)
    }

    /// Find a browse label by its 16-bit catalog ID; returns `None` when absent.
    pub fn get_browse_column_by_id(&mut self, id: i64) -> Result<Option<DevBrowseColumn>> {
        DevBrowseColumn::find(&mut self.standard, &id)
    }

    /// Find the first exact, case-sensitive plain-text label match, or `None`.
    ///
    /// Enclosing U+FFFA/U+FFFB markers are omitted from the compared label.
    pub fn get_browse_column_by_name(&mut self, name: &str) -> Result<Option<DevBrowseColumn>> {
        DevBrowseColumn::find_by_name(&mut self.standard, name)
    }

    /// Update a browse label in memory, preserving its enclosing markers.
    ///
    /// Catalog IDs and codes cannot change; saving remains explicit.
    pub fn update_browse_column(&mut self, column: DevBrowseColumn) -> Result<DevBrowseColumn> {
        column.update(&mut self.standard)
    }

    /// Load all category-menu settings in physical order, retaining repeated rows.
    pub fn get_browse_categories(&mut self) -> Result<Vec<DevBrowseCategory>> {
        DevBrowseCategory::all(&mut self.standard)
    }

    /// Load all category settings for a column ID, retaining physical order and repetitions.
    ///
    /// Returns an empty list when absent; referenced labels are not loaded.
    pub fn get_browse_categories_by_column(
        &mut self,
        column_id: i64,
    ) -> Result<Vec<DevBrowseCategory>> {
        DevBrowseCategory::by_column(&mut self.standard, &column_id)
    }

    /// Update one category-setting occurrence in memory; saving remains explicit.
    ///
    /// Column and selector cannot change. No automatic reordering or state coercion
    /// is performed. Stale snapshots are rejected; reload after rebuilding the database.
    pub fn update_browse_category(
        &mut self,
        category: DevBrowseCategory,
    ) -> Result<DevBrowseCategory> {
        category.update(&mut self.standard)
    }

    /// Load all sort-option settings in physical order, retaining repeated rows.
    pub fn get_browse_sorts(&mut self) -> Result<Vec<DevBrowseSort>> {
        DevBrowseSort::all(&mut self.standard)
    }

    /// Load all sort settings for a column ID, retaining physical order and repetitions.
    ///
    /// Returns an empty list when absent; referenced labels are not loaded.
    pub fn get_browse_sorts_by_column(&mut self, column_id: i64) -> Result<Vec<DevBrowseSort>> {
        DevBrowseSort::by_column(&mut self.standard, &column_id)
    }

    /// Update one sort-setting occurrence's raw tail in memory; saving remains explicit.
    ///
    /// Column and selector cannot change. Stale snapshots are rejected; reload after
    /// rebuilding. Tail-bit meanings remain provisional; see [`crate::models::browse`].
    pub fn update_browse_sort(&mut self, sort: DevBrowseSort) -> Result<DevBrowseSort> {
        sort.update(&mut self.standard)
    }

    // -- Color ------------------------------------------------------------------------------------

    /// Load all initialized color labels in physical order without resolving related rows.
    pub fn get_colors(&mut self) -> Result<Vec<DevColor>> {
        DevColor::all(&mut self.standard)
    }

    /// Find a color label by its 16-bit export ID; returns `None` when absent.
    pub fn get_color_by_id(&mut self, id: i64) -> Result<Option<DevColor>> {
        DevColor::find(&mut self.standard, &id)
    }

    /// Find the first exact, case-sensitive color name match, or `None` when absent.
    pub fn get_color_by_name(&mut self, name: &str) -> Result<Option<DevColor>> {
        DevColor::find_by_name(&mut self.standard, name)
    }

    /// Update a color name in memory and return a refreshed snapshot.
    ///
    /// Names must be unique (case-sensitive); existing IDs cannot change.
    /// Colors are initialized at creation and support updates only. Saving remains explicit.
    pub fn update_color(&mut self, color: DevColor) -> Result<DevColor> {
        color.update(&mut self.standard)
    }

    // -- Content ----------------------------------------------------------------------------------

    /// Load shared tracks in physical order with resolved metadata, including album artists.
    ///
    /// Each distinct related row is decoded once per call; missing metadata stays `None`.
    /// BPM is unscaled, duration is seconds, and file size is bytes. Source-library IDs
    /// come from each content row. Decoding and metadata conversion errors propagate.
    /// Results are detached snapshots; audio and analysis files are not read.
    pub fn get_tracks(&mut self) -> Result<Vec<Track>> {
        self.read_tracks(DevContent::all)
    }

    /// Load a shared track by its local export ID, or `None` if absent.
    ///
    /// See [`Self::get_tracks`] for relationship resolution, units and errors.
    pub fn get_track_by_id(&mut self, id: i64) -> Result<Option<Track>> {
        Ok(self
            .read_tracks(|conn| Ok(DevContent::find(conn, &id)?.into_iter().collect()))?
            .pop())
    }

    /// Load matching shared tracks in ascending local export-ID order.
    ///
    /// Missing IDs are omitted, repeated IDs occur once, and empty input reads no pages.
    /// See [`Self::get_tracks`] for relationship resolution, units and errors.
    pub fn get_tracks_by_ids(&mut self, ids: &[i64]) -> Result<Vec<Track>> {
        self.read_tracks(|conn| DevContent::by_ids(conn, ids))
    }

    /// Load a shared track by its stored audio path, or `None` if absent.
    ///
    /// Backslashes are converted to forward slashes without accessing the file;
    /// the path must otherwise match exactly. Repeated paths return the first
    /// physical occurrence. See [`Self::get_tracks`] for conversion errors.
    pub fn get_track_by_path(&mut self, path: &str) -> Result<Option<Track>> {
        let path = Path::new(path).to_unix();
        Ok(self
            .read_tracks(|conn| {
                Ok(DevContent::find_by_path(conn, &path.to_string_lossy())?
                    .into_iter()
                    .collect())
            })?
            .pop())
    }

    fn read_tracks(
        &mut self,
        load: impl FnOnce(&mut DeviceSqlConnection<File>) -> Result<Vec<DevContent>>,
    ) -> Result<Vec<Track>> {
        let contents = load(&mut self.standard)?;
        DevContent::resolve_many(&mut self.standard, contents)?
            .into_iter()
            .map(Track::try_from)
            .collect()
    }

    /// Load all content rows in physical order without resolving related metadata.
    pub fn get_contents(&mut self) -> Result<Vec<DevContent>> {
        DevContent::all(&mut self.standard)
    }

    /// Find content by its local export ID; returns `None` when absent.
    pub fn get_content_by_id(&mut self, id: i64) -> Result<Option<DevContent>> {
        DevContent::find(&mut self.standard, &id)
    }

    /// Find content by both source-library IDs, rather than its local export ID.
    ///
    /// Missing or zero identities return `None`. Repeated pairs return the row
    /// with the lowest local export ID.
    pub fn get_content_by_master_content_id(
        &mut self,
        master_content_id: i64,
        master_db_id: i64,
    ) -> Result<Option<DevContent>> {
        DevContent::find_by_master_content_id(&mut self.standard, master_content_id, master_db_id)
    }

    /// Load contents by source-content IDs within one source library.
    ///
    /// Returns all matches in local export-ID order, omitting missing/zero IDs and
    /// repeated input IDs. Repeated source identities retain all matching content rows.
    pub fn get_contents_by_master_content_ids(
        &mut self,
        master_content_ids: &[i64],
        master_db_id: i64,
    ) -> Result<Vec<DevContent>> {
        DevContent::by_master_content_ids(&mut self.standard, master_content_ids, master_db_id)
    }

    /// Load content and its metadata relationships, including its album artist.
    ///
    /// Missing content returns `None`; missing relationship targets remain `None`.
    pub fn get_content_with_relations(&mut self, id: i64) -> Result<Option<DevContentView>> {
        DevContent::find_with_relations(&mut self.standard, &id)
    }

    /// Load all content in physical order and resolve distinct metadata targets once.
    ///
    /// Returns detached snapshots without loading playlists, history, or My Tags.
    pub fn get_contents_with_relations(&mut self) -> Result<Vec<DevContentView>> {
        DevContent::all_with_relations(&mut self.standard)
    }

    /// Allocate a local export ID and insert content, updating existing export-property counts.
    ///
    /// The operation commits atomically in memory; saving remains explicit.
    /// Audio, artwork and analysis files are not copied or generated.
    pub fn insert_content(&mut self, content: NewDevContent) -> Result<DevContent> {
        content.insert(&mut self.standard)
    }

    /// Create content from a nonempty audio path with forward slashes and insert it in memory.
    ///
    /// The file name is derived from the path; other metadata uses [`NewDevContent`] defaults.
    pub fn create_content(&mut self, path: impl Into<String>) -> Result<DevContent> {
        self.insert_content(NewDevContent::new(path))
    }

    /// Persist edited content in memory, preserving unedited binary fields.
    ///
    /// The ID cannot change. Errors roll back the operation; saving remains explicit.
    /// Editing paths does not move files or automatically update other path fields.
    pub fn update_content(&mut self, content: DevContent) -> Result<DevContent> {
        content.update(&mut self.standard)
    }

    /// Delete content and its playlist/history occurrences, updating export-property counts.
    ///
    /// When the extended file is open, also delete its My Tag associations and
    /// recommendation pairs containing this ID on either side. Unknown tables remain intact.
    /// Returns deleted row count, including associations but excluding property updates;
    /// a missing ID returns zero. ID zero is rejected. Changes commit atomically in memory;
    /// saving the file pair remains explicit and is not atomic. Resource files are retained.
    pub fn delete_content(&mut self, id: i64) -> Result<usize> {
        match self.extended.as_mut() {
            Some(extended) => DevContent::delete_with_extended(&mut self.standard, extended, &id),
            None => DevContent::delete(&mut self.standard, &id),
        }
    }

    // -- Genre ------------------------------------------------------------------------------------

    /// Load all genres in physical order without resolving related rows.
    pub fn get_genres(&mut self) -> Result<Vec<DevGenre>> {
        DevGenre::all(&mut self.standard)
    }

    /// Find a genre by export ID; returns `None` when absent.
    pub fn get_genre_by_id(&mut self, id: i64) -> Result<Option<DevGenre>> {
        DevGenre::find(&mut self.standard, &id)
    }

    /// Find the first exact, case-sensitive name match; returns `None` when absent.
    pub fn get_genre_by_name(&mut self, name: &str) -> Result<Option<DevGenre>> {
        DevGenre::find_by_name(&mut self.standard, name)
    }

    /// Allocate a genre ID and insert it, enforcing unique, case-sensitive names.
    ///
    /// Changes remain in memory until explicitly saved.
    pub fn insert_genre(&mut self, genre: NewDevGenre) -> Result<DevGenre> {
        genre.insert(&mut self.standard)
    }

    /// Create a genre with an automatically allocated export ID and insert it in memory.
    pub fn create_genre(&mut self, name: impl Into<String>) -> Result<DevGenre> {
        self.insert_genre(NewDevGenre::new(name))
    }

    /// Persist an edited genre name in memory and return a refreshed snapshot.
    ///
    /// The ID cannot change; names must remain unique. Saving is explicit.
    pub fn update_genre(&mut self, genre: DevGenre) -> Result<DevGenre> {
        genre.update(&mut self.standard)
    }

    /// Delete a genre and clear referencing contents' genre IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero for a missing genre. ID zero is rejected.
    /// Contents and export track counts are retained; saving remains explicit.
    pub fn delete_genre(&mut self, id: i64) -> Result<usize> {
        DevGenre::delete(&mut self.standard, &id)
    }

    // -- History ----------------------------------------------------------------------------------

    /// Load all histories in physical order without loading their occurrences.
    ///
    /// Device Library histories have no folder hierarchy or type attribute.
    pub fn get_histories(&mut self) -> Result<Vec<DevHistory>> {
        DevHistory::all(&mut self.standard)
    }

    /// Find a history by export ID; returns `None` when absent.
    pub fn get_history_by_id(&mut self, id: i64) -> Result<Option<DevHistory>> {
        DevHistory::find(&mut self.standard, &id)
    }

    /// Load a history's contents in stored sequence order, retaining repeated plays.
    ///
    /// Ties retain physical occurrence order; missing content references return errors.
    /// No entries returns an empty list. Content metadata relationships are not resolved.
    pub fn get_history_contents(&mut self, id: i64) -> Result<Vec<DevContent>> {
        DevHistory::get_contents(&mut self.standard, &id)
    }

    /// Load occurrence snapshots in stored sequence order without resolving contents.
    ///
    /// Repeated plays, sequence gaps and ties are retained. These snapshots can be
    /// passed to [`Self::delete_history_content`] to delete individual occurrences.
    pub fn get_history_entries(&mut self, id: i64) -> Result<Vec<DevHistoryContent>> {
        DevHistoryContent::by_history_id(&mut self.standard, &id)
    }

    /// Allocate a history ID and insert its name in memory; saving remains explicit.
    pub fn insert_history(&mut self, history: NewDevHistory) -> Result<DevHistory> {
        history.insert(&mut self.standard)
    }

    /// Create a named history with an automatically allocated export ID.
    ///
    /// Changes remain in memory until explicitly saved.
    pub fn create_history(&mut self, name: impl Into<String>) -> Result<DevHistory> {
        self.insert_history(NewDevHistory::new(name))
    }

    /// Update a history name in memory and return a refreshed snapshot.
    /// The ID cannot change; saving remains explicit.
    pub fn update_history(&mut self, history: DevHistory) -> Result<DevHistory> {
        history.update(&mut self.standard)
    }

    /// Delete a history and its occurrences atomically in memory, retaining contents.
    ///
    /// Returns the total deleted rows, or zero for a missing history. ID zero is rejected.
    /// Other histories and playlists are unchanged; saving remains explicit.
    pub fn delete_history(&mut self, id: i64) -> Result<usize> {
        DevHistory::delete(&mut self.standard, &id)
    }

    /// Insert a history occurrence in memory, retaining its explicit stored position.
    ///
    /// Repeated tracks and position ties are allowed. Reference IDs must be nonzero;
    /// target existence is not checked. Saving remains explicit.
    pub fn insert_history_content(
        &mut self,
        entry: NewDevHistoryContent,
    ) -> Result<DevHistoryContent> {
        entry.insert(&mut self.standard)
    }

    /// Create an occurrence at an explicit stored position without shifting other entries.
    ///
    /// See [`Self::insert_history_content`] for validation and persistence semantics.
    pub fn create_history_content(
        &mut self,
        history_id: i64,
        content_id: i64,
        seq: u32,
    ) -> Result<DevHistoryContent> {
        self.insert_history_content(NewDevHistoryContent::new(history_id, content_id, seq))
    }

    /// Delete one occurrence snapshot, retaining other plays, contents and sequence gaps.
    ///
    /// Returns one on success; stale snapshots return an error. Re-fetch entries after
    /// structural edits. Changes remain in memory until explicitly saved.
    pub fn delete_history_content(&mut self, entry: DevHistoryContent) -> Result<usize> {
        entry.delete(&mut self.standard)
    }

    // -- Key --------------------------------------------------------------------------------------

    /// Load all musical keys in physical order without resolving related rows.
    pub fn get_keys(&mut self) -> Result<Vec<DevKey>> {
        DevKey::all(&mut self.standard)
    }

    /// Find a key by export ID; returns `None` when absent.
    pub fn get_key_by_id(&mut self, id: i64) -> Result<Option<DevKey>> {
        DevKey::find(&mut self.standard, &id)
    }

    /// Find the first exact, case-sensitive name match; returns `None` when absent.
    pub fn get_key_by_name(&mut self, name: &str) -> Result<Option<DevKey>> {
        DevKey::find_by_name(&mut self.standard, name)
    }

    /// Allocate an export key ID and insert it, enforcing unique, case-sensitive names.
    ///
    /// The supplied secondary ID is retained independently; saving remains explicit.
    pub fn insert_key(&mut self, key: NewDevKey) -> Result<DevKey> {
        key.insert(&mut self.standard)
    }

    /// Create a key with an automatically allocated export ID and an explicit secondary ID.
    ///
    /// The secondary ID's role remains unknown; no musical-key numbering is inferred.
    /// Changes remain in memory until explicitly saved.
    pub fn create_key(&mut self, name: impl Into<String>, secondary_id: i64) -> Result<DevKey> {
        self.insert_key(NewDevKey::new(name, secondary_id))
    }

    /// Persist an edited key name or secondary ID in memory and return a refreshed snapshot.
    ///
    /// The export ID cannot change; names must remain unique. Saving is explicit.
    pub fn update_key(&mut self, key: DevKey) -> Result<DevKey> {
        key.update(&mut self.standard)
    }

    /// Delete a key and clear referencing contents' key IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero for a missing key. ID zero is rejected.
    /// Contents and export track counts are retained; saving remains explicit.
    pub fn delete_key(&mut self, id: i64) -> Result<usize> {
        DevKey::delete(&mut self.standard, &id)
    }

    // -- Label ------------------------------------------------------------------------------------

    /// Load all labels in physical order without resolving related rows.
    pub fn get_labels(&mut self) -> Result<Vec<DevLabel>> {
        DevLabel::all(&mut self.standard)
    }

    /// Find a label by export ID; returns `None` when absent.
    pub fn get_label_by_id(&mut self, id: i64) -> Result<Option<DevLabel>> {
        DevLabel::find(&mut self.standard, &id)
    }

    /// Find the first exact, case-sensitive name match; returns `None` when absent.
    pub fn get_label_by_name(&mut self, name: &str) -> Result<Option<DevLabel>> {
        DevLabel::find_by_name(&mut self.standard, name)
    }

    /// Allocate a label ID and insert it, enforcing unique, case-sensitive names.
    /// Changes remain in memory until explicitly saved.
    pub fn insert_label(&mut self, label: NewDevLabel) -> Result<DevLabel> {
        label.insert(&mut self.standard)
    }

    /// Create a label with an automatically allocated export ID and insert it in memory.
    pub fn create_label(&mut self, name: impl Into<String>) -> Result<DevLabel> {
        self.insert_label(NewDevLabel::new(name))
    }

    /// Persist an edited label name in memory and return a refreshed snapshot.
    ///
    /// The ID cannot change; names must remain unique. Saving is explicit.
    pub fn update_label(&mut self, label: DevLabel) -> Result<DevLabel> {
        label.update(&mut self.standard)
    }

    /// Delete a label and clear referencing contents' label IDs atomically in memory.
    ///
    /// Returns the total affected rows, or zero for a missing label. ID zero is rejected.
    /// Contents and export track counts are retained; saving remains explicit.
    pub fn delete_label(&mut self, id: i64) -> Result<usize> {
        DevLabel::delete(&mut self.standard, &id)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    /// Load all tags/categories in physical order without resolving relations.
    ///
    /// All My Tag methods require the extended file; otherwise they return
    /// [`Error::MissingExtendedDatabase`]. Mutations remain in memory until saved.
    pub fn get_my_tags(&mut self) -> Result<Vec<DevMyTag>> {
        DevMyTag::all(self.extended_mut()?)
    }

    /// Find a tag/category by its source-library ID; missing IDs return `None`.
    pub fn get_my_tag_by_id(&mut self, id: i64) -> Result<Option<DevMyTag>> {
        DevMyTag::find(self.extended_mut()?, &id)
    }

    /// Load immediate children in stored sequence order; zero selects roots.
    pub fn get_my_tag_children(&mut self, id: i64) -> Result<Vec<DevMyTag>> {
        DevMyTag::by_parent_id(self.extended_mut()?, &id)
    }

    /// Load tags for a local content ID, retaining physical association order and repetitions.
    ///
    /// Missing tag targets return errors. The standard content row is not loaded.
    pub fn get_my_tag_by_content_id(&mut self, content_id: i64) -> Result<Vec<DevMyTag>> {
        DevMyTag::by_content_id(self.extended_mut()?, &content_id)
    }

    /// Load a tag's contents across both files in physical association order, retaining repeats.
    ///
    /// Missing content targets return errors. Metadata relationships are not resolved.
    pub fn get_my_tag_contents(&mut self, id: i64) -> Result<Vec<DevContent>> {
        let extended = self
            .extended
            .as_mut()
            .ok_or(Error::MissingExtendedDatabase)?;
        DevMyTag::get_contents(&mut self.standard, extended, &id)
    }

    /// Insert a tag/category, validating ancestry and maintaining sibling order.
    ///
    /// IDs must be supplied explicitly and should match the source `master.db`.
    pub fn insert_my_tag(&mut self, my_tag: NewDevMyTag) -> Result<DevMyTag> {
        my_tag.insert(self.extended_mut()?)
    }

    /// Create a tag with its source `master.db` ID. `None` selects the root or appends.
    ///
    /// Explicit positions start at [`DevMyTag::START_SEQ`]. Saving remains explicit.
    pub fn create_my_tag(
        &mut self,
        id: i64,
        name: impl Into<String>,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<DevMyTag> {
        let mut input = NewDevMyTag::tag(id, name).parent_id(parent_id.unwrap_or_default());
        input.seq = seq;
        self.insert_my_tag(input)
    }

    /// Create a category with its source `master.db` ID. `None` selects the root or appends.
    ///
    /// Explicit positions start at [`DevMyTag::START_SEQ`]. Saving remains explicit.
    pub fn create_my_tag_folder(
        &mut self,
        id: i64,
        name: impl Into<String>,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<DevMyTag> {
        let mut input = NewDevMyTag::folder(id, name).parent_id(parent_id.unwrap_or_default());
        input.seq = seq;
        self.insert_my_tag(input)
    }

    /// Update name, parent or position, validating ancestry and maintaining sibling order.
    ///
    /// IDs and attributes cannot change. Changes commit atomically in memory.
    pub fn update_my_tag(&mut self, my_tag: DevMyTag) -> Result<DevMyTag> {
        my_tag.update(self.extended_mut()?)
    }

    /// Rename a tag/category, returning one on success or zero for a missing ID.
    pub fn rename_my_tag(&mut self, id: i64, name: &str) -> Result<usize> {
        let conn = self.extended_mut()?;
        let Some(mut tag) = DevMyTag::find(conn, &id)? else {
            return Ok(0);
        };
        tag.name = name.into();
        tag.update(conn)?;
        Ok(1)
    }

    /// Count immediate children without decoding names; zero selects the root.
    pub fn count_my_tag_children(&mut self, parent_id: i64) -> Result<usize> {
        DevMyTag::count_children(self.extended_mut()?, &parent_id)
    }

    /// Read sibling positions in ascending order, retaining imported gaps and ties.
    pub fn get_my_tag_seq_numbers(&mut self, parent_id: i64) -> Result<Vec<u32>> {
        DevMyTag::get_seq_numbers(self.extended_mut()?, &parent_id)
    }

    /// Check contiguous sibling positions starting at [`DevMyTag::START_SEQ`].
    pub fn validate_my_tag_seq_numbers(&mut self, parent_id: i64) -> Result<bool> {
        DevMyTag::validate_seq_numbers(self.extended_mut()?, &parent_id)
    }

    /// Normalize sibling positions in stable sequence order, returning changed rows.
    ///
    /// The parent must be a category with valid ancestry, or zero for the root.
    pub fn reset_my_tag_seq(&mut self, parent_id: i64) -> Result<usize> {
        DevMyTag::reset_seq(self.extended_mut()?, &parent_id)
    }

    /// Move one node, maintaining sibling order and rejecting cycles or invalid parents.
    ///
    /// `None` keeps the parent or appends. Returns one if the selected node moved, else zero.
    pub fn move_my_tag(
        &mut self,
        id: i64,
        parent_id: Option<&i64>,
        seq: Option<u32>,
    ) -> Result<usize> {
        DevMyTag::move_to(self.extended_mut()?, &id, parent_id, seq)
    }

    /// Move distinct nodes as a block in caller order, atomically; `None` appends.
    ///
    /// Returns changed selected nodes, excluding shifts of other siblings.
    pub fn move_my_tags(
        &mut self,
        ids: &[i64],
        parent_id: i64,
        start_seq: Option<u32>,
    ) -> Result<usize> {
        DevMyTag::move_all_to(self.extended_mut()?, ids, &parent_id, start_seq)
    }

    /// Delete a tag/category subtree and its associations, normalizing surviving siblings.
    ///
    /// Returns deleted rows; missing IDs return zero. Contents remain intact.
    pub fn delete_my_tag(&mut self, id: i64) -> Result<usize> {
        DevMyTag::delete(self.extended_mut()?, &id)
    }

    /// Delete selected subtrees atomically, counting overlapping descendants once.
    ///
    /// Duplicate or missing input IDs reject the batch. Returns deleted nodes and associations.
    pub fn delete_my_tags(&mut self, ids: &[i64]) -> Result<usize> {
        DevMyTag::delete_all(self.extended_mut()?, ids)
    }

    /// Load all raw tag/content associations in physical order, retaining duplicates.
    pub fn get_my_tag_associations(&mut self) -> Result<Vec<DevMyTagContent>> {
        DevMyTagContent::all(self.extended_mut()?)
    }

    /// Load a tag's association snapshots without resolving contents or deduplicating pairs.
    pub fn get_my_tag_entries(&mut self, id: i64) -> Result<Vec<DevMyTagContent>> {
        DevMyTagContent::by_my_tag_id(self.extended_mut()?, &id)
    }

    /// Load a content's association snapshots without resolving tags or deduplicating pairs.
    pub fn get_my_tag_associations_by_content_id(
        &mut self,
        content_id: i64,
    ) -> Result<Vec<DevMyTagContent>> {
        DevMyTagContent::by_content_id(self.extended_mut()?, &content_id)
    }

    /// Insert an association with explicit reserved values. IDs must be nonzero.
    ///
    /// target existence is not checked and repeated pairs are allowed.
    pub fn insert_my_tag_content(&mut self, entry: NewDevMyTagContent) -> Result<DevMyTagContent> {
        entry.insert(self.extended_mut()?)
    }

    /// Associate a source-library tag ID with a local export content ID using model defaults.
    ///
    /// Repeated pairs are allowed; referenced rows are not loaded or validated.
    pub fn create_my_tag_content(
        &mut self,
        my_tag_id: i64,
        content_id: i64,
    ) -> Result<DevMyTagContent> {
        self.insert_my_tag_content(NewDevMyTagContent::new(content_id, my_tag_id))
    }

    /// Delete all occurrences of a tag/content pair, returning the deleted count.
    /// Both IDs must be nonzero. Tags and contents remain intact.
    pub fn delete_my_tag_content(&mut self, my_tag_id: i64, content_id: i64) -> Result<usize> {
        DevMyTagContent::remove(self.extended_mut()?, &content_id, &my_tag_id)
    }

    /// Delete one association snapshot, retaining duplicate occurrences and referenced rows.
    ///
    /// Returns one on success. Re-fetch after structural edits; stale snapshots return errors.
    pub fn delete_my_tag_content_entry(&mut self, entry: DevMyTagContent) -> Result<usize> {
        entry.delete(self.extended_mut()?)
    }

    // -- Playlist ---------------------------------------------------------------------------------

    /// Load all playlists and folders in physical order without resolving contents.
    ///
    /// All playlist mutations commit in memory; saving remains explicit.
    pub fn get_playlists(&mut self) -> Result<Vec<DevPlaylist>> {
        DevPlaylist::all(&mut self.standard)
    }

    /// Find a playlist or folder by export ID; missing IDs return `None`.
    pub fn get_playlist_by_id(&mut self, id: i64) -> Result<Option<DevPlaylist>> {
        DevPlaylist::find(&mut self.standard, &id)
    }

    /// Load immediate children in sibling order; zero selects roots.
    pub fn get_playlist_children(&mut self, id: i64) -> Result<Vec<DevPlaylist>> {
        DevPlaylist::by_parent_id(&mut self.standard, &id)
    }

    /// Load contents in playlist sequence order, retaining imported duplicate occurrences.
    ///
    /// Missing content references return errors; metadata relationships are not resolved.
    pub fn get_playlist_contents(&mut self, id: i64) -> Result<Vec<DevContent>> {
        DevPlaylist::get_contents(&mut self.standard, &id)
    }

    /// Find a node by exact path components. Empty, missing or ambiguous paths return `None`.
    ///
    /// Malformed ancestry and duplicate IDs return errors.
    pub fn get_playlist_by_path(&mut self, path: &[&str]) -> Result<Option<DevPlaylist>> {
        DevPlaylist::find_by_path(&mut self.standard, path)
    }

    /// Build a detached tree with children in sibling order, without loading contents.
    ///
    /// Invalid parents, cycles and duplicate IDs return errors.
    pub fn get_playlist_tree(&mut self) -> Result<Vec<DevPlaylistTreeNode>> {
        DevPlaylist::tree(&mut self.standard)
    }

    /// Read the node type; the root sentinel (zero) is treated as a folder.
    pub fn get_playlist_attribute(&mut self, id: i64) -> Result<Option<PlaylistType>> {
        DevPlaylist::get_attribute(&mut self.standard, &id)
    }

    /// Check whether an existing node is a regular playlist.
    pub fn is_playlist(&mut self, id: i64) -> Result<bool> {
        DevPlaylist::is_playlist(&mut self.standard, &id)
    }

    /// Allocate an export ID and insert a playlist/folder, validating ancestry and sibling order.
    pub fn insert_playlist(&mut self, playlist: NewDevPlaylist) -> Result<DevPlaylist> {
        playlist.insert(&mut self.standard)
    }

    /// Create a regular playlist. `None` selects the root or appends among siblings.
    ///
    /// Explicit positions start at [`DevPlaylist::START_SEQ`].
    pub fn create_playlist(
        &mut self,
        name: impl Into<String>,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<DevPlaylist> {
        let mut input = NewDevPlaylist::playlist(name).parent_id(parent_id.unwrap_or_default());
        input.seq = seq;
        self.insert_playlist(input)
    }

    /// Create a folder. `None` selects the root or appends among siblings.
    ///
    /// Explicit positions start at [`DevPlaylist::START_SEQ`].
    pub fn create_playlist_folder(
        &mut self,
        name: impl Into<String>,
        parent_id: Option<i64>,
        seq: Option<u32>,
    ) -> Result<DevPlaylist> {
        let mut input = NewDevPlaylist::folder(name).parent_id(parent_id.unwrap_or_default());
        input.seq = seq;
        self.insert_playlist(input)
    }

    /// Update name, parent or position, validating ancestry and maintaining sibling order.
    ///
    /// IDs and attributes cannot change. Changes commit atomically in memory.
    pub fn update_playlist(&mut self, playlist: DevPlaylist) -> Result<DevPlaylist> {
        playlist.update(&mut self.standard)
    }

    /// Rename a node while preserving hierarchy, position and reserved data.
    ///
    /// Returns one on success; missing IDs return [`Error::ModelNotFound`].
    pub fn rename_playlist(&mut self, id: i64, name: &str) -> Result<usize> {
        DevPlaylist::rename(&mut self.standard, &id, name)
    }

    /// Count immediate children without decoding names; zero selects roots.
    pub fn count_playlist_children(&mut self, parent_id: i64) -> Result<usize> {
        DevPlaylist::count_children(&mut self.standard, &parent_id)
    }

    /// Read sibling positions in ascending order, retaining imported gaps and ties.
    pub fn get_playlist_seq_numbers(&mut self, parent_id: i64) -> Result<Vec<u32>> {
        DevPlaylist::get_seq_numbers(&mut self.standard, &parent_id)
    }

    /// Check contiguous sibling positions starting at [`DevPlaylist::START_SEQ`].
    pub fn validate_playlist_seq_numbers(&mut self, parent_id: i64) -> Result<bool> {
        DevPlaylist::validate_seq_numbers(&mut self.standard, &parent_id)
    }

    /// Normalize siblings in stable sequence order, returning changed rows.
    ///
    /// The parent must be a folder with valid ancestry, or zero for the root.
    pub fn reset_playlist_seq(&mut self, parent_id: i64) -> Result<usize> {
        DevPlaylist::reset_seq(&mut self.standard, &parent_id)
    }

    /// Move one node, rejecting cycles and invalid parents. `None` keeps the parent or appends.
    ///
    /// Positions start at [`DevPlaylist::START_SEQ`]. Returns one if the selected node moved, else zero.
    pub fn move_playlist(
        &mut self,
        id: i64,
        parent_id: Option<&i64>,
        seq: Option<u32>,
    ) -> Result<usize> {
        DevPlaylist::move_to(&mut self.standard, &id, parent_id, seq)
    }

    /// Move distinct nodes as a block in caller order, atomically; `None` appends.
    ///
    /// The destination is measured after removing the selection. Returns changed selected nodes.
    pub fn move_playlists(
        &mut self,
        ids: &[i64],
        parent_id: i64,
        start_seq: Option<u32>,
    ) -> Result<usize> {
        DevPlaylist::move_all_to(&mut self.standard, ids, &parent_id, start_seq)
    }

    /// Delete a playlist/folder subtree and its entries, normalizing surviving siblings.
    ///
    /// Returns deleted rows; missing IDs return zero. Contents remain intact.
    pub fn delete_playlist(&mut self, id: i64) -> Result<usize> {
        DevPlaylist::delete(&mut self.standard, &id)
    }

    /// Delete selected subtrees atomically, counting overlapping descendants once.
    ///
    /// Duplicate or missing input IDs reject the batch. Returns deleted nodes and entries.
    pub fn delete_playlists(&mut self, ids: &[i64]) -> Result<usize> {
        DevPlaylist::delete_all(&mut self.standard, ids)
    }

    /// Load all playlist entry snapshots in physical order, retaining imported duplicates.
    pub fn get_playlist_associations(&mut self) -> Result<Vec<DevPlaylistContent>> {
        DevPlaylistContent::all(&mut self.standard)
    }

    /// Load a playlist's entry snapshots in stored sequence order without resolving contents.
    pub fn get_playlist_entries(&mut self, id: i64) -> Result<Vec<DevPlaylistContent>> {
        DevPlaylistContent::by_playlist_id(&mut self.standard, &id)
    }

    /// Load a content's entry snapshots ordered by playlist ID and sequence.
    pub fn get_playlist_associations_by_content_id(
        &mut self,
        content_id: i64,
    ) -> Result<Vec<DevPlaylistContent>> {
        DevPlaylistContent::by_content_id(&mut self.standard, &content_id)
    }

    /// Find a unique playlist/content pair; missing pairs return `None`, duplicates return errors.
    pub fn get_playlist_content_by_id(
        &mut self,
        playlist_id: i64,
        content_id: i64,
    ) -> Result<Option<DevPlaylistContent>> {
        DevPlaylistContent::find(&mut self.standard, &(playlist_id, content_id))
    }

    /// Insert a unique playlist/content pair, normalizing affected positions.
    ///
    /// The destination must be a regular playlist; positions start at [`DevPlaylistContent::START_SEQ`].
    pub fn insert_playlist_content(
        &mut self,
        entry: NewDevPlaylistContent,
    ) -> Result<DevPlaylistContent> {
        entry.insert(&mut self.standard)
    }

    /// Create a unique playlist/content pair; `None` appends.
    ///
    /// Explicit positions start at [`DevPlaylistContent::START_SEQ`].
    pub fn create_playlist_content(
        &mut self,
        playlist_id: i64,
        content_id: i64,
        seq: Option<u32>,
    ) -> Result<DevPlaylistContent> {
        self.insert_playlist_content(NewDevPlaylistContent::new(playlist_id, content_id, seq))
    }

    /// Count live entries without resolving contents.
    pub fn count_playlist_contents(&mut self, playlist_id: i64) -> Result<usize> {
        DevPlaylistContent::count(&mut self.standard, &playlist_id)
    }

    /// Read stored entry positions in ascending order.
    pub fn get_playlist_content_seq_numbers(&mut self, playlist_id: i64) -> Result<Vec<u32>> {
        DevPlaylistContent::get_seq_numbers(&mut self.standard, &playlist_id)
    }

    /// Check contiguous entry positions starting at [`DevPlaylistContent::START_SEQ`].
    pub fn validate_playlist_content_seq_numbers(&mut self, playlist_id: i64) -> Result<bool> {
        DevPlaylistContent::validate_seq_numbers(&mut self.standard, &playlist_id)
    }

    /// Normalize entry positions in stable sequence order, retaining imported duplicates.
    ///
    /// Returns changed rows; the destination must be a regular playlist.
    pub fn reset_playlist_content_seq(&mut self, playlist_id: i64) -> Result<usize> {
        DevPlaylistContent::reset_seq(&mut self.standard, &playlist_id)
    }

    /// Move a unique pair within its playlist; `None` appends.
    ///
    /// Positions start at [`DevPlaylistContent::START_SEQ`]. Returns one if moved, else zero.
    pub fn move_playlist_content(
        &mut self,
        playlist_id: i64,
        content_id: i64,
        seq: Option<u32>,
    ) -> Result<usize> {
        DevPlaylistContent::move_to(&mut self.standard, &(playlist_id, content_id), seq)
    }

    /// Move distinct pairs from one playlist as a block in caller order, atomically.
    ///
    /// The destination follows removal of the selection; `None` appends. Returns changed selected entries.
    pub fn move_playlist_contents(
        &mut self,
        ids: &[(i64, i64)],
        start_seq: Option<u32>,
    ) -> Result<usize> {
        DevPlaylistContent::move_all_to(&mut self.standard, ids, start_seq)
    }

    /// Delete a unique pair and close the sequence gap. Missing pairs return zero.
    ///
    /// imported duplicates return errors. The playlist and content remain intact.
    pub fn delete_playlist_content(&mut self, playlist_id: i64, content_id: i64) -> Result<usize> {
        DevPlaylistContent::delete_by_id(&mut self.standard, &(playlist_id, content_id))
    }

    /// Delete distinct pairs atomically and compact affected lists, retaining playlists and contents.
    ///
    /// Missing, repeated or ambiguous pairs reject the batch.
    pub fn delete_playlist_contents(&mut self, ids: &[(i64, i64)]) -> Result<usize> {
        DevPlaylistContent::delete_all(&mut self.standard, ids)
    }

    /// Delete one occurrence snapshot and normalize remaining positions, retaining duplicate pairs.
    ///
    /// Re-fetch after structural edits; stale snapshots return errors. Returns one on success.
    pub fn delete_playlist_content_entry(&mut self, entry: DevPlaylistContent) -> Result<usize> {
        entry.delete(&mut self.standard)
    }

    /// Remove every occurrence of a playlist/content pair, including imported duplicates.
    ///
    /// Both IDs must be nonzero. Returns deleted rows, retaining the playlist and content.
    pub fn remove_playlist_content(&mut self, playlist_id: i64, content_id: i64) -> Result<usize> {
        DevPlaylistContent::remove(&mut self.standard, &content_id, &playlist_id)
    }

    /// Remove all entries of a playlist, retaining the playlist and contents.
    pub fn delete_playlist_contents_by_playlist_id(&mut self, playlist_id: i64) -> Result<usize> {
        DevPlaylistContent::delete_by_playlist_id(&mut self.standard, &playlist_id)
    }

    /// Remove all entries of selected playlists atomically; missing IDs are ignored.
    ///
    /// Playlists and contents remain intact.
    pub fn delete_playlist_contents_by_playlist_ids(&mut self, ids: &[i64]) -> Result<usize> {
        DevPlaylistContent::delete_by_playlist_ids(&mut self.standard, ids)
    }

    /// Remove every playlist occurrence of a content ID and compact affected lists.
    ///
    /// The content itself and playlists remain intact.
    pub fn delete_playlist_contents_by_content_id(&mut self, content_id: i64) -> Result<usize> {
        DevPlaylistContent::delete_by_content_id(&mut self.standard, &content_id)
    }

    /// Remove every playlist occurrence of selected content IDs atomically and compact affected lists.
    ///
    /// Contents and playlists remain intact.
    pub fn delete_playlist_contents_by_content_ids(&mut self, ids: &[i64]) -> Result<usize> {
        DevPlaylistContent::delete_by_content_ids(&mut self.standard, ids)
    }

    // -- Property ---------------------------------------------------------------------------------

    /// Load all standard export properties in physical order, validating unique device names.
    pub fn get_properties(&mut self) -> Result<Vec<DevExportProperty>> {
        DevExportProperty::all(&mut self.standard)
    }

    /// Find a standard export property by its exact, case-sensitive device name.
    ///
    /// Missing names return `None`; duplicate names return an error.
    pub fn get_property_by_device(
        &mut self,
        device_name: &str,
    ) -> Result<Option<DevExportProperty>> {
        DevExportProperty::find(&mut self.standard, device_name)
    }

    /// Load the first standard export property, validating all property names.
    ///
    /// Returns `None` when the table is empty.
    pub fn get_first_property(&mut self) -> Result<Option<DevExportProperty>> {
        DevExportProperty::first(&mut self.standard)
    }

    /// Update a standard export property in memory, preserving unedited binary fields.
    ///
    /// Renaming requires an unused device name. Re-fetch after content mutations,
    /// which update track counts; stale snapshots are rejected. Saving remains explicit.
    pub fn update_property(&mut self, property: DevExportProperty) -> Result<DevExportProperty> {
        property.update(&mut self.standard)
    }

    /// Load all extended database properties in physical order, retaining repeated rows.
    ///
    /// Returns [`Error::MissingExtendedDatabase`] when the extended file is not open.
    pub fn get_database_properties(&mut self) -> Result<Vec<DevDatabaseProperty>> {
        DevDatabaseProperty::all(self.extended_mut()?)
    }

    /// Load the first extended database property, or `None` when its table is empty.
    ///
    /// Returns [`Error::MissingExtendedDatabase`] when the extended file is not open.
    pub fn get_first_database_property(&mut self) -> Result<Option<DevDatabaseProperty>> {
        DevDatabaseProperty::first(self.extended_mut()?)
    }

    /// Update one extended property's My Tag library identity, preserving reserved data.
    /// Does not remap tags or associations. Re-fetch after structural edits; saving is explicit.
    /// Returns [`Error::MissingExtendedDatabase`] when the extended file is not open.
    pub fn update_database_property(
        &mut self,
        property: DevDatabaseProperty,
    ) -> Result<DevDatabaseProperty> {
        property.update(self.extended_mut()?)
    }

    // -- RecommendedLike --------------------------------------------------------------------------

    /// Load all extended recommendation rows in physical order, retaining imported duplicates.
    ///
    /// Returns [`Error::MissingExtendedDatabase`] when the extended file is not open.
    pub fn get_recommend_likes(&mut self) -> Result<Vec<DevRecommendedLike>> {
        DevRecommendedLike::all(self.extended_mut()?)
    }

    /// Load recommendations containing the content ID on either side, in physical order.
    ///
    /// Referenced contents are not loaded. Requires an open extended database.
    pub fn get_recommend_like_by_content_id(
        &mut self,
        content_id: i64,
    ) -> Result<Vec<DevRecommendedLike>> {
        DevRecommendedLike::by_content_id(self.extended_mut()?, &content_id)
    }

    /// Insert a unique ordered content pair in the extended database, in memory.
    ///
    /// Reversed pairs remain distinct. IDs must be nonzero; target existence is not checked.
    /// Unknown words are retained verbatim. Requires an open extended database; saving is explicit.
    pub fn insert_recommend_like(
        &mut self,
        recommend_like: NewDevRecommendedLike,
    ) -> Result<DevRecommendedLike> {
        recommend_like.insert(self.extended_mut()?)
    }

    /// Create a recommendation row with explicit unknown words; their meanings are not inferred.
    ///
    /// See [`Self::insert_recommend_like`] for constraints and persistence semantics.
    pub fn create_recommend_like(
        &mut self,
        content_id_1: i64,
        content_id_2: i64,
        unknown_00: u64,
        unknown_10: u32,
        unknown_14: u32,
    ) -> Result<DevRecommendedLike> {
        self.insert_recommend_like(NewDevRecommendedLike::new(
            content_id_1,
            content_id_2,
            unknown_00,
            unknown_10,
            unknown_14,
        ))
    }

    /// Delete one original recommendation occurrence, retaining contents and other occurrences.
    ///
    /// Returns one on success. Re-fetch snapshots after structural edits; stale rows return errors.
    /// Requires an open extended database; changes remain in memory until explicitly saved.
    pub fn delete_recommend_like(&mut self, recommend_like: DevRecommendedLike) -> Result<usize> {
        recommend_like.delete(self.extended_mut()?)
    }
}
