// Author: Dylan Jones
// Date:   2025-09-02

use chrono::Utc;
use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;
use uuid::Uuid;

use super::agent_registry::AgentRegistry;
use super::djmd_artist::DjmdArtist;
use super::schema::{djmdAlbum, djmdContent};
use super::{Date, DateString, RandomIdGenerator};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdAlbum` table in the Rekordbox database.
///
/// This struct maps to the `djmdAlbum` table in the SQLite database used by Rekordbox.
/// It stores album-related data, allowing multiple tracks to be associated with the same album.
/// This table includes metadata such as album name, artist, and compilation status.
///
/// # Referenced by
/// * [`DjmdContent`] via `album_id` foreign key.
///
/// # References
/// * [`DjmdArtist`] via `album_artist_id` foreign key.
#[derive(
    Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset, Associations,
)]
#[diesel(table_name = djmdAlbum)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdArtist, foreign_key = album_artist_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdAlbum {
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

    /// The name of the album.
    pub name: String,
    /// An optional string representing the id of the album artist [`DjmdArtist`].
    pub album_artist_id: Option<String>,
    /// An optional string representing the path to the album's image.
    pub image_path: Option<String>,
    /// A flag if the album is a compilation.
    pub compilation: i32,
    /// An optional string used for searching the album.
    pub search_str: Option<String>,
}

impl Model for DjmdAlbum {
    type Id = str;

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

impl ModelUpdate for DjmdAlbum {
    fn update(mut self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        let existing = match Self::find(conn, &self.id)? {
            Some(e) => e,
            None => return Err(diesel::result::Error::NotFound),
        };
        let mut changes = 0;
        if self.name != existing.name {
            changes += 1;
        }
        if self.album_artist_id != existing.album_artist_id {
            changes += 1;
        }
        if self.image_path != existing.image_path {
            changes += 1;
        }
        if self.compilation != existing.compilation {
            changes += 1;
        }
        if self.search_str != existing.search_str {
            changes += 1;
        }
        if changes == 0 {
            return Ok(existing);
        }
        self.updated_at = Utc::now();
        self.rb_local_usn = Some(AgentRegistry::increment_local_usn_by(conn, changes)?);
        Ok(diesel::update(djmdAlbum::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)?)
    }
}

impl ModelDelete for DjmdAlbum {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(djmdAlbum::table.find(id)).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Remove any references to the album in the djmdContent table
        diesel::update(djmdContent::table.filter(djmdContent::album_id.eq(id)))
            .set(djmdContent::album_id.eq(None::<String>))
            .execute(conn)?;

        Ok(result)
    }

    /// Deletes multiple records from the database table.
    ///
    /// # Arguments
    /// - `conn`: A mutable reference to the database connection.
    /// - `ids`: A vector of references to the IDs of the records to delete.
    ///
    /// # Returns
    /// A `QueryResult` containing the total number of rows affected.
    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        let result =
            diesel::delete(djmdAlbum::table.filter(djmdAlbum::id.eq_any(&ids))).execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, ids.len())?;
        // Remove any references to the albums in the djmdContent table
        diesel::update(djmdContent::table.filter(djmdContent::album_id.eq_any(ids)))
            .set(djmdContent::album_id.eq(None::<String>))
            .execute(conn)?;
        Ok(result)
    }
}

impl DjmdAlbum {
    /// Queries a record from the `djmdAlbum` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdAlbum::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `djmdAlbum` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdAlbum::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdAlbum` table.
    fn generate_id(conn: &mut SqliteConnection) -> QueryResult<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                if !Self::id_exists(conn, &tmp_id)? {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }
}

/// Represents a new record insertale to the `djmdAlbum` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdAlbum;
///
/// let new = NewDjmdAlbum::new("Name").compilation(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdAlbum {
    /// The name of the album.
    pub name: String,
    /// An optional string representing the id of the album artist [`DjmdArtist`].
    pub album_artist_id: Option<String>,
    /// An optional string representing the path to the album's image.
    pub image_path: Option<String>,
    /// An optional integer indicating whether the album is a compilation.
    pub compilation: Option<i32>,
    /// An optional string used for searching the album.
    pub search_str: Option<String>,
}

impl ModelInsert for NewDjmdAlbum {
    type Model = DjmdAlbum;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        let id = Self::Model::generate_id(conn)?;
        let uuid = Uuid::new_v4().to_string();
        let usn = AgentRegistry::increment_local_usn(conn)?;
        let now = Utc::now();
        let item = Self::Model {
            id,
            uuid,
            rb_local_usn: Some(usn),
            created_at: now,
            updated_at: now,
            name: self.name,
            album_artist_id: self.album_artist_id,
            image_path: self.image_path,
            compilation: self.compilation.unwrap_or(0),
            search_str: self.search_str,
            ..Default::default()
        };
        diesel::insert_into(djmdAlbum::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdAlbum {
    /// Creates a new album record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Inserts the new album or returns an existing record if an album with the same name already exists.
    pub fn insert_if_not_exists(self, conn: &mut SqliteConnection) -> QueryResult<DjmdAlbum> {
        match DjmdAlbum::find_by_name(conn, &self.name)? {
            Some(e) => Ok(e),
            None => self.insert(conn),
        }
    }

    /// Builder for `album_artist_id`.
    pub fn album_artist_id<S: Into<String>>(mut self, id: String) -> Self {
        self.album_artist_id = Some(id.into());
        self
    }

    /// Builder for `image_path`.
    pub fn image_path<S: Into<String>>(mut self, path: String) -> Self {
        self.image_path = Some(path.into());
        self
    }

    /// Builder for `compilation`.
    pub fn compilation(mut self, compilation: i32) -> Self {
        self.compilation = Some(compilation);
        self
    }

    /// Builder for `search_str`.
    pub fn search_str<S: Into<String>>(mut self, search_str: String) -> Self {
        self.search_str = Some(search_str.into());
        self
    }
}
