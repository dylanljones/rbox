# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-05-10

from pathlib import Path

import pytest

from rbox import Anlz

DEMO_DIR = Path(__file__).parent.parent.parent.parent / ".testdata" / "RBv6" / "demo" / "rekordbox"

ANLZ_ROOT = DEMO_DIR / "share" / "PIONEER" / "USBANLZ" / "e35" / "fa187-3f34-47e2-9880-2b33cb8d1304"
ANLZ_DAT = ANLZ_ROOT / "ANLZ0000.DAT"
ANLZ_EXT = ANLZ_ROOT / "ANLZ0000.EXT"
ANLZ_2EX = ANLZ_ROOT / "ANLZ0000.2EX"
ANLZ_FILES = [ANLZ_DAT, ANLZ_EXT, ANLZ_2EX]


@pytest.mark.parametrize("path", ANLZ_FILES)
def test_parse(path):
    Anlz(path)


@pytest.mark.parametrize("path", ANLZ_FILES)
def test_get_path(path):
    anlz = Anlz(path)
    _ = anlz.get_path()


@pytest.mark.parametrize("path", ANLZ_FILES)
def test_set_path(path):
    new = "/Some/Path"
    anlz = Anlz(path)
    anlz.set_path(new)
