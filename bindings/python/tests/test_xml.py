# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

import pytest

from rbox import RekordboxXml, XmlError
from rbox.rbxml import XmlPlaylistNode, XmlTrack


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
    assert isinstance(track, XmlTrack)
    assert isinstance(xml.get_track(0), XmlTrack)
    assert isinstance(xml.get_track_by_id(track.trackid), XmlTrack)
    assert xml.get_track_by_id("missing-id") is None
    assert isinstance(xml.root_playlist, XmlPlaylistNode)
    assert isinstance(xml.to_string(), str)
    with pytest.raises(TypeError):
        xml.get_track([])


def test_mutation_and_write_signatures(fixture_copy, tmp_path):
    xml = RekordboxXml(fixture_copy("xml"))
    track = xml.new_track("api-track", "/api/track.mp3")
    assert isinstance(track, XmlTrack)
    track.name = "API title"
    # Independently constructed rows are also accepted by the update API.
    assert xml.update_track(XmlTrack("api-track", "/api/track.mp3")) is None
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
        xml.update_track(XmlTrack("missing-id", "/api/missing.mp3"))


def test_xml_track_name_and_compatibility_alias():
    from rbox import XmlTrack as PublicXmlTrack
    from rbox.rbxml import Track as LegacyTrack

    assert PublicXmlTrack is XmlTrack
    assert LegacyTrack is XmlTrack
    track = XmlTrack("1", "file://localhost/C:/Music/track.mp3")
    assert type(track).__name__ == "XmlTrack"


@pytest.mark.parametrize("name", ["Tempo", "PositionMark", "PlaylistTrack", "PlaylistNode"])
def test_xml_model_names_and_compatibility_aliases(name):
    import rbox
    from rbox import _rbox, rbxml

    model = getattr(rbox, "Xml" + name)
    assert model.__name__ == "Xml" + name
    assert getattr(rbxml, name) is model
    assert getattr(_rbox, name) is model


def test_prefixed_xml_model_constructors():
    from rbox import XmlPlaylistNode, XmlPlaylistTrack, XmlPositionMark, XmlTempo

    assert XmlTempo(0.0, 128.0, "4/4", 1).bpm == 128.0
    assert XmlPositionMark("Cue", 0, -1, 1.5, None).start == 1.5
    assert XmlPlaylistTrack("42").key == "42"
    assert XmlPlaylistNode("Folder", 0).name == "Folder"


def test_file_information(fixture_copy, tmp_path):
    xml = RekordboxXml(fixture_copy("xml"))
    values = {
        "version": "1.0.1",
        "product_name": "Tools & <Music>",
        "product_version": "2.3.4",
        "product_company": "??",
    }
    for field, value in values.items():
        assert isinstance(getattr(xml, f"get_{field}")(), str)
        getattr(xml, f"set_{field}")(value)
        assert getattr(xml, f"get_{field}")() == value
    output = tmp_path / "metadata.xml"
    xml.dump_copy(output)
    loaded = RekordboxXml(output)
    for field, value in values.items():
        assert getattr(loaded, f"get_{field}")() == value
    with pytest.raises(TypeError):
        xml.set_product_name(None)
