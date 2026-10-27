# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Database binding contracts, independent of database ordering and persistence logic."""

import pytest

from rbox import OneLibrary, OneLibraryError
from rbox.enums import PlaylistType
from rbox.one_library import models


def test_directory_and_explicit_file_constructors(fixture_copy):
    path = fixture_copy("one_library")
    directory_db = OneLibrary(str(path.parent))
    explicit_db = OneLibrary.from_path(str(path))
    assert isinstance(explicit_db, OneLibrary)
    assert len(directory_db.get_contents()) == len(explicit_db.get_contents())
    with pytest.raises(OneLibraryError):
        OneLibrary(str(path))
    with pytest.raises(OneLibraryError):
        OneLibrary.from_path(str(path.parent))


def test_missing_database_constructors(tmp_path):
    with pytest.raises(OneLibraryError):
        OneLibrary(str(tmp_path))
    with pytest.raises(OneLibraryError):
        OneLibrary.from_path(str(tmp_path / "missing.db"))
    assert not list(tmp_path.iterdir())


def test_playlist_enum_registration(one_library):
    playlist = one_library.create_playlist("enum API", None, None)
    assert isinstance(playlist.attribute, PlaylistType)
    assert playlist.attribute == PlaylistType.List


def test_raw_getters(one_library):
    db = one_library
    assert isinstance(db, OneLibrary)
    rows = db.get_contents()
    assert isinstance(rows, list)
    row = rows[0]
    assert isinstance(row, models.Content)
    assert isinstance(row.id, int)
    assert isinstance(db.get_content_by_id(row.id), models.Content)
    assert db.get_content_by_id(2147483647) is None
    assert isinstance(db.get_albums(), list)
    assert db.get_album_by_id(2147483647) is None
    assert not hasattr(row, "artist")
    # Raw models expose mutable attributes and mapping access.
    assert row["id"] == row.id
    row.title = "API title"
    assert row.title == "API title"
    with pytest.raises(TypeError):
        db.get_content_by_id([])


def test_wide_file_sizes(one_library):
    db = one_library
    row = db.get_contents()[0]
    for size in [2**40, 2**63 - 1, None]:
        row.file_size = size
        assert db.update_content(row).file_size == size
        assert db.get_content_by_id(row.id).file_size == size
        assert db.get_track_by_id(row.id).file_size == size
    with pytest.raises(OverflowError):
        row.file_size = 2**63


def test_content_lookup_by_master_identity(one_library):
    db = one_library
    row = db.get_contents()[0]
    row.master_content_id = 2**40
    row.master_db_id = -795_645_379
    db.update_content(row)
    found = db.get_content_by_master_content_id(row.master_content_id, row.master_db_id)
    assert isinstance(found, models.Content)
    assert found.id == row.id
    assert found.master_db_id == -795_645_379
    assert db.get_content_by_master_content_id(row.master_content_id, 42) is None
    assert db.get_content_by_master_content_id(0, 0) is None
    batch = db.get_contents_by_master_content_ids(
        [row.master_content_id, 2_000_000_125, row.master_content_id, 0], row.master_db_id
    )
    assert [item.id for item in batch] == [row.id]
    assert isinstance(batch[0], models.Content)
    assert db.get_contents_by_master_content_ids([], row.master_db_id) == []
    assert db.get_contents_by_master_content_ids([row.master_content_id], 42) == []
    with pytest.raises(TypeError):
        db.get_contents_by_master_content_ids(["invalid"], row.master_db_id)
    assert db.get_contents_by_master_content_ids([row.master_content_id], 3_499_321_917) == []
    with pytest.raises(OverflowError):
        db.get_contents_by_master_content_ids([2**63], row.master_db_id)
    with pytest.raises(TypeError):
        db.get_content_by_master_content_id([], row.master_db_id)
    assert db.get_content_by_master_content_id(row.master_content_id, 3_499_321_917) is None
    with pytest.raises(OverflowError):
        db.get_content_by_master_content_id(row.master_content_id, 2**63)


def test_large_tag_ids_and_optional_references(one_library):
    db = one_library
    native = db.get_my_tags()
    assert any(tag.id > 2**31 - 1 for tag in native)
    for tag in native:
        assert db.get_my_tag_by_id(tag.id).id == tag.id
    folder = db.create_my_tag_folder("wide parent", None, None)
    assert folder.id > max(tag.id for tag in native)
    child = db.create_my_tag("wide child", folder.id, None)
    assert child.parent_id == folder.id
    assert child["parent_id"] == folder.id
    db.rename_my_tag(child.id, "renamed")
    assert db.get_my_tag_by_id(child.id).name == "renamed"
    db.move_my_tags([child.id], folder.id, 0)
    assert db.get_my_tag_children(folder.id)[0].id == child.id
    db.delete_my_tags([folder.id])
    assert db.get_my_tag_by_id(child.id) is None

    row = db.get_contents()[0]
    row.content_link = 2**40
    row.master_db_id = 2**40 + 1
    db.update_content(row)
    found = db.get_content_by_id(row.id)
    assert found.content_link == row.content_link
    assert found.master_db_id == row.master_db_id


def test_create_with_large_property_id(tmp_path):
    path = tmp_path / "wide.db"
    path.touch()
    db = OneLibrary.create(str(path), 2**40)
    assert db.get_properties()[0].my_tag_master_dbid == 2**40


def test_album_mutation_signatures(one_library):
    db = one_library
    row = db.create_album("API album", None, None)
    assert isinstance(row, models.Album)
    assert isinstance(db.get_album_by_name("API album"), models.Album)
    row.name = "API renamed album"
    assert isinstance(db.update_album(row), models.Album)
    assert isinstance(db.delete_album(row.id), int)


def test_content_view(one_library):
    db = one_library
    row = db.get_contents()[0]
    artist = db.create_artist("API artist")
    row.artist_id = artist.id
    assert isinstance(db.update_content(row), models.Content)
    view = db.get_content_with_relations(row.id)
    assert isinstance(view, models.ContentView)
    assert isinstance(view.content, models.Content)
    assert isinstance(view.artist, models.Artist)
    for field in ("remixer", "original_artist", "composer", "lyricist"):
        assert isinstance(getattr(view, field), (models.Artist, type(None)))
    for field, model in (
        ("album", models.Album),
        ("genre", models.Genre),
        ("label", models.Label),
        ("key", models.Key),
        ("color", models.Color),
        ("image", models.Image),
    ):
        assert isinstance(getattr(view, field), (model, type(None)))
    views = db.get_contents_with_relations()
    assert isinstance(views, list)
    assert isinstance(views[0], models.ContentView)
    assert db.get_content_with_relations(2147483647) is None
    with pytest.raises(AttributeError):
        view.artist = None
    with pytest.raises(AttributeError):
        view.content = row
    # PyO3 row getters return copies, not references into the view.
    copy = view.content
    before = view.content.title
    copy.title = "API detached copy"
    assert view.content.title == before


def test_album_view(one_library):
    db = one_library
    artist = db.create_artist("API album artist")
    image = db.create_image("api-artwork.jpg")
    row = db.create_album("API album", artist.id, image.id)
    view = db.get_album_with_relations(row.id)
    assert isinstance(view, models.AlbumView)
    assert isinstance(view.album, models.Album)
    assert isinstance(view.artist, models.Artist)
    assert isinstance(view.image, models.Image)
    views = db.get_albums_with_relations()
    assert isinstance(views, list)
    assert isinstance(views[0], models.AlbumView)
    absent = db.create_album("API optional fields", None, None)
    assert db.get_album_with_relations(absent.id).artist is None
    assert db.get_album_with_relations(absent.id).image is None
    assert db.get_album_with_relations(2147483647) is None
    with pytest.raises(AttributeError):
        view.album = row


def test_category_view(one_library):
    db = one_library
    item = db.create_menu_item("API menu", 0)
    row = db.create_category(item.id, None, None)
    view = db.get_category_with_relations(row.id)
    assert isinstance(view, models.CategoryView)
    assert isinstance(view.category, models.Category)
    assert isinstance(view.menu_item, models.MenuItem)
    views = db.get_categories_with_relations()
    assert isinstance(views, list)
    assert isinstance(views[0], models.CategoryView)
    assert db.get_category_with_relations(2147483647) is None
    with pytest.raises(AttributeError):
        view.menu_item = None


def test_shared_track_getters_and_album_artist(one_library):
    from rbox.core import Track, TrackSource

    db = one_library
    row = db.get_contents()[0]
    artist = db.create_artist("Shared track album artist")
    album = db.create_album("Shared track album", artist.id, None)
    row.album_id = album.id
    row.master_db_id = -795645379
    db.update_content(row)
    view = db.get_content_with_relations(row.id)
    assert view.album_artist.id == artist.id
    track = db.get_track_by_id(row.id)
    assert isinstance(track, Track)
    assert track.source == TrackSource.OneLib
    assert track.id == row.id
    assert track.master_db_id == 3499321917
    assert track.master_content_id == row.master_content_id
    assert track.album_artist == artist.name
    assert db.get_track_by_path(row.path.replace("/", "\\")).id == row.id
    assert any(item.id == row.id for item in db.get_tracks())
    assert [item.id for item in db.get_tracks_by_ids([row.id, row.id, 2147483647])] == [row.id]
    assert db.get_tracks_by_ids([]) == []
    assert db.get_track_by_id(2147483647) is None
    assert db.get_track_by_path("/missing-track.wav") is None
    with pytest.raises(TypeError):
        db.get_track_by_id("not an integer")
    row.rating = 6
    db.update_content(row)
    with pytest.raises(OneLibraryError):
        db.get_track_by_id(row.id)


def test_wide_metadata(one_library):
    db = one_library
    row = db.get_contents()[0]
    fields = ["bitrate", "sampling_rate", "track_no", "bpmx100"]
    wide = 2**32 - 1
    for field in fields:
        setattr(row, field, wide)
    db.update_content(row)
    stored = db.get_content_by_id(row.id)
    assert all(getattr(stored, field) == wide for field in fields)
    track = db.get_track_by_id(row.id)
    assert track.bit_rate == wide
    assert track.sample_rate == wide
    assert track.track_no == wide
    assert track.bpm == wide / 100
    for value in [2**63 - 1, None]:
        for field in fields:
            setattr(row, field, value)
        db.update_content(row)
        stored = db.get_content_by_id(row.id)
        assert all(getattr(stored, field) == value for field in fields)
    for field in fields:
        with pytest.raises(OverflowError):
            setattr(row, field, 2**63)
