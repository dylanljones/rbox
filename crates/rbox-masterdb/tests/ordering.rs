// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Ordering integration tests for the master database.

use diesel::prelude::*;
use rbox_db::connection::ConnectionOptions;
use rbox_db::*;
use rbox_masterdb::{establish_connection, MasterDb, MasterPlaylistXml};
use rbox_testing::data as common;
use rbox_testing::ordering;
use rstest::rstest;

#[rstest]
#[case(6)]
#[case(7)]
fn masterdb_tree_snapshot_and_public_delete(#[case] version: usize) {
    use rbox_masterdb::models::*;
    use rbox_masterdb::schema::djmdPlaylist;
    let fixture = common::setup_master_db_path(version);
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
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
    use rbox_masterdb::schema::djmdPlaylist;
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
    use rbox_masterdb::{enums::PlaylistType, models::*};
    let fixture = common::setup_master_db_path(version);
    let mut conn = establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    rbox_testing::master_tree_updates!(conn, DjmdPlaylist, NewDjmdPlaylist, PlaylistType::List);
    rbox_testing::master_tree_updates!(
        conn,
        DjmdHistory,
        NewDjmdHistory,
        rbox_db::enums::HistoryType::List
    );
    rbox_testing::master_tree_updates!(
        conn,
        DjmdMyTag,
        NewDjmdMyTag,
        rbox_db::enums::MyTagType::List
    );
    rbox_testing::master_tree_updates!(
        conn,
        DjmdRelatedTracks,
        NewDjmdRelatedTracks,
        rbox_masterdb::enums::RelatedTracksType::List
    );
    rbox_testing::master_tree_updates!(
        conn,
        DjmdSampler,
        NewDjmdSampler,
        rbox_masterdb::enums::SamplerType::List
    );
}

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_history_ordering(#[case] version: usize) {
    use rbox_masterdb::models::NewDjmdHistory;
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    ordering::tree_matrix::<rbox_masterdb::models::DjmdHistory>(
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
    use rbox_masterdb::models::NewDjmdPlaylist;
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    ordering::tree_matrix::<rbox_masterdb::models::DjmdPlaylist>(
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
    use rbox_masterdb::models::NewDjmdMyTag;
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    ordering::tree_matrix::<rbox_masterdb::models::DjmdMyTag>(
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
    use rbox_masterdb::models::NewDjmdRelatedTracks;
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    ordering::tree_matrix::<rbox_masterdb::models::DjmdRelatedTracks>(
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
    use rbox_masterdb::models::NewDjmdSampler;
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    ordering::tree_matrix::<rbox_masterdb::models::DjmdSampler>(
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

#[rstest]
#[case(6)]
#[case(7)]
fn djmd_song_history_ordering(#[case] version: usize) {
    use rbox_masterdb::models::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdHistory::new(
        "ordering p",
        rbox_db::enums::HistoryType::try_from(0).unwrap(),
    )
    .insert(&mut conn)
    .unwrap();
    let q = NewDjmdHistory::new(
        "ordering q",
        rbox_db::enums::HistoryType::try_from(0).unwrap(),
    )
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
    use rbox_masterdb::{enums::PlaylistType, models::*};
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdPlaylist::new("ordering p", PlaylistType::try_from(0).unwrap())
        .insert(&mut conn)
        .unwrap();
    let q = NewDjmdPlaylist::new("ordering q", PlaylistType::try_from(0).unwrap())
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
    use rbox_masterdb::models::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdMyTag::new(
        "ordering p",
        rbox_db::enums::MyTagType::try_from(0).unwrap(),
    )
    .insert(&mut conn)
    .unwrap();
    let q = NewDjmdMyTag::new(
        "ordering q",
        rbox_db::enums::MyTagType::try_from(0).unwrap(),
    )
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
    use rbox_masterdb::models::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdRelatedTracks::new("ordering p", rbox_masterdb::enums::RelatedTracksType::List)
        .insert(&mut conn)
        .unwrap();
    let q = NewDjmdRelatedTracks::new("ordering q", rbox_masterdb::enums::RelatedTracksType::List)
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
    use rbox_masterdb::models::*;
    let fixture = common::setup_master_db_path(version);
    let mut conn =
        rbox_masterdb::establish_connection(fixture.path(), &ConnectionOptions::default()).unwrap();
    let content_ids: Vec<_> = DjmdContent::all(&mut conn)
        .unwrap()
        .into_iter()
        .take(6)
        .map(|r| r.id)
        .collect();
    assert_eq!(content_ids.len(), 6);
    let p = NewDjmdSampler::new("ordering p", rbox_masterdb::enums::SamplerType::List)
        .insert(&mut conn)
        .unwrap();
    let q = NewDjmdSampler::new("ordering q", rbox_masterdb::enums::SamplerType::List)
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

#[rstest]
#[case(6)]
#[case(7)]
fn handler_batch_moves_preserve_order_and_xml(#[case] version: usize) {
    use rbox_masterdb::models::*;
    let fixture = common::setup_master_db_dir_path(version);
    let mut db = MasterDb::new(fixture.path().join("rekordbox/master.db")).unwrap();
    db.set_unsafe_writes(true);
    let p = db
        .create_playlist_folder("source".into(), None, None)
        .unwrap();
    let q = db
        .create_playlist_folder("target".into(), None, None)
        .unwrap();
    let a = db
        .create_playlist("a".into(), Some(p.id.clone()), None, None, None)
        .unwrap();
    let b = db
        .create_playlist("b".into(), Some(p.id.clone()), None, None, None)
        .unwrap();
    let c = db
        .create_playlist("c".into(), Some(q.id.clone()), None, None, None)
        .unwrap();
    db.move_playlists(vec![&b.id, &a.id], &q.id, Some(1))
        .unwrap();
    let mut conn = db.pool.get().unwrap();
    ordering::assert_order::<DjmdPlaylist>(&mut conn, &p.id, &[]);
    ordering::assert_order::<DjmdPlaylist>(
        &mut conn,
        &q.id,
        &[b.id.clone(), a.id.clone(), c.id.clone()],
    );
    let xml = MasterPlaylistXml::load(db.playlist_xml_path().unwrap());
    for id in [&a.id, &b.id, &c.id] {
        let row = DjmdPlaylist::find(&mut conn, id).unwrap().unwrap();
        let node = xml.get_playlist(id.clone()).unwrap();
        assert_eq!(node.parent_id, row.parent_id);
        assert!(
            (node.timestamp - row.updated_at.naive_utc())
                .num_seconds()
                .abs()
                <= 1
        );
    }
    let before = DjmdPlaylist::ordered(&mut conn, &q.id).unwrap();
    assert!(db.move_playlists(vec![&a.id, &a.id], &p.id, None).is_err());
    assert!(db
        .move_playlists(vec![&a.id, "missing"], &p.id, None)
        .is_err());
    assert!(db.move_playlists(vec![&a.id], &q.id, Some(0)).is_err());
    assert_eq!(DjmdPlaylist::ordered(&mut conn, &q.id).unwrap(), before);
    assert!(db.move_playlists(vec![], &q.id, None).unwrap().is_empty());

    let content = DjmdContent::all(&mut conn).unwrap();
    let s1 = db
        .create_playlist_song(&a.id, &content[0].id, None)
        .unwrap();
    let s2 = db
        .create_playlist_song(&a.id, &content[1].id, Some(1))
        .unwrap();
    let s3 = db
        .create_playlist_song(&a.id, &content[2].id, None)
        .unwrap();
    db.move_playlist_song(&s1.id, 1).unwrap();
    ordering::assert_order::<DjmdSongPlaylist>(
        &mut conn,
        &a.id,
        &[s1.id.clone(), s2.id.clone(), s3.id.clone()],
    );
    db.move_playlist_songs(vec![&s3.id, &s1.id], 1).unwrap();
    ordering::assert_order::<DjmdSongPlaylist>(
        &mut conn,
        &a.id,
        &[s3.id.clone(), s1.id.clone(), s2.id.clone()],
    );
    assert!(db.move_playlist_songs(vec![&s1.id, &s1.id], 1).is_err());
    assert!(db.move_playlist_song(&s1.id, 0).is_err());
    assert_eq!(db.delete_playlist_song(&s1.id).unwrap(), 1);
    ordering::assert_order::<DjmdSongPlaylist>(&mut conn, &a.id, &[s3.id.clone(), s2.id.clone()]);
    assert_eq!(db.delete_playlist_songs(vec![&s3.id, &s2.id]).unwrap(), 2);
    ordering::assert_order::<DjmdSongPlaylist>(&mut conn, &a.id, &[]);
    assert_eq!(db.delete_playlist_songs(vec![]).unwrap(), 0);
}
