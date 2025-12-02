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
use super::schema::{djmdContent, djmdKey};
use super::{Date, DateString, RandomIdGenerator};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdKey` table in the Rekordbox database.
///
/// This struct maps to the `djmdKey` table in the SQLite database used by Rekordbox.
/// It stores information about musical keys, including their names and sequence order.
///
/// # Referenced by
/// * [`DjmdContent`] via `key_id` foreign key.
#[derive(Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = djmdKey)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdKey {
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

    /// The name of the musical key.
    pub name: String,
    /// An optional integer representing the sequence/order of the key.
    pub seq: Option<i32>,
}

impl Model for DjmdKey {
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

impl ModelUpdate for DjmdKey {
    fn update(mut self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        let existing = match Self::find(conn, &self.id)? {
            Some(e) => e,
            None => return Err(diesel::result::Error::NotFound),
        };
        let mut changes = 0;
        if self.name != existing.name {
            changes += 1;
        }
        if self.seq != existing.seq {
            changes += 1;
        }
        if changes == 0 {
            return Ok(existing);
        }
        self.updated_at = Utc::now();
        self.rb_local_usn = Some(AgentRegistry::increment_local_usn_by(conn, changes)?);
        diesel::update(djmdKey::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for DjmdKey {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(djmdKey::table.find(id)).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Reorder the seq numbers
        Self::reset_seq(conn)?;
        // Remove any references to the key in the djmdContent table
        diesel::update(djmdContent::table.filter(djmdContent::key_id.eq(id)))
            .set(djmdContent::key_id.eq(None::<String>))
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
            diesel::delete(djmdKey::table.filter(djmdKey::id.eq_any(&ids))).execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, ids.len())?;
        // Reorder the seq numbers
        Self::reset_seq(conn)?;
        // Remove any references to the key in the djmdContent table
        diesel::update(djmdContent::table.filter(djmdContent::key_id.eq_any(&ids)))
            .set(djmdContent::key_id.eq(None::<String>))
            .execute(conn)?;
        Ok(result)
    }
}

impl DjmdKey {
    /// Queries a record from the `djmdKey` table by its `scale_name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdKey::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Counts the number of records in the `djmdKey` table.
    pub fn count(conn: &mut SqliteConnection) -> QueryResult<i32> {
        Ok(djmdKey::table.count().get_result::<i64>(conn)? as i32)
    }

    /// Checks if a record with the given `scale_name` exists in the `djmdKey` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdKey::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Reset the `seq` numbers of all records from the `djmdKey` table in the current order.
    pub fn reset_seq(conn: &mut SqliteConnection) -> QueryResult<usize> {
        diesel::sql_query(
            "WITH ordered AS (SELECT id, ROW_NUMBER() OVER (ORDER BY seq) AS new_seq FROM djmdKey)
            UPDATE djmdKey SET seq = (SELECT new_seq FROM ordered WHERE ordered.id = djmdKey.id);",
        )
        .execute(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdKey` table.
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

/// Represents a new record insertale to the `djmdKey` table.
///
/// Implements the builder pattern for optional values of the new record.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdKey;
///
/// let new = NewDjmdKey::new("C").seq(1);
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdKey {
    /// The name of the musical key.
    pub name: String,
    /// An optional integer representing the sequence/order of the key.
    pub seq: Option<i32>,
}

impl ModelInsert for NewDjmdKey {
    type Model = DjmdKey;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<DjmdKey> {
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
            seq: self.seq,
            ..Default::default()
        };
        let result: Self::Model = diesel::insert_into(djmdKey::table)
            .values(item)
            .get_result(conn)?;
        if let Some(_seq) = result.seq {
            DjmdKey::reset_seq(conn)?;
        }
        Ok(result)
    }
}

impl NewDjmdKey {
    /// Creates a new record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Inserts the new key or returns an existing record if a key with the same name already exists.
    pub fn insert_if_not_exists(self, conn: &mut SqliteConnection) -> QueryResult<DjmdKey> {
        match DjmdKey::find_by_name(conn, &self.name)? {
            Some(e) => Ok(e),
            None => self.insert(conn),
        }
    }

    /// Builder for `seq`.
    pub fn seq(mut self, seq: i32) -> Self {
        self.seq = Some(seq);
        self
    }
}
