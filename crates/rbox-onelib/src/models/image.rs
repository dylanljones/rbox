// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! SQLite row models and operations for `Image`.

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use rbox_derives::PyMutableMapping;

use crate::schema::{album, content, image};
use rbox_db::{Model, ModelDelete, ModelInsert, ModelUpdate};

/// Represents the `image` table in the Rekordbox One Library database.
///
/// This struct maps to the `image` table in the One Library export database.
/// It stores the relative file paths of all images on the device.
///
/// # Referenced by
/// * [`Album`] via `image_id` foreign key.
/// * [`Content`] via `image_id` foreign key.
#[derive(Debug, Clone, PartialEq, HasQuery, Identifiable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = image)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Image {
    /// The unique identifier of the image.
    pub id: i64,
    /// The relative file path of the image.
    pub path: String,
}

impl Model for Image {
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

impl ModelUpdate for Image {
    fn update(self, conn: &mut SqliteConnection) -> QueryResult<Self> {
        diesel::update(image::table.find(self.id.clone()))
            .set(self)
            .get_result(conn)
    }
}

impl ModelDelete for Image {
    fn delete(conn: &mut SqliteConnection, id: &Self::Id) -> QueryResult<usize> {
        let result = diesel::delete(image::table.find(id)).execute(conn)?;
        // Remove any references to the image in the album table
        diesel::update(album::table.filter(album::image_id.eq(id)))
            .set(album::image_id.eq(None::<i64>))
            .execute(conn)?;
        // Remove any references to the image in the content table
        diesel::update(content::table.filter(content::image_id.eq(id)))
            .set(content::image_id.eq(None::<i64>))
            .execute(conn)?;
        Ok(result)
    }

    fn delete_all(conn: &mut SqliteConnection, ids: Vec<&Self::Id>) -> QueryResult<usize> {
        let result = diesel::delete(image::table.filter(image::id.eq_any(&ids))).execute(conn)?;
        // Remove any references to the image in the album table
        diesel::update(album::table.filter(album::image_id.eq_any(&ids)))
            .set(album::image_id.eq(None::<i64>))
            .execute(conn)?;
        // Remove any references to the image in the content table
        diesel::update(content::table.filter(content::image_id.eq_any(&ids)))
            .set(content::image_id.eq(None::<i64>))
            .execute(conn)?;
        Ok(result)
    }
}

impl Image {
    /// Queries a record from the `image` table by its `path`.
    pub fn find_by_path(conn: &mut SqliteConnection, path: &str) -> QueryResult<Option<Self>> {
        Self::query()
            .filter(image::path.eq(path))
            .first(conn)
            .optional()
    }

    /// Checks if a record with the given `path` exists in the `image` table.
    pub fn path_exists(conn: &mut SqliteConnection, path: &str) -> QueryResult<bool> {
        let query = Self::query().filter(image::path.eq(path));
        diesel::dsl::select(diesel::dsl::exists(query)).get_result(conn)
    }
}

/// Represents a new record insertale to the `image` table.
#[derive(Debug, Clone, PartialEq, Insertable)]
#[diesel(table_name = image)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object, get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewImage {
    /// The relative file path of the image.
    pub path: String,
}

impl ModelInsert for NewImage {
    type Model = Image;

    fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Self::Model> {
        diesel::insert_into(image::table)
            .values(self)
            .get_result(conn)
    }

    fn insert_all(conn: &mut SqliteConnection, items: Vec<Self>) -> QueryResult<Vec<Self::Model>> {
        diesel::insert_into(image::table)
            .values(items)
            .get_results(conn)
    }
}

impl NewImage {
    /// Creates a new image record with the required fields.
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self { path: path.into() }
    }
}
