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
use super::schema::{djmdAlbum, djmdArtist};
use super::{Date, DateString, RandomIdGenerator};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdArtist` table in the Rekordbox database.
///
/// This struct maps to the `djmdArtist` table in the SQLite database used by Rekordbox.
/// It stores artist-related data, allowing multiple tracks to be associated with the same artist.
///
/// # Referenced by
/// * [`DjmdContent`] via `artist_id` foreign key.
/// * [`DjmdAlbum`] via `album_artist_id` foreign key.
#[derive(Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = djmdArtist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdArtist {
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

    /// An optional string representing the name of the artist.
    pub name: String,
    /// An optional string used for searching the artist.
    pub search_str: Option<String>,
}

impl Model for DjmdArtist {
    type Id = str;

    fn all(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        Self::query().load(conn)
    }

    fn find(conn: &mut SqliteConnection, id: &str) -> QueryResult<Option<Self>> {
        Self::query().find(id).first(conn).optional()
    }

    fn id_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
        diesel::dsl::select(diesel::dsl::exists(Self::query().find(id))).get_result(conn)
    }
}

impl ModelUpdate for DjmdArtist {
    fn update(mut self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        let existing = match Self::find(conn, &self.id)? {
            Some(e) => e,
            None => return Err(diesel::result::Error::NotFound),
        };
        let mut changes = 0;
        if self.name != existing.name {
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
        diesel::update(djmdArtist::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for DjmdArtist {
    fn delete(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
        let result = diesel::delete(djmdArtist::table.find(id)).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Remove any references to the artist in the djmdContent table
        diesel::sql_query(
            r#"UPDATE djmdContent
             SET ArtistID    = CASE WHEN ArtistID    =? THEN NULL ELSE ArtistID END,
                 RemixerID   = CASE WHEN RemixerID   =? THEN NULL ELSE RemixerID END,
                 OrgArtistID = CASE WHEN OrgArtistID =? THEN NULL ELSE OrgArtistID END,
                 ComposerID  = CASE WHEN ComposerID  =? THEN NULL ELSE ComposerID END,
                 Lyricist    = CASE WHEN Lyricist    =? THEN NULL ELSE Lyricist END
             WHERE ArtistID =? OR RemixerID =? OR OrgArtistID =? OR ComposerID =? OR Lyricist =?;"#,
        )
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Text, _>(id)
        .execute(conn)?;
        // Remove any references to the artist in the djmdAlbum table
        diesel::update(djmdAlbum::table.filter(djmdAlbum::album_artist_id.eq(id)))
            .set(djmdAlbum::album_artist_id.eq(None::<String>))
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
            diesel::delete(djmdArtist::table.filter(djmdArtist::id.eq_any(&ids))).execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, result)?;
        // Remove any references to the artist in the djmdContent table
        let ids_json = serde_json::to_string(&ids)
            .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?;
        diesel::sql_query(
            r#"WITH ids(val) AS (SELECT value FROM json_each(?))
            UPDATE djmdContent
            SET ArtistID    = CASE WHEN ArtistID    IN (SELECT val FROM ids) THEN NULL ELSE ArtistID END,
                RemixerID   = CASE WHEN RemixerID   IN (SELECT val FROM ids) THEN NULL ELSE RemixerID END,
                OrgArtistID = CASE WHEN OrgArtistID IN (SELECT val FROM ids) THEN NULL ELSE OrgArtistID END,
                ComposerID  = CASE WHEN ComposerID  IN (SELECT val FROM ids) THEN NULL ELSE ComposerID END,
                Lyricist    = CASE WHEN Lyricist    IN (SELECT val FROM ids) THEN NULL ELSE Lyricist END
            WHERE artist_id   IN (SELECT val FROM ids)
               OR RemixerID   IN (SELECT val FROM ids)
               OR OrgArtistID IN (SELECT val FROM ids)
               OR ComposerID  IN (SELECT val FROM ids)
               OR Lyricist    IN (SELECT val FROM ids);"#
        )
            .bind::<diesel::sql_types::Text, _>(ids_json)
            .execute(conn)?;
        // Remove any references to the artist in the djmdAlbum table
        diesel::update(djmdAlbum::table.filter(djmdAlbum::album_artist_id.eq_any(ids)))
            .set(djmdAlbum::album_artist_id.eq(None::<String>))
            .execute(conn)?;
        Ok(result)
    }
}

impl DjmdArtist {
    /// Queries a record from the `djmdArtist` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdArtist::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `djmdArtist` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdArtist::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdArtist` table.
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

/// Represents a new record insertale to the `djmdArtist` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdArtist;
///
/// let new = NewDjmdArtist::new("Name".into()).search_str("Search".into());
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdArtist {
    /// The name of the artist.
    pub name: String,
    /// An optional string used for searching the artist.
    pub search_str: Option<String>,
}

impl ModelInsert for NewDjmdArtist {
    type Model = DjmdArtist;

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
            search_str: self.search_str,
            ..Default::default()
        };
        diesel::insert_into(djmdArtist::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdArtist {
    /// Creates a new artist record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Inserts the new artist or returns an existing record if an artist with the same name already exists.
    pub fn insert_if_not_exists(self, conn: &mut SqliteConnection) -> QueryResult<DjmdArtist> {
        match DjmdArtist::find_by_name(conn, &self.name)? {
            Some(e) => Ok(e),
            None => self.insert(conn),
        }
    }

    /// Builder for `search_str`.
    pub fn search_str<S: Into<String>>(mut self, search_str: S) -> Self {
        self.search_str = Some(search_str.into());
        self
    }
}
