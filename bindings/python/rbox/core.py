# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Shared track models, paths, library discovery, and Rekordbox options.

This namespace exposes the Python bindings for ``rbox::core``. For example::

    from rbox import core

    directory = core.anlz_dir_usb("/Contents/Artist/Track.flac")
    artwork = core.artwork_dir_usb(20)
"""

from ._rbox import (
    FileType,
    LibraryLocation,
    LocationSource,
    LookupPaths,
    OptionsError,
    ProgramLocation,
    RekordboxOptions,
    Track,
    TrackMetadata,
    anlz_dir_local,
    anlz_dir_usb,
    app_dir,
    artwork_dir_local,
    artwork_dir_usb,
    default_master_db_path,
    find_programs,
    find_programs_in,
    image_path_local,
    is_rekordbox_running,
    locate_library,
    master_db_path,
    options_json_path,
    rekordbox_agent_app_dir,
    rekordbox_app_dir,
)

__all__ = [
    "FileType",
    "LibraryLocation",
    "LocationSource",
    "LookupPaths",
    "OptionsError",
    "ProgramLocation",
    "RekordboxOptions",
    "Track",
    "TrackMetadata",
    "anlz_dir_local",
    "anlz_dir_usb",
    "app_dir",
    "artwork_dir_local",
    "artwork_dir_usb",
    "default_master_db_path",
    "find_programs",
    "find_programs_in",
    "image_path_local",
    "is_rekordbox_running",
    "locate_library",
    "master_db_path",
    "options_json_path",
    "rekordbox_agent_app_dir",
    "rekordbox_app_dir",
]
