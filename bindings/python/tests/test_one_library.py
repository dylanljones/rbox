# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Database binding contracts, independent of database ordering and persistence logic."""

import pytest

from rbox import OneLibrary
from rbox.one_library import models


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
