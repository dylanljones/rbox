// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::color)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Color {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::color)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewColor {
    pub name: String,
}

impl NewColor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
