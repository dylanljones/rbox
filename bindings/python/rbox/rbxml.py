# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

from pathlib import Path
from typing import List, Optional, Union

from ._rbox import (
    PyRekordboxXml as _PyRekordboxXml,
    XmlPlaylistNode,
    XmlPlaylistTrack,
    XmlPositionMark,
    XmlTempo,
    XmlTrack,
)


class RekordboxXml:
    def __init__(self, path: Union[str, Path]) -> None:
        p = Path(path)
        if p.exists():
            self._xml = _PyRekordboxXml.load(str(path))
        else:
            self._xml = _PyRekordboxXml(str(path))

    def get_version(self) -> str:
        """Return the XML format version (DJ_PLAYLISTS Version)."""
        return self._xml.get_version()

    def set_version(self, value: str) -> None:
        """Set the XML format version (DJ_PLAYLISTS Version); save with dump or dump_copy."""
        self._xml.set_version(value)

    def get_product_name(self) -> str:
        """Return the name of the application that produced the XML."""
        return self._xml.get_product_name()

    def set_product_name(self, value: str) -> None:
        """Set the name of the application that produced the XML; save with dump or dump_copy."""
        self._xml.set_product_name(value)

    def get_product_version(self) -> str:
        """Return the version of the application that produced the XML."""
        return self._xml.get_product_version()

    def set_product_version(self, value: str) -> None:
        """Set the version of the application that produced the XML; save with dump or dump_copy."""
        self._xml.set_product_version(value)

    def get_product_company(self) -> str:
        """Return the company that produced the application."""
        return self._xml.get_product_company()

    def set_product_company(self, value: str) -> None:
        """Set the company that produced the application; save with dump or dump_copy."""
        self._xml.set_product_company(value)

    @property
    def tracks(self) -> list[XmlTrack]:
        return self._xml.tracks

    @property
    def root_playlist(self) -> XmlPlaylistNode:
        return self._xml.root_playlist

    def to_string(self) -> str:
        return self._xml.to_string()

    def dump_copy(self, path: Union[str, Path]) -> None:
        self._xml.dump_copy(str(path))

    def dump(self) -> None:
        self._xml.dump()

    def get_tracks(self) -> List[XmlTrack]:
        return self._xml.tracks

    def get_track(self, index: int) -> Optional[XmlTrack]:
        return self._xml.get_track(index)

    def get_track_by_id(self, track_id: str) -> Optional[XmlTrack]:
        return self._xml.get_track_by_id(track_id)

    def get_track_by_location(self, location: Union[str, Path]) -> Optional[XmlTrack]:
        loc = str(location).replace("\\", "/")
        return self._xml.get_track_by_location(loc)

    def get_track_by_key(self, key: Union[str, Path], key_type: int) -> Optional[XmlTrack]:
        key = str(key).replace("\\", "/")
        return self._xml.get_track_by_key(key, key_type)

    def add_track(self, track: XmlTrack) -> None:
        self._xml.add_track(track)

    def new_track(self, track_id: str, location: str) -> XmlTrack:
        return self._xml.new_track(track_id, location)

    def update_track(self, track: XmlTrack) -> None:
        self._xml.update_track(track)

    def remove_track(self, track_id: str) -> None:
        self._xml.remove_track(track_id)

    def clear_tracks(self) -> None:
        self._xml.clear_tracks()


# Deprecated compatibility alias; prefer XmlTrack.
Track = XmlTrack

# Deprecated compatibility alias; prefer XmlTempo.
Tempo = XmlTempo

# Deprecated compatibility alias; prefer XmlPositionMark.
PositionMark = XmlPositionMark

# Deprecated compatibility alias; prefer XmlPlaylistTrack.
PlaylistTrack = XmlPlaylistTrack

# Deprecated compatibility alias; prefer XmlPlaylistNode.
PlaylistNode = XmlPlaylistNode
