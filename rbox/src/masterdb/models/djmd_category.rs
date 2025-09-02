// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::djmd_menu_items::DjmdMenuItems;
use super::schema;
use super::util::{Date, DateString};

/// Represents the `DjmdCategory` table in the Rekordbox database.
///
/// This struct maps to the `DjmdCategory` table in the SQLite database used by Rekordbox.
/// It stores category-related data, which can be associated with menu items.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Associations,
)]
#[diesel(table_name = schema::djmdCategory)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DjmdMenuItems, foreign_key = menu_item_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DjmdCategory {
    /// A unique identifier for the entry.
    pub id: String,
    /// A unique universal identifier for the entry.
    pub uuid: String,
    /// An integer representing the data status in Rekordbox.
    pub rb_data_status: i32,
    /// An integer representing the local data status in Rekordbox.
    pub rb_local_data_status: i32,
    /// An integer indicating whether the entry is locally deleted.
    pub rb_local_deleted: i32,
    /// An integer indicating whether the entry is locally synced.
    pub rb_local_synced: i32,
    /// An optional integer representing the update sequence number.
    pub usn: Option<i32>,
    /// An optional integer representing the local update sequence number.
    pub rb_local_usn: Option<i32>,
    /// The timestamp when the entry was created, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub created_at: Date,
    /// The timestamp when the entry was last updated, serialized/deserialized as `DateString`.
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub updated_at: Date,

    /// An optional string representing the id of the associated [`DjmdMenuItems`].
    pub menu_item_id: Option<String>,
    /// An optional integer representing the sequence/order of the category.
    pub seq: Option<i32>,
    /// An optional integer indicating whether the category is disabled.
    pub disable: Option<i32>,
    /// An optional integer representing the info order for the category.
    pub info_order: Option<i32>,
}
