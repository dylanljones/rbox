# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

import pytest

from rbox import Anlz, AnlzError
from rbox.anlz import (
    VBR,
    Beat,
    BeatGrid,
    Phrase,
    SeekIndex,
    SeekPoint,
    SongStructure,
    SongStructureData,
    UnknownSection,
    WaveformColumn,
    WaveformPreview,
)


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


def test_extended_grid_fields(fixture_copy):
    dat = Anlz(fixture_copy("DAT"))
    ext = Anlz(fixture_copy("EXT"))
    beats = dat.get_beat_grid().beats
    grid = ext.get_extended_beat_grid()
    assert grid is not None
    assert grid.first_beat.time == beats[0].time
    assert grid.last_beat.time == beats[-1].time
    assert grid.checksum == sum(b.beat_number + b.tempo + b.time for b in beats) % 2**32
    assert grid.offset_ms == 0
    assert grid.offset_flags == 0
    assert len(grid.beats) == len(beats)
    assert all(0 <= b.microseconds <= 999 for b in grid.beats)


def test_native_error(tmp_path):
    with pytest.raises(FileNotFoundError):
        Anlz(tmp_path / "missing.DAT")
    invalid = tmp_path / "invalid.DAT"
    invalid.write_bytes(b"invalid ANLZ")
    with pytest.raises(AnlzError):
        Anlz(invalid)


@pytest.mark.parametrize("extension", ["DAT", "EXT", "2EX"])
@pytest.mark.parametrize("version", [6, 7])
def test_editable_section_snapshots_round_trip(fixture_copy, tmp_path, extension, version):
    source = fixture_copy(extension, version)
    original = source.read_bytes()
    anlz = Anlz(source)
    data = anlz.get_data()
    assert len(data["sections"]) == anlz.section_count()
    for index, section in enumerate(data["sections"]):
        assert anlz.contains(section["tag"])
        assert anlz.get_section(index)["tag"] == section["tag"]
        assert anlz.get_sections_by_tag(section["tag"])
        anlz.set_section_at(index, section)
    output = tmp_path / ("snapshot." + extension)
    anlz.dump_copy(output)
    assert output.read_bytes() == original
    assert source.read_bytes() == original


@pytest.mark.parametrize("extension", ["dat", "ext", "2ex"])
def test_create_set_insert_and_remove(tmp_path, extension):
    path = tmp_path / ("created." + extension.upper())
    anlz = getattr(Anlz, "new_" + extension)(path)
    assert not path.exists()
    assert anlz.section_count() == 0
    assert anlz.get_path() is None
    assert anlz.get_section(0) is None
    anlz.set_path("/music/test.aiff")
    grid = BeatGrid(beats=[Beat(1, 12000, 100)], header_flags=0x80000)
    anlz.set_beat_grid(grid)
    grid.beats[0].time = 999
    assert anlz.get_beat_grid().beats[0].time == 100
    anlz.set_beat_grid(grid)
    assert anlz.get_beat_grid().beats[0].time == 999
    grid.beats.append(Beat(2, 12000, 1499))
    assert len(anlz.get_beat_grid().beats) == 1
    anlz.set_beat_grid(grid)
    assert len(anlz.get_beat_grid().beats) == 2
    grid.beats.pop()
    anlz.set_beat_grid(grid)
    assert len(anlz.get_beat_grid().beats) == 1
    unknown = {"tag": "Unknown", "content": UnknownSection(list(b"ZZZZ"), [1, 2], [3, 4])}
    anlz.push_section(unknown)
    anlz.insert_section(1, unknown)
    assert len(anlz.get_sections_by_tag("ZZZZ")) == 2
    assert anlz.remove_section_by_tag("ZZZZ")
    assert anlz.remove_sections_by_tag("ZZZZ") == 1
    assert anlz.remove_sections_by_tag("ZZZZ") == 0
    anlz.push_section(unknown)
    anlz.dump()
    reloaded = Anlz(path)
    assert reloaded.get_section_by_tag("ZZZZ")["content"].header_data == bytes([1, 2])
    assert reloaded.get_section_by_tag("ZZZZ")["content"].content_data == bytes([3, 4])
    assert anlz.remove_beat_grid()
    assert not anlz.remove_beat_grid()
    assert anlz.remove_path()
    anlz.remove_section(0)
    assert anlz.section_count() == 0


def test_invalid_edits_are_atomic(tmp_path):
    anlz = Anlz.new_dat(tmp_path / "invalid.DAT")
    anlz.set_waveform_preview(WaveformPreview([WaveformColumn(1, 0)]))
    for action in (
        lambda: anlz.set_waveform_preview(WaveformPreview([WaveformColumn(32, 0)])),
        lambda: anlz.set_section({"tag": "PQTZ", "content": WaveformPreview([])}),
        lambda: anlz.set_section({"tag": "Unknown", "content": UnknownSection(list(b"PQTZ"), [], [])}),
        lambda: anlz.remove_section(99),
        lambda: anlz.contains("not-a-tag"),
    ):
        with pytest.raises((AnlzError, ValueError, TypeError)):
            action()
        assert anlz.section_count() == 1
        assert anlz.get_waveform_preview().data[0].height == 1


def test_precise_seek_values_and_song_structure(tmp_path):
    anlz = Anlz.new_ext(tmp_path / "metadata.EXT")
    sample = 2**60 + 1
    anlz.set_seek_index(SeekIndex(0, sample, [SeekPoint(sample, 2**64 - 1, 1024)]))
    assert anlz.get_seek_index().entries[0].sample == sample
    assert anlz.get_seek_index().entries[0].offset == 2**64 - 1
    with pytest.raises(ValueError):
        anlz.set_vbr(VBR(0, [0] * 399, 0))
    assert anlz.get_vbr() is None
    vbr = VBR(0, [0] * 400, 0)
    vbr.index[0] = 123
    anlz.set_vbr(vbr)
    assert anlz.get_vbr().index[0] == 123
    assert anlz.remove_vbr()
    phrase = Phrase(
        index=1,
        beat=1,
        kind=1,
        k1=0,
        k2=0,
        b=0,
        beat2=0,
        beat3=0,
        beat4=0,
        k3=0,
        fill=0,
        beat_fill=0,
        u1=7,
        u2=8,
        u3=9,
        u4=10,
        u5=11,
    )
    data = SongStructureData(2, 8, 0, [phrase], 12, 13, 14, 15)
    anlz.set_song_structure(SongStructure(1, True, data))
    data.end_beat = 16
    assert anlz.get_song_structure().end_beat == 8
    anlz.set_song_structure_data(data)
    snapshot = anlz.get_song_structure_section()
    assert snapshot.version == 1 and snapshot.is_encrypted
    assert snapshot.data.end_beat == 16
    assert snapshot.data.phrases[0].u5 == 11
    anlz.dump()
    reloaded = Anlz(tmp_path / "metadata.EXT")
    assert reloaded.get_song_structure_section().data.phrases[0].u1 == 7
    assert reloaded.get_seek_index().entries[0].offset == 2**64 - 1
    assert anlz.into_data()["header"]["code"] == b"PMAI"
    assert anlz.section_count() == 2


def test_cue_subtype_helpers(fixture_copy):
    anlz = Anlz(fixture_copy("EXT"))
    for name in ("hot_cues", "memory_cues", "extended_hot_cues", "extended_memory_cues"):
        snapshot = getattr(anlz, "get_" + name)()
        assert snapshot is not None
        assert getattr(anlz, "remove_" + name)()
        assert not getattr(anlz, "remove_" + name)()
        getattr(anlz, "set_" + name)(snapshot)
        assert getattr(anlz, "get_" + name)().list_type == snapshot.list_type


@pytest.mark.parametrize("extension", ["DAT", "EXT", "2EX"])
@pytest.mark.parametrize("version", [6, 7])
def test_typed_setters_and_removers(fixture_copy, tmp_path, extension, version):
    source = fixture_copy(extension, version)
    original = source.read_bytes()
    anlz = Anlz(source)
    methods = {
        "PQTZ": "beat_grid",
        "PQT2": "extended_beat_grid",
        "PCOB": "cue_list",
        "PCO2": "extended_cue_list",
        "PVBR": "vbr",
        "PVB2": "seek_index",
        "PWV2": "tiny_waveform_preview",
        "PWAV": "waveform_preview",
        "PWV3": "waveform_detail",
        "PWV4": "waveform_color_preview",
        "PWV5": "waveform_color_detail",
        "PWV6": "waveform_3band_preview",
        "PWV7": "waveform_3band_detail",
        "PWVC": "waveform_3band_calibration",
        "PSSI": "song_structure",
        "PVDI": "vocal_detection",
    }
    for section in anlz.get_data()["sections"]:
        name = methods.get(section["tag"])
        if name is not None:
            getattr(anlz, "set_" + name)(section["content"])
    output = tmp_path / ("typed." + extension)
    anlz.dump_copy(output)
    assert output.read_bytes() == original
    for name in methods.values():
        while getattr(anlz, "remove_" + name)():
            pass
        assert not getattr(anlz, "remove_" + name)()


def test_static_beat_grid_helpers():
    grid = BeatGrid.new_static_bpm(120.0, 100, 2100)
    assert [b.time for b in grid.beats] == [100, 600, 1100, 1600, 2100]
    assert [b.beat_number for b in grid.beats] == [1, 2, 3, 4, 1]
    assert all(b.tempo == 12000 for b in grid.beats)
    grid.header_flags = 0xABCDFF83
    grid.set_static_bpm(100.0, 200, 1400)
    assert grid.header_flags == 0xABCDFF83
    assert [b.time for b in grid.beats] == [200, 800, 1400]
    for bpm in (float("nan"), float("inf"), 0.999, 300.001):
        with pytest.raises(AnlzError):
            BeatGrid.new_static_bpm(bpm, 0, 1000)
        with pytest.raises(AnlzError):
            grid.set_static_bpm(bpm, 0, 1000)
        assert [b.time for b in grid.beats] == [200, 800, 1400]
        assert grid.header_flags == 0xABCDFF83
    for bpm in (1.0, 300.0):
        assert BeatGrid.new_static_bpm(bpm, 0, 1000).beats[0].tempo == int(bpm * 100)
    with pytest.raises(AnlzError):
        grid.set_static_bpm(120.0, 1001, 1000)
    with pytest.raises(OverflowError):
        BeatGrid.new_static_bpm(120.0, -1, 1000)
    with pytest.raises(TypeError):
        BeatGrid.new_static_bpm(120.0, 0.5, 1000)
