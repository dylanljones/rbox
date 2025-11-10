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
use super::schema::{album, artist, content, image};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

type JoinTuple = (Album, Option<Artist>, Option<Image>);

/// Represents the `album` table in the Rekordbox Device Library Plus database.
///
/// This struct maps to the `album` table in the Device Library Plus export database.
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
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
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

impl Album {
    /// Queries all records from the `album` table.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    /// Queries a record from the `album` table by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

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

    /// Updates an existing record in the `album` table.
    pub fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(album::table.find(self.id))
            .set(self)
            .get_result(conn)
    }

    /// Cascade deletion of a single record
    fn cascade_delete(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        // Remove any references to the album in the content table
        diesel::update(content::table.filter(content::album_id.eq(id)))
            .set(content::album_id.eq(None::<i32>))
            .execute(conn)
    }

    /// Cascade deletion of multiple records
    fn cascade_deletes(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        // Remove any references to the albums in the content table
        diesel::update(content::table.filter(content::album_id.eq_any(ids)))
            .set(content::album_id.eq(None::<i32>))
            .execute(conn)
    }

    /// Deletes a record from the `album` table and removes any references to it.
    pub fn delete(conn: &mut SqliteConnection, id: i32) -> QueryResult<usize> {
        let mut result = diesel::delete(album::table.find(id)).execute(conn)?;
        result += Self::cascade_delete(conn, id)?;
        Ok(result)
    }

    /// Deletes multiple records from the `album` table and removes any references to them.
    pub fn delete_many(conn: &mut SqliteConnection, ids: &[i32]) -> QueryResult<usize> {
        let mut result =
            diesel::delete(album::table.filter(album::id.eq_any(ids))).execute(conn)?;
        result += Self::cascade_deletes(conn, ids)?;
        Ok(result)
    }
}

/// Represents a complete album with its associated artist and image.
#[derive(Debug, Clone, PartialEq)]
pub struct AlbumJoin {
    /// The unique identifier of the album.
    pub id: i32,
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

    /// The associated [`Artist`]
    pub artist: Option<Artist>,
    /// The associated [`Image`]
    pub image: Option<Image>,
}

impl AlbumJoin {
    fn from_join(tuple: JoinTuple) -> Self {
        let mut result = Self::from(tuple.0);
        result.artist = tuple.1;
        result.image = tuple.2;
        result
    }

    /// Queries all records from the `album` table with joined associations.
    pub fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        let results: Vec<JoinTuple> = album::table
            .left_outer_join(artist::table)
            .left_outer_join(image::table)
            .load(conn)?;
        Ok(results.into_iter().map(Self::from_join).collect())
    }

    /// Queries a record from the `album` table with joined associations by its `id`.
    pub fn find(conn: &mut SqliteConnection, id: i32) -> QueryResult<Option<Self>> {
        let result: Option<JoinTuple> = album::table
            .find(id)
            .left_outer_join(artist::table)
            .left_outer_join(image::table)
            .first(conn)
            .optional()?;
        Ok(result.map(Self::from_join))
    }

    /// Queries a record from the `album` table with joined associations by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        let result: Option<JoinTuple> = album::table
            .filter(album::name.eq(name))
            .left_outer_join(artist::table)
            .left_outer_join(image::table)
            .first(conn)
            .optional()?;
        Ok(result.map(Self::from_join))
    }
}

impl Into<Album> for AlbumJoin {
    fn into(self) -> Album {
        Album {
            id: self.id,
            name: self.name,
            artist_id: self.artist_id,
            image_id: self.image_id,
            is_compilation: self.is_compilation,
            name_for_search: self.name_for_search,
        }
    }
}

impl From<Album> for AlbumJoin {
    fn from(raw: Album) -> Self {
        AlbumJoin {
            id: raw.id,
            name: raw.name,
            artist_id: raw.artist_id,
            image_id: raw.image_id,
            is_compilation: raw.is_compilation,
            name_for_search: raw.name_for_search,
            artist: None,
            image: None,
        }
    }
}

/// Represents a new record insertale to the `album` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::devlib_plus::models::NewAlbum;
///
/// let new = NewAlbum::new("Name".into()).compilation(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = album)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
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

impl NewAlbum {
    /// Creates a new album record with the required fields.
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// Inserts the new album into the database.
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Album> {
        diesel::insert_into(album::table)
            .values(self)
            .get_result(conn)
    }

    /// Inserts multiple new albums into the database.
    pub fn insert_all(conn: &mut SqliteConnection, items: &[NewAlbum]) -> QueryResult<Vec<Album>> {
        diesel::insert_into(album::table)
            .values(items)
            .get_results(conn)
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
    pub fn name_for_search(mut self, search_str: String) -> Self {
        self.name_for_search = Some(search_str);
        self
    }
}

#[cfg(feature = "master-db")]
impl Into<NewAlbum> for crate::masterdb::models::DjmdAlbum {
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
