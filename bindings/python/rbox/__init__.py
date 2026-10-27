# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Rekordbox databases, analysis files, settings, and shared core utilities."""

from . import core, rbxml
from ._rbox import (
    AnlzError,
    DeviceLibraryError,
    Error,
    MasterDbError,
    OneLibraryError,
    SettingError,
)
from .anlz import Anlz, TagType

# Preserve the existing top-level API as aliases of the core namespace.
from .core import (
    LibraryLocation,
    LocationSource,
    LookupPaths,
    OptionsError,
    ProgramLocation,
    RekordboxOptions,
    Track,
    TrackMetadata,
    TrackSource,
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
from .devlib import DeviceLibrary
from .masterdb import MasterDb
from .one_library import OneLibrary
from .rbxml import RekordboxXml
from .settings import Setting
