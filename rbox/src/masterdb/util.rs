// Author: Dylan Jones
// Date:   2025-05-15

use chrono::{DateTime, TimeZone, Utc};
use diesel::backend::Backend;
use diesel::deserialize::{FromSql, Result as DResult};
use diesel::serialize::{IsNull, Output, Result as SResult, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use std::cell::RefCell;
use std::rc::Rc;

use super::models::DjmdPlaylistTreeItem;

pub type Date = DateTime<Utc>;

const DATEFMT: &str = "%Y-%m-%d %H:%M:%S%.3f %:z";

/// Parse a Rekordbox date string into a chrono DateTime<Utc>
pub fn parse_datetime(s: &str) -> anyhow::Result<Date> {
    // If string contains additional info in brackets (...), remove it
    let idx = s.find('(');
    let s = if let Some(idx) = idx { &s[..idx] } else { s };
    DateTime::parse_from_str(s, DATEFMT)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| anyhow::anyhow!("Failed to parse datetime: {}", e))
}

/// Format a chrono DateTime<Tz> into a Rekordbox date string
pub fn format_datetime<Tz: TimeZone>(dt: &DateTime<Tz>) -> String {
    dt.with_timezone(&Utc).format(DATEFMT).to_string()
}

/// Wrapper for DateTime<Utc> for diesel serialization/deserialization
#[derive(Debug, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub struct DateString(pub Date);

impl From<DateString> for Date {
    fn from(s: DateString) -> Self {
        s.0
    }
}

impl From<Date> for DateString {
    fn from(dt: Date) -> Self {
        DateString(dt)
    }
}

impl<B> FromSql<Text, B> for DateString
where
    B: Backend,
    String: FromSql<Text, B>,
{
    fn from_sql(bytes: B::RawValue<'_>) -> DResult<Self> {
        let s = <String as FromSql<Text, B>>::from_sql(bytes)?;
        Ok(parse_datetime(&s).map(DateString).map_err(|_| {
            let msg = format!("Invalid datetime string: {}", s);
            diesel::result::Error::DeserializationError(msg.into())
        })?)
    }
}

impl ToSql<Text, diesel::sqlite::Sqlite> for DateString
where
    str: ToSql<Text, diesel::sqlite::Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> SResult {
        let s = format_datetime(&self.0); // Format the DateTime<Utc> as a string
        out.set_value(s); // Set the value in the output buffer
        Ok(IsNull::No)
    }
}

/// Trait for converting between Rekordbox date strings and chrono DateTime
pub trait RekordboxDateString {
    fn from_date<Tz: TimeZone>(dt: DateTime<Tz>) -> Self;
    fn into_date(self) -> anyhow::Result<Date>;
}

impl RekordboxDateString for String {
    fn from_date<Tz: TimeZone>(dt: DateTime<Tz>) -> Self {
        format_datetime(&dt)
    }

    fn into_date(self) -> anyhow::Result<Date> {
        parse_datetime(&self)
    }
}

/// Recursively sort a playlist tree item and its children
fn sort_tree(item: &mut Rc<RefCell<DjmdPlaylistTreeItem>>) {
    // Sort the current node's children by a chosen criterion (e.g., ID or name)
    item.borrow_mut().children.sort_by(|a, b| {
        let a_seq = a.borrow().seq;
        let b_seq = b.borrow().seq;
        a_seq.cmp(&b_seq)
    });
    // Recursively sort each child
    for child in &mut item.borrow_mut().children {
        sort_tree(child);
    }
}

/// Sort a list of playlist tree items and their children
pub fn sort_tree_list(tree: &mut Vec<Rc<RefCell<DjmdPlaylistTreeItem>>>) {
    // Sort the root nodes
    tree.sort_by(|a, b| {
        let a_seq = a.borrow().seq;
        let b_seq = b.borrow().seq;
        a_seq.cmp(&b_seq)
    });
    // Sort each tree item recursively
    for item in tree {
        sort_tree(item);
    }
}
