// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! File type codes shared by Rekordbox formats. Raw model fields retain unknown codes.

#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Known Rekordbox file type codes; use raw integers to retain unknown values.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, eq, eq_int))]
#[cfg_attr(feature = "napi", napi)]
pub enum FileType {
    /// Content that rekordbox could not read; this is a known on-disk code.
    #[default]
    INVALID = 0,
    MP3 = 1,
    MP4 = 3,
    M4A = 4,
    FLAC = 5,
    ALAC = 6,
    WAV = 11,
    AIFF = 12,
    VIDEO = 16,
    StreamApple = 26,
}

impl TryFrom<i32> for FileType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::INVALID),
            1 => Ok(Self::MP3),
            3 => Ok(Self::MP4),
            4 => Ok(Self::M4A),
            5 => Ok(Self::FLAC),
            6 => Ok(Self::ALAC),
            11 => Ok(Self::WAV),
            12 => Ok(Self::AIFF),
            16 => Ok(Self::VIDEO),
            26 => Ok(Self::StreamApple),
            _ => Err("Invalid value for FileType".into()),
        }
    }
}

impl FileType {
    /// AAC audio uses the same database code as M4A.
    pub const AAC: Self = Self::M4A;
    /// Alternate extension spelling for AIFF.
    pub const AIF: Self = Self::AIFF;

    /// Infer a file type from an extension without its leading dot.
    ///
    /// Matching is case-insensitive. M4A defaults to AAC (code 4); distinguishing
    /// ALAC (code 6) in the same container requires inspecting the audio codec.
    pub fn try_from_extension(ext: &str) -> std::result::Result<Self, String> {
        match ext.to_lowercase().as_str() {
            "mp3" => Ok(Self::MP3),
            "mp4" => Ok(Self::MP4),
            "m4a" | "aac" => Ok(Self::M4A),
            "alac" => Ok(Self::ALAC),
            "flac" => Ok(Self::FLAC),
            "wav" => Ok(Self::WAV),
            "aiff" | "aif" => Ok(Self::AIFF),
            "avi" | "m4v" | "mov" | "mpg" => Ok(Self::VIDEO),
            &_ => Err(format!("Unknown file type '{}'", ext)),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl FileType {
    #[classattr]
    #[pyo3(name = "AAC")]
    fn aac_alias() -> Self {
        Self::AAC
    }

    #[classattr]
    #[pyo3(name = "AIF")]
    fn aif_alias() -> Self {
        Self::AIF
    }
}
