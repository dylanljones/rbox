# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-05-15

from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

from ._rbox import PyRekordboxXml as _PyRekordboxXml


@dataclass
class Base:
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class Tempo(Base):
    inizio: float
    bpm: float
    metro: str
    battito: int


@dataclass
class PositionMark(Base):
    name: str
    mark_type: int
    num: int
    start: float
    end: Optional[float]


@dataclass
class Track(Base):
    trackid: str
    location: str
    name: Optional[str] = None
    artist: Optional[str] = None
    composer: Optional[str] = None
    album: Optional[str] = None
    grouping: Optional[str] = None
    genre: Optional[str] = None
    kind: Optional[str] = None
    size: Optional[int] = None
    totaltime: Optional[float] = None
    discnumber: Optional[int] = None
    tracknumber: Optional[int] = None
    year: Optional[int] = None
    averagebpm: Optional[float] = None
    datemodified: Optional[str] = None
    dateadded: Optional[str] = None
    bitrate: Optional[int] = None
    samplerate: Optional[float] = None
    comments: Optional[str] = None
    playcount: Optional[int] = None
    lastplayed: Optional[str] = None
    rating: Optional[int] = None
    remixer: Optional[str] = None
    tonality: Optional[str] = None
    label: Optional[str] = None
    mix: Optional[str] = None
    colour: Optional[str] = None
    tempos: List[Tempo] = field(default_factory=list)
    position_marks: List[PositionMark] = field(default_factory=list)

    def __post_init__(self):
        if self.tempos and isinstance(self.tempos[0], dict):
            self.tempos = [Tempo(**data) for data in self.tempos]  # type: ignore
        if self.position_marks and isinstance(self.position_marks[0], dict):
            self.position_marks = [PositionMark(**mark) for mark in self.position_marks]  # type: ignore

    def to_dict(self) -> Dict[str, Any]:
        data = super().to_dict()
        data["tempos"] = [tempo.to_dict() for tempo in self.tempos]
        data["position_marks"] = [mark.to_dict() for mark in self.position_marks]
        return data


class RekordboxXml:
    def __init__(self, path: Union[str, Path]) -> None:
        p = Path(path)
        if p.exists():
            self._xml = _PyRekordboxXml.load(str(path))
        else:
            self._xml = _PyRekordboxXml(str(path))

    def dump_copy(self, path: Union[str, Path]) -> None:
        self._xml.dump_copy(str(path))

    def dump(self) -> None:
        self._xml.dump()

    def get_tracks(self) -> List[Track]:
        return [Track(**item) for item in self._xml.get_tracks()]

    def get_track_by_id(self, track_id: str) -> Optional[Track]:
        item = self._xml.get_track_by_id(track_id)
        return Track(**item) if item else None

    def get_track_by_location(self, location: Union[str, Path]) -> Optional[Track]:
        loc = str(location).replace("\\", "/")
        item = self._xml.get_track_by_location(loc)
        return Track(**item) if item else None

    def get_track_by_key(self, key: Union[str, Path], key_type: int) -> Optional[Track]:
        key = str(key).replace("\\", "/")
        item = self._xml.get_track_by_key(key, key_type)
        return Track(**item) if item else None

    def add_track(self, track: Track) -> None:
        self._xml.add_track(track.to_dict())

    def update_track(self, track: Track) -> None:
        self._xml.update_track(track.to_dict())

    def remove_track(self, track_id: str) -> None:
        self._xml.remove_track(track_id)
