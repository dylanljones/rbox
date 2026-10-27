# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Deprecated compatibility package; use rbox.onelib instead."""

import warnings as _warnings

from ..onelib import OneLibrary
from .models import *

_warnings.warn(
    "rbox.one_library is deprecated; use rbox.onelib instead.",
    DeprecationWarning,
    stacklevel=2,
)
