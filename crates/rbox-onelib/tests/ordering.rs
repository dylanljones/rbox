// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Ordering integration tests for One Library.

mod support;
use rbox_db::test_support::ordering;
use support::{one_tree_updates, visible_handler_test, visible_matrix, ConnectionFixture};

use diesel::prelude::*;
use rbox_db::connection::ConnectionOptions;
use rbox_db::ordering::SequenceVisible;
use rbox_db::*;

#[test]
fn one_library_tree_snapshot_has_stable_sibling_order() {
    use rbox_onelib::models::*;
    use rbox_onelib::schema::playlist;
    let mut conn = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
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

#[test]
fn one_library_association_deletion_and_rollback() {
    use rbox_onelib::models::*;
    let mut conn = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    let content = conn.seed_content_ids();
    let playlists = (0..3)
        .map(|i| {
            NewPlaylist::playlist(format!("p{i}"))
                .insert(&mut conn)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let histories = (0..3)
        .map(|i| {
            NewHistory::new(format!("h{i}"), rbox_db::enums::HistoryType::List)
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
    use rbox_onelib::{enums::PlaylistType, models::*};
    let mut conn = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    one_tree_updates!(conn, Playlist, NewPlaylist, PlaylistType::List);
    one_tree_updates!(conn, History, NewHistory, rbox_db::enums::HistoryType::List);
    one_tree_updates!(conn, MyTag, NewMyTag, rbox_db::enums::MyTagType::List);
}

#[test]
fn high_level_one_library_positions_are_unambiguous() {
    let fixture = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    let db = rbox_onelib::database::OneLibrary::new(fixture.path()).unwrap();
    let p = db.create_playlist_folder("parent", None, None).unwrap();
    let a = db.create_playlist("a", Some(p.id), None).unwrap();
    let b = db.create_playlist("b", Some(p.id), Some(0)).unwrap();
    let c = db.create_playlist("c", Some(p.id), None).unwrap();
    let mut conn = rbox_onelib::connection::establish_connection(
        fixture.path(),
        &ConnectionOptions::default(),
    )
    .unwrap();
    ordering::assert_order::<rbox_onelib::models::Playlist>(&mut conn, &p.id, &[b.id, a.id, c.id]);
}

visible_matrix!(
    category_visibility_and_ordering,
    Category,
    NewCategory,
    category,
    ConnectionFixture::one_library(rbox_onelib::connection::establish_connection)
);
visible_matrix!(
    sort_visibility_and_ordering,
    Sort,
    NewSort,
    sort,
    ConnectionFixture::one_library(rbox_onelib::connection::establish_connection)
);

#[test]
fn history_ordering() {
    use rbox_onelib::models::NewHistory;
    let mut conn = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    ordering::tree_matrix::<rbox_onelib::models::History>(
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
    use rbox_onelib::models::NewPlaylist;
    let mut conn = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    ordering::tree_matrix::<rbox_onelib::models::Playlist>(
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
    use rbox_onelib::models::NewMyTag;
    let mut conn = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    ordering::tree_matrix::<rbox_onelib::models::MyTag>(
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

#[test]
fn history_content_ordering() {
    use rbox_onelib::models::*;
    let mut conn = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    let content_ids = conn.seed_content_ids();
    let p = NewHistory::new(
        "ordering p",
        rbox_db::enums::HistoryType::try_from(0).unwrap(),
    )
    .insert(&mut conn)
    .unwrap();
    let q = NewHistory::new(
        "ordering q",
        rbox_db::enums::HistoryType::try_from(0).unwrap(),
    )
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
    use rbox_onelib::{enums::PlaylistType, models::*};
    let mut conn = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    let content_ids = conn.seed_content_ids();
    let p = NewPlaylist::new("ordering p", PlaylistType::try_from(0).unwrap())
        .insert(&mut conn)
        .unwrap();
    let q = NewPlaylist::new("ordering q", PlaylistType::try_from(0).unwrap())
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

#[test]
fn handler_playlist_and_membership_moves() {
    use rbox_onelib::{database::OneLibrary, models::*};
    let mut fixture = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    let tracks = fixture.seed_content_ids();
    let db = OneLibrary::new(fixture.path()).unwrap();
    let p = db.create_playlist_folder("source", None, None).unwrap();
    let q = db.create_playlist_folder("target", None, None).unwrap();
    let a = db.create_playlist("a", Some(p.id), None).unwrap();
    let b = db.create_playlist("b", Some(p.id), None).unwrap();
    let c = db.create_playlist("c", Some(q.id), None).unwrap();
    db.move_playlist(a.id, Some(&q.id), Some(0)).unwrap();
    ordering::assert_order::<Playlist>(&mut fixture, &p.id, &[b.id]);
    ordering::assert_order::<Playlist>(&mut fixture, &q.id, &[a.id, c.id]);
    db.move_playlists(vec![&c.id, &b.id], q.id, Some(0))
        .unwrap();
    ordering::assert_order::<Playlist>(&mut fixture, &q.id, &[c.id, b.id, a.id]);
    assert!(db.move_playlists(vec![&a.id, &a.id], p.id, None).is_err());
    assert!(db.move_playlist(a.id, None, Some(-1)).is_err());
    assert_eq!(db.move_playlists(vec![], q.id, None).unwrap(), 0);
    for &id in &tracks[..3] {
        db.create_playlist_content(a.id, id, None).unwrap();
    }
    db.move_playlist_content(a.id, tracks[2], Some(0)).unwrap();
    let ids = [(a.id, tracks[0]), (a.id, tracks[1]), (a.id, tracks[2])];
    ordering::assert_order::<PlaylistContent>(&mut fixture, &a.id, &[ids[2], ids[0], ids[1]]);
    db.move_playlist_contents(vec![&ids[1], &ids[0]], Some(0))
        .unwrap();
    ordering::assert_order::<PlaylistContent>(&mut fixture, &a.id, &[ids[1], ids[0], ids[2]]);
    assert!(db
        .move_playlist_contents(vec![&ids[0], &ids[0]], None)
        .is_err());
    assert!(db.move_playlist_content(a.id, tracks[0], Some(-1)).is_err());
    assert_eq!(db.delete_playlist_content(a.id, tracks[0]).unwrap(), 1);
    ordering::assert_order::<PlaylistContent>(&mut fixture, &a.id, &[ids[1], ids[2]]);
    assert_eq!(
        db.delete_playlist_contents(vec![&ids[1], &ids[2]]).unwrap(),
        2
    );
    ordering::assert_order::<PlaylistContent>(&mut fixture, &a.id, &[]);
    assert_eq!(db.delete_playlist(a.id).unwrap(), 1);
    ordering::assert_order::<Playlist>(&mut fixture, &q.id, &[c.id, b.id]);
    assert_eq!(db.delete_playlists(vec![&c.id, &b.id]).unwrap(), 2);
    ordering::assert_order::<Playlist>(&mut fixture, &q.id, &[]);
}

#[test]
fn handler_my_tag_moves_and_deletions() {
    use rbox_onelib::{database::OneLibrary, models::MyTag};
    let mut fixture = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    let db = OneLibrary::new(fixture.path()).unwrap();
    let p = db.create_my_tag_folder("source", None, None).unwrap();
    let q = db.create_my_tag_folder("target", None, None).unwrap();
    let a = db.create_my_tag("a", Some(p.id), None).unwrap();
    let b = db.create_my_tag("b", Some(p.id), Some(0)).unwrap();
    db.move_my_tag(a.id, Some(&q.id), Some(0)).unwrap();
    ordering::assert_order::<MyTag>(&mut fixture, &p.id, &[b.id]);
    db.move_my_tags(vec![&b.id, &a.id], q.id, Some(0)).unwrap();
    ordering::assert_order::<MyTag>(&mut fixture, &q.id, &[b.id, a.id]);
    assert!(db.move_my_tag(a.id, None, Some(-1)).is_err());
    assert!(db.move_my_tags(vec![&a.id, &a.id], p.id, None).is_err());
    assert_eq!(db.move_my_tags(vec![], p.id, None).unwrap(), 0);
    assert_eq!(db.delete_my_tag(b.id).unwrap(), 1);
    ordering::assert_order::<MyTag>(&mut fixture, &q.id, &[a.id]);
    assert_eq!(db.delete_my_tags(vec![&a.id, &q.id]).unwrap(), 2);
    assert!(MyTag::find(&mut fixture, &a.id).unwrap().is_none());
}

visible_handler_test!(
    handler_category_visibility,
    Category,
    create_category,
    update_category,
    delete_category,
    ConnectionFixture::one_library(rbox_onelib::connection::establish_connection)
);
visible_handler_test!(
    handler_sort_visibility,
    Sort,
    create_sort,
    update_sort,
    delete_sort,
    ConnectionFixture::one_library(rbox_onelib::connection::establish_connection)
);

#[test]
fn handler_history_membership_compaction() {
    use rbox_onelib::{database::OneLibrary, models::*};
    let mut fixture = ConnectionFixture::one_library(rbox_onelib::connection::establish_connection);
    let tracks = fixture.seed_content_ids();
    let db = OneLibrary::new(fixture.path()).unwrap();
    let history = db
        .create_history("history", rbox_db::enums::HistoryType::List)
        .unwrap();
    for &id in &tracks[..3] {
        db.create_history_content(history.id, id).unwrap();
    }
    ordering::assert_order::<HistoryContent>(
        &mut fixture,
        &history.id,
        &tracks[..3]
            .iter()
            .map(|&id| (history.id, id))
            .collect::<Vec<_>>(),
    );
    assert_eq!(db.delete_history_content(history.id, tracks[1]).unwrap(), 1);
    ordering::assert_order::<HistoryContent>(
        &mut fixture,
        &history.id,
        &[(history.id, tracks[0]), (history.id, tracks[2])],
    );
    assert_eq!(db.delete_history(history.id).unwrap(), 1);
    assert!(HistoryContent::ordered(&mut fixture, &history.id)
        .unwrap()
        .is_empty());
}
