// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Typed errors returned by the ANLZ reader, editor, and writer.

use std::path::PathBuf;

use crate::sections::AnlzTag;

pub type Result<T> = std::result::Result<T, AnlzError>;

/// The structural reason that an ANLZ file is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidFileReason {
    HeaderOrLength,
    TruncatedSectionHeader,
    SectionOffsetOverflow,
    InvalidSectionBounds,
    SectionSizeMismatch,
    BackingFileTruncated,
}

impl std::fmt::Display for InvalidFileReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::HeaderOrLength => "invalid file header or declared file length",
            Self::TruncatedSectionHeader => "truncated section header",
            Self::SectionOffsetOverflow => "section offset overflow",
            Self::InvalidSectionBounds => "invalid section bounds",
            Self::SectionSizeMismatch => "section parser did not consume its declared size",
            Self::BackingFileTruncated => "backing file was truncated",
        })
    }
}

/// Errors returned while reading, modifying, or writing an ANLZ file.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AnlzError {
    /// An underlying filesystem operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// A binary structure could not be decoded or encoded.
    #[error(transparent)]
    Binary(#[from] binrw::error::Error),

    /// The file or a section has invalid outer structure.
    #[error("invalid ANLZ file at byte {offset}: {reason}")]
    InvalidFile {
        reason: InvalidFileReason,
        offset: u64,
    },

    /// Decoding a lazily loaded section failed.
    #[error("failed to decode section {index} ({tag:?}) at byte {offset}: {source}")]
    SectionDecode {
        index: usize,
        tag: AnlzTag,
        offset: u64,
        #[source]
        source: Box<AnlzError>,
    },

    /// A section's public tag does not describe its content variant.
    #[error("section tag {tag:?} does not match its content")]
    TagContentMismatch { tag: AnlzTag },

    /// The requested destination cannot identify an output file.
    #[error("invalid ANLZ output path: {0}")]
    InvalidOutputPath(PathBuf),

    /// The serialized file cannot be represented by ANLZ's 32-bit size fields.
    #[error("ANLZ output size {size} exceeds the 32-bit format limit")]
    OutputTooLarge { size: u64 },

    /// Replacing the destination with the completed temporary file failed.
    #[error("failed to replace {path}: {source}")]
    Persist {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// An internal lazy-loading invariant was violated.
    #[error("invalid ANLZ handler state: {0}")]
    InvalidState(&'static str),
}
