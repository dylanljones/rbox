// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::sort)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Sort {
    pub id: i32,
    pub menu_item_id: i32,
    pub sequence_no: i32,
    pub is_visible: i32,
    pub is_selected_as_sub_column: i32,
}

#[derive(Debug, Clone, PartialEq, Insertable)]
#[diesel(table_name = schema::sort)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewSort {
    pub id: i32,
    pub menu_item_id: i32,
    pub sequence_no: i32,
    pub is_visible: i32,
    pub is_selected_as_sub_column: i32,
}
