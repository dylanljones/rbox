// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::playlist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Playlist {
    pub id: i32,
    pub sequence_no: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Insertable)]
#[diesel(table_name = schema::playlist)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewPlaylist {
    pub sequence_no: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = schema::playlist_content)]
#[diesel(primary_key(playlist_id, content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PlaylistContent {
    pub playlist_id: i32,
    pub content_id: i32,
    pub sequence_no: i32,
}
