# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Editable ANLZ snapshots. Call a setter to apply changes, then dump to save."""

from enum import Enum
from pathlib import Path
from typing import List, Literal, Optional, TypedDict, Union

from ._rbox import (
    VBR,
    BeatGrid,
    CueList,
    ExtendedBeatGrid,
    ExtendedCueList,
    PyAnlz as _PyAnlz,
    SeekIndex,
    SongStructure,
    SongStructureData,
    TinyWaveformPreview,
    UnknownSection,
    VocalDetection,
    Waveform3BandCalibration,
    Waveform3BandDetail,
    Waveform3BandPreview,
    WaveformColorDetail,
    WaveformColorPreview,
    WaveformDetail,
    WaveformPreview,
)


class TagType(Enum):
    """Enum for tag types."""

    BeatGrid = "BeatGrid"
    ExtendedBeatGrid = "ExtendedBeatGrid"
    CueList = "CueList"
    ExtendedCueList = "ExtendedCueList"
    Path = "Path"
    VBR = "VBR"
    SeekIndex = "SeekIndex"
    VocalDetection = "VocalDetection"
    Waveform3BandCalibration = "Waveform3BandCalibration"
    WaveformPreview = "WaveformPreview"
    TinyWaveformPreview = "TinyWaveformPreview"
    WaveformDetail = "WaveformDetail"
    WaveformColorPreview = "WaveformColorPreview"
    WaveformColorDetail = "WaveformColorDetail"
    Waveform3BandPreview = "Waveform3BandPreview"
    Waveform3BandDetail = "Waveform3BandDetail"
    SongStructure = "SongStructure"


class BeatGridSection(TypedDict):
    tag: Literal["PQTZ"]
    content: BeatGrid


class ExtendedBeatGridSection(TypedDict):
    tag: Literal["PQT2"]
    content: ExtendedBeatGrid


class CueListSection(TypedDict):
    tag: Literal["PCOB"]
    content: CueList


class ExtendedCueListSection(TypedDict):
    tag: Literal["PCO2"]
    content: ExtendedCueList


class PathSection(TypedDict):
    tag: Literal["PPTH"]
    content: str


class VBRSection(TypedDict):
    tag: Literal["PVBR"]
    content: VBR


class SeekIndexSection(TypedDict):
    tag: Literal["PVB2"]
    content: SeekIndex


class TinyWaveformPreviewSection(TypedDict):
    tag: Literal["PWV2"]
    content: TinyWaveformPreview


class WaveformPreviewSection(TypedDict):
    tag: Literal["PWAV"]
    content: WaveformPreview


class WaveformDetailSection(TypedDict):
    tag: Literal["PWV3"]
    content: WaveformDetail


class WaveformColorPreviewSection(TypedDict):
    tag: Literal["PWV4"]
    content: WaveformColorPreview


class WaveformColorDetailSection(TypedDict):
    tag: Literal["PWV5"]
    content: WaveformColorDetail


class Waveform3BandPreviewSection(TypedDict):
    tag: Literal["PWV6"]
    content: Waveform3BandPreview


class Waveform3BandDetailSection(TypedDict):
    tag: Literal["PWV7"]
    content: Waveform3BandDetail


class Waveform3BandCalibrationSection(TypedDict):
    tag: Literal["PWVC"]
    content: Waveform3BandCalibration


class SongStructureSection(TypedDict):
    tag: Literal["PSSI"]
    content: SongStructure


class VocalDetectionSection(TypedDict):
    tag: Literal["PVDI"]
    content: VocalDetection


class UnknownTaggedSection(TypedDict):
    tag: Literal["Unknown"]
    content: UnknownSection


AnlzSection = Union[
    BeatGridSection,
    ExtendedBeatGridSection,
    CueListSection,
    ExtendedCueListSection,
    PathSection,
    VBRSection,
    SeekIndexSection,
    TinyWaveformPreviewSection,
    WaveformPreviewSection,
    WaveformDetailSection,
    WaveformColorPreviewSection,
    WaveformColorDetailSection,
    Waveform3BandPreviewSection,
    Waveform3BandDetailSection,
    Waveform3BandCalibrationSection,
    SongStructureSection,
    VocalDetectionSection,
    UnknownTaggedSection,
]


class AnlzHeader(TypedDict):
    code: bytes
    size: int
    total_size: int


class AnlzData(TypedDict):
    header: AnlzHeader
    header_data: bytes
    sections: List[AnlzSection]


def _tag(value: Union[TagType, str]) -> str:
    return value.value if isinstance(value, TagType) else value


class Anlz:
    def into_data(self) -> AnlzData:
        """Return an eager snapshot, leaving this handler usable."""
        return self._anlz.into_data()

    def get_song_structure_section(self) -> Optional[SongStructure]:
        return self._anlz.get_song_structure_section()

    def set_song_structure_data(self, value: SongStructureData) -> None:
        self._anlz.set_song_structure_data(value)

    def __init__(self, path: Union[str, Path]) -> None:
        path = Path(path)
        if not path.exists():
            raise FileNotFoundError(f"File not found: {path}")
        self._anlz = _PyAnlz(str(path))

    def dump_copy(self, path: Union[str, Path]) -> None:
        self._anlz.dump_copy(str(path))

    def dump(self) -> None:
        self._anlz.dump()

    def get_tags(self) -> List[str]:
        return self._anlz.get_tags()

    def contains(self, tag: Union[TagType, str]) -> bool:
        return self._anlz.contains(_tag(tag))

    def get_beat_grid(self) -> Optional[BeatGrid]:
        return self._anlz.get_beat_grid()

    def get_extended_beat_grid(self) -> Optional[ExtendedBeatGrid]:
        return self._anlz.get_extended_beat_grid()

    def get_hot_cues(self) -> Optional[CueList]:
        return self._anlz.get_hot_cues()

    def get_memory_cues(self) -> Optional[CueList]:
        return self._anlz.get_memory_cues()

    def get_extended_hot_cues(self) -> Optional[ExtendedCueList]:
        return self._anlz.get_extended_hot_cues()

    def get_extended_memory_cues(self) -> Optional[ExtendedCueList]:
        return self._anlz.get_extended_memory_cues()

    def get_path(self) -> Optional[str]:
        return self._anlz.get_path()

    def set_path(self, path: Union[str, Path]) -> None:
        self._anlz.set_path(str(path))

    def get_vbr_data(self) -> Optional[bytes]:
        return self._anlz.get_vbr_data()

    def get_tiny_waveform_preview(self) -> Optional[TinyWaveformPreview]:
        return self._anlz.get_tiny_waveform_preview()

    def get_waveform_preview(self) -> Optional[WaveformPreview]:
        return self._anlz.get_waveform_preview()

    def get_waveform_detail(self) -> Optional[WaveformDetail]:
        return self._anlz.get_waveform_detail()

    def get_waveform_color_preview(self) -> Optional[WaveformColorPreview]:
        return self._anlz.get_waveform_color_preview()

    def get_waveform_color_detail(self) -> Optional[WaveformColorDetail]:
        return self._anlz.get_waveform_color_detail()

    def get_waveform_3band_preview(self) -> Optional[Waveform3BandPreview]:
        return self._anlz.get_waveform_3band_preview()

    def get_waveform_3band_detail(self) -> Optional[Waveform3BandDetail]:
        return self._anlz.get_waveform_3band_detail()

    def get_song_structure(self) -> Optional[SongStructureData]:
        return self._anlz.get_song_structure()

    def get_vbr(self) -> Optional[VBR]:
        return self._anlz.get_vbr()

    def get_seek_index(self) -> Optional[SeekIndex]:
        return self._anlz.get_seek_index()

    def get_waveform_3band_calibration(self) -> Optional[Waveform3BandCalibration]:
        return self._anlz.get_waveform_3band_calibration()

    def get_vocal_detection(self) -> Optional[VocalDetection]:
        return self._anlz.get_vocal_detection()

    @classmethod
    def new_dat(cls, path: Union[str, Path]) -> "Anlz":
        """Create an empty analysis file in memory; dump writes it."""
        instance = cls.__new__(cls)
        instance._anlz = _PyAnlz.new_dat(str(path))
        return instance

    @classmethod
    def new_ext(cls, path: Union[str, Path]) -> "Anlz":
        """Create an empty analysis file in memory; dump writes it."""
        instance = cls.__new__(cls)
        instance._anlz = _PyAnlz.new_ext(str(path))
        return instance

    @classmethod
    def new_2ex(cls, path: Union[str, Path]) -> "Anlz":
        """Create an empty analysis file in memory; dump writes it."""
        instance = cls.__new__(cls)
        instance._anlz = _PyAnlz.new_2ex(str(path))
        return instance

    @classmethod
    def load(cls, path: Union[str, Path]) -> "Anlz":
        return cls(path)

    def section_count(self) -> int:
        return self._anlz.section_count()

    def get_section(self, index: int) -> Optional[AnlzSection]:
        return self._anlz.get_section(index)

    def get_section_by_tag(self, tag: Union[TagType, str]) -> Optional[AnlzSection]:
        return self._anlz.get_section_by_tag(_tag(tag))

    def get_sections_by_tag(self, tag: Union[TagType, str]) -> List[AnlzSection]:
        return self._anlz.get_sections_by_tag(_tag(tag))

    def get_data(self) -> AnlzData:
        """Explicitly read every section into a detached snapshot."""
        return self._anlz.get_data()

    def remove_section(self, index: int) -> None:
        self._anlz.remove_section(index)

    def remove_section_by_tag(self, tag: Union[TagType, str]) -> bool:
        return self._anlz.remove_section_by_tag(_tag(tag))

    def remove_sections_by_tag(self, tag: Union[TagType, str]) -> int:
        return self._anlz.remove_sections_by_tag(_tag(tag))

    def insert_section(self, index: int, section: AnlzSection) -> None:
        self._anlz.insert_section(index, section)

    def set_section_at(self, index: int, section: AnlzSection) -> None:
        self._anlz.set_section_at(index, section)

    def push_section(self, section: AnlzSection) -> None:
        self._anlz.push_section(section)

    def set_section(self, section: AnlzSection) -> None:
        self._anlz.set_section(section)

    def set_beat_grid(self, value: BeatGrid) -> None:
        self._anlz.set_beat_grid(value)

    def remove_beat_grid(self) -> bool:
        return self._anlz.remove_beat_grid()

    def set_extended_beat_grid(self, value: ExtendedBeatGrid) -> None:
        self._anlz.set_extended_beat_grid(value)

    def remove_extended_beat_grid(self) -> bool:
        return self._anlz.remove_extended_beat_grid()

    def set_cue_list(self, value: CueList) -> None:
        self._anlz.set_cue_list(value)

    def remove_cue_list(self) -> bool:
        return self._anlz.remove_cue_list()

    def set_extended_cue_list(self, value: ExtendedCueList) -> None:
        self._anlz.set_extended_cue_list(value)

    def remove_extended_cue_list(self) -> bool:
        return self._anlz.remove_extended_cue_list()

    def set_vbr(self, value: VBR) -> None:
        self._anlz.set_vbr(value)

    def remove_vbr(self) -> bool:
        return self._anlz.remove_vbr()

    def set_seek_index(self, value: SeekIndex) -> None:
        self._anlz.set_seek_index(value)

    def remove_seek_index(self) -> bool:
        return self._anlz.remove_seek_index()

    def set_tiny_waveform_preview(self, value: TinyWaveformPreview) -> None:
        self._anlz.set_tiny_waveform_preview(value)

    def remove_tiny_waveform_preview(self) -> bool:
        return self._anlz.remove_tiny_waveform_preview()

    def set_waveform_preview(self, value: WaveformPreview) -> None:
        self._anlz.set_waveform_preview(value)

    def remove_waveform_preview(self) -> bool:
        return self._anlz.remove_waveform_preview()

    def set_waveform_detail(self, value: WaveformDetail) -> None:
        self._anlz.set_waveform_detail(value)

    def remove_waveform_detail(self) -> bool:
        return self._anlz.remove_waveform_detail()

    def set_waveform_color_preview(self, value: WaveformColorPreview) -> None:
        self._anlz.set_waveform_color_preview(value)

    def remove_waveform_color_preview(self) -> bool:
        return self._anlz.remove_waveform_color_preview()

    def set_waveform_color_detail(self, value: WaveformColorDetail) -> None:
        self._anlz.set_waveform_color_detail(value)

    def remove_waveform_color_detail(self) -> bool:
        return self._anlz.remove_waveform_color_detail()

    def set_waveform_3band_preview(self, value: Waveform3BandPreview) -> None:
        self._anlz.set_waveform_3band_preview(value)

    def remove_waveform_3band_preview(self) -> bool:
        return self._anlz.remove_waveform_3band_preview()

    def set_waveform_3band_detail(self, value: Waveform3BandDetail) -> None:
        self._anlz.set_waveform_3band_detail(value)

    def remove_waveform_3band_detail(self) -> bool:
        return self._anlz.remove_waveform_3band_detail()

    def set_waveform_3band_calibration(self, value: Waveform3BandCalibration) -> None:
        self._anlz.set_waveform_3band_calibration(value)

    def remove_waveform_3band_calibration(self) -> bool:
        return self._anlz.remove_waveform_3band_calibration()

    def set_song_structure(self, value: SongStructure) -> None:
        self._anlz.set_song_structure(value)

    def remove_song_structure(self) -> bool:
        return self._anlz.remove_song_structure()

    def set_vocal_detection(self, value: VocalDetection) -> None:
        self._anlz.set_vocal_detection(value)

    def remove_vocal_detection(self) -> bool:
        return self._anlz.remove_vocal_detection()

    def set_hot_cues(self, value: CueList) -> None:
        self._anlz.set_hot_cues(value)

    def remove_hot_cues(self) -> bool:
        return self._anlz.remove_hot_cues()

    def set_memory_cues(self, value: CueList) -> None:
        self._anlz.set_memory_cues(value)

    def remove_memory_cues(self) -> bool:
        return self._anlz.remove_memory_cues()

    def set_extended_hot_cues(self, value: ExtendedCueList) -> None:
        self._anlz.set_extended_hot_cues(value)

    def remove_extended_hot_cues(self) -> bool:
        return self._anlz.remove_extended_hot_cues()

    def set_extended_memory_cues(self, value: ExtendedCueList) -> None:
        self._anlz.set_extended_memory_cues(value)

    def remove_extended_memory_cues(self) -> bool:
        return self._anlz.remove_extended_memory_cues()

    def remove_path(self) -> bool:
        return self._anlz.remove_path()
