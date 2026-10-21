// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Date parsing and Diesel error propagation for One Library date fields.

use chrono::{DateTime, Utc};
use diesel::{Connection, RunQueryDsl, SqliteConnection};
use rbox_onelib::error::Error;
use rbox_onelib::models::{format_datetime, parse_datetime, DateString, OneLibraryDateString};

#[test]
fn dates_parse_at_midnight_utc_with_optional_whitespace() {
    let expected = DateTime::parse_from_rfc3339("2024-02-29T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    for input in ["2024-02-29", " \t2024-02-29\r\n"] {
        assert_eq!(parse_datetime(input).unwrap(), expected);
        assert_eq!(input.to_owned().into_date().unwrap(), expected);
    }
    assert_eq!(format_datetime(&expected), "2024-02-29");
}

#[test]
fn invalid_dates_preserve_input_and_parse_error() {
    for input in [
        "",
        " ",
        "2023-02-29",
        "2024-13-01",
        "2024-01-32",
        "not a date",
        " 2024-02-30 ",
        "2024-02-29T12:00:00Z",
    ] {
        let expected = chrono::NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d").unwrap_err();
        match input.to_owned().into_date().unwrap_err() {
            Error::InvalidDateTime { value, source } => {
                assert_eq!(value, input);
                assert_eq!(source, expected);
            }
            other => panic!("Unexpected error: {other:?}"),
        }
    }
}

#[test]
fn diesel_deserialization_preserves_the_typed_error() {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    let error = diesel::select(diesel::dsl::sql::<diesel::sql_types::Text>(
        "' 2023-02-29 '",
    ))
    .get_result::<DateString>(&mut conn)
    .unwrap_err();
    match error {
        diesel::result::Error::DeserializationError(source) => {
            // Diesel can add field context around the error returned by FromSql.
            let mut source: &(dyn std::error::Error + 'static) = source.as_ref();
            while !source.is::<Error>() {
                source = source
                    .source()
                    .expect("typed date error in the source chain");
            }
            assert!(matches!(source.downcast_ref::<Error>(),
                Some(Error::InvalidDateTime { value, .. }) if value == " 2023-02-29 "));
            assert!(source.source().unwrap().is::<chrono::ParseError>());
        }
        other => panic!("Unexpected Diesel error: {other:?}"),
    }
}

#[test]
fn diesel_round_trip_uses_the_utc_date_and_discards_time() {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    let input = DateTime::parse_from_rfc3339("2024-03-01T01:30:00+02:00").unwrap();
    assert_eq!(String::from_date(input), "2024-02-29");
    let value = DateString::from(input.with_timezone(&Utc));
    let loaded = diesel::select(
        diesel::dsl::sql::<diesel::sql_types::Text>("").bind::<diesel::sql_types::Text, _>(value),
    )
    .get_result::<DateString>(&mut conn)
    .unwrap();
    assert_eq!(loaded.0, parse_datetime("2024-02-29").unwrap());
}
