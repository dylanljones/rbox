# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

import pytest

from rbox import Setting, SettingError
from rbox.settings import OnAirDisplay


@pytest.mark.parametrize(
    "factory,filename",
    [
        ("new_mysetting", "MYSETTING.DAT"),
        ("new_mysetting2", "MYSETTING2.DAT"),
        ("new_djmmysetting", "DJMMYSETTING.DAT"),
        ("new_devsetting", "DEVSETTING.DAT"),
    ],
)
def test_factories_and_writers(tmp_path, factory, filename):
    setting = getattr(Setting, factory)(str(tmp_path / filename))
    assert isinstance(setting, Setting)
    assert setting.dump() is None
    copy_dir = tmp_path / "copy"
    copy_dir.mkdir()
    assert setting.dump_copy(str(copy_dir / filename)) is None
    assert isinstance(Setting.load(str(tmp_path / filename)), Setting)


def test_enum_conversion(tmp_path):
    setting = Setting.new_mysetting(str(tmp_path / "MYSETTING.DAT"))
    assert isinstance(setting.get_on_air_display(), OnAirDisplay)
    assert setting.set_on_air_display(OnAirDisplay.On) is None
    assert setting.get_on_air_display() is OnAirDisplay.On
    assert setting.set_on_air_display("Off") is None
    assert setting.get_on_air_display() is OnAirDisplay.Off
    with pytest.raises(ValueError):
        setting.set_on_air_display("not-an-enum-value")


def test_native_error(tmp_path):
    with pytest.raises(SettingError):
        Setting.load(str(tmp_path / "missing.DAT"))
