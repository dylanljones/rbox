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
use super::schema::{djmdContent, djmdLabel};
use super::{Date, DateString, RandomIdGenerator};
use crate::model_traits::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

/// Represents the `djmdLabel` table in the Rekordbox database.
///
/// This struct maps to the `djmdLabel` table in the SQLite database used by Rekordbox.
/// It stores information about labels, including their names and metadata.
///
/// # Referenced by
/// * [`DjmdContent`] via `label_id` foreign key.
#[derive(Debug, Clone, PartialEq, Default, HasQuery, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = djmdLabel)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct DjmdLabel {
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

    /// The name of the label.
    pub name: String,
}

impl Model for DjmdLabel {
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

impl ModelUpdate for DjmdLabel {
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
        diesel::update(djmdLabel::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for DjmdLabel {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(djmdLabel::table.find(id)).execute(conn)?;
        AgentRegistry::increment_local_usn(conn)?;
        // Remove any references to the label in the djmdContent table
        diesel::update(djmdContent::table.filter(djmdContent::label_id.eq(id)))
            .set(djmdContent::label_id.eq(None::<String>))
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
            diesel::delete(djmdLabel::table.filter(djmdLabel::id.eq_any(&ids))).execute(conn)?;
        AgentRegistry::increment_local_usn_by(conn, ids.len())?;
        // Remove any references to the labels in the djmdContent table
        diesel::update(djmdContent::table.filter(djmdContent::label_id.eq_any(&ids)))
            .set(djmdContent::label_id.eq(None::<String>))
            .execute(conn)?;
        Ok(result)
    }
}

impl DjmdLabel {
    /// Queries a record from the `djmdLabel` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(djmdLabel::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `djmdLabel` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(djmdLabel::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }

    /// Generates a new unique identifier for a record in the `djmdLabel` table.
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

/// Represents a new record insertale to the `djmdLabel` table.
///
/// # Examples
/// ```rust
/// use rbox::masterdb::models::NewDjmdLabel;
///
/// let new = NewDjmdArtist::new("Name");
/// println!("{:?}", new);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewDjmdLabel {
    /// The name of the label.
    pub name: String,
}

impl ModelInsert for NewDjmdLabel {
    type Model = DjmdLabel;

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
            ..Default::default()
        };
        diesel::insert_into(djmdLabel::table)
            .values(item)
            .get_result(conn)
    }
}

impl NewDjmdLabel {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }

    /// Inserts the new label or returns an existing record if an label with the same name already exists.
    pub fn insert_if_not_exists(self, conn: &mut SqliteConnection) -> QueryResult<DjmdLabel> {
        match DjmdLabel::find_by_name(conn, &self.name)? {
            Some(e) => Ok(e),
            None => self.insert(conn),
        }
    }
}
