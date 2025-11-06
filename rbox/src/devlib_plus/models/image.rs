// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::image)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Image {
    pub id: i32,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Insertable)]
#[diesel(table_name = schema::image)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewImage {
    pub path: String,
}
