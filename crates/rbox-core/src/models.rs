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

impl Into<TrackMetadata> for Track {
    fn into(self) -> TrackMetadata {
        TrackMetadata {
            path: self.path,
            file_type: self.file_type,
            rating: self.rating,
            search_str: self.search_str,
            tag: self.tag,
            title: self.title,
            artist: self.artist,
            album: self.album,
            album_artist: self.album_artist,
            genre: self.genre,
            bpm: self.bpm,
            length: self.length,
            track_no: self.track_no,
            bit_rate: self.bit_rate,
            bit_depth: self.bit_depth,
            commnt: self.commnt,
            release_year: self.release_year,
            remixer: self.remixer,
            label: self.label,
            org_artist: self.org_artist,
            key: self.key,
            stock_date: self.stock_date,
            file_size: self.file_size,
            disc_no: self.disc_no,
            composer: self.composer,
            subtitle: self.subtitle,
            sample_rate: self.sample_rate,
            release_date: self.release_date,
            date_created: self.date_created,
            lyricist: self.lyricist,
            isrc: self.isrc,
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
