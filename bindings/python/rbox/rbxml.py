# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-05-15

from pathlib import Path
from typing import List, Optional, Union

from ._rbox import PlaylistNode, PyRekordboxXml as _PyRekordboxXml, Track


class RekordboxXml:
    def __init__(self, path: Union[str, Path]) -> None:
        p = Path(path)
        if p.exists():
            self._xml = _PyRekordboxXml.load(str(path))
        else:
            self._xml = _PyRekordboxXml(str(path))

    @property
    def tracks(self) -> list[Track]:
        return self._xml.tracks

    @property
    def root_playlist(self) -> PlaylistNode:
        return self._xml.root_playlist

    def to_string(self) -> str:
        return self._xml.to_string()

    def dump_copy(self, path: Union[str, Path]) -> None:
        self._xml.dump_copy(str(path))

    def dump(self) -> None:
        self._xml.dump()

    def get_tracks(self) -> List[Track]:
        return self._xml.tracks

    def get_track(self, index: int) -> Optional[Track]:
        return self._xml.get_track(index)

    def get_track_by_id(self, track_id: str) -> Optional[Track]:
        return self._xml.get_track_by_id(track_id)

    def get_track_by_location(self, location: Union[str, Path]) -> Optional[Track]:
        loc = str(location).replace("\\", "/")
        return self._xml.get_track_by_location(loc)

    def get_track_by_key(self, key: Union[str, Path], key_type: int) -> Optional[Track]:
        key = str(key).replace("\\", "/")
        return self._xml.get_track_by_key(key, key_type)

    def add_track(self, track: Track) -> None:
        self._xml.add_track(track)

    def new_track(self, track_id: str, location: str) -> Track:
        return self._xml.new_track(track_id, location)

    def update_track(self, track: Track) -> None:
        self._xml.update_track(track)

    def remove_track(self, track_id: str) -> None:
        self._xml.remove_track(track_id)

    def clear_tracks(self) -> None:
        self._xml.clear_tracks()
