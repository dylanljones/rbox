// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::artist::Artist;
use super::image::Image;
use crate::schema::{album, artist, content, image};
use rbox_db::{load_related, Model, ModelDelete, ModelInsert, ModelRelations, ModelUpdate};
#[cfg(feature = "master-db")]
use rbox_masterdb::models::DjmdAlbum;

/// Represents the `album` table in the Rekordbox One Library database.
///
/// This struct maps to the `album` table in the One Library export database.
/// It stores album-related data, allowing multiple tracks to be associated with the same album.
/// This table includes metadata such as album name, artist, and compilation status.
///
/// # Referenced by
/// * [`Content`] via `album_id` foreign key.
///
/// # References
/// * [`Artist`] via `artist_id` foreign key.
/// * [`Image`] via `image_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset, Associations)]
#[diesel(table_name = album)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(Artist, foreign_key = artist_id))]
#[diesel(belongs_to(Image, foreign_key = image_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Album {
    /// The unique identifier of the album.
    pub id: i32,
    /// The name of the album.
    pub name: String,
    /// An optional foreign key referencing the album [`Artist`].
    pub artist_id: Option<i32>,
    /// An optional foreign key referencing the album artwork [`Image`].
    pub image_id: Option<i32>,
    /// A flag if the album is a compilation.
    pub is_compilation: i32,
    /// An optional string used for searching the album.
    pub name_for_search: Option<String>,
}

/// Album snapshot with all direct metadata relationships resolved.
///
/// Field access performs no queries, and editing this view does not persist changes.
/// Related rows are not expanded recursively. Missing targets remain `None`;
/// the original row retains its foreign keys.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct AlbumView {
    /// Original album row, including its foreign keys.
    pub album: Album,
    /// Resolved artist.
    pub artist: Option<Artist>,
    /// Resolved image.
    pub image: Option<Image>,
}

impl ModelRelations for Album {
    type Resolved = AlbumView;

    fn resolve_many(
        conn: &mut SqliteConnection,
        models: Vec<Self>,
    ) -> QueryResult<Vec<Self::Resolved>> {
        if models.is_empty() {
            return Ok(Vec::new());
        }
        conn.transaction(|conn| {
            let artists = load_related(
                conn,
                models.iter().filter_map(|row| row.artist_id),
                |conn, ids| Artist::query().filter(artist::id.eq_any(ids)).load(conn),
                |row| row.id,
            )?;
            let images = load_related(
                conn,
                models.iter().filter_map(|row| row.image_id),
                |conn, ids| Image::query().filter(image::id.eq_any(ids)).load(conn),
                |row| row.id,
            )?;
            Ok(models
                .into_iter()
                .map(|album| AlbumView {
                    artist: album
                        .artist_id
                        .as_ref()
                        .and_then(|id| artists.get(id))
                        .cloned(),
                    image: album
                        .image_id
                        .as_ref()
                        .and_then(|id| images.get(id))
                        .cloned(),
                    album,
                })
                .collect())
        })
    }
}

impl Model for Album {
    type Id = i32;

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }
}

impl ModelUpdate for Album {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(album::table.find(self.id))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Album {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let mut result = diesel::delete(album::table.find(id)).execute(conn)?;
        // Remove any references to the album in the content table
        result += diesel::update(content::table.filter(content::album_id.eq(id)))
            .set(content::album_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let mut result =
            diesel::delete(album::table.filter(album::id.eq_any(&ids))).execute(conn)?;
        // Remove any references to the albums in the content table
        result += diesel::update(content::table.filter(content::album_id.eq_any(&ids)))
            .set(content::album_id.eq(None::<i32>))
            .execute(conn)?;
        Ok(result)
    }
}

impl Album {
    /// Queries a record from the `album` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(album::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `album` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(album::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }
}

/// Represents a new record insertale to the `album` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox_onelib::models::NewAlbum;
///
/// let new = NewAlbum::new("Name").compilation(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = album)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewAlbum {
    /// The name of the album.
    pub name: String,
    /// An optional foreign key referencing the album artist [`Artist`].
    pub artist_id: Option<i32>,
    /// An optional foreign key referencing the album artwork [`Image`].
    pub image_id: Option<i32>,
    /// A flag if the album is a compilation.
    pub is_compilation: i32,
    /// An optional string used for searching the album.
    pub name_for_search: Option<String>,
}

impl ModelInsert for NewAlbum {
    type Model = Album;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        diesel::insert_into(album::table)
            .values(self)
            .get_result(conn)
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        diesel::insert_into(album::table)
            .values(items)
            .get_results(conn)
    }
}

impl NewAlbum {
    /// Creates a new album record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Builder for `artist_id`.
    pub fn artist_id(mut self, id: i32) -> Self {
        self.artist_id = Some(id);
        self
    }

    /// Builder for `image_id`.
    pub fn image_id(mut self, id: i32) -> Self {
        self.image_id = Some(id);
        self
    }

    /// Builder for `is_complation`.
    pub fn compilation(mut self, compilation: i32) -> Self {
        self.is_compilation = compilation;
        self
    }

    /// Builder for `name_for_search`.
    pub fn name_for_search<S: Into<String>>(mut self, search_str: S) -> Self {
        self.name_for_search = Some(search_str.into());
        self
    }
}

#[cfg(feature = "master-db")]
impl Into<NewAlbum> for DjmdAlbum {
    fn into(self) -> NewAlbum {
        NewAlbum {
            name: self.name,
            artist_id: None,
            image_id: None,
            is_compilation: self.compilation,
            name_for_search: self.search_str,
        }
    }
}
