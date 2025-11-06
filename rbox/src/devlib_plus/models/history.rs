// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::history)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct History {
    pub id: i32,
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub history_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::history)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewHistory {
    pub id: i32,
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub history_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = schema::history_content)]
#[diesel(primary_key(history_id, content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct HistoryContent {
    pub history_id: i32,
    pub content_id: i32,
    pub sequence_no: i32,
}
