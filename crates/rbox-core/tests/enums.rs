// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared enum codes remain available without database dependencies.

use rbox_core::{
    Analyzed, FileType, HistoryType, HotCueBanklistType, MyTagType, PlaylistType,
    RelatedTracksType, SamplerType,
};

#[test]
fn database_enum_codes_do_not_require_diesel() {
    macro_rules! check {
        ($ty:ty, $($variant:ident = $code:expr),+ $(,)?) => {{
            $(
                assert_eq!(<$ty>::try_from($code).unwrap(), <$ty>::$variant);
                let code: i32 = <$ty>::$variant.into();
                assert_eq!(code, $code);
            )+
            assert!(<$ty>::try_from(99).is_err());
            assert_eq!(<$ty>::default(), <$ty>::List);
        }};
    }
    check!(HistoryType, List = 0, Folder = 1);
    check!(HotCueBanklistType, List = 0, Folder = 1);
    check!(MyTagType, List = 0, Folder = 1);
    check!(RelatedTracksType, List = 11, Folder = 10);
    check!(SamplerType, List = 2, Folder = 3, All = 5, Capture = 6);
    for (value, code) in [
        (Analyzed::NotAnalyzed, 0),
        (Analyzed::Standard, 105),
        (Analyzed::Advanced, 121),
        (Analyzed::Locked, 233),
    ] {
        assert_eq!(Analyzed::try_from(code).unwrap(), value);
        let stored: i32 = value.into();
        assert_eq!(stored, code);
    }
    assert_eq!(Analyzed::default(), Analyzed::NotAnalyzed);
    assert!(Analyzed::try_from(99).is_err());
    check!(
        PlaylistType,
        List = 0,
        Folder = 1,
        SmartList = 4,
        CloudLibrarySync = -128
    );
}

#[cfg(feature = "diesel")]
#[test]
fn diesel_enum_round_trips_reject_unknown_codes() {
    use diesel::sql_types::Integer;
    use diesel::{Connection, RunQueryDsl, SqliteConnection};

    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    macro_rules! check {
        ($ty:ty, $($variant:ident),+ $(,)?) => {{
            $(
                let value = <$ty>::$variant;
                let loaded = diesel::select(diesel::dsl::sql::<Integer>("").bind::<Integer, _>(value))
                    .get_result::<$ty>(&mut conn).unwrap();
                assert_eq!(loaded, value);
            )+
            assert!(diesel::select(diesel::dsl::sql::<Integer>("99")).get_result::<$ty>(&mut conn).is_err());
        }};
    }
    check!(HistoryType, List, Folder);
    check!(HotCueBanklistType, List, Folder);
    check!(MyTagType, List, Folder);
    check!(RelatedTracksType, List, Folder);
    check!(SamplerType, List, Folder, All, Capture);
    check!(PlaylistType, List, Folder, SmartList, CloudLibrarySync);
    check!(
        FileType,
        INVALID,
        MP3,
        MP4,
        M4A,
        FLAC,
        ALAC,
        WAV,
        AIFF,
        VIDEO,
        StreamApple
    );
}

#[test]
fn file_type_codes_and_extension_aliases_are_preserved() {
    for (value, code) in [
        (FileType::INVALID, 0),
        (FileType::MP3, 1),
        (FileType::MP4, 3),
        (FileType::M4A, 4),
        (FileType::FLAC, 5),
        (FileType::ALAC, 6),
        (FileType::WAV, 11),
        (FileType::AIFF, 12),
        (FileType::VIDEO, 16),
        (FileType::StreamApple, 26),
    ] {
        assert_eq!(value as i32, code);
        assert_eq!(FileType::try_from(code).unwrap(), value);
    }
    for code in [-1, 2, i32::MAX] {
        assert!(FileType::try_from(code).is_err());
    }
    assert_eq!(FileType::try_from_extension("AAC").unwrap(), FileType::AAC);
    assert_eq!(FileType::try_from_extension("aif").unwrap(), FileType::AIF);
    assert_eq!(FileType::try_from_extension("m4a").unwrap(), FileType::M4A);
    assert_eq!(
        FileType::try_from_extension("alac").unwrap(),
        FileType::ALAC
    );
    assert!(FileType::try_from_extension("unknown").is_err());
}
