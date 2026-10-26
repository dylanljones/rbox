// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: MIT

//! Device Library enum re-exports shared with other database formats.
//!
//! Playlist codes retain the values accepted by the existing reader. Creation
//! remains restricted to lists and folders by the playlist model.

pub use rbox_core::{FileType, MyTagType, PlaylistType};
