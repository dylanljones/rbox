// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared ordering contract tests. Each adapter runs the same finite, exhaustive matrices.
//! Cases execute in rollback transactions so fixtures and subsequent cases remain unchanged.

use diesel::prelude::*;
use diesel::result::Error;
use rbox::model_traits::{Model, ModelDelete, ModelList, ModelTree, OrderedModel};
use std::borrow::Borrow;
use std::fmt::Debug;

type Key<M> = <<M as Model>::Id as ToOwned>::Owned;

#[track_caller]
pub fn case(conn: &mut SqliteConnection, f: impl FnOnce(&mut SqliteConnection) -> QueryResult<()>) {
    let location = std::panic::Location::caller();
    let result = conn.transaction(|conn| {
        f(conn).unwrap_or_else(|error| panic!("ordering scenario at {location}: {error}"));
        Err::<(), Error>(Error::RollbackTransaction)
    });
    assert!(matches!(result, Err(Error::RollbackTransaction)));
}

fn keys<M: OrderedModel>(rows: &[M]) -> Vec<Key<M>> {
    rows.iter().map(M::order_id).collect()
}

fn registry(conn: &mut SqliteConnection) -> Option<i32> {
    rbox::masterdb::AgentRegistry::local_usn(conn).ok()
}

pub fn assert_order<M>(conn: &mut SqliteConnection, parent: &M::ParentId, expected: &[Key<M>])
where
    M: OrderedModel,
    Key<M>: PartialEq + Debug,
{
    let rows = M::ordered(conn, parent).unwrap();
    assert_eq!(keys::<M>(&rows), expected);
    assert_eq!(
        rows.iter().map(M::order_position).collect::<Vec<_>>(),
        (M::START_SEQ..M::START_SEQ + rows.len() as i32).collect::<Vec<_>>()
    );
    assert!(M::validate_seq_numbers(conn, parent).unwrap());
}

/// Every ordered subset, including empty, singletons, noncontiguous selections and permutations.
fn selections(n: usize) -> Vec<Vec<usize>> {
    fn visit(n: usize, prefix: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        out.push(prefix.clone());
        for i in 0..n {
            if !prefix.contains(&i) {
                prefix.push(i);
                visit(n, prefix, out);
                prefix.pop();
            }
        }
    }
    let mut out = Vec::new();
    visit(n, &mut Vec::new(), &mut out);
    out
}

pub fn tree_matrix<M>(
    conn: &mut SqliteConnection,
    root: &M::Id,
    missing: &M::Id,
    create: impl Fn(&mut SqliteConnection, &M::Id, Option<i32>) -> QueryResult<M>,
    table: &str,
) where
    M: ModelTree + ModelDelete + Debug + PartialEq,
    Key<M>: Clone + PartialEq + Debug,
{
    let p = create(conn, root, None).unwrap().order_id();
    let q = create(conn, root, None).unwrap().order_id();
    let untouched = M::ordered(conn, root).unwrap();
    assert_eq!(
        M::move_all_to(conn, &[], missing, Some(i32::MIN)).unwrap(),
        0
    );
    assert!(create(conn, missing, None).is_err());

    // All insertion slots, all single moves, all deletions, for sizes zero through five.
    for n in 0..=5 {
        case(conn, |conn| {
            let mut initial = Vec::new();
            for _ in 0..n {
                initial.push(create(conn, p.borrow(), None)?.order_id());
            }
            for slot in 0..=n {
                case(conn, |conn| {
                    let inserted =
                        create(conn, p.borrow(), Some(M::START_SEQ + slot as i32))?.order_id();
                    let mut expected = initial.clone();
                    expected.insert(slot, inserted);
                    assert_order::<M>(conn, p.borrow(), &expected);
                    Ok(())
                });
            }
            for source in 0..n {
                for destination in 0..n {
                    case(conn, |conn| {
                        let before = M::ordered(conn, p.borrow())?;
                        let usn = registry(conn);
                        M::move_to(
                            conn,
                            initial[source].borrow(),
                            None,
                            Some(M::START_SEQ + destination as i32),
                        )?;
                        let mut expected = initial.clone();
                        let item = expected.remove(source);
                        expected.insert(destination, item);
                        assert_order::<M>(conn, p.borrow(), &expected);
                        if source == destination {
                            assert_eq!(M::ordered(conn, p.borrow())?, before);
                            assert_eq!(registry(conn), usn);
                        }
                        Ok(())
                    });
                }
                case(conn, |conn| {
                    M::move_to(conn, initial[source].borrow(), None, None)?;
                    let mut expected = initial.clone();
                    let item = expected.remove(source);
                    expected.push(item);
                    assert_order::<M>(conn, p.borrow(), &expected);
                    Ok(())
                });
                case(conn, |conn| {
                    M::delete(conn, initial[source].borrow())?;
                    let mut expected = initial.clone();
                    expected.remove(source);
                    assert_order::<M>(conn, p.borrow(), &expected);
                    Ok(())
                });
            }
            for invalid in [M::START_SEQ - 1, M::START_SEQ + n as i32 + 1, i32::MAX] {
                let before = M::ordered(conn, p.borrow())?;
                assert!(create(conn, p.borrow(), Some(invalid)).is_err());
                assert_eq!(M::ordered(conn, p.borrow())?, before);
            }
            Ok(())
        });
    }

    // Every ordered subset of four siblings, at every valid final position.
    case(conn, |conn| {
        let initial: Vec<_> = (0..4)
            .map(|_| create(conn, p.borrow(), None).unwrap().order_id())
            .collect();
        for selection in selections(4) {
            let ids: Vec<&M::Id> = selection.iter().map(|&i| initial[i].borrow()).collect();
            case(conn, |conn| {
                M::delete_all(conn, ids.clone())?;
                let expected = initial
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !selection.contains(i))
                    .map(|(_, id)| id.clone())
                    .collect::<Vec<_>>();
                assert_order::<M>(conn, p.borrow(), &expected);
                Ok(())
            });
            for slot in 0..=4 - selection.len() {
                case(conn, |conn| {
                    let before = M::ordered(conn, p.borrow())?;
                    let usn = registry(conn);
                    M::move_all_to(conn, &ids, p.borrow(), Some(M::START_SEQ + slot as i32))?;
                    let mut expected = initial
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !selection.contains(i))
                        .map(|(_, id)| id.clone())
                        .collect::<Vec<_>>();
                    expected.splice(slot..slot, selection.iter().map(|&i| initial[i].clone()));
                    assert_order::<M>(conn, p.borrow(), &expected);
                    if expected == initial {
                        assert_eq!(M::ordered(conn, p.borrow())?, before);
                        assert_eq!(registry(conn), usn);
                    }
                    Ok(())
                });
            }
            case(conn, |conn| {
                M::move_all_to(conn, &ids, p.borrow(), None)?;
                let mut expected = initial
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !selection.contains(i))
                    .map(|(_, id)| id.clone())
                    .collect::<Vec<_>>();
                expected.extend(selection.iter().map(|&i| initial[i].clone()));
                assert_order::<M>(conn, p.borrow(), &expected);
                Ok(())
            });
        }
        let before = M::ordered(conn, p.borrow())?;
        let usn = registry(conn);
        for ids in [
            vec![initial[0].borrow(), initial[0].borrow()],
            vec![initial[0].borrow(), missing],
        ] {
            assert!(M::move_all_to(conn, &ids, p.borrow(), None).is_err());
            assert_eq!(M::ordered(conn, p.borrow())?, before);
        }
        for invalid in [M::START_SEQ - 1, M::START_SEQ + 4, i32::MAX] {
            assert!(M::move_to(conn, initial[0].borrow(), None, Some(invalid)).is_err());
            assert!(M::move_all_to(
                conn,
                &[initial[0].borrow(), initial[1].borrow()],
                p.borrow(),
                Some(invalid)
            )
            .is_err());
            assert_eq!(M::ordered(conn, p.borrow())?, before);
        }
        assert!(M::move_to(conn, missing, None, None).is_err());
        assert!(M::move_to(conn, initial[0].borrow(), Some(missing), None).is_err());
        assert!(M::move_to(conn, p.borrow(), Some(p.borrow()), None).is_err());
        assert!(M::move_to(conn, p.borrow(), Some(initial[0].borrow()), None).is_err());
        assert!(M::move_all_to(
            conn,
            &[p.borrow(), initial[1].borrow()],
            initial[0].borrow(),
            None
        )
        .is_err());
        assert_eq!(M::ordered(conn, p.borrow())?, before);

        // A cycle already present in the target ancestry is rejected without looping.
        case(conn, |conn| {
            let id_column = if table.starts_with("djmd") {
                "ID".to_owned()
            } else {
                format!("{table}_id")
            };
            let parent_column = if table.starts_with("djmd") {
                "ParentID".to_owned()
            } else {
                format!("{table}_id_parent")
            };
            let first = format!("{:?}", initial[0]).replace('"', "");
            let second = format!("{:?}", initial[1]).replace('"', "");
            diesel::sql_query(format!(
                "UPDATE {table} SET {parent_column} = '{second}' WHERE {id_column} = '{first}'"
            ))
            .execute(conn)?;
            diesel::sql_query(format!(
                "UPDATE {table} SET {parent_column} = '{first}' WHERE {id_column} = '{second}'"
            ))
            .execute(conn)?;
            assert!(
                M::move_to(conn, initial[2].borrow(), Some(initial[0].borrow()), None).is_err()
            );
            assert!(M::delete_all(conn, vec![initial[0].borrow(), initial[1].borrow()]).is_err());
            Ok(())
        });

        // INSERT failure after siblings have shifted must roll the shifts back.
        diesel::sql_query(format!("CREATE TEMP TRIGGER fail_insert BEFORE INSERT ON {table} BEGIN SELECT RAISE(ABORT, 'injected insert failure'); END")).execute(conn)?;
        assert!(create(conn, p.borrow(), Some(M::START_SEQ)).is_err());
        assert_eq!(M::ordered(conn, p.borrow())?, before);
        assert_eq!(registry(conn), usn);
        diesel::sql_query("DROP TRIGGER fail_insert").execute(conn)?;
        // Fail the final write of a single move, after an earlier range update succeeded.
        let id_column = if table.starts_with("djmd") {
            "ID".to_string()
        } else {
            format!("{table}_id")
        };
        let key = format!("{:?}", initial[0]).replace('"', "");
        let position = if table.starts_with("djmd") {
            "Seq"
        } else {
            "sequenceNo"
        };
        diesel::sql_query(format!("CREATE TEMP TRIGGER fail_move BEFORE UPDATE ON {table} WHEN OLD.{id_column} = '{key}' BEGIN SELECT RAISE(ABORT, 'injected move failure'); END")).execute(conn)?;
        assert!(M::move_to(conn, initial[0].borrow(), None, None).is_err());
        assert_eq!(M::ordered(conn, p.borrow())?, before);
        assert!(M::move_all_to(
            conn,
            &[initial[1].borrow(), initial[0].borrow()],
            p.borrow(),
            None
        )
        .is_err());
        assert_eq!(M::ordered(conn, p.borrow())?, before);
        assert_eq!(registry(conn), usn);
        diesel::sql_query("DROP TRIGGER fail_move").execute(conn)?;

        // Duplicate/gapped positions repair deterministically by primary key, and repair is idempotent.
        diesel::sql_query(format!(
            "UPDATE {table} SET {position} = 7 WHERE {id_column} = '{key}'"
        ))
        .execute(conn)?;
        let sorted = keys::<M>(&M::ordered(conn, p.borrow())?);
        M::reset_seq(conn, p.borrow())?;
        assert_order::<M>(conn, p.borrow(), &sorted);
        let repaired = M::ordered(conn, p.borrow())?;
        assert_eq!(M::reset_seq(conn, p.borrow())?, 0);
        assert_eq!(M::ordered(conn, p.borrow())?, repaired);
        Ok(())
    });

    // Every permutation/subset drawn from two collections, targeting either collection.
    case(conn, |conn| {
        let initial: Vec<_> = (0..4)
            .map(|i| {
                create(conn, if i < 2 { p.borrow() } else { q.borrow() }, None)
                    .unwrap()
                    .order_id()
            })
            .collect();
        for selection in selections(4) {
            for destination in 0..2 {
                let target = if destination == 0 {
                    p.borrow()
                } else {
                    q.borrow()
                };
                let remaining = (destination * 2..destination * 2 + 2)
                    .filter(|i| !selection.contains(i))
                    .collect::<Vec<_>>();
                for slot in 0..=remaining.len() {
                    case(conn, |conn| {
                        let ids: Vec<&M::Id> =
                            selection.iter().map(|&i| initial[i].borrow()).collect();
                        M::move_all_to(conn, &ids, target, Some(M::START_SEQ + slot as i32))?;
                        for group in 0..2 {
                            let mut expected: Vec<_> = (group * 2..group * 2 + 2)
                                .filter(|i| !selection.contains(i))
                                .map(|i| initial[i].clone())
                                .collect();
                            if group == destination {
                                expected.splice(
                                    slot..slot,
                                    selection.iter().map(|&i| initial[i].clone()),
                                );
                            }
                            assert_order::<M>(
                                conn,
                                if group == 0 { p.borrow() } else { q.borrow() },
                                &expected,
                            );
                        }
                        Ok(())
                    });
                }
            }
        }
        Ok(())
    });
    assert_eq!(M::ordered(conn, root).unwrap(), untouched);

    // Ancestor/descendant selections delete each subtree once, in either input order.
    for reverse in [false, true] {
        case(conn, |conn| {
            let branch = create(conn, p.borrow(), None)?.order_id();
            let child = create(conn, branch.borrow(), None)?.order_id();
            let grandchild = create(conn, child.borrow(), None)?.order_id();
            let sibling = create(conn, p.borrow(), None)?.order_id();
            let mut ids: Vec<&M::Id> = vec![branch.borrow(), child.borrow(), grandchild.borrow()];
            if reverse {
                ids.reverse();
            }
            assert_eq!(M::delete_all(conn, ids)?, 3);
            assert_order::<M>(conn, p.borrow(), &[sibling]);
            for id in [&branch, &child, &grandchild] {
                assert!(M::find(conn, id.borrow())?.is_none());
            }
            Ok(())
        });
    }
}

pub fn list_matrix<M>(
    conn: &mut SqliteConnection,
    parent: &M::ParentId,
    other: &M::ParentId,
    create: impl Fn(&mut SqliteConnection, &M::ParentId, usize, Option<i32>) -> QueryResult<M>,
    table: &str,
) where
    M: ModelList + ModelDelete + Debug + PartialEq,
    Key<M>: Clone + PartialEq + Debug,
{
    assert_eq!(M::move_all_to(conn, &[], Some(i32::MIN)).unwrap(), 0);
    for n in 0..=5 {
        case(conn, |conn| {
            let initial = (0..n)
                .map(|i| create(conn, parent, i, None).unwrap().order_id())
                .collect::<Vec<_>>();
            for slot in 0..=n {
                case(conn, |conn| {
                    let added =
                        create(conn, parent, 5, Some(M::START_SEQ + slot as i32))?.order_id();
                    let mut expected = initial.clone();
                    expected.insert(slot, added);
                    assert_order::<M>(conn, parent, &expected);
                    Ok(())
                });
            }
            for source in 0..n {
                case(conn, |conn| {
                    M::move_to(conn, initial[source].borrow(), None)?;
                    let mut expected = initial.clone();
                    let row = expected.remove(source);
                    expected.push(row);
                    assert_order::<M>(conn, parent, &expected);
                    Ok(())
                });
            }
            assert!(create(conn, parent, 5, Some(-2)).is_err());
            assert!(create(conn, parent, 5, Some(i32::MAX)).is_err());
            assert_order::<M>(conn, parent, &initial);
            Ok(())
        });
    }
    case(conn, |conn| {
        let initial: Vec<_> = (0..4)
            .map(|i| create(conn, parent, i, None).unwrap().order_id())
            .collect();
        let other_row = create(conn, other, 4, None)?;
        let unrelated = M::ordered(conn, other)?;
        for source in 0..4 {
            for destination in 0..4 {
                case(conn, |conn| {
                    M::move_to(
                        conn,
                        initial[source].borrow(),
                        Some(M::START_SEQ + destination as i32),
                    )?;
                    let mut expected = initial.clone();
                    let item = expected.remove(source);
                    expected.insert(destination, item);
                    assert_order::<M>(conn, parent, &expected);
                    Ok(())
                });
            }
            case(conn, |conn| {
                M::delete(conn, initial[source].borrow())?;
                let mut expected = initial.clone();
                expected.remove(source);
                assert_order::<M>(conn, parent, &expected);
                Ok(())
            });
        }
        for slot in 0..=4 {
            case(conn, |conn| {
                let added = create(conn, parent, 5, Some(M::START_SEQ + slot as i32))?.order_id();
                let mut expected = initial.clone();
                expected.insert(slot, added);
                assert_order::<M>(conn, parent, &expected);
                Ok(())
            });
        }
        for selection in selections(4) {
            case(conn, |conn| {
                let ids: Vec<&M::Id> = selection.iter().map(|&i| initial[i].borrow()).collect();
                assert_eq!(M::delete_all(conn, ids)?, selection.len());
                let expected = initial
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !selection.contains(i))
                    .map(|(_, id)| id.clone())
                    .collect::<Vec<_>>();
                assert_order::<M>(conn, parent, &expected);
                Ok(())
            });
            case(conn, |conn| {
                let ids: Vec<&M::Id> = selection.iter().map(|&i| initial[i].borrow()).collect();
                M::move_all_to(conn, &ids, None)?;
                let mut expected = initial
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !selection.contains(i))
                    .map(|(_, id)| id.clone())
                    .collect::<Vec<_>>();
                expected.extend(selection.iter().map(|&i| initial[i].clone()));
                assert_order::<M>(conn, parent, &expected);
                Ok(())
            });
            for slot in 0..=4 - selection.len() {
                case(conn, |conn| {
                    let ids: Vec<&M::Id> = selection.iter().map(|&i| initial[i].borrow()).collect();
                    let before = M::ordered(conn, parent)?;
                    M::move_all_to(conn, &ids, Some(M::START_SEQ + slot as i32))?;
                    let mut expected = initial
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !selection.contains(i))
                        .map(|(_, id)| id.clone())
                        .collect::<Vec<_>>();
                    expected.splice(slot..slot, selection.iter().map(|&i| initial[i].clone()));
                    assert_order::<M>(conn, parent, &expected);
                    if expected == initial {
                        assert_eq!(M::ordered(conn, parent)?, before);
                    }
                    Ok(())
                });
            }
        }
        let before = M::ordered(conn, parent)?;
        let other_id = other_row.order_id();
        assert!(M::move_all_to(conn, &[initial[0].borrow(), other_id.borrow()], None).is_err());
        assert!(M::move_all_to(conn, &[initial[0].borrow(), initial[0].borrow()], None).is_err());
        for invalid in [M::START_SEQ - 1, M::START_SEQ + 4, i32::MAX] {
            assert!(M::move_to(conn, initial[0].borrow(), Some(invalid)).is_err());
            assert!(M::move_all_to(
                conn,
                &[initial[0].borrow(), initial[1].borrow()],
                Some(invalid)
            )
            .is_err());
        }
        assert_eq!(M::ordered(conn, parent)?, before);
        case(conn, |conn| {
            M::delete(conn, initial[1].borrow())?;
            let before = M::ordered(conn, parent)?;
            assert!(M::move_to(conn, initial[1].borrow(), None).is_err());
            assert!(
                M::move_all_to(conn, &[initial[0].borrow(), initial[1].borrow()], None).is_err()
            );
            assert_eq!(M::ordered(conn, parent)?, before);
            Ok(())
        });
        let seq_column = if table.starts_with("djmd") {
            "TrackNo"
        } else {
            "seq"
        };
        diesel::sql_query(format!("CREATE TEMP TRIGGER fail_move BEFORE UPDATE ON {table} WHEN OLD.{seq_column} = {} AND NEW.{seq_column} = {} BEGIN SELECT RAISE(ABORT, 'injected move failure'); END", M::START_SEQ, M::START_SEQ + 3)).execute(conn)?;
        let usn = registry(conn);
        assert!(M::move_to(conn, initial[0].borrow(), None).is_err());
        assert_eq!(M::ordered(conn, parent)?, before);
        assert!(M::move_all_to(conn, &[initial[1].borrow(), initial[0].borrow()], None).is_err());
        assert_eq!(M::ordered(conn, parent)?, before);
        assert_eq!(registry(conn), usn);
        diesel::sql_query("DROP TRIGGER fail_move").execute(conn)?;
        diesel::sql_query(format!("CREATE TEMP TRIGGER fail_delete BEFORE DELETE ON {table} WHEN OLD.{seq_column} = {} BEGIN SELECT RAISE(ABORT, 'injected delete failure'); END", M::START_SEQ)).execute(conn)?;
        assert!(M::delete_all(conn, vec![initial[3].borrow(), initial[0].borrow()]).is_err());
        assert_eq!(M::ordered(conn, parent)?, before);
        assert_eq!(registry(conn), usn);
        diesel::sql_query("DROP TRIGGER fail_delete").execute(conn)?;
        diesel::sql_query(format!("CREATE TEMP TRIGGER fail_insert BEFORE INSERT ON {table} BEGIN SELECT RAISE(ABORT, 'injected failure'); END")).execute(conn)?;
        assert!(create(conn, parent, 5, Some(M::START_SEQ)).is_err());
        assert_eq!(M::ordered(conn, parent)?, before);
        diesel::sql_query("DROP TRIGGER fail_insert").execute(conn)?;
        assert_eq!(M::ordered(conn, other)?, unrelated);
        Ok(())
    });
}
