// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: MIT

//! Shared track metadata and library-track models, including optional bindings.

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use std::path::Path;

/// Source-neutral track data. Names are resolved to relationships during planning.
/// Unknown metadata stays `None`; length is seconds,
/// bitrate is kbps, sample rate is Hz, and file size is bytes.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all))]
pub struct TrackMetadata {
    /// Local file path; planning makes it absolute and normalizes separators.
    pub path: std::path::PathBuf,
    /// Explicit format, otherwise inferred from the extension.
    pub file_type: Option<i32>,
    /// Rating in the master database's 0..=5 scale.
    pub rating: Option<u8>,
    pub search_str: Option<String>,
    pub tag: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    /// Tempo in beats per minute, without database scaling.
    pub bpm: Option<f64>,
    /// Duration in seconds, including fractional seconds.
    pub length: Option<f64>,
    pub track_no: Option<u32>,
    pub bit_rate: Option<u32>,
    pub bit_depth: Option<u16>,
    pub commnt: Option<String>,
    pub release_year: Option<u16>,
    pub remixer: Option<String>,
    pub label: Option<String>,
    pub org_artist: Option<String>,
    /// Musical key name from tags; resolved to a djmdKey ID by the planner.
    pub key: Option<String>,
    pub stock_date: Option<String>,
    /// Explicit size in bytes. When absent, the planner reads filesystem metadata.
    pub file_size: Option<u64>,
    pub disc_no: Option<u32>,
    pub composer: Option<String>,
    pub subtitle: Option<String>,
    pub sample_rate: Option<u32>,
    pub release_date: Option<String>,
    pub date_created: Option<String>,
    pub lyricist: Option<String>,
    pub isrc: Option<String>,
}

impl TrackMetadata {
    /// Create input with only the path populated.
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
}

/// A track in the Rekordbox library.
///
/// This is the same as `TrackMetadata` but with additional fields for the Rekordbox database.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all))]
pub struct Track {
    /// Unique Rekordbox track ID.
    pub id: String,
    /// Unique Rekordbox track UUID.
    pub uuid: String,
    pub path: std::path::PathBuf,
    pub file_type: Option<i32>,
    pub rating: Option<u8>,
    pub search_str: Option<String>,
    pub tag: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    /// Tempo in beats per minute, without database scaling.
    pub bpm: Option<f64>,
    /// Duration in seconds, including fractional seconds.
    pub length: Option<f64>,
    pub track_no: Option<u32>,
    pub bit_rate: Option<u32>,
    pub bit_depth: Option<u16>,
    pub commnt: Option<String>,
    pub release_year: Option<u16>,
    pub remixer: Option<String>,
    pub label: Option<String>,
    pub org_artist: Option<String>,
    pub key: Option<String>,
    pub stock_date: Option<String>,
    pub file_size: Option<u64>,
    pub disc_no: Option<u32>,
    pub composer: Option<String>,
    pub subtitle: Option<String>,
    pub sample_rate: Option<u32>,
    pub release_date: Option<String>,
    pub date_created: Option<String>,
    pub lyricist: Option<String>,
    pub isrc: Option<String>,
}

impl Track {
    /// Create input with only the path populated.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            ..Default::default()
        }
    }
}

impl From<TrackMetadata> for Track {
    fn from(metadata: TrackMetadata) -> Self {
        Self {
            id: String::new(),
            uuid: String::new(),
            path: metadata.path,
            file_type: metadata.file_type,
            rating: metadata.rating,
            search_str: metadata.search_str,
            tag: metadata.tag,
            title: metadata.title,
            artist: metadata.artist,
            album: metadata.album,
            album_artist: metadata.album_artist,
            genre: metadata.genre,
            bpm: metadata.bpm,
            length: metadata.length,
            track_no: metadata.track_no,
            bit_rate: metadata.bit_rate,
            bit_depth: metadata.bit_depth,
            commnt: metadata.commnt,
            release_year: metadata.release_year,
            remixer: metadata.remixer,
            label: metadata.label,
            org_artist: metadata.org_artist,
            key: metadata.key,
            stock_date: metadata.stock_date,
            file_size: metadata.file_size,
            disc_no: metadata.disc_no,
            composer: metadata.composer,
            subtitle: metadata.subtitle,
            sample_rate: metadata.sample_rate,
            release_date: metadata.release_date,
            date_created: metadata.date_created,
            lyricist: metadata.lyricist,
            isrc: metadata.isrc,
        }
    }
}

impl From<Track> for TrackMetadata {
    fn from(track: Track) -> Self {
        TrackMetadata {
            path: track.path,
            file_type: track.file_type,
            rating: track.rating,
            search_str: track.search_str,
            tag: track.tag,
            title: track.title,
            artist: track.artist,
            album: track.album,
            album_artist: track.album_artist,
            genre: track.genre,
            bpm: track.bpm,
            length: track.length,
            track_no: track.track_no,
            bit_rate: track.bit_rate,
            bit_depth: track.bit_depth,
            commnt: track.commnt,
            release_year: track.release_year,
            remixer: track.remixer,
            label: track.label,
            org_artist: track.org_artist,
            key: track.key,
            stock_date: track.stock_date,
            file_size: track.file_size,
            disc_no: track.disc_no,
            composer: track.composer,
            subtitle: track.subtitle,
            sample_rate: track.sample_rate,
            release_date: track.release_date,
            date_created: track.date_created,
            lyricist: track.lyricist,
            isrc: track.isrc,
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl TrackMetadata {
    #[new]
    fn py_new(path: std::path::PathBuf) -> Self {
        Self::new(path)
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Track {
    #[new]
    fn py_new(path: std::path::PathBuf) -> Self {
        Self::new(path)
    }

    /// Copy metadata into a track with empty database identifiers.
    #[staticmethod]
    #[pyo3(name = "from_metadata")]
    fn py_from_metadata(metadata: TrackMetadata) -> Self {
        metadata.into()
    }

    /// Copy the track's metadata, omitting its database identifiers.
    #[pyo3(name = "to_metadata")]
    fn py_to_metadata(&self) -> TrackMetadata {
        self.clone().into()
    }
}
