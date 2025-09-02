// Author: Dylan Jones
// Date:   2025-05-01

//! # Rekordbox master.db database models
//!
//! This module defines the data models that map to the tables in the Rekordbox SQLite database.
//! It uses Diesel ORM for database interactions and provides structures for various tables,
//! including content, playlists, artists, albums, and more. These models facilitate reading,
//! writing, and updating data in the Rekordbox database.
//!
//! ## Features
//! - Comprehensive mapping of Rekordbox database tables to Rust structs.
//! - Support for Diesel ORM operations like querying, inserting, and updating records.
//! - Associations between related tables for seamless navigation of relationships.
//!
//! ## Usage
//! Import the necessary models and use them with Diesel's query builder to interact with the
//! database.

mod agent_registry;
mod content_active_censor;
mod content_cue;
mod content_file;
mod djmd_active_sensor;
mod djmd_album;
mod djmd_artist;
mod djmd_category;
mod djmd_color;
mod djmd_content;
mod djmd_cue;
mod djmd_device;
mod djmd_genre;
mod djmd_history;
mod djmd_hot_cue_bank_list;
mod djmd_key;
mod djmd_label;
mod djmd_menu_items;
mod djmd_mixer_param;
mod djmd_my_tag;
mod djmd_playlist;
mod djmd_property;
mod djmd_recommend_like;
mod djmd_related_tracks;
mod djmd_sampler;
mod djmd_song_tag_list;
mod djmd_sort;
mod image_file;
mod setting_file;
mod uuid_map;

use super::schema;
use super::util;

pub use agent_registry::{AgentRegistry, CloudAgentRegistry};
pub use content_active_censor::ContentActiveCensor;
pub use content_cue::ContentCue;
pub use content_file::ContentFile;
pub use djmd_active_sensor::DjmdActiveCensor;
pub use djmd_album::DjmdAlbum;
pub use djmd_artist::DjmdArtist;
pub use djmd_category::DjmdCategory;
pub use djmd_color::DjmdColor;
pub use djmd_content::DjmdContent;
pub use djmd_cue::DjmdCue;
pub use djmd_device::DjmdDevice;
pub use djmd_genre::DjmdGenre;
pub use djmd_history::{DjmdHistory, DjmdSongHistory};
pub use djmd_hot_cue_bank_list::{DjmdHotCueBanklist, DjmdSongHotCueBanklist, HotCueBanklistCue};
pub use djmd_key::DjmdKey;
pub use djmd_label::DjmdLabel;
pub use djmd_menu_items::DjmdMenuItems;
pub use djmd_mixer_param::DjmdMixerParam;
pub use djmd_my_tag::{DjmdMyTag, DjmdSongMyTag};
pub use djmd_playlist::{DjmdPlaylist, DjmdPlaylistTreeItem, DjmdSongPlaylist};
pub use djmd_property::{DjmdCloudProperty, DjmdProperty};
pub use djmd_recommend_like::DjmdRecommendLike;
pub use djmd_related_tracks::{DjmdRelatedTracks, DjmdSongRelatedTracks};
pub use djmd_sampler::{DjmdSampler, DjmdSongSampler};
pub use djmd_song_tag_list::DjmdSongTagList;
pub use djmd_sort::DjmdSort;
pub use image_file::ImageFile;
pub use setting_file::SettingFile;
pub use util::{format_datetime, parse_datetime, sort_tree_list, RekordboxDateString};
pub use uuid_map::UuidIDMap;
