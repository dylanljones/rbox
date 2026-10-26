# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Public core-model constructors, properties, and conversions."""

from pathlib import Path

import pytest

from rbox.core import Track, TrackMetadata, TrackSource


@pytest.mark.parametrize("model", [TrackMetadata, Track])
def test_constructor_and_properties(model):
    row = model(Path("music/ÃƒÅ“ber den Wolken.mp3"))
    assert row.path == Path("music/ÃƒÅ“ber den Wolken.mp3")
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
    assert track.id is None
    assert track.uuid == ""
    track.id = 42
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


def test_track_export_metadata_and_source():
    track = Track("track.mp3")
    assert track.source == TrackSource.Unknown
    for field in (
        "master_content_id",
        "master_db_id",
        "artwork_path",
        "analysis_data_path",
        "color",
        "play_count",
        "hot_cue_auto_load",
        "delivery_control",
        "delivery_comment",
    ):
        assert getattr(track, field) is None
    track.id = 7
    track.master_content_id = 42
    track.master_db_id = 100
    track.source = TrackSource.DeviceLib
    track.artwork_path = "artwork/cover.jpg"
    track.analysis_data_path = Path("ANLZ/ANLZ0000.DAT")
    track.color = 3
    track.play_count = 123
    track.hot_cue_auto_load = "unknown-setting"
    track.delivery_control = "on"
    track.delivery_comment = "Comment"
    assert track.id != track.master_content_id
    assert track.master_db_id == 100
    assert track.source == TrackSource.DeviceLib
    assert track.artwork_path == Path("artwork/cover.jpg")
    assert track.analysis_data_path == Path("ANLZ/ANLZ0000.DAT")
    assert track.hot_cue_auto_load == "unknown-setting"
    for field, maximum in [
        ("color", 255),
        ("play_count", 2**32 - 1),
        ("id", 2**32 - 1),
        ("master_content_id", 2**32 - 1),
        ("master_db_id", 2**32 - 1),
    ]:
        for value in (-1, maximum + 1):
            with pytest.raises(OverflowError):
                setattr(track, field, value)
    rebuilt = Track.from_metadata(track.to_metadata())
    assert rebuilt.source == TrackSource.Unknown
    assert rebuilt.master_content_id is None
    assert rebuilt.artwork_path is None
