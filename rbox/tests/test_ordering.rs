// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

#![cfg(all(feature = "master-db", feature = "one-library"))]

mod common;

use common::ordering;

use diesel::prelude::*;
use rbox::model_traits::*;
use rstest::rstest;

#[test]
fn one_library_tree_snapshot_has_stable_sibling_order() {
    use rbox::one_library::schema::playlist;
    use rbox::one_library::*;
    let mut conn = one_connection();
    let root = NewPlaylist::folder("root").insert(&mut conn).unwrap();
    let mut ids = Vec::new();
    for _ in 0..4 {
        ids.push(
            NewPlaylist::folder("child")
                .parent_id(root.id)
                .insert(&mut conn)
                .unwrap()
                .id,
        );
    }
    let grandchild = NewPlaylist::playlist("grandchild")
        .parent_id(ids[2])
        .insert(&mut conn)
        .unwrap();
    Playlist::move_all_to(&mut conn, &[&ids[3], &ids[1]], &root.id, Some(0)).unwrap();
    let tree = Playlist::tree(&mut conn).unwrap();
    let root_node = tree
        .iter()
        .find(|node| node.borrow().id == root.id)
        .unwrap()
        .borrow();
    assert_eq!(
        root_node
            .children
            .iter()
            .map(|n| n.borrow().id)
            .collect::<Vec<_>>(),
        vec![ids[3], ids[1], ids[0], ids[2]]
    );
    assert_eq!(
        root_node.children[3].borrow().children[0].borrow().id,
        grandchild.id
    );
    drop(root_node);
    diesel::update(playlist::table.filter(playlist::parent_id.eq(root.id)))
        .set(playlist::seq.eq(0))
        .execute(&mut *conn)
        .unwrap();
    ids.sort();
    for _ in 0..8 {
        let tree = Playlist::tree(&mut conn).unwrap();
        let node = tree
            .iter()
            .find(|node| node.borrow().id == root.id)
            .unwrap()
            .borrow();
        assert_eq!(
            node.children
                .iter()
                .map(|n| n.borrow().id)
                .collect::<Vec<_>>(),
            ids
        );
    }
}

#[rstest]
#[case(6)]
#[case(7)]
fn masterdb_tree_snapshot_and_public_delete(#[case] version: usize) {
    use rbox::masterdb::schema::djmdPlaylist;
    use rbox::masterdb::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn = establish_connection(fixture.path().to_str().unwrap()).unwrap();
    let root = NewDjmdPlaylist::folder("root").insert(&mut conn).unwrap();
    let mut ids = Vec::new();
    for _ in 0..4 {
        ids.push(
            NewDjmdPlaylist::folder("child")
                .parent_id(&root.id)
                .insert(&mut conn)
                .unwrap()
                .id,
        );
    }
    let grandchild = NewDjmdPlaylist::playlist("grandchild")
        .parent_id(&ids[2])
        .insert(&mut conn)
        .unwrap();
    DjmdPlaylist::move_all_to(&mut conn, &[&ids[3], &ids[1]], &root.id, Some(1)).unwrap();
    let tree = DjmdPlaylist::tree(&mut conn).unwrap();
    let root_node = tree
        .iter()
        .find(|node| node.borrow().id == root.id)
        .unwrap()
        .borrow();
    assert_eq!(
        root_node
            .children
            .iter()
            .map(|n| n.borrow().id.clone())
            .collect::<Vec<_>>(),
        vec![
            ids[3].clone(),
            ids[1].clone(),
            ids[0].clone(),
            ids[2].clone()
        ]
    );
    assert_eq!(
        root_node.children[3].borrow().children[0].borrow().id,
        grandchild.id
    );
    drop(root_node);
    ordering::case(&mut conn, |conn| {
        diesel::update(djmdPlaylist::table.filter(djmdPlaylist::parent_id.eq(&root.id)))
            .set(djmdPlaylist::seq.eq(1))
            .execute(conn)?;
        let mut sorted = ids.clone();
        sorted.sort();
        for _ in 0..8 {
            let tree = DjmdPlaylist::tree(conn)?;
            let node = tree
                .iter()
                .find(|node| node.borrow().id == root.id)
                .unwrap()
                .borrow();
            assert_eq!(
                node.children
                    .iter()
                    .map(|n| n.borrow().id.clone())
                    .collect::<Vec<_>>(),
                sorted
            );
        }
        Ok(())
    });
    let deleted = DjmdPlaylist::delete_all(&mut conn, vec![&grandchild.id, &ids[2]]).unwrap();
    assert_eq!(deleted.len(), 2);
    assert!(deleted.contains(&ids[2]));
    assert!(deleted.contains(&grandchild.id));
    assert!(DjmdPlaylist::validate_seq_numbers(&mut conn, &root.id).unwrap());
}

#[test]
fn masterdb_playlist_xml_tracks_both_sides_of_moves_and_deletions() {
    use rbox::masterdb::schema::djmdPlaylist;
    use rbox::masterdb::*;
    let fixture = common::setup_master_db_dir_path(6);
    let mut db = MasterDb::new(fixture.path().join("rekordbox/master.db")).unwrap();
    db.set_unsafe_writes(true);
    let p = db.create_playlist_folder("p".into(), None, None).unwrap();
    let q = db.create_playlist_folder("q".into(), None, None).unwrap();
    let a = db
        .create_playlist_folder("a".into(), Some(p.id.clone()), None)
        .unwrap();
    let b = db
        .create_playlist_folder("b".into(), Some(p.id.clone()), None)
        .unwrap();
    let c = db
        .create_playlist_folder("c".into(), Some(q.id.clone()), None)
        .unwrap();
    let mut conn = db.pool.get().unwrap();
    // Make stale XML timestamps distinguishable without sleeps or wall-clock assumptions.
    let mut xml = MasterPlaylistXml::load(db.playlist_xml_path().unwrap());
    for id in [&a.id, &b.id, &c.id] {
        xml.update(
            id.clone(),
            chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
        );
        diesel::update(djmdPlaylist::table.find(id))
            .set(djmdPlaylist::updated_at.eq("1970-01-01 00:00:00.000 +00:00"))
            .execute(&mut conn)
            .unwrap();
    }
    xml.dump().unwrap();
    drop(conn);
    db.move_playlist(&a.id, Some(1), Some(q.id.clone()))
        .unwrap();
    let xml = MasterPlaylistXml::load(db.playlist_xml_path().unwrap());
    for id in [&a.id, &b.id, &c.id] {
        let row = db.get_playlist_by_id(id).unwrap().unwrap();
        let node = xml.get_playlist(id.clone()).unwrap();
        assert_eq!(node.parent_id, row.parent_id);
        assert!(
            (node.timestamp - row.updated_at.naive_utc())
                .num_seconds()
                .abs()
                <= 1
        );
    }
    db.delete_playlist(&a.id).unwrap();
    let xml = MasterPlaylistXml::load(db.playlist_xml_path().unwrap());
    assert!(xml.get_playlist(a.id.clone()).is_none());
    let row = db.get_playlist_by_id(&c.id).unwrap().unwrap();
    assert!(
        (xml.get_playlist(c.id).unwrap().timestamp - row.updated_at.naive_utc())
            .num_seconds()
            .abs()
            <= 1
    );
    assert!(db
        .move_playlist(&"nonexistent".to_owned(), None, None)
        .is_err());
}

#[rstest]
#[case(6)]
#[case(7)]
fn masterdb_updates_and_bulk_rollback(#[case] version: usize) {
    use rbox::masterdb::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn = establish_connection(fixture.path().to_str().unwrap()).unwrap();
    macro_rules! check {
        ($model:ident, $new:ident, $leaf:expr) => {{
            let p = $new::folder("update parent").insert(&mut conn).unwrap();
            let q = $new::folder("update destination")
                .insert(&mut conn)
                .unwrap();
            let mut rows = Vec::new();
            for _ in 0..4 {
                rows.push(
                    $new::folder("child")
                        .parent_id(&p.id)
                        .insert(&mut conn)
                        .unwrap(),
                );
            }
            for source in 0..4 {
                for target in 0..4 {
                    ordering::case(&mut conn, |conn| {
                        let mut row = rows[source].clone();
                        row.seq = target as i32 + 1;
                        row.update(conn)?;
                        let mut expected = rows.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
                        let id = expected.remove(source);
                        expected.insert(target, id);
                        ordering::assert_order::<$model>(conn, &p.id, &expected);
                        Ok(())
                    });
                }
            }
            ordering::case(&mut conn, |conn| {
                let mut row = rows[0].clone();
                row.parent_id = q.id.clone();
                row.seq = 1;
                row.update(conn)?;
                ordering::assert_order::<$model>(
                    conn,
                    &p.id,
                    &rows[1..].iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
                );
                ordering::assert_order::<$model>(conn, &q.id, &[rows[0].id.clone()]);
                Ok(())
            });
            let usn = AgentRegistry::local_usn(&mut conn).unwrap();
            let before = $model::ordered(&mut conn, &p.id).unwrap();
            assert_eq!(rows[0].clone().update(&mut conn).unwrap(), rows[0]);
            assert_eq!(AgentRegistry::local_usn(&mut conn).unwrap(), usn);
            let mut cyclic = p.clone();
            cyclic.parent_id = rows[0].id.clone();
            assert!(cyclic.update(&mut conn).is_err());
            let mut invalid_kind = p.clone();
            invalid_kind.attribute = $leaf;
            assert!(invalid_kind.update(&mut conn).is_err());
            // The first insertion succeeds, then the invalid parent rejects the next one.
            assert!($new::insert_all(
                &mut conn,
                vec![
                    $new::folder("valid").parent_id(&p.id).seq(1),
                    $new::folder("invalid").parent_id("missing-ordering-parent"),
                ]
            )
            .is_err());
            // Likewise, a bulk update must roll back an earlier successful move.
            let mut first = rows[0].clone();
            first.seq = 4;
            let mut invalid = rows[1].clone();
            invalid.seq = i32::MAX;
            assert!($model::update_all(&mut conn, vec![first, invalid]).is_err());
            assert_eq!($model::ordered(&mut conn, &p.id).unwrap(), before);
            assert_eq!(AgentRegistry::local_usn(&mut conn).unwrap(), usn);
            let leaf = $new::new("leaf", $leaf).insert(&mut conn).unwrap();
            assert!($new::folder("bad parent")
                .parent_id(&leaf.id)
                .insert(&mut conn)
                .is_err());
            assert!($model::move_to(&mut conn, &rows[0].id, Some(&leaf.id), None).is_err());
        }};
    }
    check!(
        DjmdPlaylist,
        NewDjmdPlaylist,
        rbox::enums::PlaylistType::List
    );
    check!(DjmdHistory, NewDjmdHistory, rbox::enums::HistoryType::List);
    check!(DjmdMyTag, NewDjmdMyTag, rbox::enums::MyTagType::List);
    check!(
        DjmdRelatedTracks,
        NewDjmdRelatedTracks,
        rbox::enums::RelatedTracksType::List
    );
    check!(DjmdSampler, NewDjmdSampler, rbox::enums::SamplerType::List);
}

#[test]
fn one_library_association_deletion_and_rollback() {
    use rbox::one_library::*;
    let mut conn = one_connection();
    let content = content_ids(&mut conn);
    let playlists = (0..3)
        .map(|i| {
            NewPlaylist::playlist(format!("p{i}"))
                .insert(&mut conn)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let histories = (0..3)
        .map(|i| {
            NewHistory::new(format!("h{i}"), rbox::enums::HistoryType::List)
                .insert(&mut conn)
                .unwrap()
        })
        .collect::<Vec<_>>();
    for p in &playlists {
        for &id in &content {
            PlaylistContent::new(p.id, id).insert(&mut conn).unwrap();
        }
    }
    for h in &histories {
        for &id in &content {
            HistoryContent::new(h.id, id).insert(&mut conn).unwrap();
        }
    }
    let original_playlists = PlaylistContent::all(&mut conn).unwrap();
    let original_histories = HistoryContent::all(&mut conn).unwrap();
    // Duplicate composite keys fail after opening a slot; neither list may retain a gap.
    assert!(PlaylistContent::new(playlists[0].id, content[2])
        .seq(0)
        .insert(&mut conn)
        .is_err());
    assert!(HistoryContent::new(histories[0].id, content[2])
        .seq(0)
        .insert(&mut conn)
        .is_err());
    assert_eq!(PlaylistContent::all(&mut conn).unwrap(), original_playlists);
    assert_eq!(HistoryContent::all(&mut conn).unwrap(), original_histories);

    for remove in [
        vec![content[0]],
        vec![content[2]],
        vec![content[5]],
        vec![content[1], content[3]],
        content.clone(),
    ] {
        ordering::case(&mut conn, |conn| {
            if remove.len() == 1 {
                Content::delete(conn, &remove[0])?;
            } else {
                Content::delete_all(conn, remove.iter().collect())?;
            }
            for p in &playlists {
                let expected = content
                    .iter()
                    .filter(|id| !remove.contains(id))
                    .map(|&id| (p.id, id))
                    .collect::<Vec<_>>();
                ordering::assert_order::<PlaylistContent>(conn, &p.id, &expected);
            }
            for h in &histories {
                let expected = content
                    .iter()
                    .filter(|id| !remove.contains(id))
                    .map(|&id| (h.id, id))
                    .collect::<Vec<_>>();
                ordering::assert_order::<HistoryContent>(conn, &h.id, &expected);
            }
            Ok(())
        });
    }
    // Fail compaction of playlists after content deletion and history compaction have succeeded.
    diesel::sql_query("CREATE TEMP TRIGGER fail_compact BEFORE UPDATE ON playlist_content BEGIN SELECT RAISE(ABORT, 'injected failure'); END").execute(&mut *conn).unwrap();
    assert!(Content::delete(&mut conn, &content[0]).is_err());
    assert!(Content::id_exists(&mut conn, &content[0]).unwrap());
    assert_eq!(PlaylistContent::all(&mut conn).unwrap(), original_playlists);
    assert_eq!(HistoryContent::all(&mut conn).unwrap(), original_histories);
    diesel::sql_query("DROP TRIGGER fail_compact")
        .execute(&mut *conn)
        .unwrap();
    // Repair duplicate positions in two distinct composite-key collections.
    for table in ["playlist_content", "history_content"] {
        diesel::sql_query(format!("UPDATE {table} SET sequenceNo = 9"))
            .execute(&mut *conn)
            .unwrap();
    }
    for p in &playlists {
        PlaylistContent::reset_seq(&mut conn, &p.id).unwrap();
        ordering::assert_order::<PlaylistContent>(
            &mut conn,
            &p.id,
            &content.iter().map(|&id| (p.id, id)).collect::<Vec<_>>(),
        );
    }
    for h in &histories {
        HistoryContent::reset_seq(&mut conn, &h.id).unwrap();
        ordering::assert_order::<HistoryContent>(
            &mut conn,
            &h.id,
            &content.iter().map(|&id| (h.id, id)).collect::<Vec<_>>(),
        );
    }
}

#[test]
fn one_library_bulk_insert_and_invalid_parents() {
    use rbox::one_library::*;
    let mut conn = one_connection();
    macro_rules! check {
        ($model:ident, $new:ident, $leaf:expr) => {{
            let parent = $new::folder("parent").insert(&mut conn).unwrap();
            let first = $new::folder("first")
                .parent_id(parent.id)
                .insert(&mut conn)
                .unwrap();
            assert!($new::insert_all(
                &mut conn,
                vec![
                    $new::folder("valid")
                        .parent_id(parent.id)
                        .seq($model::START_SEQ),
                    $new::folder("invalid").parent_id(i32::MAX),
                ]
            )
            .is_err());
            ordering::assert_order::<$model>(&mut conn, &parent.id, &[first.id]);
            let leaf = $new::new("leaf", $leaf).insert(&mut conn).unwrap();
            assert!($new::folder("bad parent")
                .parent_id(leaf.id)
                .insert(&mut conn)
                .is_err());
            assert!($model::move_to(&mut conn, &first.id, Some(&leaf.id), None).is_err());
        }};
    }
    check!(Playlist, NewPlaylist, rbox::enums::PlaylistType::List);
    check!(History, NewHistory, rbox::enums::HistoryType::List);
    check!(MyTag, NewMyTag, rbox::enums::MyTagType::List);
}

#[test]
fn high_level_one_library_positions_are_unambiguous() {
    let fixture = one_connection();
    let db = rbox::OneLibrary::new(fixture._file.path()).unwrap();
    let p = db.create_playlist_folder("parent", None, None).unwrap();
    let a = db.create_playlist("a", Some(p.id), None).unwrap();
    let b = db.create_playlist("b", Some(p.id), Some(0)).unwrap();
    let c = db.create_playlist("c", Some(p.id), None).unwrap();
    let mut conn =
        rbox::one_library::establish_connection(fixture._file.path().to_str().unwrap()).unwrap();
    ordering::assert_order::<rbox::one_library::Playlist>(&mut conn, &p.id, &[b.id, a.id, c.id]);
}

macro_rules! visible_matrix {
    ($test:ident, $model:ident, $new:ident, $table:ident) => {
        #[test]
        fn $test() {
            use rbox::one_library::{$model, $new};
            use rbox::one_library::schema::$table;
            let mut conn = one_connection();
            diesel::delete($table::table).execute(&mut *conn).unwrap();
            let ordered = |conn: &mut SqliteConnection| -> Vec<$model> {
                $model::all(conn).unwrap().into_iter().filter(|r| r.is_visible == 1)
                    .collect::<Vec<_>>().tap_sort()
            };
            for n in 0..=5 {
                ordering::case(&mut conn, |conn| {
                    let mut ids = Vec::new();
                    for _ in 0..n { ids.push($new::new(1).visible(1).insert(conn)?.id); }
                    let hidden = $new::new(1).visible(0).seq(999).insert(conn)?;
                    assert_eq!(hidden.seq, 0);
                    for slot in 1..=n+1 {
                        ordering::case(conn, |conn| {
                            let row = $new::new(1).visible(1).seq(slot).insert(conn)?;
                            let mut expected = ids.clone();
                            expected.insert((slot-1) as usize, row.id);
                            assert_eq!(ordered(conn).iter().map(|r| r.id).collect::<Vec<_>>(), expected);
                            assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                            Ok(())
                        });
                        ordering::case(conn, |conn| {
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
                            ordering::case(conn, |conn| {
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
                            ordering::case(conn, |conn| {
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
                    for invalid in [-1, n+2, i32::MAX] {
                        assert!($new::new(1).visible(1).seq(invalid).insert(conn).is_err());
                        assert_eq!($model::all(conn)?, before);
                    }
                    for invalid in [-1, 2] {
                        let mut row = hidden.clone();
                        row.is_visible = invalid;
                        assert!(row.update(conn).is_err());
                        assert!($new::new(1).visible(invalid).insert(conn).is_err());
                        assert_eq!($model::all(conn)?, before);
                    }
                    ordering::case(conn, |conn| {
                        let mut row = hidden.clone();
                        row.is_visible = 1;
                        row.seq = 0;
                        let row = row.update(conn)?;
                        assert_eq!(row.seq, n+1);
                        assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                        Ok(())
                    });
                    if n > 0 {
                        ordering::case(conn, |conn| {
                            diesel::update($table::table.filter($table::is_visible.eq(1)))
                                .set($table::seq.eq(10)).execute(conn)?;
                            <$model as SequenceVisible>::reset_seq(conn)?;
                            assert!(<$model as SequenceVisible>::validate_seq_numbers(conn)?);
                            assert_eq!(<$model as SequenceVisible>::reset_seq(conn)?, 0);
                            assert_eq!($model::find(conn, &hidden.id)?.unwrap(), hidden);
                            Ok(())
                        });
                        ordering::case(conn, |conn| {
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
// Keep this test helper local: only these two models expose the same visible sequence fields.
trait SortVisible: Sized {
    fn tap_sort(self) -> Self;
}
impl SortVisible for Vec<rbox::one_library::Category> {
    fn tap_sort(mut self) -> Self {
        self.sort_by_key(|r| (r.seq, r.id));
        self
    }
}
impl SortVisible for Vec<rbox::one_library::Sort> {
    fn tap_sort(mut self) -> Self {
        self.sort_by_key(|r| (r.seq, r.id));
        self
    }
}
visible_matrix!(
    category_visibility_and_ordering,
    Category,
    NewCategory,
    category
);
visible_matrix!(sort_visibility_and_ordering, Sort, NewSort, sort);

#[test]
fn sequence_validation_boundaries() {
    assert!(validate_seq_numbers(&[], 0));
    assert!(validate_seq_numbers(&[0, 1, 2], 0));
    assert!(validate_seq_numbers(&[1, 2, 3], 1));
    assert!(validate_seq_numbers(&[i32::MAX], i32::MAX));
    assert!(!validate_seq_numbers(&[i32::MAX, i32::MIN], i32::MAX));
    assert!(!validate_seq_numbers(&[0, 0], 0));
    assert!(!validate_seq_numbers(&[0, 2], 0));
    assert!(!validate_seq_numbers(&[2, 1], 1));
}

struct OneFixture {
    conn: SqliteConnection,
    _file: tempfile::NamedTempFile,
}

impl std::ops::Deref for OneFixture {
    type Target = SqliteConnection;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl std::ops::DerefMut for OneFixture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

fn one_connection() -> OneFixture {
    let file = common::setup_one_library_path();
    let mut conn = rbox::one_library::establish_connection(file.path().to_str().unwrap()).unwrap();
    // Isolate the ordering collections; the exported fixture contains unsigned tag IDs
    // larger than the current i32 model supports for generated IDs.
    for table in [
        "myTag_content",
        "myTag",
        "playlist_content",
        "playlist",
        "history_content",
        "history",
    ] {
        diesel::sql_query(format!("DELETE FROM {table}"))
            .execute(&mut conn)
            .unwrap();
    }
    OneFixture { conn, _file: file }
}

fn content_ids(conn: &mut SqliteConnection) -> Vec<i32> {
    // These tests exercise associations, so only track identity is needed.
    (1_000_000..1_000_006)
        .map(|id| {
            diesel::sql_query(format!("INSERT INTO content(content_id) VALUES ({id})"))
                .execute(conn)
                .unwrap();
            id
        })
        .collect()
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_history_ordering(#[case] version: usize) {
    use rbox::masterdb::NewDjmdHistory;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    ordering::tree_matrix::<rbox::masterdb::DjmdHistory>(
        &mut conn,
        "root",
        "missing-ordering-id",
        |conn, parent, seq| {
            let mut item = NewDjmdHistory::folder("ordering test");
            item.parent_id = Some(parent.to_owned());
            item.seq = seq;
            item.insert(conn)
        },
        "djmdHistory",
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_playlist_ordering(#[case] version: usize) {
    use rbox::masterdb::NewDjmdPlaylist;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    ordering::tree_matrix::<rbox::masterdb::DjmdPlaylist>(
        &mut conn,
        "root",
        "missing-ordering-id",
        |conn, parent, seq| {
            let mut item = NewDjmdPlaylist::folder("ordering test");
            item.parent_id = Some(parent.to_owned());
            item.seq = seq;
            item.insert(conn)
        },
        "djmdPlaylist",
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_my_tag_ordering(#[case] version: usize) {
    use rbox::masterdb::NewDjmdMyTag;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    ordering::tree_matrix::<rbox::masterdb::DjmdMyTag>(
        &mut conn,
        "root",
        "missing-ordering-id",
        |conn, parent, seq| {
            let mut item = NewDjmdMyTag::folder("ordering test");
            item.parent_id = Some(parent.to_owned());
            item.seq = seq;
            item.insert(conn)
        },
        "djmdMyTag",
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_related_tracks_ordering(#[case] version: usize) {
    use rbox::masterdb::NewDjmdRelatedTracks;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    ordering::tree_matrix::<rbox::masterdb::DjmdRelatedTracks>(
        &mut conn,
        "root",
        "missing-ordering-id",
        |conn, parent, seq| {
            let mut item = NewDjmdRelatedTracks::folder("ordering test");
            item.parent_id = Some(parent.to_owned());
            item.seq = seq;
            item.insert(conn)
        },
        "djmdRelatedTracks",
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_sampler_ordering(#[case] version: usize) {
    use rbox::masterdb::NewDjmdSampler;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    ordering::tree_matrix::<rbox::masterdb::DjmdSampler>(
        &mut conn,
        "root",
        "missing-ordering-id",
        |conn, parent, seq| {
            let mut item = NewDjmdSampler::folder("ordering test");
            item.parent_id = Some(parent.to_owned());
            item.seq = seq;
            item.insert(conn)
        },
        "djmdSampler",
    );
}

#[test]
fn history_ordering() {
    use rbox::one_library::NewHistory;
    let mut conn = one_connection();
    ordering::tree_matrix::<rbox::one_library::History>(
        &mut conn,
        &0,
        &i32::MAX,
        |conn, parent, seq| {
            let mut item = NewHistory::folder("ordering test");
            item.parent_id = *parent;
            item.seq = seq;
            item.insert(conn)
        },
        "history",
    );
}

#[test]
fn playlist_ordering() {
    use rbox::one_library::NewPlaylist;
    let mut conn = one_connection();
    ordering::tree_matrix::<rbox::one_library::Playlist>(
        &mut conn,
        &0,
        &i32::MAX,
        |conn, parent, seq| {
            let mut item = NewPlaylist::folder("ordering test");
            item.parent_id = *parent;
            item.seq = seq;
            item.insert(conn)
        },
        "playlist",
    );
}

#[test]
fn my_tag_ordering() {
    use rbox::one_library::NewMyTag;
    let mut conn = one_connection();
    ordering::tree_matrix::<rbox::one_library::MyTag>(
        &mut conn,
        &0,
        &i32::MAX,
        |conn, parent, seq| {
            let mut item = NewMyTag::folder("ordering test");
            item.parent_id = *parent;
            item.seq = seq;
            item.insert(conn)
        },
        "myTag",
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_song_history_ordering(#[case] version: usize) {
    use rbox::masterdb::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdHistory::new("ordering p", rbox::enums::HistoryType::try_from(0).unwrap())
        .insert(&mut conn)
        .unwrap();
    let q = NewDjmdHistory::new("ordering q", rbox::enums::HistoryType::try_from(0).unwrap())
        .insert(&mut conn)
        .unwrap();
    ordering::list_matrix::<DjmdSongHistory>(
        &mut conn,
        &p.id,
        &q.id,
        |conn, parent, index, seq| {
            let mut entry = NewDjmdSongHistory::new(parent.to_owned(), content_ids[index].clone());
            entry.track_no = seq;
            entry.insert(conn)
        },
        "djmdSongHistory",
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_song_playlist_ordering(#[case] version: usize) {
    use rbox::masterdb::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdPlaylist::new(
        "ordering p",
        rbox::enums::PlaylistType::try_from(0).unwrap(),
    )
    .insert(&mut conn)
    .unwrap();
    let q = NewDjmdPlaylist::new(
        "ordering q",
        rbox::enums::PlaylistType::try_from(0).unwrap(),
    )
    .insert(&mut conn)
    .unwrap();
    ordering::list_matrix::<DjmdSongPlaylist>(
        &mut conn,
        &p.id,
        &q.id,
        |conn, parent, index, seq| {
            let mut entry = NewDjmdSongPlaylist::new(parent, &content_ids[index]);
            entry.track_no = seq;
            entry.insert(conn)
        },
        "djmdSongPlaylist",
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_song_my_tag_ordering(#[case] version: usize) {
    use rbox::masterdb::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdMyTag::new("ordering p", rbox::enums::MyTagType::try_from(0).unwrap())
        .insert(&mut conn)
        .unwrap();
    let q = NewDjmdMyTag::new("ordering q", rbox::enums::MyTagType::try_from(0).unwrap())
        .insert(&mut conn)
        .unwrap();
    ordering::list_matrix::<DjmdSongMyTag>(
        &mut conn,
        &p.id,
        &q.id,
        |conn, parent, index, seq| {
            let mut entry = NewDjmdSongMyTag::new(parent, &content_ids[index]);
            entry.track_no = seq;
            entry.insert(conn)
        },
        "djmdSongMyTag",
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_song_related_tracks_ordering(#[case] version: usize) {
    use rbox::masterdb::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdRelatedTracks::new("ordering p", rbox::enums::RelatedTracksType::List)
        .insert(&mut conn)
        .unwrap();
    let q = NewDjmdRelatedTracks::new("ordering q", rbox::enums::RelatedTracksType::List)
        .insert(&mut conn)
        .unwrap();
    ordering::list_matrix::<DjmdSongRelatedTracks>(
        &mut conn,
        &p.id,
        &q.id,
        |conn, parent, index, seq| {
            let mut entry = NewDjmdSongRelatedTracks::new(parent, &content_ids[index]);
            entry.track_no = seq;
            entry.insert(conn)
        },
        "djmdSongRelatedTracks",
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_song_sampler_ordering(#[case] version: usize) {
    use rbox::masterdb::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn = rbox::masterdb::establish_connection(fixture.path().to_str().unwrap()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdSampler::new("ordering p", rbox::enums::SamplerType::List)
        .insert(&mut conn)
        .unwrap();
    let q = NewDjmdSampler::new("ordering q", rbox::enums::SamplerType::List)
        .insert(&mut conn)
        .unwrap();
    ordering::list_matrix::<DjmdSongSampler>(
        &mut conn,
        &p.id,
        &q.id,
        |conn, parent, index, seq| {
            let mut entry = NewDjmdSongSampler::new(parent, &content_ids[index]);
            entry.track_no = seq;
            entry.insert(conn)
        },
        "djmdSongSampler",
    );
}

#[test]
fn history_content_ordering() {
    use rbox::one_library::*;
    let mut conn = one_connection();
    let content_ids = content_ids(&mut conn);
    let p = NewHistory::new("ordering p", rbox::enums::HistoryType::try_from(0).unwrap())
        .insert(&mut conn)
        .unwrap();
    let q = NewHistory::new("ordering q", rbox::enums::HistoryType::try_from(0).unwrap())
        .insert(&mut conn)
        .unwrap();
    ordering::list_matrix::<HistoryContent>(
        &mut conn,
        &p.id,
        &q.id,
        |conn, parent, index, seq| {
            let mut entry = HistoryContent::new(*parent, content_ids[index]);
            entry.seq = seq.unwrap_or(-1);
            entry.insert(conn)
        },
        "history_content",
    );
}

#[test]
fn playlist_content_ordering() {
    use rbox::one_library::*;
    let mut conn = one_connection();
    let content_ids = content_ids(&mut conn);
    let p = NewPlaylist::new(
        "ordering p",
        rbox::enums::PlaylistType::try_from(0).unwrap(),
    )
    .insert(&mut conn)
    .unwrap();
    let q = NewPlaylist::new(
        "ordering q",
        rbox::enums::PlaylistType::try_from(0).unwrap(),
    )
    .insert(&mut conn)
    .unwrap();
    ordering::list_matrix::<PlaylistContent>(
        &mut conn,
        &p.id,
        &q.id,
        |conn, parent, index, seq| {
            let mut entry = PlaylistContent::new(*parent, content_ids[index]);
            entry.seq = seq.unwrap_or(-1);
            entry.insert(conn)
        },
        "playlist_content",
    );
}
