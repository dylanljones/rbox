// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared enums retain their database codes and work with Diesel in downstream crates.

use diesel::{Connection, RunQueryDsl, SqliteConnection};
use rbox_db::enums::{HistoryType, HotCueBanklistType, MyTagType};

#[test]
fn shared_enums_round_trip_and_reject_unknown_values() {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    macro_rules! check {
        ($ty:ty) => {
            for (value, code) in [(<$ty>::List, 0), (<$ty>::Folder, 1)] {
                assert_eq!(value as i32, code);
                assert_eq!(<$ty>::try_from(code).unwrap(), value);
                let loaded = diesel::select(
                    diesel::dsl::sql::<diesel::sql_types::Integer>("")
                        .bind::<diesel::sql_types::Integer, _>(value),
                )
                .get_result::<$ty>(&mut conn)
                .unwrap();
                assert_eq!(loaded, value);
            }
            assert!(<$ty>::try_from(99).is_err());
            assert!(
                diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("99"))
                    .get_result::<$ty>(&mut conn)
                    .is_err()
            );
        };
    }
    check!(HistoryType);
    check!(HotCueBanklistType);
    check!(MyTagType);
}
