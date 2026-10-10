# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

import pytest

from rbox import Anlz, AnlzError
from rbox.anlz import BeatGrid


@pytest.mark.parametrize("extension", ["DAT", "EXT", "2EX"])
def test_load_and_getters(fixture_copy, extension):
    anlz = Anlz(fixture_copy(extension))
    assert isinstance(anlz, Anlz)
    tags = anlz.get_tags()
    assert isinstance(tags, list)
    assert all(isinstance(tag, str) for tag in tags)
    assert isinstance(anlz.get_path(), (str, type(None)))
    assert isinstance(anlz.get_beat_grid(), (BeatGrid, type(None)))


def test_setters_and_writers(fixture_copy, tmp_path):
    anlz = Anlz(fixture_copy("DAT"))
    assert anlz.set_path("/api/track.mp3") is None
    assert anlz.get_path() == "/api/track.mp3"
    assert anlz.dump() is None
    assert anlz.dump_copy(tmp_path / "copy.DAT") is None


def test_native_error(tmp_path):
    with pytest.raises(FileNotFoundError):
        Anlz(tmp_path / "missing.DAT")
    invalid = tmp_path / "invalid.DAT"
    invalid.write_bytes(b"invalid ANLZ")
    with pytest.raises(AnlzError):
        Anlz(invalid)
