# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Shared-path binding exports, integer conversion, and return values."""

import pytest

from rbox.core import (
    anlz_dir_local,
    anlz_dir_usb,
    artwork_dir_local,
    artwork_dir_usb,
    image_path_local,
)


@pytest.mark.parametrize(
    "track_path,expected",
    [
        ("/Contents/Artist/Track.flac", "/PIONEER/USBANLZ/P02F/0000E9C7"),
        ("/Contents/🎵.mp3", "/PIONEER/USBANLZ/P072/0001722E"),
    ],
)
def test_anlz_dir_usb(track_path, expected):
    assert anlz_dir_usb(track_path=track_path) == expected


LOCAL_HELPERS = [
    (anlz_dir_local, "/PIONEER/USBANLZ/0e8/0d4cc-06a6-4574-ba18-1c2301ea9924"),
    (artwork_dir_local, "/PIONEER/Artwork/0e8/0d4cc-06a6-4574-ba18-1c2301ea9924"),
    (image_path_local, "/PIONEER/Artwork/0e8/0d4cc-06a6-4574-ba18-1c2301ea9924/artwork.jpg"),
]


@pytest.mark.parametrize("helper,expected", LOCAL_HELPERS)
@pytest.mark.parametrize(
    "uuid",
    [
        "0e80d4cc-06a6-4574-ba18-1c2301ea9924",
        "0E80D4CC-06A6-4574-BA18-1C2301EA9924",
        "0e80d4cc06a64574ba181c2301ea9924",
    ],
)
def test_local_paths(helper, expected, uuid):
    assert helper(uuid=uuid) == expected


@pytest.mark.parametrize("helper,expected", LOCAL_HELPERS)
@pytest.mark.parametrize("uuid", ["", "not-a-uuid", "0e80d4cc-06a6-4574-ba18-1c2301ea992g"])
def test_local_paths_invalid_uuid(helper, expected, uuid):
    with pytest.raises(ValueError, match="Invalid UUID"):
        helper(uuid)


@pytest.mark.parametrize("helper", [anlz_dir_usb, anlz_dir_local, artwork_dir_local, image_path_local])
@pytest.mark.parametrize("value", [None, 123, True])
def test_path_helper_wrong_type(helper, value):
    with pytest.raises(TypeError):
        helper(value)


@pytest.mark.parametrize(
    "artwork_id,folder",
    [
        (0, "00001"),
        (19, "00001"),
        (20, "00002"),
        (39, "00002"),
        (40, "00003"),
        (4294967295, "214748365"),
    ],
)
def test_artwork_dir_usb(artwork_id, folder):
    assert artwork_dir_usb(artwork_id=artwork_id) == f"/PIONEER/Artwork/{folder}"


@pytest.mark.parametrize("artwork_id", [-1, 4294967296])
def test_artwork_id_out_of_range(artwork_id):
    with pytest.raises(OverflowError):
        artwork_dir_usb(artwork_id)


@pytest.mark.parametrize("artwork_id", [1.5, "20", None])
def test_artwork_id_wrong_type(artwork_id):
    with pytest.raises(TypeError):
        artwork_dir_usb(artwork_id)
