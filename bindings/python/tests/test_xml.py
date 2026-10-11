# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

import pytest

from rbox import RekordboxXml, XmlError
from rbox.rbxml import PlaylistNode, Track


def test_load_errors_are_xml_errors(tmp_path):
    with pytest.raises(XmlError):
        RekordboxXml(str(tmp_path / "missing.xml"))
    malformed = tmp_path / "malformed.xml"
    malformed.write_text("<invalid>", encoding="utf-8")
    with pytest.raises(XmlError):
        RekordboxXml(str(malformed))


def test_load_and_getters(fixture_copy):
    xml = RekordboxXml(fixture_copy("xml"))
    assert isinstance(xml, RekordboxXml)
    assert isinstance(xml.get_tracks(), list)
    track = xml.get_tracks()[0]
    assert isinstance(track, Track)
    assert isinstance(xml.get_track(0), Track)
    assert isinstance(xml.get_track_by_id(track.trackid), Track)
    assert xml.get_track_by_id("missing-id") is None
    assert isinstance(xml.root_playlist, PlaylistNode)
    assert isinstance(xml.to_string(), str)
    with pytest.raises(TypeError):
        xml.get_track([])


def test_mutation_and_write_signatures(fixture_copy, tmp_path):
    xml = RekordboxXml(fixture_copy("xml"))
    track = xml.new_track("api-track", "/api/track.mp3")
    assert isinstance(track, Track)
    track.name = "API title"
    # Independently constructed rows are also accepted by the update API.
    assert xml.update_track(Track("api-track", "/api/track.mp3")) is None
    assert xml.remove_track("api-track") is None
    assert xml.dump() is None
    assert xml.dump_copy(tmp_path / "copy.xml") is None


@pytest.mark.parametrize("source", ["index", "id", "list", "new"])
def test_update_returned_track(fixture_copy, source):
    xml = RekordboxXml(fixture_copy("xml"))
    if source == "index":
        track = xml.get_track(0)
    elif source == "id":
        track = xml.get_track_by_id(xml.get_track(0).trackid)
    elif source == "list":
        track = xml.get_tracks()[0]
    else:
        track = xml.new_track("api-track", "/api/track.mp3")
    track.name = "API updated title"
    assert xml.update_track(track) is None
    assert xml.get_track_by_id(track.trackid).name == "API updated title"


def test_update_unknown_track(fixture_copy):
    xml = RekordboxXml(fixture_copy("xml"))
    with pytest.raises(XmlError):
        xml.update_track(Track("missing-id", "/api/missing.mp3"))
