// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Content snapshots, queries and transactional mutations over DeviceSQL.
//!
//! Content is the model name for DeviceSQL tracks, matching masterdb and OneLibrary
//! terminology. The underlying binary representation remains `TrackRow`.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Seek};

use binrw::BinRead;
use rbox_core::{PathExt, Track, TrackSource};
use std::path::PathBuf;

use super::{
    load_related, text, text_error, unsupported, update_export_count, validate_id, Album, Artist,
    Artwork, Color, Genre, HistoryContent, Key, Label, MyTagContent, PlaylistContent,
    RecommendedLike,
};
use crate::binary::{
    DeviceSqlString, ExtendedRow, PageType, Row, RowRecord, StandardRow, StringOffsets, TableKind,
    TrackHeader, TrackRow, TrackStrings,
};
pub use crate::devicesql::TrackCreationOptions;
use crate::devicesql::{DeviceSqlConnection, Transaction};

use crate::{Error, Result};

/// A detached track from the standard database's track table.
///
/// Named `Content` for consistency with masterdb and OneLibrary models-old; this wraps
/// the binary [`TrackRow`] and uses [`PageType::Tracks`].
///
/// Public fields contain decoded values, like the masterdb/onelib models-old.
/// Editing a field only changes this snapshot; call [Self::update] to persist edits.
/// Use [`NewContent`] to insert a fresh track without an existing raw row.
/// IDs, empty strings, date text and paths retain their stored meaning. Unknown
/// fields and string encodings remain available through [`Self::raw_record`].
/// ISRC decoding is explicit because its frame can have an unknown encoding.
///
/// Plain queries never load relationships. Use [`Self::resolve`] or
/// [`Self::all_with_relations`] to explicitly construct a [`ContentView`].
/// The extended database has no track table and returns [`Error::WrongDatabaseKind`].
#[derive(Clone, PartialEq, Eq)]
pub struct Content {
    /// Export track ID, used by playlists and tag associations.
    pub id: u32,
    /// Main artist ID.
    pub artist_id: u32,
    /// Album ID.
    pub album_id: u32,
    /// Genre ID.
    pub genre_id: u32,
    /// Label ID.
    pub label_id: u32,
    /// Musical-key ID.
    pub key_id: u32,
    /// Artwork-reference ID.
    pub artwork_id: u32,
    /// Composer artist ID.
    pub composer_id: u32,
    /// Original artist ID.
    pub original_artist_id: u32,
    /// Remixer artist ID.
    pub remixer_id: u32,
    /// Content identity in the source library, separate from the export ID.
    pub master_content_id: u32,
    /// Source library identity.
    pub master_db_id: u32,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Audio file size in bytes.
    pub file_size: u32,
    /// Stored bitrate, normally kb/s.
    pub bitrate: u32,
    /// Tempo in BPM multiplied by 100.
    pub bpmx100: u32,
    /// Track duration in seconds.
    pub length: u16,
    /// Track number within the release.
    pub track_no: u32,
    /// Disc number.
    pub disc_no: u16,
    /// Stored play count.
    pub play_count: u16,
    /// Release year.
    pub release_year: u16,
    /// Sample depth in bits per sample.
    pub bit_depth: u16,
    /// Raw color ID, including zero.
    pub color_id: u8,
    /// Raw star rating; unrecognized values are retained.
    pub rating: u8,
    /// Raw audio format code; unrecognized codes are retained.
    pub file_type: u16,
    /// Display title.
    pub title: String,
    /// DJ comment.
    pub comment: String,
    /// Mix/remix name.
    pub mix_name: String,
    /// Lyricist text.
    pub lyricist: String,
    /// Track message, separate from the comment.
    pub message: String,
    /// Original publication setting text.
    pub publish_track_info: String,
    /// Original hot-cue auto-load setting text.
    pub hot_cue_auto_load: String,
    /// Collection date as stored text.
    pub date_added: String,
    /// Release date as stored text.
    pub release_date: String,
    /// Analysis date as stored text.
    pub analysis_date: String,
    /// Analysis-file path, preserving original separators.
    pub analysis_path: String,
    /// Audio basename.
    pub file_name: String,
    /// Audio path, preserving original separators.
    pub path: String,
    isrc_override: Option<String>,
    raw: RowRecord,
}

/// Content snapshot with resolved metadata, following masterdb's content-view pattern.
///
/// Field access performs no queries. Zero IDs and missing targets are represented
/// by `None`; the original IDs and raw row remain in [`Self::content`].
/// Related models-old are detached rows, with only the album artist additionally
/// resolved. Editing the view does not persist changes or refresh related fields.
///
/// ```
/// use rbox_devlib::{binary::DatabaseKind, DeviceSqlConnection};
/// use rbox_devlib::models::{Content, NewArtist, NewContent};
/// # fn main() -> rbox_devlib::Result<()> {
/// let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
/// let artist = NewArtist::new(1, "Artist").insert(&mut conn)?;
/// let mut input = NewContent::new(10, "/Contents/Track.wav");
/// input.artist_id = artist.id;
/// input.insert(&mut conn)?;
/// let view = Content::find_with_relations(&mut conn, &10)?.unwrap();
/// assert_eq!(view.artist.as_ref().map(|artist| artist.name.as_str()), Some("Artist"));
/// assert_eq!(view.id, 10); // Content fields remain accessible through Deref.
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentView {
    /// Original content snapshot, including unresolved foreign keys.
    pub content: Content,
    /// Main artist.
    pub artist: Option<Artist>,
    /// Artist credited as remixer.
    pub remixer: Option<Artist>,
    /// Artist credited as original artist.
    pub original_artist: Option<Artist>,
    /// Artist credited as composer.
    pub composer: Option<Artist>,
    /// Referenced album.
    pub album: Option<Album>,
    /// Artist referenced by the album's artist_id, independently of the content artist.
    pub album_artist: Option<Artist>,
    /// Referenced genre.
    pub genre: Option<Genre>,
    /// Referenced label.
    pub label: Option<Label>,
    /// Referenced musical key.
    pub key: Option<Key>,
    /// Referenced color.
    pub color: Option<Color>,
    /// Referenced artwork path record; no image file is loaded.
    pub artwork: Option<Artwork>,
}

/// Access content fields without performing relationship queries.
impl std::ops::Deref for ContentView {
    type Target = Content;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

/// Targets decoded once per resolution batch; not retained across mutations.
struct ContentRelations {
    artists: HashMap<u32, Artist>,
    albums: HashMap<u32, Album>,
    genres: HashMap<u32, Genre>,
    labels: HashMap<u32, Label>,
    keys: HashMap<u32, Key>,
    artworks: HashMap<u32, Artwork>,
    colors: HashMap<u16, Color>,
}

impl ContentRelations {
    fn load<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        contents: &[Content],
    ) -> Result<Self> {
        conn.check_kind(Content::TABLE)?;
        let albums = load_related(conn, contents.iter().map(|row| row.album_id), Album::find)?;
        let artists = load_related(
            conn,
            contents
                .iter()
                .flat_map(|row| {
                    [
                        row.artist_id,
                        row.remixer_id,
                        row.original_artist_id,
                        row.composer_id,
                    ]
                })
                .chain(albums.values().map(|album| album.artist_id)),
            Artist::find,
        )?;
        Ok(Self {
            albums,
            artists,
            genres: load_related(conn, contents.iter().map(|row| row.genre_id), Genre::find)?,
            labels: load_related(conn, contents.iter().map(|row| row.label_id), Label::find)?,
            keys: load_related(conn, contents.iter().map(|row| row.key_id), Key::find)?,
            artworks: load_related(
                conn,
                contents.iter().map(|row| row.artwork_id),
                Artwork::find,
            )?,
            colors: load_related(
                conn,
                contents.iter().map(|row| u16::from(row.color_id)),
                Color::find,
            )?,
        })
    }

    fn view(&self, content: Content) -> ContentView {
        let album = self.albums.get(&content.album_id).cloned();
        let album_artist = album
            .as_ref()
            .and_then(|album| self.artists.get(&album.artist_id))
            .cloned();
        ContentView {
            artist: self.artists.get(&content.artist_id).cloned(),
            remixer: self.artists.get(&content.remixer_id).cloned(),
            original_artist: self.artists.get(&content.original_artist_id).cloned(),
            composer: self.artists.get(&content.composer_id).cloned(),
            album,
            album_artist,
            genre: self.genres.get(&content.genre_id).cloned(),
            label: self.labels.get(&content.label_id).cloned(),
            key: self.keys.get(&content.key_id).cloned(),
            artwork: self.artworks.get(&content.artwork_id).cloned(),
            color: self.colors.get(&u16::from(content.color_id)).cloned(),
            content,
        }
    }
}

impl std::fmt::Debug for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Content")
            .field("id", &self.id)
            .field("artist_id", &self.artist_id)
            .field("album_id", &self.album_id)
            .field("genre_id", &self.genre_id)
            .field("label_id", &self.label_id)
            .field("key_id", &self.key_id)
            .field("artwork_id", &self.artwork_id)
            .field("composer_id", &self.composer_id)
            .field("original_artist_id", &self.original_artist_id)
            .field("remixer_id", &self.remixer_id)
            .field("master_content_id", &self.master_content_id)
            .field("master_db_id", &self.master_db_id)
            .field("sample_rate", &self.sample_rate)
            .field("file_size", &self.file_size)
            .field("bitrate", &self.bitrate)
            .field("bpmx100", &self.bpmx100)
            .field("length", &self.length)
            .field("track_no", &self.track_no)
            .field("disc_no", &self.disc_no)
            .field("play_count", &self.play_count)
            .field("release_year", &self.release_year)
            .field("bit_depth", &self.bit_depth)
            .field("color_id", &self.color_id)
            .field("rating", &self.rating)
            .field("file_type", &self.file_type)
            .field("title", &self.title)
            .field("comment", &self.comment)
            .field("mix_name", &self.mix_name)
            .field("lyricist", &self.lyricist)
            .field("message", &self.message)
            .field("publish_track_info", &self.publish_track_info)
            .field("hot_cue_auto_load", &self.hot_cue_auto_load)
            .field("date_added", &self.date_added)
            .field("release_date", &self.release_date)
            .field("analysis_date", &self.analysis_date)
            .field("analysis_path", &self.analysis_path)
            .field("file_name", &self.file_name)
            .field("path", &self.path)
            .field("isrc", &self.isrc())
            .finish()
    }
}

impl Content {
    /// Standard database table containing tracks.
    pub const TABLE: TableKind = TableKind::Standard(PageType::Tracks);

    /// Stream live tracks in page-chain and slot order.
    ///
    /// Creating the iterator does not read pages. Each advance decodes one row;
    /// malformed rows produce errors instead of being skipped.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live tracks without loading related tables.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Find a track by its export ID; a missing ID returns `None`.
    ///
    /// The engine builds a reusable ID index from scalar headers on first use.
    /// Only the selected row's strings are decoded. Duplicate IDs are an error.
    pub fn find<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<Option<Self>> {
        let location = conn
            .index_rows(Self::TABLE, Self::read_id)?
            .get(id)
            .copied();
        location
            .map(|location| Self::try_from(conn.read_indexed_row(Self::TABLE, location)?))
            .transpose()
    }

    /// Check an export ID using the engine's index, without decoding strings.
    pub fn id_exists<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<bool> {
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(id))
    }

    /// Load matching export IDs in ascending order, omitting missing and repeated IDs.
    ///
    /// Only matching rows have their strings decoded. Empty input reads no pages.
    pub fn by_ids<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        ids: &[u32],
    ) -> Result<Vec<Self>> {
        conn.check_kind(Self::TABLE)?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut ids = ids.to_vec();
        ids.sort_unstable();
        ids.dedup();
        let index = conn.index_rows(Self::TABLE, Self::read_id)?;
        let locations: Vec<_> = ids.iter().filter_map(|id| index.get(id).copied()).collect();
        locations
            .into_iter()
            .map(|location| Self::try_from(conn.read_indexed_row(Self::TABLE, location)?))
            .collect()
    }

    /// Find the first live row whose stored audio path matches exactly.
    ///
    /// Comparison is case-sensitive and does not normalize separators or access files.
    /// If paths repeat, the first row in physical page/slot order is returned.
    pub fn find_by_path<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        path: &str,
    ) -> Result<Option<Self>> {
        for row in Self::iter(conn)? {
            let row = row?;
            if row.path == path {
                return Ok(Some(row));
            }
        }
        Ok(None)
    }

    /// Check for an exact stored audio path, using the same comparison as `find_by_path`.
    pub fn path_exists<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        path: &str,
    ) -> Result<bool> {
        Ok(Self::find_by_path(conn, path)?.is_some())
    }

    /// Find content by its source-library identity, not its local export ID.
    ///
    /// Both IDs use their unsigned representation, as in `rbox_core::Track`.
    /// Zero means an absent source identity and returns `None`. If the pair occurs
    /// more than once, return the row with the lowest export ID. Scans scalar
    /// headers and decodes strings only for the selected row.
    pub fn find_by_master_content_id<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        master_content_id: u32,
        master_db_id: u32,
    ) -> Result<Option<Self>> {
        conn.check_kind(Self::TABLE)?;
        if master_content_id == 0 || master_db_id == 0 {
            return Ok(None);
        }
        let mut selected: Option<(u32, RowRecord)> = None;
        for record in conn.iter_raw(Self::TABLE)? {
            let record = record?;
            let header = Self::read_header(&record)?;
            if header.master_content_id == master_content_id
                && header.master_db_id == master_db_id
                && selected.as_ref().is_none_or(|(id, _)| header.id < *id)
            {
                selected = Some((header.id, record));
            }
        }
        selected
            .map(|(_, record)| Self::try_from(record))
            .transpose()
    }

    /// Load all contents matching source-content IDs within one source library.

    /// IDs use DeviceLibrary's unsigned representation. Missing and zero IDs are
    /// omitted; repeated input IDs do not repeat rows. A zero library ID or empty
    /// input returns an empty vector without reading pages. Results are ordered by
    /// local export ID, including all exported rows when a source identity repeats.
    /// Scans scalar headers once and decodes strings only for matching rows.
    /// Extended connections are rejected, including for empty input.
    pub fn by_master_content_ids<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        master_content_ids: &[u32],
        master_db_id: u32,
    ) -> Result<Vec<Self>> {
        conn.check_kind(Self::TABLE)?;
        if master_db_id == 0 || master_content_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids: HashSet<_> = master_content_ids
            .iter()
            .copied()
            .filter(|id| *id != 0)
            .collect();
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut rows = Vec::new();
        for record in conn.iter_raw(Self::TABLE)? {
            let record = record?;
            let header = Self::read_header(&record)?;
            if header.master_db_id == master_db_id && ids.contains(&header.master_content_id) {
                rows.push(Self::try_from(record)?);
            }
        }
        rows.sort_unstable_by_key(|row| row.id);
        Ok(rows)
    }

    /// Persist edited public fields, preserving current unknown data and unedited frames.
    ///
    /// Re-resolves the original export ID; changing it is rejected. Only fields
    /// changed from this snapshot are applied; conflicts on the same field use
    /// last-write-wins. ISRC is edited through [`Self::set_isrc`]; otherwise its
    /// frame is preserved. Analysis flags, content-link and unknown strings are
    /// retained from the current binary row. Paths retain supplied separators;
    /// editing the audio path requires a nonempty value and does not rename files
    /// or automatically change the basename or analysis path.
    /// The engine handles relocation. Errors roll back; saving remains explicit.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw.clone())?;
        validate_id(Self::TABLE, self.id)?;
        if self.id != original.id {
            return Err(Error::InvalidModelField {
                table: Self::TABLE,
                field: "id",
                reason: "an existing content's ID cannot be changed",
            });
        }
        if self.path != original.path && self.path.is_empty() {
            return Err(Error::InvalidModelField {
                table: Self::TABLE,
                field: "path",
                reason: "audio path must not be empty",
            });
        }
        let mut tx = conn.transaction();
        let location = tx
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&original.id)
            .copied()
            .ok_or(Error::ModelNotFound {
                table: Self::TABLE,
                id: original.id,
            })?;
        let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
        let record = tx.read(&handle)?;
        let Row::Standard(StandardRow::Track(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if self.artist_id != original.artist_id {
            row.header.artist_id = self.artist_id;
        }
        if self.album_id != original.album_id {
            row.header.album_id = self.album_id;
        }
        if self.genre_id != original.genre_id {
            row.header.genre_id = self.genre_id;
        }
        if self.label_id != original.label_id {
            row.header.label_id = self.label_id;
        }
        if self.key_id != original.key_id {
            row.header.key_id = self.key_id;
        }
        if self.artwork_id != original.artwork_id {
            row.header.artwork_id = self.artwork_id;
        }
        if self.composer_id != original.composer_id {
            row.header.composer_id = self.composer_id;
        }
        if self.original_artist_id != original.original_artist_id {
            row.header.original_artist_id = self.original_artist_id;
        }
        if self.remixer_id != original.remixer_id {
            row.header.remixer_id = self.remixer_id;
        }
        if self.master_content_id != original.master_content_id {
            row.header.master_content_id = self.master_content_id;
        }
        if self.master_db_id != original.master_db_id {
            row.header.master_db_id = self.master_db_id;
        }
        if self.sample_rate != original.sample_rate {
            row.header.sample_rate = self.sample_rate;
        }
        if self.file_size != original.file_size {
            row.header.file_size = self.file_size;
        }
        if self.bitrate != original.bitrate {
            row.header.bitrate = self.bitrate;
        }
        if self.bpmx100 != original.bpmx100 {
            row.header.tempo = self.bpmx100;
        }
        if self.length != original.length {
            row.header.duration = self.length;
        }
        if self.track_no != original.track_no {
            row.header.track_number = self.track_no;
        }
        if self.disc_no != original.disc_no {
            row.header.disc_number = self.disc_no;
        }
        if self.play_count != original.play_count {
            row.header.play_count = self.play_count;
        }
        if self.release_year != original.release_year {
            row.header.year = self.release_year;
        }
        if self.bit_depth != original.bit_depth {
            row.header.sample_depth = self.bit_depth;
        }
        if self.color_id != original.color_id {
            row.header.color_id = self.color_id;
        }
        if self.rating != original.rating {
            row.header.rating = self.rating;
        }
        if self.file_type != original.file_type {
            row.header.audio_format = self.file_type;
        }
        if self.title != original.title {
            row.text.title = text(&self.title)?;
        }
        if self.comment != original.comment {
            row.text.comment = text(&self.comment)?;
        }
        if self.mix_name != original.mix_name {
            row.text.mix_name = text(&self.mix_name)?;
        }
        if self.lyricist != original.lyricist {
            row.text.lyricist = text(&self.lyricist)?;
        }
        if self.message != original.message {
            row.text.message = text(&self.message)?;
        }
        if self.publish_track_info != original.publish_track_info {
            row.text.publish_track_info = text(&self.publish_track_info)?;
        }
        if self.hot_cue_auto_load != original.hot_cue_auto_load {
            row.text.hot_cue_auto_load = text(&self.hot_cue_auto_load)?;
        }
        if self.date_added != original.date_added {
            row.text.date_added = text(&self.date_added)?;
        }
        if self.release_date != original.release_date {
            row.text.release_date = text(&self.release_date)?;
        }
        if self.analysis_date != original.analysis_date {
            row.text.analysis_date = text(&self.analysis_date)?;
        }
        if self.analysis_path != original.analysis_path {
            row.text.analysis_path = text(&self.analysis_path)?;
        }
        if self.file_name != original.file_name {
            row.text.filename = text(&self.file_name)?;
        }
        if self.path != original.path {
            row.text.file_path = text(&self.path)?;
        }
        if let Some(isrc) = &self.isrc_override {
            if !original.isrc().is_ok_and(|value| value == *isrc) {
                row.text.isrc = DeviceSqlString::encode_isrc(isrc)?;
            }
        }
        let handle = tx.update(&handle, &Row::Standard(StandardRow::Track(row)))?;
        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Delete from a standard-only database, including playlist/history occurrences.
    ///
    /// For a library with exportExt.pdb, use [Self::delete_with_extended] or
    /// [crate::DeviceLibrary::delete_content] to also remove tag/recommendation links.
    /// Updates all existing export-property track counts. Parent playlists/histories,
    /// catalog rows, media, artwork and ANLZ files remain. Surviving occurrence
    /// sequence values are preserved (including gaps and ties).
    /// Returns deleted row count, excluding property-count updates; a missing ID
    /// returns zero without cleaning dangling links. ID zero is rejected.
    /// The operation commits atomically in memory; saving remains explicit.
    pub fn delete<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        validate_id(Self::TABLE, *id)?;
        let mut tx = conn.transaction();
        let count = Self::delete_standard(&mut tx, *id)?;
        tx.commit()?;
        Ok(count)
    }

    /// Delete content and all known occurrences/associations across both databases.
    ///
    /// In addition to standard cleanup, deletes every My Tag association and every
    /// recommendation pair containing the export ID on either side. Unknown tables
    /// are preserved. Returns the total deleted row count, excluding property updates.
    /// Both overlays are published only after all fallible operations and source
    /// checks succeed. Disk saving is still separate and not atomic across files.
    pub fn delete_with_extended<R: Read + Seek, E: Read + Seek>(
        standard: &mut DeviceSqlConnection<R>,
        extended: &mut DeviceSqlConnection<E>,
        id: &u32,
    ) -> Result<usize> {
        standard.check_kind(Self::TABLE)?;
        extended.check_kind(MyTagContent::TABLE)?;
        validate_id(Self::TABLE, *id)?;
        let mut tx = standard.transaction();
        let mut ext = extended.transaction();
        let count = Self::delete_standard(&mut tx, *id)?;
        if count == 0 {
            return Ok(0);
        }
        let mut total = count;
        for table in [MyTagContent::TABLE, RecommendedLike::TABLE] {
            total = total
                .checked_add(delete_content_links(&mut ext, table, *id)?)
                .ok_or(Error::MutationOverflow)?;
        }
        tx.commit_with(ext)?;
        Ok(total)
    }

    fn delete_standard<R: Read + Seek>(tx: &mut Transaction<'_, R>, id: u32) -> Result<usize> {
        let index = tx.index_rows(Self::TABLE, Self::read_id)?;
        let Some(location) = index.get(&id).copied() else {
            return Ok(0);
        };
        let remaining = u32::try_from(index.len() - 1).map_err(|_| Error::MutationOverflow)?;
        let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
        tx.delete(&handle)?;
        let mut total = 1usize;
        for table in [PlaylistContent::TABLE, HistoryContent::TABLE] {
            total = total
                .checked_add(delete_content_links(tx, table, id)?)
                .ok_or(Error::MutationOverflow)?;
        }
        update_export_count(tx, remaining)?;
        Ok(total)
    }

    /// Resolve this snapshot's metadata references, including the album artist.
    /// Does not reload the content row; local field edits are retained. Zero and
    /// missing target IDs become `None`, while decoding and index errors propagate.
    pub fn resolve<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<ContentView> {
        let relations = ContentRelations::load(conn, std::slice::from_ref(&self))?;
        Ok(relations.view(self))
    }

    /// Resolve a batch, decoding each distinct related row once per call.
    /// Input order, repeated contents and local snapshot edits are retained.
    /// Empty input performs no reads; an extended connection is always rejected.
    pub fn resolve_many<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        contents: Vec<Self>,
    ) -> Result<Vec<ContentView>> {
        let relations = ContentRelations::load(conn, &contents)?;
        Ok(contents
            .into_iter()
            .map(|content| relations.view(content))
            .collect())
    }

    /// Find one content row and explicitly load its metadata relationships.
    /// A missing content ID returns `None` without reading related tables.
    pub fn find_with_relations<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<Option<ContentView>> {
        Self::find(conn, id)?
            .map(|content| content.resolve(conn))
            .transpose()
    }

    /// Fetch all content rows and resolve shared metadata targets in a batch.
    /// Physical content order is retained. Playlists, history and extended My Tags
    /// are not loaded; the view contains metadata relationships only.
    pub fn all_with_relations<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<Vec<ContentView>> {
        let contents = Self::all(conn)?;
        Self::resolve_many(conn, contents)
    }

    /// Original row window and location, including unknown fields and slack.
    ///
    /// This remains the original snapshot even if public model fields change.
    /// Its location is not a stable mutation handle after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }

    /// Return the edited ISRC, or decode the original frame.
    /// Unrecognized original encodings return an error until explicitly replaced.
    pub fn isrc(&self) -> Result<String> {
        if let Some(value) = &self.isrc_override {
            return Ok(value.clone());
        }
        let Row::Standard(StandardRow::Track(row)) = self.raw.decode()? else {
            return Err(unsupported(&self.raw));
        };
        row.text
            .isrc
            .isrc_text()
            .map_err(|source| text_error(&self.raw, "isrc", source))
    }

    /// Edit the ISRC in this snapshot; call [`Self::update`] to persist it.
    /// Accepts twelve ASCII alphanumeric characters, or an empty string to clear.
    /// Invalid input leaves the snapshot unchanged. Unknown original encodings
    /// are preserved unless this setter explicitly replaces them.
    pub fn set_isrc(&mut self, value: impl Into<String>) -> Result<&mut Self> {
        let value = value.into();
        DeviceSqlString::encode_isrc(&value)?;
        self.isrc_override = Some(value);
        Ok(self)
    }

    pub(super) fn read_id(record: &RowRecord) -> Result<u32> {
        Ok(Self::read_header(record)?.id)
    }

    fn read_header(record: &RowRecord) -> Result<TrackHeader> {
        if record.table() != Self::TABLE {
            return Err(unsupported(record));
        }
        let header =
            TrackHeader::read_le(&mut Cursor::new(record.as_bytes())).map_err(|source| {
                Error::RowDecode {
                    page: record.page(),
                    table: record.table(),
                    slot: record.slot().slot,
                    source,
                }
            })?;
        if header.subtype != TrackHeader::SUBTYPE {
            return Err(unsupported(record));
        }
        Ok(header)
    }
}

/// Select live occurrences without interpreting their unknown fields, then tombstone
/// each selected slot. Duplicate associations and both sides of pairs are handled.
fn delete_content_links<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    table: TableKind,
    id: u32,
) -> Result<usize> {
    let mut locations = Vec::new();
    for record in tx.iter_raw(table)? {
        let record = record?;
        let matches = match record.decode()? {
            Row::Standard(StandardRow::PlaylistEntry(row)) => row.track_id == id,
            Row::Standard(StandardRow::HistoryEntry(row)) => row.track_id == id,
            Row::Extended(ExtendedRow::TrackMyTag(row)) => row.track_id == id,
            Row::Extended(ExtendedRow::RecommendLike(row)) => {
                row.track_id_1 == id || row.track_id_2 == id
            }
            _ => return Err(unsupported(&record)),
        };
        if matches {
            locations.push((record.page(), record.slot().slot));
        }
    }
    let count = locations.len();
    for (page, slot) in locations {
        let handle = tx.locate(table, page, slot)?;
        tx.delete(&handle)?;
    }
    Ok(count)
}

impl TryFrom<RowRecord> for Content {
    type Error = Error;

    /// Decode a track row, retaining the original bytes alongside its fields.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Standard(StandardRow::Track(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        let text = |field, value: &DeviceSqlString| {
            value
                .text()
                .map_err(|source| text_error(&record, field, source))
        };
        Ok(Self {
            id: row.header.id,
            artist_id: row.header.artist_id,
            album_id: row.header.album_id,
            genre_id: row.header.genre_id,
            label_id: row.header.label_id,
            key_id: row.header.key_id,
            artwork_id: row.header.artwork_id,
            composer_id: row.header.composer_id,
            original_artist_id: row.header.original_artist_id,
            remixer_id: row.header.remixer_id,
            master_content_id: row.header.master_content_id,
            master_db_id: row.header.master_db_id,
            sample_rate: row.header.sample_rate,
            file_size: row.header.file_size,
            bitrate: row.header.bitrate,
            bpmx100: row.header.tempo,
            length: row.header.duration,
            track_no: row.header.track_number,
            disc_no: row.header.disc_number,
            play_count: row.header.play_count,
            release_year: row.header.year,
            bit_depth: row.header.sample_depth,
            color_id: row.header.color_id,
            rating: row.header.rating,
            file_type: row.header.audio_format,
            title: text("title", &row.text.title)?,
            comment: text("comment", &row.text.comment)?,
            mix_name: text("mix_name", &row.text.mix_name)?,
            lyricist: text("lyricist", &row.text.lyricist)?,
            message: text("message", &row.text.message)?,
            publish_track_info: text("publish_track_info", &row.text.publish_track_info)?,
            hot_cue_auto_load: text("hot_cue_auto_load", &row.text.hot_cue_auto_load)?,
            date_added: text("date_added", &row.text.date_added)?,
            release_date: text("release_date", &row.text.release_date)?,
            analysis_date: text("analysis_date", &row.text.analysis_date)?,
            analysis_path: text("analysis_path", &row.text.analysis_path)?,
            file_name: text("file_name", &row.text.filename)?,
            path: text("path", &row.text.file_path)?,
            isrc_override: None,
            raw: record,
        })
    }
}

/// Input for a fresh track; no source row or existing PDB is required.
///
/// Supply an unused, nonzero export ID and the exported audio path. Metadata
/// starts at zero or empty text; no source-library identity or analysis path is
/// inferred. `creation` supplies overridable reserved values from the engine's
/// supported writer profile. Populate metadata and ANLZ references before export.
///
/// Insertion returns a [`Content`] with an engine-created raw row, commits in memory,
/// and updates any existing export-property track counts. Call the connection's
/// save methods to persist. Related rows and audio/ANLZ assets are not created.
///
/// ```
/// use rbox_devlib::{binary::DatabaseKind, DeviceSqlConnection};
/// use rbox_devlib::models::NewContent;
///
/// # fn main() -> rbox_devlib::Result<()> {
/// let mut conn = DeviceSqlConnection::create(DatabaseKind::Standard)?;
/// let mut new = NewContent::new(1, "/Contents/Artist/Track.flac");
/// new.title = "Track".into();
/// new.bpmx100 = 12800;
/// new.analysis_path = "/PIONEER/USBANLZ/P02F/0000E9C7/ANLZ0000.DAT".into();
/// let track = new.insert(&mut conn)?;
/// assert_eq!(track.id, 1);
/// assert_eq!(track.title, "Track");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewContent {
    /// Export track ID, used by playlists and tag associations.
    pub id: u32,
    /// Main artist ID.
    pub artist_id: u32,
    /// Album ID.
    pub album_id: u32,
    /// Genre ID.
    pub genre_id: u32,
    /// Label ID.
    pub label_id: u32,
    /// Musical-key ID.
    pub key_id: u32,
    /// Artwork-reference ID.
    pub artwork_id: u32,
    /// Composer artist ID.
    pub composer_id: u32,
    /// Original artist ID.
    pub original_artist_id: u32,
    /// Remixer artist ID.
    pub remixer_id: u32,
    /// Content identity in the source library, separate from the export ID.
    pub master_content_id: u32,
    /// Source library identity.
    pub master_db_id: u32,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Audio file size in bytes.
    pub file_size: u32,
    /// Stored bitrate, normally kb/s.
    pub bitrate: u32,
    /// Tempo in BPM multiplied by 100.
    pub bpmx100: u32,
    /// Track duration in seconds.
    pub length: u16,
    /// Track number within the release.
    pub track_no: u32,
    /// Disc number.
    pub disc_no: u16,
    /// Stored play count.
    pub play_count: u16,
    /// Release year.
    pub release_year: u16,
    /// Sample depth in bits per sample.
    pub bit_depth: u16,
    /// Raw color ID, including zero.
    pub color_id: u8,
    /// Raw star rating; unrecognized values are retained.
    pub rating: u8,
    /// Raw audio format code; unrecognized codes are retained.
    pub file_type: u16,
    /// Display title.
    pub title: String,
    /// DJ comment.
    pub comment: String,
    /// Mix/remix name.
    pub mix_name: String,
    /// Lyricist text.
    pub lyricist: String,
    /// Track message, separate from the comment.
    pub message: String,
    /// Original publication setting text.
    pub publish_track_info: String,
    /// Original hot-cue auto-load setting text.
    pub hot_cue_auto_load: String,
    /// Collection date as stored text.
    pub date_added: String,
    /// Release date as stored text.
    pub release_date: String,
    /// Analysis date as stored text.
    pub analysis_date: String,
    /// Analysis-file path, preserving original separators.
    pub analysis_path: String,
    /// Audio basename.
    pub file_name: String,
    /// Audio path, preserving original separators.
    pub path: String,
    /// ISRC text; empty means absent, otherwise twelve ASCII alphanumeric characters.
    pub isrc: String,
    /// Reserved authoring values; defaults describe a writer policy, not parser invariants.
    pub creation: TrackCreationOptions,
}

impl NewContent {
    /// Start a track with an explicit export ID and audio path using forward slashes.
    ///
    /// The file name is the last slash-separated component of the supplied path.
    /// Other metadata remains zero or empty. Insertion validates the ID and path.
    pub fn new(id: u32, path: impl Into<String>) -> Self {
        let path = path.into();
        let file_name = path.rsplit('/').next().unwrap_or_default().to_owned();
        Self {
            id,
            path,
            file_name,
            ..Self::default()
        }
    }

    /// Insert a fresh row and return its decoded snapshot.
    ///
    /// Duplicate IDs, encoding errors and allocation failures leave the connection
    /// unchanged. Existing export-property counts are updated in the same transaction;
    /// a bare database without property rows remains supported. This commits to the
    /// connection's overlay only, not to disk.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Content> {
        conn.check_kind(Content::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        let index = tx.index_rows(Content::TABLE, Content::read_id)?;
        if index.contains_key(&self.id) {
            return Err(Error::DuplicateId {
                table: Content::TABLE,
                id: self.id,
            });
        }
        let count = u32::try_from(index.len())
            .map_err(|_| Error::MutationOverflow)?
            .checked_add(1)
            .ok_or(Error::MutationOverflow)?;
        let handle = tx.insert(Content::TABLE, &Row::Standard(StandardRow::Track(row)))?;
        update_export_count(&mut tx, count)?;
        // Perform every fallible conversion before publishing the transaction.
        let model = Content::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Build a binary row without assigning a page, slot, or string offsets.
    ///
    /// Useful for explicit binary transactions. Direct engine inserts do not
    /// check duplicate model IDs or maintain export-property counts.
    pub fn to_binary(&self) -> Result<TrackRow> {
        if self.id == 0 {
            return Err(Error::InvalidModelField {
                table: Content::TABLE,
                field: "id",
                reason: "export track IDs must be nonzero",
            });
        }
        if self.path.is_empty() {
            return Err(Error::InvalidModelField {
                table: Content::TABLE,
                field: "path",
                reason: "audio path must not be empty",
            });
        }
        Ok(TrackRow {
            header: TrackHeader {
                subtype: TrackHeader::SUBTYPE,
                index_shift: 0,
                content_link: self.creation.content_link,
                raw_analysis: self.creation.raw_analysis,
                id: self.id,
                artist_id: self.artist_id,
                album_id: self.album_id,
                genre_id: self.genre_id,
                label_id: self.label_id,
                key_id: self.key_id,
                artwork_id: self.artwork_id,
                composer_id: self.composer_id,
                original_artist_id: self.original_artist_id,
                remixer_id: self.remixer_id,
                master_content_id: self.master_content_id,
                master_db_id: self.master_db_id,
                sample_rate: self.sample_rate,
                file_size: self.file_size,
                bitrate: self.bitrate,
                tempo: self.bpmx100,
                duration: self.length,
                track_number: self.track_no,
                disc_number: self.disc_no,
                play_count: self.play_count,
                year: self.release_year,
                sample_depth: self.bit_depth,
                color_id: self.color_id,
                rating: self.rating,
                audio_format: self.file_type,
            },
            strings: StringOffsets::Far {
                marker: self.creation.string_marker,
                offsets: [0; 21],
            },
            text: TrackStrings {
                isrc: DeviceSqlString::encode_isrc(&self.isrc)?,
                title: text(&self.title)?,
                comment: text(&self.comment)?,
                mix_name: text(&self.mix_name)?,
                lyricist: text(&self.lyricist)?,
                message: text(&self.message)?,
                publish_track_info: text(&self.publish_track_info)?,
                hot_cue_auto_load: text(&self.hot_cue_auto_load)?,
                date_added: text(&self.date_added)?,
                release_date: text(&self.release_date)?,
                analysis_date: text(&self.analysis_date)?,
                analysis_path: text(&self.analysis_path)?,
                filename: text(&self.file_name)?,
                file_path: text(&self.path)?,
                unknown_02: text(&self.creation.unknown_strings[0])?,
                unknown_03: text(&self.creation.unknown_strings[1])?,
                unknown_04: text(&self.creation.unknown_strings[2])?,
                unknown_08: text(&self.creation.unknown_strings[3])?,
                unknown_09: text(&self.creation.unknown_strings[4])?,
                unknown_13: text(&self.creation.unknown_strings[5])?,
                unknown_18: text(&self.creation.unknown_strings[6])?,
            },
        })
    }
}

/// Convert resolved metadata without querying the database or loading assets.
/// Stored zeroes and empty strings are retained; absent relationships stay `None`.
/// Analysis dates have no equivalent in `Track` and are not mapped to creation dates.
/// Unknown ISRC encodings and ratings outside 0..=5 return an error.
impl TryFrom<&ContentView> for Track {
    type Error = Error;

    fn try_from(view: &ContentView) -> Result<Self> {
        let row = &view.content;
        if row.rating > 5 {
            return Err(track_mapping_error(
                "rating",
                "rating must be between 0 and 5",
            ));
        }
        Ok(Self {
            id: Some(row.id),
            source: TrackSource::DeviceLib,
            master_content_id: (row.master_content_id != 0).then_some(row.master_content_id),
            master_db_id: (row.master_db_id != 0).then_some(row.master_db_id),
            path: PathBuf::from(&row.path),
            file_type: Some(i32::from(row.file_type)),
            artwork_path: view
                .artwork
                .as_ref()
                .map(|value| PathBuf::from(&value.path)),
            analysis_data_path: Some(PathBuf::from(&row.analysis_path)),
            color: Some(row.color_id),
            play_count: Some(u32::from(row.play_count)),
            hot_cue_auto_load: Some(row.hot_cue_auto_load.clone()),
            delivery_control: Some(row.publish_track_info.clone()),
            delivery_comment: Some(row.message.clone()),
            title: Some(row.title.clone()),
            commnt: Some(row.comment.clone()),
            subtitle: Some(row.mix_name.clone()),
            lyricist: Some(row.lyricist.clone()),
            stock_date: Some(row.date_added.clone()),
            release_date: Some(row.release_date.clone()),
            isrc: Some(row.isrc()?),
            rating: Some(row.rating),
            bpm: Some(f64::from(row.bpmx100) / 100.0),
            length: Some(f64::from(row.length)),
            track_no: Some(row.track_no),
            disc_no: Some(u32::from(row.disc_no)),
            bit_rate: Some(row.bitrate),
            bit_depth: Some(row.bit_depth),
            sample_rate: Some(row.sample_rate),
            release_year: Some(row.release_year),
            file_size: Some(u64::from(row.file_size)),
            artist: view.artist.as_ref().map(|value| value.name.clone()),
            album: view.album.as_ref().map(|value| value.name.clone()),
            album_artist: view.album_artist.as_ref().map(|value| value.name.clone()),
            remixer: view.remixer.as_ref().map(|value| value.name.clone()),
            org_artist: view
                .original_artist
                .as_ref()
                .map(|value| value.name.clone()),
            composer: view.composer.as_ref().map(|value| value.name.clone()),
            genre: view.genre.as_ref().map(|value| value.name.clone()),
            label: view.label.as_ref().map(|value| value.name.clone()),
            key: view.key.as_ref().map(|value| value.name.clone()),
            ..Self::default()
        })
    }
}

impl TryFrom<ContentView> for Track {
    type Error = Error;

    fn try_from(view: ContentView) -> Result<Self> {
        Self::try_from(&view)
    }
}

fn track_mapping_error(field: &'static str, reason: &'static str) -> Error {
    Error::InvalidModelField {
        table: Content::TABLE,
        field,
        reason,
    }
}

/// Build an insertion row without database access or asset copying.
///
/// `Track::id` must be a nonzero u32 export ID; assign a destination ID first when
/// importing from another library. Absent master IDs use the stored zero sentinel.
/// Relationship and artwork IDs remain zero: resolve names and assign destination
/// references before inserting. Paths use Unix separators; the basename is derived
/// from the audio path. BPM is rounded to hundredths and length to whole seconds.
/// Missing values use the normal `NewContent` defaults. Unsupported metadata
/// (UUID, search text, tags and creation date) is omitted, and analysis dates are
/// not inferred. Reserved binary values use the engine's creation defaults.
impl TryFrom<&Track> for NewContent {
    type Error = Error;

    fn try_from(input: &Track) -> Result<Self> {
        let id = input
            .id
            .ok_or_else(|| track_mapping_error("id", "export ID must be assigned"))?;
        validate_id(Content::TABLE, id)?;
        let path = input
            .path
            .normalize_unix_string()
            .ok_or_else(|| track_mapping_error("path", "path must be UTF-8"))?;
        if path.is_empty() {
            return Err(track_mapping_error("path", "audio path must not be empty"));
        }
        let rounded = |field, value: f64, scale: f64, max: f64| -> Result<u32> {
            let result = (value * scale).round();
            if !value.is_finite() || value < 0.0 || result > max {
                return Err(track_mapping_error(
                    field,
                    "value must be finite, nonnegative and fit the stored range",
                ));
            }
            Ok(result as u32)
        };
        if input.rating.is_some_and(|value| value > 5) {
            return Err(track_mapping_error(
                "rating",
                "rating must be between 0 and 5",
            ));
        }
        let mut row = Self::new(id, path);
        row.master_content_id = input.master_content_id.unwrap_or_default();
        row.master_db_id = input.master_db_id.unwrap_or_default();
        row.file_type = u16::try_from(input.file_type.unwrap_or_default())
            .map_err(|_| track_mapping_error("file_type", "value exceeds u16 range"))?;
        row.file_size = u32::try_from(input.file_size.unwrap_or_default())
            .map_err(|_| track_mapping_error("file_size", "value exceeds u32 range"))?;
        row.disc_no = u16::try_from(input.disc_no.unwrap_or_default())
            .map_err(|_| track_mapping_error("disc_no", "value exceeds u16 range"))?;
        row.play_count = u16::try_from(input.play_count.unwrap_or_default())
            .map_err(|_| track_mapping_error("play_count", "value exceeds u16 range"))?;
        row.bpmx100 = rounded(
            "bpm",
            input.bpm.unwrap_or_default(),
            100.0,
            f64::from(u32::MAX),
        )?;
        row.length = rounded(
            "length",
            input.length.unwrap_or_default(),
            1.0,
            f64::from(u16::MAX),
        )? as u16;
        row.rating = input.rating.unwrap_or_default();
        row.color_id = input.color.unwrap_or_default();
        row.track_no = input.track_no.unwrap_or_default();
        row.bitrate = input.bit_rate.unwrap_or_default();
        row.bit_depth = input.bit_depth.unwrap_or_default();
        row.sample_rate = input.sample_rate.unwrap_or_default();
        row.release_year = input.release_year.unwrap_or_default();
        row.title = input.title.clone().unwrap_or_default();
        row.comment = input.commnt.clone().unwrap_or_default();
        row.mix_name = input.subtitle.clone().unwrap_or_default();
        row.lyricist = input.lyricist.clone().unwrap_or_default();
        row.date_added = input.stock_date.clone().unwrap_or_default();
        row.release_date = input.release_date.clone().unwrap_or_default();
        row.hot_cue_auto_load = input.hot_cue_auto_load.clone().unwrap_or_default();
        row.publish_track_info = input.delivery_control.clone().unwrap_or_default();
        row.message = input.delivery_comment.clone().unwrap_or_default();
        row.analysis_path = input
            .analysis_data_path
            .as_ref()
            .map(|path| {
                path.normalize_unix_string()
                    .ok_or_else(|| track_mapping_error("analysis_path", "path must be UTF-8"))
            })
            .transpose()?
            .unwrap_or_default();
        row.isrc = input.isrc.clone().unwrap_or_default();
        DeviceSqlString::encode_isrc(&row.isrc)?;
        Ok(row)
    }
}

impl TryFrom<Track> for NewContent {
    type Error = Error;

    fn try_from(input: Track) -> Result<Self> {
        Self::try_from(&input)
    }
}
