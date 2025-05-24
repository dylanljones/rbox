# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-05-10

from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import List, Optional, Union

from ._rbox import PyAnlz as _PyAnlz


class TagType(Enum):
    """Enum for tag types."""

    BeatGrid = "BeatGrid"
    ExtendedBeatGrid = "ExtendedBeatGrid"
    CueList = "CueList"
    ExtendedCueList = "ExtendedCueList"
    Path = "Path"
    VBR = "VBR"
    WaveformPreview = "WaveformPreview"
    TinyWaveformPreview = "TinyWaveformPreview"
    WaveformDetail = "WaveformDetail"
    WaveformColorPreview = "WaveformColorPreview"
    WaveformColorDetail = "WaveformColorDetail"
    Waveform3BandPreview = "Waveform3BandPreview"
    Waveform3BandDetail = "Waveform3BandDetail"
    SongStructure = "SongStructure"


@dataclass
class Beat:
    beat_number: int
    tempo: int
    time: int


@dataclass
class ExtBeat:
    beat_number: int


@dataclass
class Cue:
    hot_cue: int
    status: int
    order_first: int
    order_last: int
    cue_type: int
    time: int
    loop_time: int


@dataclass
class ExtendedCue:
    hot_cue: int
    cue_type: int
    time: int
    loop_time: int
    color: int
    loop_numerator: int
    loop_denominator: int
    comment: str
    hot_cue_color_index: int
    red: int
    green: int
    blue: int


class Anlz:
    def __init__(self, path: Union[str, Path]) -> None:
        path = Path(path)
        if not path.exists():
            raise FileNotFoundError(f"File not found: {path}")
        self._anlz = _PyAnlz(str(path))

    def dump_copy(self, path: Union[str, Path]) -> None:
        self._anlz.dump_copy(str(path))

    def dump(self) -> None:
        self._anlz.dump()

    def contains(self, tag: TagType) -> bool:
        return self._anlz.contains(tag.value)

    def get_beat_grid(self) -> List[Beat]:
        return [Beat(**data) for data in self._anlz.get_beat_grid()]

    def get_extended_beat_grid(self) -> List[ExtBeat]:
        return [ExtBeat(**data) for data in self._anlz.get_extended_beat_grid()]

    def get_hot_cues(self) -> List[Cue]:
        return [Cue(**data) for data in self._anlz.get_hot_cues()]

    def get_memory_cues(self) -> List[Cue]:
        return [Cue(**data) for data in self._anlz.get_memory_cues()]

    def get_extended_hot_cues(self) -> List[ExtendedCue]:
        return [ExtendedCue(**data) for data in self._anlz.get_extended_hot_cues()]

    def get_extended_memory_cues(self) -> List[ExtendedCue]:
        return [ExtendedCue(**data) for data in self._anlz.get_extended_memory_cues()]

    def get_tags(self) -> List[str]:
        return self._anlz.get_tags()

    def get_path(self) -> Optional[str]:
        return self._anlz.get_path()

    def set_path(self, path: Union[str, Path]) -> None:
        self._anlz.set_path(str(path))
