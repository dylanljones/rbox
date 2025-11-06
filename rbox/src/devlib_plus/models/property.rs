// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;
use super::util::{Date, DateString};

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset, Insertable)]
#[diesel(table_name = schema::property)]
#[diesel(primary_key(device_name))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Property {
    pub device_name: String,
    pub db_version: i32,
    pub number_of_contents: i32,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_date: Date,
    pub back_ground_color_type: Option<i32>,
    pub my_tag_master_dbid: Option<i32>,
}
