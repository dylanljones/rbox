// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use rbox::masterdb::{LogicalOperator, Operator, Property, SmartList};

/// Build the content-ID SQL for a single-condition smart list.
fn sql_of(property: Property, operator: Operator, left: &str) -> String {
    let mut smart = SmartList::new("0", LogicalOperator::All);
    smart.new_condition(property, operator, left.to_string(), None, None);
    smart.build_content_id_sql().unwrap()
}

// -- String properties ----------------------------------------------------------------------------

#[test]
fn test_string_contains() {
    let sql = sql_of(Property::Name, Operator::Contains, "house");
    assert_eq!(
        sql,
        "SELECT DISTINCT djmdContent.ID AS id FROM djmdContent \
         WHERE djmdContent.Title LIKE '%house%'"
    );
}

#[test]
fn test_string_escapes_single_quotes() {
    let sql = sql_of(Property::Name, Operator::Equal, "O'Brien");
    assert!(sql.contains("djmdContent.Title = 'O''Brien'"), "got: {sql}");
}

#[test]
fn test_string_all_operators() {
    assert!(sql_of(Property::Comments, Operator::Equal, "x").contains("djmdContent.Commnt = 'x'"));
    assert!(
        sql_of(Property::Comments, Operator::NotEqual, "x").contains("djmdContent.Commnt != 'x'")
    );
    assert!(sql_of(Property::Comments, Operator::Contains, "x").contains("LIKE '%x%'"));
    assert!(sql_of(Property::Comments, Operator::NotContains, "x").contains("NOT LIKE '%x%'"));
    assert!(sql_of(Property::Comments, Operator::StartsWith, "x").contains("LIKE 'x%'"));
    assert!(sql_of(Property::Comments, Operator::EndsWith, "x").contains("LIKE '%x'"));
}

// -- Joins ----------------------------------------------------------------------------------------

#[test]
fn test_artist_join_uses_alias() {
    let sql = sql_of(Property::Artist, Operator::Equal, "Daft Punk");
    assert!(
        sql.contains("LEFT JOIN djmdArtist AS sl_artist ON djmdContent.ArtistID = sl_artist.ID"),
        "got: {sql}"
    );
    assert!(sql.contains("sl_artist.Name = 'Daft Punk'"), "got: {sql}");
}

#[test]
fn test_key_join_uses_scale_name() {
    let sql = sql_of(Property::Key, Operator::Equal, "Am");
    assert!(
        sql.contains("LEFT JOIN djmdKey AS sl_key ON djmdContent.KeyID = sl_key.ID"),
        "got: {sql}"
    );
    assert!(sql.contains("sl_key.ScaleName = 'Am'"), "got: {sql}");
}

#[test]
fn test_album_artist_two_hop_join_order() {
    let sql = sql_of(Property::AlbumArtist, Operator::Equal, "VA");
    // The album join must come before the artist join that depends on it.
    let album_idx = sql.find("djmdAlbum AS sl_aa_album").expect("album join");
    let artist_idx = sql
        .find("djmdArtist AS sl_album_artist")
        .expect("album artist join");
    assert!(album_idx < artist_idx, "join order wrong: {sql}");
    assert!(
        sql.contains("sl_aa_album.AlbumArtistID = sl_album_artist.ID"),
        "got: {sql}"
    );
    assert!(sql.contains("sl_album_artist.Name = 'VA'"), "got: {sql}");
}

#[test]
fn test_multiple_artist_roles_have_distinct_aliases() {
    // Artist, original artist, producer and remixer all reference djmdArtist and must not
    // collide on the table alias.
    let mut smart = SmartList::new("0", LogicalOperator::All);
    smart.new_condition(Property::Artist, Operator::Equal, "a".into(), None, None);
    smart.new_condition(
        Property::OriginalArtist,
        Operator::Equal,
        "b".into(),
        None,
        None,
    );
    smart.new_condition(Property::Producer, Operator::Equal, "c".into(), None, None);
    smart.new_condition(Property::Remixer, Operator::Equal, "d".into(), None, None);
    let sql = smart.build_content_id_sql().unwrap();
    for alias in ["sl_artist", "sl_org_artist", "sl_composer", "sl_remixer"] {
        assert!(sql.contains(alias), "missing alias {alias} in: {sql}");
    }
}

// -- Numeric properties ---------------------------------------------------------------------------

#[test]
fn test_number_range_is_inclusive() {
    let mut smart = SmartList::new("0", LogicalOperator::All);
    smart.new_condition(
        Property::BPM,
        Operator::InRange,
        "120".into(),
        Some("130".into()),
        None,
    );
    let sql = smart.build_content_id_sql().unwrap();
    assert!(
        sql.contains("djmdContent.BPM >= '120' AND djmdContent.BPM <= '130'"),
        "got: {sql}"
    );
}

#[test]
fn test_grouping_uses_color_id() {
    let sql = sql_of(Property::Grouping, Operator::Equal, "1");
    assert!(sql.contains("djmdContent.ColorID = '1'"), "got: {sql}");
}

// -- Date properties ------------------------------------------------------------------------------

#[test]
fn test_date_equal_compares_date_prefix() {
    let sql = sql_of(Property::DateReleased, Operator::Equal, "2020-01-01");
    assert!(
        sql.contains("substr(djmdContent.ReleaseDate, 1, 10) = '2020-01-01'"),
        "got: {sql}"
    );
}

#[test]
fn test_date_in_last_days() {
    let mut smart = SmartList::new("0", LogicalOperator::All);
    smart.new_condition(
        Property::DateCreated,
        Operator::InLast,
        "6".into(),
        None,
        Some("day".into()),
    );
    let sql = smart.build_content_id_sql().unwrap();
    assert!(
        sql.contains("substr(djmdContent.created_at, 1, 10) >= date('now', '-6 days')"),
        "got: {sql}"
    );
}

#[test]
fn test_date_not_in_last_months() {
    let mut smart = SmartList::new("0", LogicalOperator::All);
    smart.new_condition(
        Property::StockDate,
        Operator::NotInLast,
        "3".into(),
        None,
        Some("month".into()),
    );
    let sql = smart.build_content_id_sql().unwrap();
    assert!(
        sql.contains("substr(djmdContent.StockDate, 1, 10) < date('now', '-3 months')"),
        "got: {sql}"
    );
}

// -- My tags --------------------------------------------------------------------------------------

#[test]
fn test_mytag_has_tag() {
    let sql = sql_of(Property::MyTag, Operator::Contains, "123456");
    assert!(
        sql.contains(
            "djmdContent.ID IN (SELECT djmdSongMyTag.ContentID FROM djmdSongMyTag \
             WHERE djmdSongMyTag.MyTagID = '123456')"
        ),
        "got: {sql}"
    );
}

#[test]
fn test_mytag_not_has_tag() {
    let sql = sql_of(Property::MyTag, Operator::NotContains, "123456");
    assert!(sql.contains("djmdContent.ID NOT IN ("), "got: {sql}");
}

#[test]
fn test_mytag_negative_id_is_normalized() {
    // Rekordbox stores tag IDs as signed 32-bit values; -1 maps back to 2^32 - 1.
    let sql = sql_of(Property::MyTag, Operator::Contains, "-1");
    assert!(
        sql.contains("djmdSongMyTag.MyTagID = '4294967295'"),
        "got: {sql}"
    );
}

// -- Logical operator -----------------------------------------------------------------------------

#[test]
fn test_logical_operator_any_uses_or() {
    let mut smart = SmartList::new("0", LogicalOperator::Any);
    smart.new_condition(Property::Name, Operator::Contains, "a".into(), None, None);
    smart.new_condition(
        Property::Comments,
        Operator::Contains,
        "b".into(),
        None,
        None,
    );
    let sql = smart.build_content_id_sql().unwrap();
    assert!(sql.contains(" OR "), "got: {sql}");
}

#[test]
fn test_logical_operator_all_uses_and() {
    let mut smart = SmartList::new("0", LogicalOperator::All);
    smart.new_condition(Property::Name, Operator::Contains, "a".into(), None, None);
    smart.new_condition(
        Property::Comments,
        Operator::Contains,
        "b".into(),
        None,
        None,
    );
    let sql = smart.build_content_id_sql().unwrap();
    assert!(sql.contains(" AND "), "got: {sql}");
}

// -- Edge cases -----------------------------------------------------------------------------------

#[test]
fn test_mixname_is_skipped() {
    // `mixName` has no column mapping, so the condition contributes no filter.
    let sql = sql_of(Property::MixName, Operator::Contains, "club");
    assert_eq!(sql, "SELECT DISTINCT djmdContent.ID AS id FROM djmdContent");
    assert!(!sql.contains("WHERE"), "got: {sql}");
}

#[test]
fn test_no_conditions_has_no_where() {
    let smart = SmartList::new("0", LogicalOperator::All);
    let sql = smart.build_content_id_sql().unwrap();
    assert_eq!(sql, "SELECT DISTINCT djmdContent.ID AS id FROM djmdContent");
}

#[test]
fn test_unsupported_operator_errors_not_panics() {
    // A text property with a numeric operator is invalid; it should error, not panic.
    let mut smart = SmartList::new("0", LogicalOperator::All);
    smart.new_condition(Property::Name, Operator::Greater, "x".into(), None, None);
    assert!(smart.build_content_id_sql().is_err());
}

#[test]
fn test_every_property_builds_valid_sql() {
    // Smoke test: every property must produce a clause (no panics / todos) for a
    // representative operator, so any Rekordbox smart list can be evaluated.
    let cases: [(Property, Operator, &str, Option<&str>); 23] = [
        (Property::Artist, Operator::Contains, "x", None),
        (Property::Album, Operator::Contains, "x", None),
        (Property::AlbumArtist, Operator::Contains, "x", None),
        (Property::OriginalArtist, Operator::Contains, "x", None),
        (Property::BPM, Operator::Greater, "120", None),
        (Property::Grouping, Operator::Equal, "1", None),
        (Property::Comments, Operator::Contains, "x", None),
        (Property::Producer, Operator::Contains, "x", None),
        (Property::StockDate, Operator::Less, "2020-01-01", None),
        (Property::DateCreated, Operator::InLast, "7", Some("day")),
        (Property::Counter, Operator::Greater, "1", None),
        (Property::Filename, Operator::Contains, "x", None),
        (Property::Genre, Operator::Equal, "House", None),
        (Property::Key, Operator::Equal, "Am", None),
        (Property::Label, Operator::Contains, "x", None),
        (Property::MixName, Operator::Contains, "x", None),
        (Property::MyTag, Operator::Contains, "1", None),
        (Property::Rating, Operator::Equal, "5", None),
        (
            Property::DateReleased,
            Operator::Greater,
            "2020-01-01",
            None,
        ),
        (Property::Remixer, Operator::Contains, "x", None),
        (Property::Duration, Operator::Less, "300", None),
        (Property::Name, Operator::Contains, "x", None),
        (Property::Year, Operator::Equal, "2020", None),
    ];
    for (property, operator, left, unit) in cases {
        let mut smart = SmartList::new("0", LogicalOperator::All);
        smart.new_condition(
            property.clone(),
            operator,
            left.to_string(),
            None,
            unit.map(|u| u.to_string()),
        );
        let sql = smart.build_content_id_sql();
        assert!(sql.is_ok(), "property {property:?} failed: {sql:?}");
    }
}
