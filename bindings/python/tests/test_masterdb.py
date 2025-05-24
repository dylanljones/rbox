# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-05-07

import shutil
from pathlib import Path

import pytest

from rbox import MasterDb
from rbox.masterdb import models

DEMO_DIR = Path(__file__).parent.parent.parent.parent / ".testdata" / "RBv6" / "demo" / "rekordbox"
DEMO_DB_SRC = DEMO_DIR / "master.db"
DEMO_DB = DEMO_DIR / "master-copy.db"


@pytest.fixture
def db():
    """Return a clean Rekordbox database instance."""
    shutil.copy(DEMO_DB_SRC, DEMO_DB)
    db = MasterDb(DEMO_DB)
    yield db


def test_open_master_db(db):
    pass


def test_get_local_usn(db):
    usn = db.get_local_usn()
    assert usn > 0


# -- Album ----------------------------------------------------------------------------------------


def test_get_albums(db):
    _ = db.get_album()


def test_get_album(db):
    item = db.get_album_by_id("1234")
    assert item is None


def test_insert_album(db):
    item = db.insert_album("Name")
    assert isinstance(item, models.DjmdAlbum)

    id_ = item.ID
    album = db.get_album_by_id(id_)
    assert album is not None


def test_get_album_by_name(db):
    item = db.get_album_by_name("Name")
    assert item is None

    db.insert_album("Name")

    item = db.get_album_by_name("Name")
    assert item is not None


def test_update_album(db):
    item = db.insert_album("Name")
    assert isinstance(item, models.DjmdAlbum)

    id_ = item.ID
    new_item = db.get_album_by_id(id_)
    assert new_item is not None

    # Update the album name
    new_name = "New Name"
    new_item.Name = new_name
    db.update_album(new_item)

    # Verify the update
    updated = db.get_album_by_id(id_)
    assert updated is not None
    assert updated.Name == new_name
