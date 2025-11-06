// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;

use super::schema;

/// Represents an artist in the database.
///
/// This struct is used to map the `artist` table in the database and provides
/// Diesel ORM annotations for querying and manipulating artist records.
#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = schema::artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Artist {
    /// The unique identifier of the artist.
    pub id: i32,
    /// The name of the artist.
    pub name: String,
    /// An optional field for storing a search-optimized version of the artist name.
    pub name_for_search: Option<String>,
}

/// Represents a new artist to be inserted into the database.
///
/// This struct is used for creating new artist records in the `artist` table
/// and includes Diesel ORM annotations for database interactions.
#[derive(PartialEq, Debug, Clone, Default, Insertable)]
#[diesel(table_name = schema::artist)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewArtist {
    /// The name of the artist.
    pub name: String,
    /// An optional field for storing a search-optimized version of the artist name.
    pub name_for_search: Option<String>,
}

impl NewArtist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            name_for_search: None,
        }
    }
}

// Clones of Artist for associating with Content

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Remixer {
    pub id: i32,
    pub name: String,
    pub name_for_search: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct OriginalArtist {
    pub id: i32,
    pub name: String,
    pub name_for_search: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Composer {
    pub id: i32,
    pub name: String,
    pub name_for_search: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::artist)]
#[diesel(primary_key(id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Lyricist {
    pub id: i32,
    pub name: String,
    pub name_for_search: Option<String>,
}
