// Author: Dylan Jones
// Date:   2025-09-02

use chrono::NaiveDate;
use diesel::backend::Backend;
use diesel::deserialize::{FromSql, Result as DResult};
use diesel::serialize::{IsNull, Output, Result as SResult, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};

use crate::error::{Error, Result};

pub type Date = NaiveDate;

const DATEFMT: &str = "%Y-%m-%d";

/// Parse a Devicelib Plus date string into a chrono NaiveDate.
pub fn parse_datetime(s: &str) -> Result<Date> {
    Date::parse_from_str(s, DATEFMT)
        .map_err(|e| Error::Error(format!("Failed to parse datetime: {}", e)))
}

/// Format a chrono DateTime<Tz> into a Devicelib Plus date string.
pub fn format_datetime(dt: &Date) -> String {
    dt.format(DATEFMT).to_string()
}

/// Wrapper for NaiveDate for diesel serialization/deserialization
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
