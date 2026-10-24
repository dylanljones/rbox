# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Discovery bindings use isolated profiles and copied databases."""

import json

import pytest

from rbox import Error, MasterDb, MasterDbError
from rbox.core import LibraryLocation, LocationSource, LookupPaths, ProgramLocation, find_programs_in


def test_missing_profile_and_errors(tmp_path):
    paths = LookupPaths.under(tmp_path)
    location = paths.locate()
    assert isinstance(location, LibraryLocation)
    assert location.db_path == tmp_path / "Pioneer/rekordbox/master.db"
    assert location.source == LocationSource.Default
    assert not location.available
    assert not list(tmp_path.iterdir())
    with pytest.raises(AttributeError):
        location.available = True
    with pytest.raises(MasterDbError):
        MasterDb.from_location(location)
    assert not location.db_path.exists()
    paths.default_library_dirs = []
    with pytest.raises(Error):
        paths.locate()


def test_options_location_can_open_masterdb(fixture_copy, tmp_path):
    database = fixture_copy("masterdb")
    options = tmp_path / "options.json"
    analysis = tmp_path / "custom-analysis"
    settings = tmp_path / "custom-settings"
    options.write_text(
        json.dumps(
            {
                "options": [
                    ["db-path", str(database)],
                    ["analysis-data-root-path", str(analysis)],
                    ["settings-root-path", str(settings)],
                ]
            }
        ),
        encoding="utf-8",
    )
    paths = LookupPaths(tmp_path / "missing.settings", options, [tmp_path / "fallback"])
    assert paths.options_file == options
    paths.settings_file = tmp_path / "other.settings"
    assert paths.settings_file == tmp_path / "other.settings"
    location = paths.locate()
    assert location.source == LocationSource.Options
    assert location.db_path == database
    assert location.analysis_root == analysis
    assert location.settings_root == settings
    assert location.available
    db = MasterDb.from_location(location)
    assert db.get_contents()
    options.write_text("{bad json", encoding="utf-8")
    with pytest.raises(Error):
        paths.locate()


def test_program_results(tmp_path):
    executable = tmp_path / "rekordbox 7.2.1" / "rekordbox.exe"
    executable.parent.mkdir()
    executable.write_bytes(b"")
    programs = find_programs_in([tmp_path])
    assert len(programs) == 1
    assert isinstance(programs[0], ProgramLocation)
    assert programs[0].executable == executable
    assert programs[0].version == [7, 2, 1]
    assert find_programs_in([tmp_path / "missing"]) == []
