# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-06-01

from pathlib import Path
from typing import Dict, Union

from ._rbox import PyMasterDb as _PyMasterDb
from .anlz import Anlz


class MasterDb(_PyMasterDb):
    def __init__(self, path: Union[str, Path]) -> None:
        super().__init__(str(path))

    def get_content_anlz_dir(self, content_id: str) -> Path:
        return Path(super().get_content_anlz_dir(content_id))

    def get_content_anlz_paths(self, content_id: str) -> Dict[str, Path]:
        """Get the ANLZ paths for a content ID."""
        paths = super().get_content_anlz_paths(content_id)
        return {ext: Path(p) for ext, p in paths.items()}

    def get_content_anlz_files(self, content_id: str) -> Dict[str, Anlz]:
        """Get the ANLZ file handlers for a content ID."""
        paths = self.get_content_anlz_paths(content_id)
        return {ext: Anlz(p) for ext, p in paths.items()}
