// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Content snapshots and explicit read queries over the DeviceSQL engine.
//!
//! Content is the model name for DeviceSQL tracks, matching masterdb and OneLibrary
//! terminology. The underlying binary representation remains `TrackRow`.

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};

use binrw::BinRead;

use super::{
    load_related, text, text_error, unsupported, update_export_count, Album, Artist, Artwork,
    Color, Genre, Key, Label,
};
use crate::binary::{
    DeviceSqlString, PageType, Row, RowRecord, StandardRow, StringOffsets, TableKind, TrackHeader,
    TrackRow, TrackStrings,
};
use crate::devicesql::DeviceSqlConnection;
pub use crate::devicesql::TrackCreationOptions;

use crate::{Error, Result};

/// A detached track from the standard database's track table.
///
/// Named `Content` for consistency with masterdb and OneLibrary models-old; this wraps
/// the binary [`TrackRow`] and uses [`PageType::Tracks`].
///
/// Public fields contain decoded values, like the masterdb/onelib models-old.
/// Editing a field only changes this snapshot; model updates are not implemented.
/// Use [`super::NewContent`] to insert a fresh track without an existing raw row.
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

    /// Decode the original ISRC frame, or report an unrecognized encoding.
    pub fn isrc(&self) -> Result<String> {
        let Row::Standard(StandardRow::Track(row)) = self.raw.decode()? else {
            return Err(unsupported(&self.raw));
        };
        row.text
            .isrc
            .isrc_text()
            .map_err(|source| text_error(&self.raw, "isrc", source))
    }

    pub(super) fn read_id(record: &RowRecord) -> Result<u32> {
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
        Ok(header.id)
    }
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
