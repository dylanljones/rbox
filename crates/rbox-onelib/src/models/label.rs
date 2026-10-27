// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! SQLite row models and operations for `Label`.

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::{PyConstructor, PyMutableMapping};

use crate::schema::{content, label};
use rbox_db::{Model, ModelDelete, ModelInsert, ModelUpdate};
#[cfg(feature = "master-db")]
use rbox_masterdb::models::DjmdLabel;

/// Represents the `label` table in the Rekordbox One Library database.
///
/// This struct maps to the `label` table in the One Library export database.
/// It stores label-related data, allowing multiple tracks to be associated with the same label.
///
/// # Referenced by
/// * [`Content`] via `label_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = label)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Label {
    /// The unique identifier of the label.
    pub id: i64,
    /// The name of the label.
    pub name: String,
}

impl Model for Label {
    type Id = i64;

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

impl ModelUpdate for Label {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(label::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Label {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(label::table.find(id)).execute(conn)?;
        // Remove any references to the label in the content table
        diesel::update(content::table.filter(content::label_id.eq(id)))
            .set(content::label_id.eq(None::<i64>))
            .execute(conn)?;
        Ok(result)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let result = diesel::delete(label::table.filter(label::id.eq_any(&ids))).execute(conn)?;
        // Remove any references to the label in the content table
        diesel::update(content::table.filter(content::label_id.eq_any(&ids)))
            .set(content::label_id.eq(None::<i64>))
            .execute(conn)?;
        Ok(result)
    }
}

impl Label {
    /// Queries a record from the `label` table by its `name`.
    pub fn find_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(label::name.eq(name))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `name` exists in the `label` table.
    pub fn name_exists(conn: &mut SqliteConnection, name: &str) -> QueryResult<bool> {
        let query = Self::query().filter(label::name.eq(name));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }
}

/// Represents a new record insertale to the `label` table.
#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = label)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping, PyConstructor))]
#[cfg_attr(feature = "napi", napi(object))]
#[cfg_attr(feature = "pyo3", py_constructor(required(name)))]
pub struct NewLabel {
    /// The name of the label.
    pub name: String,
}

impl ModelInsert for NewLabel {
    type Model = Label;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        diesel::insert_into(label::table)
            .values(self)
            .get_result(conn)
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        diesel::insert_into(label::table)
            .values(items)
            .get_results(conn)
    }
}

impl NewLabel {
    /// Creates a new label record with the required fields.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[cfg(feature = "master-db")]
impl Into<NewLabel> for DjmdLabel {
    fn into(self) -> NewLabel {
        NewLabel { name: self.name }
    }
}
