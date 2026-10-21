// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Core track objects with checked JavaScript integer and path conversions.

use napi::bindgen_prelude::BigInt;
use napi::{Error, Result};
use napi_derive::napi;
use rbox::core::models::{Track as CoreTrack, TrackMetadata as CoreTrackMetadata};

/// Create metadata with only the path populated.
#[napi]
pub fn create_track_metadata(path: String) -> TrackMetadata {
    CoreTrackMetadata::new(path).into()
}

/// Create a library track with empty database identifiers.
#[napi]
pub fn create_track(path: String) -> Track {
    CoreTrack::new(path).into()
}

/// Copy metadata into a track with empty database identifiers.
#[napi]
pub fn track_from_metadata(metadata: TrackMetadata) -> Result<Track> {
    Ok(CoreTrack::from(CoreTrackMetadata::try_from(metadata)?).into())
}

/// Copy metadata, omitting database identifiers.
#[napi]
pub fn track_to_metadata(track: Track) -> Result<TrackMetadata> {
    let metadata: CoreTrackMetadata = CoreTrack::try_from(track)?.into();
    Ok(metadata.into())
}

fn unsigned(value: f64, max: f64, field: &str) -> Result<u32> {
    if !value.is_finite() || value < 0.0 || value > max || value.fract() != 0.0 {
        return Err(Error::from_reason(format!(
            "{field} must be an integer between 0 and {max}"
        )));
    }
    Ok(value as u32)
}

fn file_size(value: BigInt) -> Result<u64> {
    let (negative, value, lossless) = value.get_u64();
    if negative || !lossless {
        return Err(Error::from_reason(
            "fileSize must fit an unsigned 64-bit integer",
        ));
    }
    Ok(value)
}

/// Shared track data. BPM is unscaled, length is seconds, fileSize is bytes as bigint.
#[napi(object)]
pub struct TrackMetadata {
    pub path: String,
    pub file_type: Option<i32>,
    pub rating: Option<f64>,
    pub search_str: Option<String>,
    pub tag: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub bpm: Option<f64>,
    pub length: Option<f64>,
    pub track_no: Option<f64>,
    pub bit_rate: Option<f64>,
    pub bit_depth: Option<f64>,
    pub commnt: Option<String>,
    pub release_year: Option<f64>,
    pub remixer: Option<String>,
    pub label: Option<String>,
    pub org_artist: Option<String>,
    pub key: Option<String>,
    pub stock_date: Option<String>,
    pub file_size: Option<BigInt>,
    pub disc_no: Option<f64>,
    pub composer: Option<String>,
    pub subtitle: Option<String>,
    pub sample_rate: Option<f64>,
    pub release_date: Option<String>,
    pub date_created: Option<String>,
    pub lyricist: Option<String>,
    pub isrc: Option<String>,
}

impl From<CoreTrackMetadata> for TrackMetadata {
    fn from(value: CoreTrackMetadata) -> Self {
        Self {
            path: value.path.to_string_lossy().into_owned(),
            file_type: value.file_type,
            rating: value.rating.map(f64::from),
            search_str: value.search_str,
            tag: value.tag,
            title: value.title,
            artist: value.artist,
            album: value.album,
            album_artist: value.album_artist,
            genre: value.genre,
            bpm: value.bpm,
            length: value.length,
            track_no: value.track_no.map(f64::from),
            bit_rate: value.bit_rate.map(f64::from),
            bit_depth: value.bit_depth.map(f64::from),
            commnt: value.commnt,
            release_year: value.release_year.map(f64::from),
            remixer: value.remixer,
            label: value.label,
            org_artist: value.org_artist,
            key: value.key,
            stock_date: value.stock_date,
            file_size: value.file_size.map(BigInt::from),
            disc_no: value.disc_no.map(f64::from),
            composer: value.composer,
            subtitle: value.subtitle,
            sample_rate: value.sample_rate.map(f64::from),
            release_date: value.release_date,
            date_created: value.date_created,
            lyricist: value.lyricist,
            isrc: value.isrc,
        }
    }
}

impl TryFrom<TrackMetadata> for CoreTrackMetadata {
    type Error = Error;
    fn try_from(value: TrackMetadata) -> Result<Self> {
        Ok(Self {
            path: value.path.into(),
            file_type: value.file_type,
            rating: value
                .rating
                .map(|n| unsigned(n, u8::MAX as f64, "rating").map(|n| n as u8))
                .transpose()?,
            search_str: value.search_str,
            tag: value.tag,
            title: value.title,
            artist: value.artist,
            album: value.album,
            album_artist: value.album_artist,
            genre: value.genre,
            bpm: value.bpm,
            length: value.length,
            track_no: value
                .track_no
                .map(|n| unsigned(n, u32::MAX as f64, "track_no").map(|n| n as u32))
                .transpose()?,
            bit_rate: value
                .bit_rate
                .map(|n| unsigned(n, u32::MAX as f64, "bit_rate").map(|n| n as u32))
                .transpose()?,
            bit_depth: value
                .bit_depth
                .map(|n| unsigned(n, u16::MAX as f64, "bit_depth").map(|n| n as u16))
                .transpose()?,
            commnt: value.commnt,
            release_year: value
                .release_year
                .map(|n| unsigned(n, u16::MAX as f64, "release_year").map(|n| n as u16))
                .transpose()?,
            remixer: value.remixer,
            label: value.label,
            org_artist: value.org_artist,
            key: value.key,
            stock_date: value.stock_date,
            file_size: value.file_size.map(file_size).transpose()?,
            disc_no: value
                .disc_no
                .map(|n| unsigned(n, u32::MAX as f64, "disc_no").map(|n| n as u32))
                .transpose()?,
            composer: value.composer,
            subtitle: value.subtitle,
            sample_rate: value
                .sample_rate
                .map(|n| unsigned(n, u32::MAX as f64, "sample_rate").map(|n| n as u32))
                .transpose()?,
            release_date: value.release_date,
            date_created: value.date_created,
            lyricist: value.lyricist,
            isrc: value.isrc,
        })
    }
}

/// Shared track data. BPM is unscaled, length is seconds, fileSize is bytes as bigint.
#[napi(object)]
pub struct Track {
    pub id: String,
    pub uuid: String,
    pub path: String,
    pub file_type: Option<i32>,
    pub rating: Option<f64>,
    pub search_str: Option<String>,
    pub tag: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub bpm: Option<f64>,
    pub length: Option<f64>,
    pub track_no: Option<f64>,
    pub bit_rate: Option<f64>,
    pub bit_depth: Option<f64>,
    pub commnt: Option<String>,
    pub release_year: Option<f64>,
    pub remixer: Option<String>,
    pub label: Option<String>,
    pub org_artist: Option<String>,
    pub key: Option<String>,
    pub stock_date: Option<String>,
    pub file_size: Option<BigInt>,
    pub disc_no: Option<f64>,
    pub composer: Option<String>,
    pub subtitle: Option<String>,
    pub sample_rate: Option<f64>,
    pub release_date: Option<String>,
    pub date_created: Option<String>,
    pub lyricist: Option<String>,
    pub isrc: Option<String>,
}

impl From<CoreTrack> for Track {
    fn from(value: CoreTrack) -> Self {
        Self {
            id: value.id,
            uuid: value.uuid,
            path: value.path.to_string_lossy().into_owned(),
            file_type: value.file_type,
            rating: value.rating.map(f64::from),
            search_str: value.search_str,
            tag: value.tag,
            title: value.title,
            artist: value.artist,
            album: value.album,
            album_artist: value.album_artist,
            genre: value.genre,
            bpm: value.bpm,
            length: value.length,
            track_no: value.track_no.map(f64::from),
            bit_rate: value.bit_rate.map(f64::from),
            bit_depth: value.bit_depth.map(f64::from),
            commnt: value.commnt,
            release_year: value.release_year.map(f64::from),
            remixer: value.remixer,
            label: value.label,
            org_artist: value.org_artist,
            key: value.key,
            stock_date: value.stock_date,
            file_size: value.file_size.map(BigInt::from),
            disc_no: value.disc_no.map(f64::from),
            composer: value.composer,
            subtitle: value.subtitle,
            sample_rate: value.sample_rate.map(f64::from),
            release_date: value.release_date,
            date_created: value.date_created,
            lyricist: value.lyricist,
            isrc: value.isrc,
        }
    }
}

impl TryFrom<Track> for CoreTrack {
    type Error = Error;
    fn try_from(value: Track) -> Result<Self> {
        Ok(Self {
            id: value.id,
            uuid: value.uuid,
            path: value.path.into(),
            file_type: value.file_type,
            rating: value
                .rating
                .map(|n| unsigned(n, u8::MAX as f64, "rating").map(|n| n as u8))
                .transpose()?,
            search_str: value.search_str,
            tag: value.tag,
            title: value.title,
            artist: value.artist,
            album: value.album,
            album_artist: value.album_artist,
            genre: value.genre,
            bpm: value.bpm,
            length: value.length,
            track_no: value
                .track_no
                .map(|n| unsigned(n, u32::MAX as f64, "track_no").map(|n| n as u32))
                .transpose()?,
            bit_rate: value
                .bit_rate
                .map(|n| unsigned(n, u32::MAX as f64, "bit_rate").map(|n| n as u32))
                .transpose()?,
            bit_depth: value
                .bit_depth
                .map(|n| unsigned(n, u16::MAX as f64, "bit_depth").map(|n| n as u16))
                .transpose()?,
            commnt: value.commnt,
            release_year: value
                .release_year
                .map(|n| unsigned(n, u16::MAX as f64, "release_year").map(|n| n as u16))
                .transpose()?,
            remixer: value.remixer,
            label: value.label,
            org_artist: value.org_artist,
            key: value.key,
            stock_date: value.stock_date,
            file_size: value.file_size.map(file_size).transpose()?,
            disc_no: value
                .disc_no
                .map(|n| unsigned(n, u32::MAX as f64, "disc_no").map(|n| n as u32))
                .transpose()?,
            composer: value.composer,
            subtitle: value.subtitle,
            sample_rate: value
                .sample_rate
                .map(|n| unsigned(n, u32::MAX as f64, "sample_rate").map(|n| n as u32))
                .transpose()?,
            release_date: value.release_date,
            date_created: value.date_created,
            lyricist: value.lyricist,
            isrc: value.isrc,
        })
    }
}
