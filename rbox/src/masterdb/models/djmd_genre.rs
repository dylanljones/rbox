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
use super::schema::{djmdContent, djmdGenre};
use super::{Date, DateString, RandomIdGenerator};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdGenre` table in the Rekordbox database.
///
/// This struct maps to the `djmdGenre` table in the SQLite database used by Rekordbox.
/// It stores genre-related data, allowing tracks to be categorized by genre.
///
/// # Referenced by
/// * [`DjmdContent`] via `genre_id` foreign key.
#[derive(Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = djmdGenre)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdGenre {
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

    /// An optional string representing the name of the genre.
    pub name: String,
}

impl Model for DjmdGenre {
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

impl ModelUpdate for DjmdGenre {
    fn update(mut self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        let existing = match Self::find(conn, &self.id)? {
            Some(e) => e,
            None => return Err(diesel::result::Error::NotFound),
        };
        let mut changes = 0;
        if self.name != existing.name {
            changes += 1;
        }
        if changes == 0 {
            return Ok(existing);
        }
        self.updated_at = Utc::now();
        self.rb_local_usn = Some(AgentRegistry::increment_local_usn_by(conn, changes)?);
        diesel::update(djmdGenre::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for DjmdGenre {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(djmdGenre::table.find(id)).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Remove any references to the album in the djmdContent table
        diesel::update(djmdContent::table.filter(djmdContent::genre_id.eq(id)))
            .set(djmdContent::genre_id.eq(None::<String>))
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
        let result =
            diesel::delete(djmdGenre::table.filter(djmdGenre::id.eq_any(&ids))).execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, result)?;
        // Remove any references to the genres in the djmdContent table
        diesel::update(djmdContent::table.filter(djmdContent::genre_id.eq_any(ids)))
            .set(djmdContent::genre_id.eq(None::<String>))
            .execute(conn)?;
        Ok(result)
    }
}

impl DjmdGenre {
    /// Queries a record from the `djmdGenre` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdGenre::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `djmdGenre` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdGenre::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdGenre` table.
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

/// Represents a new record insertale to the `djmdGenre` table.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdGenre;
///
/// let new = NewDjmdGenre::new("Name");
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdGenre {
    /// The name of the genre.
    pub name: String,
}

impl ModelInsert for NewDjmdGenre {
    type Model = DjmdGenre;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<DjmdGenre> {
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
            ..Default::default()
        };
        diesel::insert_into(djmdGenre::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdGenre {
    /// Creates a new genre record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }

    /// Inserts the new genre or returns an existing record if a genre with the same name already exists.
    pub fn insert_if_not_exists(self, conn: &mut SqliteConnection) -> QueryResult<DjmdGenre> {
        match DjmdGenre::find_by_name(conn, &self.name)? {
            Some(e) => Ok(e),
            None => self.insert(conn),
        }
    }
}
