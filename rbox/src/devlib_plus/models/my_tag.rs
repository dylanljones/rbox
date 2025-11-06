// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::myTag)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MyTag {
    pub id: i32,
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub my_tag_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::myTag)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewMyTag {
    pub sequence_no: i32,
    pub name: String,
    pub attribute: i32,
    pub my_tag_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = schema::myTag_content)]
#[diesel(primary_key(my_tag_id, content_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MyTagContent {
    pub my_tag_id: i32,
    pub content_id: i32,
}
