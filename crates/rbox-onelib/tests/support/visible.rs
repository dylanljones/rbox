// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared contract for visible-only sequences.

/// Run the visibility contract for a One Library category or sort model.
macro_rules! visible_matrix {
    ($test:ident, $model:ident, $new:ident, $table:ident, $connection:expr) => {
        #[test]
        fn $test() {
            use rbox_onelib::models::{$model, $new};
            use rbox_onelib::schema::$table;
            let mut conn = $connection;
            diesel::delete($table::table).execute(&mut *conn).unwrap();
            let ordered = |conn: &mut SqliteConnection| -> Vec<$model> {
                let mut rows = $model::all(conn).unwrap().into_iter().filter(|r| r.is_visible == 1)
                    .collect::<Vec<_>>();
                rows.sort_by_key(|row| (row.seq, row.id));
                rows
            };
            for n in 0..=5 {
                $crate::ordering::case(&mut conn, |conn| {
                    let mut ids = Vec::new();
                    for _ in 0..n { ids.push($new::new(1).visible(1).insert(conn)?.id); }
                    let hidden = $new::new(1).visible(0).seq(999).insert(conn)?;
                    assert_eq!(hidden.seq, 0);
                    for slot in 1..=n+1 {
                        $crate::ordering::case(conn, |conn| {
                            let row = $new::new(1).visible(1).seq(slot).insert(conn)?;
                            let mut expected = ids.clone();
                            expected.insert((slot-1) as usize, row.id);
                            assert_eq!(ordered(conn).iter().map(|r| r.id).collect::<Vec<_>>(), expected);
                            assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                            Ok(())
                        });
                        $crate::ordering::case(conn, |conn| {
                            let mut row = hidden.clone();
                            row.is_visible = 1;
                            row.seq = slot;
                            row.update(conn)?;
                            let mut expected = ids.clone();
                            expected.insert((slot-1) as usize, hidden.id);
                            assert_eq!(ordered(conn).iter().map(|r| r.id).collect::<Vec<_>>(), expected);
                            assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                            Ok(())
                        });
                    }
                    for source in 0..n as usize {
                        for destination in 1..=n {
                            $crate::ordering::case(conn, |conn| {
                                let mut row = $model::find(conn, &ids[source])?.unwrap();
                                row.seq = destination;
                                row.update(conn)?;
                                let mut expected = ids.clone();
                                let id = expected.remove(source);
                                expected.insert((destination-1) as usize, id);
                                assert_eq!(ordered(conn).iter().map(|r| r.id).collect::<Vec<_>>(), expected);
                                assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                                Ok(())
                            });
                        }
                        for hide in [false, true] {
                            $crate::ordering::case(conn, |conn| {
                                if hide {
                                    let mut row = $model::find(conn, &ids[source])?.unwrap();
                                    row.is_visible = 0;
                                    assert_eq!(row.update(conn)?.seq, 0);
                                } else { $model::delete(conn, &ids[source])?; }
                                let mut expected = ids.clone();
                                expected.remove(source);
                                assert_eq!(ordered(conn).iter().map(|r| r.id).collect::<Vec<_>>(), expected);
                                assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                                Ok(())
                            });
                        }
                    }
                    let before = $model::all(conn)?;
                    // Bulk operations must roll back earlier successful writes.
                    assert!($new::insert_all(conn, vec![
                        $new::new(1).visible(1).seq(1),
                        $new::new(1).visible(2),
                    ]).is_err());
                    assert_eq!($model::all(conn)?, before);
                    let mut valid = hidden.clone();
                    valid.is_visible = 1;
                    valid.seq = 1;
                    let mut invalid = hidden.clone();
                    invalid.is_visible = 2;
                    assert!($model::update_all(conn, vec![valid, invalid]).is_err());
                    assert_eq!($model::all(conn)?, before);
                    assert_eq!(hidden.clone().update(conn)?, hidden);
                    for invalid in [-1, n+2, i32::MAX] {
                        assert!($new::new(1).visible(1).seq(invalid).insert(conn).is_err());
                        let mut row = hidden.clone();
                        row.is_visible = 1;
                        row.seq = invalid;
                        assert!(row.update(conn).is_err());
                        assert_eq!($model::all(conn)?, before);
                    }
                    for invalid in [-1, 2] {
                        let mut row = hidden.clone();
                        row.is_visible = invalid;
                        assert!(row.update(conn).is_err());
                        assert!($new::new(1).visible(invalid).insert(conn).is_err());
                        assert_eq!($model::all(conn)?, before);
                    }
                    $crate::ordering::case(conn, |conn| {
                        let mut row = hidden.clone();
                        row.is_visible = 1;
                        row.seq = 0;
                        let row = row.update(conn)?;
                        assert_eq!(row.seq, n+1);
                        assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                        Ok(())
                    });
                    if n > 0 {
                        $crate::ordering::case(conn, |conn| {
                            diesel::update($table::table.filter($table::is_visible.eq(1)))
                                .set($table::seq.eq(10)).execute(conn)?;
                            <$model as SequenceVisible>::reset_seq(conn)?;
                            assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                            assert_eq!(<$model as SequenceVisible>::reset_seq(conn)?, 0);
                            assert_eq!($model::find(conn, &hidden.id)?.unwrap(), hidden);
                            Ok(())
                        });
                        $crate::ordering::case(conn, |conn| {
                            let deleted = $model::delete_all(conn, ids.iter().step_by(2).chain(std::iter::once(&hidden.id)).collect())?;
                            assert_eq!(deleted, (ids.len() + 1) / 2 + 1);
                            assert_eq!($model::delete_all(conn, vec![])?, 0);
                            assert_eq!($model::delete_all(conn, vec![&hidden.id])?, 0);
                            assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                            Ok(())
                        });
                        // A trigger rejects the final visibility change after sibling shifts.
                        diesel::sql_query(format!("CREATE TEMP TRIGGER fail_visible BEFORE UPDATE ON {} WHEN OLD.isVisible = 0 AND NEW.isVisible = 1 BEGIN SELECT RAISE(ABORT, 'injected failure'); END", stringify!($table))).execute(conn)?;
                        let mut row = hidden.clone();
                        row.is_visible = 1;
                        row.seq = 1;
                        assert!(row.update(conn).is_err());
                        assert_eq!($model::all(conn)?, before);
                        diesel::sql_query("DROP TRIGGER fail_visible").execute(conn)?;
                    }
                    Ok(())
                });
            }
        }
    };
}

/// Verify the public handler methods for a visible One Library sequence.
macro_rules! visible_handler_test {
    ($test:ident, $model:ident, $create:ident, $update:ident, $delete:ident, $setup:expr) => {
        #[test]
        fn $test() {
            use rbox_onelib::{database::OneLibrary, models::$model};
            let mut fixture = $setup;
            let db = OneLibrary::new(fixture.path()).unwrap();
            // Remove fixture rows using model operations so no sequence gaps remain.
            let original = $model::all(&mut fixture).unwrap();
            $model::delete_all(&mut fixture, original.iter().map(|row| &row.id).collect()).unwrap();
            let a = db.$create(1, None, Some(1)).unwrap();
            let b = db.$create(1, Some(1), Some(1)).unwrap();
            assert_eq!(
                <$model as SequenceVisible>::get_seq_numbers(&mut fixture).unwrap(),
                vec![1, 2]
            );
            assert_eq!($model::find(&mut fixture, &a.id).unwrap().unwrap().seq, 2);
            let mut moved = a.clone();
            moved.seq = 1;
            db.$update(moved).unwrap();
            assert_eq!($model::find(&mut fixture, &b.id).unwrap().unwrap().seq, 2);
            let mut hidden = $model::find(&mut fixture, &a.id).unwrap().unwrap();
            hidden.is_visible = 0;
            assert_eq!(db.$update(hidden).unwrap().seq, 0);
            assert_eq!($model::find(&mut fixture, &b.id).unwrap().unwrap().seq, 1);
            let before = $model::all(&mut fixture).unwrap();
            assert!(db.$create(1, Some(-1), Some(1)).is_err());
            assert_eq!($model::all(&mut fixture).unwrap(), before);
            assert_eq!(db.$delete(b.id).unwrap(), 1);
            assert_eq!(
                <$model as SequenceVisible>::count_visible(&mut fixture).unwrap(),
                0
            );
            assert_eq!(db.$delete(a.id).unwrap(), 1);
        }
    };
}

pub(crate) use {visible_handler_test, visible_matrix};
