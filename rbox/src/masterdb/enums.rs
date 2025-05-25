// Author: Dylan Jones
// Date:   2025-05-03

use std::convert::TryFrom;

#[repr(i32)]
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    MP3 = 0,
    MP3_2 = 1,
    M4A = 4,
    FLAC = 5,
    WAV = 11,
    AIFF = 12,
}

impl TryFrom<i32> for FileType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FileType::MP3),
            1 => Ok(FileType::MP3_2),
            4 => Ok(FileType::M4A),
            5 => Ok(FileType::FLAC),
            11 => Ok(FileType::WAV),
            12 => Ok(FileType::AIFF),
            _ => Err("Invalid value for FileType"),
        }
    }
}

impl FileType {
    pub fn try_from_extension(ext: &str) -> anyhow::Result<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Ok(FileType::MP3),
            "m4a" => Ok(FileType::M4A),
            "flac" => Ok(FileType::FLAC),
            "wav" => Ok(FileType::WAV),
            "aiff" => Ok(FileType::AIFF),
            "aif" => Ok(FileType::AIFF),
            &_ => Err(anyhow::anyhow!("Unknown file type '{}'", ext)),
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, PartialEq)]
pub enum Analyzed {
    NotAnalyzed = 0,
    Standard = 105,
    Advanced = 121,
    Locked = 233,
}

impl TryFrom<i32> for Analyzed {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Analyzed::NotAnalyzed),
            105 => Ok(Analyzed::Standard),
            121 => Ok(Analyzed::Advanced),
            233 => Ok(Analyzed::Locked),
            _ => Err("Invalid value for Analyzed"),
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisUpdated {
    Normal = 0,
    Advanced = 1,
}

impl TryFrom<i32> for AnalysisUpdated {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AnalysisUpdated::Normal),
            1 => Ok(AnalysisUpdated::Advanced),
            _ => Err("Invalid value for AnalysisUpdated"),
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, PartialEq)]
pub enum PlaylistType {
    Playlist = 0,
    Folder = 1,
    SmartPlaylist = 4,
}

impl TryFrom<i32> for PlaylistType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PlaylistType::Playlist),
            1 => Ok(PlaylistType::Folder),
            4 => Ok(PlaylistType::SmartPlaylist),
            _ => Err("Invalid value for PlaylistType"),
        }
    }
}
