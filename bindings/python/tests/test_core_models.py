# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Public core-model constructors, properties, and conversions."""

from pathlib import Path

import pytest

from rbox.core import Track, TrackMetadata


@pytest.mark.parametrize("model", [TrackMetadata, Track])
def test_constructor_and_properties(model):
    row = model(Path("music/Über den Wolken.mp3"))
    assert row.path == Path("music/Über den Wolken.mp3")
    assert row.title is None
    assert row.bpm is None
    row.path = "music/updated.wav"
    row.title = "Track title"
    row.bpm = 128.5
    row.length = 123.25
    row.file_type = 999  # Raw codes retain unknown formats.
    assert row.path == Path("music/updated.wav")
    assert row.title == "Track title"
    assert row.bpm == 128.5
    assert row.length == 123.25
    assert row.file_type == 999
    row.title = None
    assert row.title is None
    with pytest.raises(TypeError):
        row.rating = "invalid"
    with pytest.raises(TypeError):
        model(None)


def test_metadata_conversion_copies_values():
    metadata = TrackMetadata("music/track.mp3")
    metadata.album_artist = "Artist"
    metadata.bpm = 127.123456789
    metadata.file_size = 123456
    track = Track.from_metadata(metadata)
    assert track.id == track.uuid == ""
    track.id = "42"
    track.uuid = "track-uuid"
    track.title = "Edited"
    assert metadata.title is None
    copied = track.to_metadata()
    assert isinstance(copied, TrackMetadata)
    assert copied.album_artist == "Artist"
    assert copied.bpm == 127.123456789
    assert copied.file_size == 123456
    assert copied.title == "Edited"
    assert not hasattr(copied, "id")
    copied.title = None
    assert track.title == "Edited"


def test_unsigned_fields_reject_negative_and_overflow():
    metadata = TrackMetadata("track.wav")
    metadata.file_size = 2**64 - 1
    assert metadata.file_size == 2**64 - 1
    for field, maximum in [
        ("rating", 255),
        ("bit_depth", 65535),
        ("release_year", 65535),
        ("track_no", 2**32 - 1),
        ("disc_no", 2**32 - 1),
        ("sample_rate", 2**32 - 1),
        ("bit_rate", 2**32 - 1),
        ("file_size", 2**64 - 1),
    ]:
        for value in (-1, maximum + 1):
            with pytest.raises(OverflowError):
                setattr(metadata, field, value)


def test_track_and_xml_track_are_distinct():
    from rbox.rbxml import Track as LegacyXmlTrack, XmlTrack

    assert Track.__name__ == "Track"
    assert Track is not XmlTrack
    assert LegacyXmlTrack is XmlTrack
