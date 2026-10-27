# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Database binding contracts, independent of database ordering and persistence logic."""

from pathlib import Path

import pytest

from rbox import MasterDb, MasterDbError, Track, TrackSource
from rbox.masterdb import models


def test_raw_getters(master_db):
    db = master_db
    assert isinstance(db, MasterDb)
    assert callable(db.add_content)
    rows = db.get_contents()
    assert isinstance(rows, list)
    row = rows[0]
    assert isinstance(row, models.DjmdContent)
    assert isinstance(row.id, str)
    assert isinstance(db.get_content_by_id(row.id), models.DjmdContent)
    assert db.get_content_by_id("missing-id") is None
    assert isinstance(db.get_albums(), list)
    assert db.get_album_by_id("missing-id") is None
    assert not hasattr(row, "artist")
    # Raw models expose mutable attributes and mapping access.
    assert row["id"] == row.id
    row.title = "API title"
    assert row.title == "API title"
    with pytest.raises(TypeError):
        db.get_content_by_id([])


def test_wide_file_sizes(master_db):
    row = master_db.get_contents()[0]
    for size in [2**40, 2**63 - 1, None]:
        row.file_size = size
        master_db.update_content(row)
        assert master_db.get_content_by_id(row.id).file_size == size
        assert master_db.get_track_by_id(row.id).file_size == size
    with pytest.raises(OverflowError):
        row.file_size = 2**63


def test_resolved_track_helpers(master_db):
    db = master_db
    row = db.get_contents()[0]
    row.artist_id = db.create_artist("Track helper artist").id
    row.bpm = 12825
    row.composer_id = None
    row.image_path = "/artwork/cover.jpg"
    row.analysis_data_path = "/PIONEER/USBANLZ/example/ANLZ0000.DAT"
    row.color_id = "3"
    row.dj_play_count = 123
    row.hot_cue_auto_load = "unknown-setting"
    row.delivery_control = "on"
    row.delivery_comment = "Delivery comment"
    db.update_content(row)
    track = db.get_track_by_id(row.id)
    assert isinstance(track, Track)
    assert track.id == int(row.id)
    assert track.artist == "Track helper artist"
    assert track.bpm == 128.25
    assert track.composer is None
    assert isinstance(track.path, Path)
    assert track.master_content_id == int(row.id)
    assert track.master_db_id == int(db.get_properties()[0].db_id)
    assert track.source == TrackSource.MasterDb
    assert track.artwork_path == Path(row.image_path)
    assert track.analysis_data_path == Path(row.analysis_data_path)
    assert track.color == 3
    assert track.play_count == 123
    assert track.hot_cue_auto_load == "unknown-setting"
    assert track.delivery_control == "on"
    assert track.delivery_comment == "Delivery comment"
    assert db.get_track_by_path(str(track.path)).id == int(row.id)
    assert all(isinstance(item, Track) for item in db.get_tracks())
    assert [item.id for item in db.get_tracks_by_ids([row.id, "missing", row.id])] == [int(row.id)]
    assert db.get_tracks_by_ids([]) == []
    assert db.get_track_by_id("missing") is None
    assert db.get_track_by_path("/missing/track.mp3") is None
    with pytest.raises(TypeError):
        db.get_tracks_by_ids([123])
    row.rating = 6
    db.update_content(row)
    with pytest.raises(MasterDbError, match="rating"):
        db.get_track_by_id(row.id)


def test_album_mutation_signatures(master_db):
    db = master_db
    row = db.create_album("API album", None, None, None)
    assert isinstance(row, models.DjmdAlbum)
    assert isinstance(db.get_album_by_name("API album"), models.DjmdAlbum)
    row.name = "API renamed album"
    assert isinstance(db.update_album(row), models.DjmdAlbum)
    assert db.delete_album(row.id) is None


def test_content_view(master_db):
    db = master_db
    row = db.get_contents()[0]
    artist = db.create_artist("API artist")
    row.artist_id = artist.id
    album_artist = db.create_artist("API album artist")
    row.album_id = db.create_album("API content album", album_artist.id, None, None).id
    assert db.update_content(row) is None
    view = db.get_content_with_relations(row.id)
    assert isinstance(view, models.DjmdContentView)
    assert isinstance(view.content, models.DjmdContent)
    assert isinstance(view.artist, models.DjmdArtist)
    assert view.album_artist.id == album_artist.id
    for field in ("remixer", "original_artist", "composer"):
        assert isinstance(getattr(view, field), (models.DjmdArtist, type(None)))
    for field, model in (
        ("album", models.DjmdAlbum),
        ("genre", models.DjmdGenre),
        ("label", models.DjmdLabel),
        ("key", models.DjmdKey),
        ("color", models.DjmdColor),
    ):
        assert isinstance(getattr(view, field), (model, type(None)))
    views = db.get_contents_with_relations()
    assert isinstance(views, list)
    assert isinstance(views[0], models.DjmdContentView)
    assert db.get_content_with_relations("missing-id") is None
    with pytest.raises(AttributeError):
        view.artist = None
    with pytest.raises(AttributeError):
        view.content = row
    # PyO3 row getters return copies, not references into the view.
    copy = view.content
    before = view.content.title
    copy.title = "API detached copy"
    assert view.content.title == before


def test_album_view(master_db):
    db = master_db
    artist = db.create_artist("API album artist")

    row = db.create_album("API album", artist.id, None, None)
    view = db.get_album_with_relations(row.id)
    assert isinstance(view, models.DjmdAlbumView)
    assert isinstance(view.album, models.DjmdAlbum)
    assert isinstance(view.artist, models.DjmdArtist)

    views = db.get_albums_with_relations()
    assert isinstance(views, list)
    assert isinstance(views[0], models.DjmdAlbumView)
    absent = db.create_album("API optional fields", None, None, None)
    assert db.get_album_with_relations(absent.id).artist is None

    assert db.get_album_with_relations("missing-id") is None
    with pytest.raises(AttributeError):
        view.album = row
