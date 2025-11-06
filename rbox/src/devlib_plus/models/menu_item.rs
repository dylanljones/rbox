// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::menuItem)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MenuItem {
    pub id: i32,
    pub kind: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Insertable)]
#[diesel(table_name = schema::menuItem)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewMenuItem {
    pub kind: i32,
    pub name: String,
}
