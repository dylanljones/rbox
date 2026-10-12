# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Shared fixture copies for binding API tests; Rust tests cover data semantics."""

import shutil
from pathlib import Path

import pytest

from rbox import MasterDb, OneLibrary

TESTDATA = Path(__file__).resolve().parents[3] / ".testdata"
DEMO = TESTDATA / "RBv6" / "demo"
ANLZ = DEMO / "rekordbox/share/PIONEER/USBANLZ/e35/fa187-3f34-47e2-9880-2b33cb8d1304"
FIXTURES = {
    "masterdb": DEMO / "rekordbox/master.db",
    "one_library": TESTDATA / "RBv7/demo/export/PIONEER/rekordbox/exportLibrary.db",
    "xml": DEMO / "database.xml",
    **{extension: ANLZ / f"ANLZ0000.{extension}" for extension in ("DAT", "EXT", "2EX")},
}


@pytest.fixture
def fixture_copy(tmp_path):
    """Copy a fixture before opening it, including readers that create WAL files."""

    def copy(name, version=6):
        source = FIXTURES[name]
        if version != 6:
            assert version == 7 and name in ("DAT", "EXT", "2EX")
            source = TESTDATA / "RBv7/demo/export/PIONEER/USBANLZ/P016/0000875E" / source.name
        destination = tmp_path / source.name
        shutil.copyfile(source, destination)
        return destination

    return copy


@pytest.fixture
def master_db(fixture_copy):
    return MasterDb(str(fixture_copy("masterdb")))


@pytest.fixture
def one_library(fixture_copy):
    return OneLibrary(str(fixture_copy("one_library")))
