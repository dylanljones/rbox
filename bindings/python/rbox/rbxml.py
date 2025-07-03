# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-05-15

from pathlib import Path
from typing import List, Optional, Union

from ._rbox import PyRekordboxXml as _PyRekordboxXml, Track


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
        return self._xml.get_tracks()

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

    def update_track(self, track: Track) -> None:
        self._xml.update_track(track)

    def remove_track(self, track_id: str) -> None:
        self._xml.remove_track(track_id)
