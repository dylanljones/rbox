// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use super::schema::artist;
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `artist` table in the Rekordbox One Library database.
///
/// This struct maps to the `artist` table in the One Library export database.
/// It stores artist-related data, allowing multiple tracks to be associated with the same artist.
///
/// # Referenced by
/// * [`Album`] via `artist_id` foreign key.
/// * [`Content`] via `artist_id`, `remixer_id`, `original_artist_id`, `composer_id` and `lyricist_id` foreign keys.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(table_name = artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Artist {
    /// The unique identifier of the artist.
    pub id: i32,
    /// The name of the artist.
    pub name: String,
    /// An optional field for storing a search-optimized version of the artist name.
    pub name_for_search: Option<String>,
}

impl Model for Artist {
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

impl ModelUpdate for Artist {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(artist::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Artist {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let mut result = diesel::delete(artist::table.find(id)).execute(conn)?;
        // Remove any references to the artist in the content table
        result += diesel::sql_query(
            "UPDATE content
             SET artist_id_artist         = CASE WHEN artist_id_artist          =? THEN NULL ELSE artist_id_artist END,
                 artist_id_remixer        = CASE WHEN artist_id_remixer         =? THEN NULL ELSE artist_id_remixer END,
                 artist_id_originalArtist = CASE WHEN artist_id_originalArtist  =? THEN NULL ELSE artist_id_originalArtist END,
                 artist_id_composer       = CASE WHEN artist_id_composer        =? THEN NULL ELSE artist_id_composer END,
                 artist_id_lyricist       = CASE WHEN artist_id_lyricist        =? THEN NULL ELSE artist_id_lyricist END
             WHERE artist_id_artist         =?
                OR artist_id_remixer        =?
                OR artist_id_originalArtist =?
                OR artist_id_composer       =?
                OR artist_id_lyricist       =?"
        )
            .bind::<diesel::sql_types::Integer, _>(id)
            .bind::<diesel::sql_types::Integer, _>(id)
            .bind::<diesel::sql_types::Integer, _>(id)
            .bind::<diesel::sql_types::Integer, _>(id)
            .bind::<diesel::sql_types::Integer, _>(id)
            .bind::<diesel::sql_types::Integer, _>(id)
            .bind::<diesel::sql_types::Integer, _>(id)
            .bind::<diesel::sql_types::Integer, _>(id)
            .bind::<diesel::sql_types::Integer, _>(id)
            .bind::<diesel::sql_types::Integer, _>(id)
            .execute(conn)?;
        Ok(result)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let mut result =
            diesel::delete(artist::table.filter(artist::id.eq_any(&ids))).execute(conn)?;
        // Remove any references to the artists in the content table
        let ids_json = serde_json::to_string(&ids)
            .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?;
        result += diesel::sql_query(
            "WITH ids(val) AS (SELECT value FROM json_each(?))
            UPDATE content
            SET artist_id_artist          = CASE WHEN artist_id_artist          IN (SELECT val FROM ids) THEN NULL ELSE artist_id_artist END,
                artist_id_remixer         = CASE WHEN artist_id_remixer         IN (SELECT val FROM ids) THEN NULL ELSE artist_id_remixer END,
                artist_id_originalArtist  = CASE WHEN artist_id_originalArtist  IN (SELECT val FROM ids) THEN NULL ELSE artist_id_originalArtist END,
                artist_id_composer        = CASE WHEN artist_id_composer        IN (SELECT val FROM ids) THEN NULL ELSE artist_id_composer END,
                artist_id_lyricist        = CASE WHEN artist_id_lyricist        IN (SELECT val FROM ids) THEN NULL ELSE artist_id_lyricist END
            WHERE artist_id_artist         IN (SELECT val FROM ids)
               OR artist_id_remixer        IN (SELECT val FROM ids)
               OR artist_id_originalArtist IN (SELECT val FROM ids)
               OR artist_id_composer       IN (SELECT val FROM ids)
               OR artist_id_lyricist       IN (SELECT val FROM ids)"
        )
            .bind::<diesel::sql_types::Text, _>(ids_json)
            .execute(conn)?;
        Ok(result)
    }
}

impl Artist {
    /// Queries a record from the `artist` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(artist::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `artist` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(artist::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }
}

/// Represents a new record insertale to the `artist` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::one_library::models::NewArtist;
///
/// let new = NewArtist::new("Name").name_for_search("Name");
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = artist)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewArtist {
    /// The name of the artist.
    pub name: String,
    /// An optional field for storing a search-optimized version of the artist name.
    pub name_for_search: Option<String>,
}

impl ModelInsert for NewArtist {
    type Model = Artist;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        diesel::insert_into(artist::table)
            .values(self)
            .get_result(conn)
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        diesel::insert_into(artist::table)
            .values(items)
            .get_results(conn)
    }
}

impl NewArtist {
    /// Creates a new artist record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Builder for `name_for_search`.
    pub fn name_for_search<S: Into<String>>(mut self, search_str: S) -> Self {
        self.name_for_search = Some(search_str.into());
        self
    }
}

#[cfg(feature = "master-db")]
impl Into<NewArtist> for crate::masterdb::models::DjmdArtist {
    fn into(self) -> NewArtist {
        NewArtist {
            name: self.name,
            name_for_search: self.search_str,
        }
    }
}

// Clones of Artist for associating with Content

#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable)]
#[diesel(table_name = artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Remixer {
    pub id: i32,
    pub name: String,
    pub name_for_search: Option<String>,
}

#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable)]
#[diesel(table_name = artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct OriginalArtist {
    pub id: i32,
    pub name: String,
    pub name_for_search: Option<String>,
}

#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable)]
#[diesel(table_name = artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Composer {
    pub id: i32,
    pub name: String,
    pub name_for_search: Option<String>,
}

#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable)]
#[diesel(table_name = artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Lyricist {
    pub id: i32,
    pub name: String,
    pub name_for_search: Option<String>,
}
