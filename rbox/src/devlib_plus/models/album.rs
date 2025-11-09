// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::artist::Artist;
use super::image::Image;
use super::schema;
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

type JoinTuple = (AlbumRaw, Option<Artist>, Option<Image>);

/// Represents the raw structure of an album in the database.
///
/// This struct is used to map the `album` table in the database and provides
/// Diesel ORM annotations for querying and manipulating album records.
#[derive(
    Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset, Associations,
)]
#[diesel(table_name = schema::album)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(Artist, foreign_key = artist_id))]
#[diesel(belongs_to(Image, foreign_key = image_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct AlbumRaw {
    /// The unique identifier of the album.
    pub id: i32,
    /// The name of the album.
    pub name: String,
    /// The optional foreign key referencing the artist associated with the album.
    pub artist_id: Option<i32>,
    /// The optional foreign key referencing the image associated with the album.
    pub image_id: Option<i32>,
    /// Indicates whether the album is a compilation (1 for true, 0 for false).
    pub is_complation: i32,
    /// An optional field for storing a search-optimized version of the album name.
    pub name_for_search: Option<String>,
}

/// Represents a complete album with its associated artist and image.
///
/// This struct is used to encapsulate the full details of an album, including
/// its relationships with the `Artist` and `Image` entities.
#[derive(Debug, Clone, PartialEq)]
pub struct Album {
    /// The unique identifier of the album.
    pub id: i32,
    /// The name of the album.
    pub name: String,
    /// The optional foreign key referencing the artist associated with the album.
    pub artist_id: Option<i32>,
    /// The optional foreign key referencing the image associated with the album.
    pub image_id: Option<i32>,
    /// Indicates whether the album is a compilation (1 for true, 0 for false).
    pub is_complation: i32,
    /// An optional field for storing a search-optimized version of the album name.
    pub name_for_search: Option<String>,

    /// The associated artist entity, if available.
    pub artist: Option<Artist>,
    /// The associated image entity, if available.
    pub image: Option<Image>,
}

impl Into<AlbumRaw> for Album {
    fn into(self) -> AlbumRaw {
        AlbumRaw {
            id: self.id,
            name: self.name,
            artist_id: self.artist_id,
            image_id: self.image_id,
            is_complation: self.is_complation,
            name_for_search: self.name_for_search,
        }
    }
}

impl From<AlbumRaw> for Album {
    fn from(raw: AlbumRaw) -> Self {
        Album {
            id: raw.id,
            name: raw.name,
            artist_id: raw.artist_id,
            image_id: raw.image_id,
            is_complation: raw.is_complation,
            name_for_search: raw.name_for_search,
            artist: None,
            image: None,
        }
    }
}

impl Album {
    pub fn from_join(tuple: JoinTuple) -> Self {
        let mut result = Self::from(tuple.0);
        result.artist = tuple.1;
        result.image = tuple.2;
        result
    }
}

/// Represents a new album to be inserted into the database.
///
/// This struct is used for creating new album records in the `album` table
/// and includes Diesel ORM annotations for database interactions.
#[derive(PartialEq, Debug, Clone, Default, Insertable)]
#[diesel(table_name = schema::album)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewAlbum {
    /// The name of the album.
    pub name: String,
    /// The optional foreign key referencing the artist associated with the album.
    pub artist_id: Option<i32>,
    /// The optional foreign key referencing the image associated with the album.
    pub image_id: Option<i32>,
    /// Indicates whether the album is a compilation (1 for true, 0 for false).
    pub is_complation: i32,
    /// An optional field for storing a search-optimized version of the album name.
    pub name_for_search: Option<String>,
}

impl NewAlbum {
    pub fn new(
        name: String,
        artist_id: Option<i32>,
        image_id: Option<i32>,
        compilation: Option<i32>,
    ) -> Self {
        let is_complation = compilation.unwrap_or(0);
        NewAlbum {
            name,
            artist_id,
            image_id,
            is_complation,
            name_for_search: None,
        }
    }
}

#[cfg(feature = "master-db")]
impl Into<NewAlbum> for crate::masterdb::models::DjmdAlbum {
    fn into(self) -> NewAlbum {
        NewAlbum {
            name: self.name,
            artist_id: None,
            image_id: None,
            is_complation: self.compilation,
            name_for_search: self.search_str,
        }
    }
}
