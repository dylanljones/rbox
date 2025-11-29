// Author: Dylan Jones
// Date:   2025-05-03

use std::convert::TryFrom;

#[repr(i32)]
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    MP3 = 0,
    MP3_2 = 1,
    MP4 = 3,
    ALAC = 4,
    FLAC = 5,
    M4A = 6,
    WAV = 11,
    AIFF = 12,
}

impl TryFrom<i32> for FileType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::MP3),
            1 => Ok(Self::MP3_2),
            3 => Ok(Self::MP4),
            4 => Ok(Self::ALAC),
            5 => Ok(Self::FLAC),
            6 => Ok(Self::M4A),
            11 => Ok(Self::WAV),
            12 => Ok(Self::AIFF),
            _ => Err("Invalid value for FileType".into()),
        }
    }
}

impl FileType {
    pub fn try_from_extension(ext: &str) -> std::result::Result<Self, String> {
        match ext.to_lowercase().as_str() {
            "mp3" => Ok(Self::MP3),
            "mp4" => Ok(Self::MP4),
            "m4a" => Ok(Self::ALAC),
            "flac" => Ok(Self::FLAC),
            "wav" => Ok(Self::WAV),
            "aiff" => Ok(Self::AIFF),
            "aif" => Ok(Self::AIFF),
            &_ => Err(format!("Unknown file type '{}'", ext)),
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
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Analyzed::NotAnalyzed),
            105 => Ok(Analyzed::Standard),
            121 => Ok(Analyzed::Advanced),
            233 => Ok(Analyzed::Locked),
            _ => Err("Invalid value for Analyzed".into()),
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
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Advanced),
            _ => Err("Invalid value for AnalysisUpdated".into()),
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, PartialEq)]
pub enum PlaylistType {
    Playlist = 0,
    Folder = 1,
    SmartPlaylist = 4,
    CloudLibrarySync = -128,
}

impl TryFrom<i32> for PlaylistType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Playlist),
            1 => Ok(Self::Folder),
            4 => Ok(Self::SmartPlaylist),
            -128 => Ok(Self::CloudLibrarySync),
            _ => Err(format!("Invalid value for PlaylistType: {}", value)),
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, PartialEq)]
pub enum MyTagType {
    MyTag = 0,
    Folder = 1,
}

impl TryFrom<i32> for MyTagType {
    type Error = String;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::MyTag),
            1 => Ok(Self::Folder),
            _ => Err("Invalid value for MyTagType".into()),
        }
    }
}
