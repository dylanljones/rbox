// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::djmd_content::DjmdContent;
use super::schema;
use super::util::{Date, DateString};

/// Represents the `DjmdPlaylist` table in the Rekordbox database.
///
/// This struct maps to the `DjmdPlaylist` table in the SQLite database used by Rekordbox.
/// It stores information about playlists, including metadata such as sequence, name, image path,
/// attributes, and parent relationships.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdPlaylist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdPlaylist, foreign_key = parent_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdPlaylist {
    /// A unique identifier for the entry.
    pub id: String,
    /// A unique universal identifier for the entry.
    pub uuid: String,
    /// An integer representing the data status in Rekordbox.
    pub rb_data_status: i32,
    /// An integer representing the local data status in Rekordbox.
    pub rb_local_data_status: i32,
    /// An integer indicating whether the entry is locally deleted.
    pub rb_local_deleted: i32,
    /// An integer indicating whether the entry is locally synced.
    pub rb_local_synced: i32,
    /// An optional integer representing the update sequence number.
    pub usn: Option<i32>,
    /// An optional integer representing the local update sequence number.
    pub rb_local_usn: Option<i32>,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    /// An optional integer representing the sequence/order of the playlist.
    pub seq: Option<i32>,
    /// An optional string representing the name of the playlist.
    pub name: Option<String>,
    /// An optional string representing the path to the image associated with the playlist.
    pub image_path: Option<String>,
    /// An optional integer representing additional attributes of the playlist.
    pub attribute: Option<i32>,
    /// An optional string representing the ID of the parent [`DjmdPlaylist`] entry.
    pub parent_id: Option<String>,
    /// An optional string containing the smart list configuration for the playlist.
    pub smart_list: Option<String>,
}

impl DjmdPlaylist {
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        name: String,
        attribute: i32,
        parent_id: String,
        seq: i32,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            seq: Some(seq),
            name: Some(name.clone()),
            image_path: image_path.clone(),
            attribute: Some(attribute),
            parent_id: Some(parent_id.clone()),
            smart_list: smart_list.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DjmdPlaylistTreeItem {
    pub id: String,
    pub uuid: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: Date,
    pub updated_at: Date,

    pub seq: Option<i32>,
    pub name: Option<String>,
    pub image_path: Option<String>,
    pub attribute: Option<i32>,
    pub parent_id: Option<String>,
    pub smart_list: Option<String>,
    pub children: Vec<Rc<RefCell<Self>>>,
}

impl DjmdPlaylistTreeItem {
    pub fn from_playlist(playlist: DjmdPlaylist) -> Self {
        Self {
            id: playlist.id,
            uuid: playlist.uuid,
            rb_data_status: playlist.rb_data_status,
            rb_local_data_status: playlist.rb_local_data_status,
            rb_local_deleted: playlist.rb_local_deleted,
            rb_local_synced: playlist.rb_local_synced,
            usn: playlist.usn,
            rb_local_usn: playlist.rb_local_usn,
            created_at: playlist.created_at,
            updated_at: playlist.updated_at,

            seq: playlist.seq,
            name: playlist.name,
            image_path: playlist.image_path,
            attribute: playlist.attribute,
            parent_id: playlist.parent_id,
            smart_list: playlist.smart_list,

            children: Vec::new(),
        }
    }
}

/// Represents the `DjmdSongPlaylist` table in the Rekordbox database.
///
/// This struct maps to the `DjmdSongPlaylist` table in the SQLite database used by Rekordbox.
/// It stores information about the relationship between songs and playlists, including
/// metadata such as track number and associated playlist or content IDs.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdSongPlaylist)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdPlaylist, foreign_key = playlist_id))]
#[diesel(belongs_to(DjmdContent, foreign_key = content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdSongPlaylist {
    /// A unique identifier for the entry.
    pub id: String,
    /// A unique universal identifier for the entry.
    pub uuid: String,
    /// An integer representing the data status in Rekordbox.
    pub rb_data_status: i32,
    /// An integer representing the local data status in Rekordbox.
    pub rb_local_data_status: i32,
    /// An integer indicating whether the entry is locally deleted.
    pub rb_local_deleted: i32,
    /// An integer indicating whether the entry is locally synced.
    pub rb_local_synced: i32,
    /// An optional integer representing the update sequence number.
    pub usn: Option<i32>,
    /// An optional integer representing the local update sequence number.
    pub rb_local_usn: Option<i32>,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    /// An optional string representing the ID of the associated playlist in [`DjmdPlaylist`].
    pub playlist_id: Option<String>,
    /// An optional string representing the ID of the associated content in [`DjmdContent`].
    pub content_id: Option<String>,
    /// An optional integer representing the track number in the playlist.
    pub track_no: Option<i32>,
}

impl DjmdSongPlaylist {
    pub fn new(
        id: String,
        uuid: String,
        usn: i32,
        now: Date,
        playlist_id: String,
        content_id: String,
        seq: i32,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: id.clone(),
            uuid: uuid.clone(),
            rb_data_status: 0,
            rb_local_data_status: 0,
            rb_local_deleted: 0,
            rb_local_synced: 0,
            usn: None,
            rb_local_usn: Some(usn),
            created_at: now.clone(),
            updated_at: now.clone(),
            playlist_id: Some(playlist_id.clone()),
            content_id: Some(content_id.clone()),
            track_no: Some(seq),
        })
    }
}
