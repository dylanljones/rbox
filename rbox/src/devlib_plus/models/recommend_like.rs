// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;
use super::util::{Date, DateString};

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = schema::recommendedLike)]
#[diesel(primary_key(content_id_1, content_id_2))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RecommendedLike {
    pub content_id_1: i32,
    pub content_id_2: i32,
    pub rating: i32,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_date: Date,
}
