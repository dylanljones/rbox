// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::hotCueBankList)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct HotCueBankList {
    pub id: i32,
    pub sequence_no: i32,
    pub name: Option<String>,
    pub image_id: Option<i32>,
    pub attribute: i32,
    pub hot_cue_bank_list_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::hotCueBankList)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewHotCueBankList {
    pub sequence_no: i32,
    pub name: Option<String>,
    pub image_id: Option<i32>,
    pub attribute: i32,
    pub hot_cue_bank_list_id_parent: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = schema::hotCueBankList_cue)]
#[diesel(primary_key(hot_cue_bank_list_id, cue_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct HotCueBankListCue {
    pub hot_cue_bank_list_id: i32,
    pub cue_id: i32,
    pub sequence_no: i32,
}
