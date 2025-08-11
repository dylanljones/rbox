// Author: Dylan Jones
// Date:   2025-08-11

use rbox::settings::*;
use std::path::PathBuf;

mod common;

fn path(dir: &PathBuf, file_name: &str, key: &str, value: &str) -> PathBuf {
    dir.join(key).join(value).join(file_name)
}

fn load(dir: &PathBuf, file_name: &str, key: &str, value: &str) -> anyhow::Result<Setting> {
    Setting::load(path(dir, file_name, key, value))
}

fn assert_file(expected: &PathBuf, actual: &PathBuf) {
    let expected_contents = std::fs::read(expected).expect("Failed to read expected file");
    let actual_contents = std::fs::read(actual).expect("Failed to read actual file");
    assert_eq!(
        expected_contents, actual_contents,
        "File contents do not match"
    );
}

fn assert_get_auto_cue(dir: &PathBuf, name: &str, value: AutoCue) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "auto_cue", name)?;
    assert_eq!(sett.get_auto_cue()?, value);
    Ok(())
}

fn assert_set_auto_cue(dir: &PathBuf, name: &str, value: AutoCue) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "auto_cue", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_auto_cue(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_auto_cue_level(dir: &PathBuf, name: &str, value: AutoCueLevel) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "auto_cue_level", name)?;
    assert_eq!(sett.get_auto_cue_level()?, value);
    Ok(())
}

fn assert_set_auto_cue_level(dir: &PathBuf, name: &str, value: AutoCueLevel) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "auto_cue_level", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_auto_cue_level(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_disc_slot_illumination(
    dir: &PathBuf,
    name: &str,
    value: DiscSlotIllumination,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "disc_slot_illumination", name)?;
    assert_eq!(sett.get_disc_slot_illumination()?, value);
    Ok(())
}

fn assert_set_disc_slot_illumination(
    dir: &PathBuf,
    name: &str,
    value: DiscSlotIllumination,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "disc_slot_illumination", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_disc_slot_illumination(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_eject_lock(dir: &PathBuf, name: &str, value: EjectLock) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "eject_lock", name)?;
    assert_eq!(sett.get_eject_lock()?, value);
    Ok(())
}

fn assert_set_eject_lock(dir: &PathBuf, name: &str, value: EjectLock) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "eject_lock", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_eject_lock(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_hotcue_autoload(
    dir: &PathBuf,
    name: &str,
    value: HotCueAutoLoad,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "hotcue_autoload", name)?;
    assert_eq!(sett.get_hotcue_autoload()?, value);
    Ok(())
}

fn assert_set_hotcue_autoload(
    dir: &PathBuf,
    name: &str,
    value: HotCueAutoLoad,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "hotcue_autoload", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_hotcue_autoload(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_hotcue_color(dir: &PathBuf, name: &str, value: HotCueColor) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "hotcue_color", name)?;
    assert_eq!(sett.get_hotcue_color()?, value);
    Ok(())
}

fn assert_set_hotcue_color(dir: &PathBuf, name: &str, value: HotCueColor) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "hotcue_color", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_hotcue_color(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_jog_mode(dir: &PathBuf, name: &str, value: JogMode) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "jog_mode", name)?;
    assert_eq!(sett.get_jog_mode()?, value);
    Ok(())
}

fn assert_set_jog_mode(dir: &PathBuf, name: &str, value: JogMode) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "jog_mode", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_mode(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_jog_ring_brightness(
    dir: &PathBuf,
    name: &str,
    value: JogRingBrightness,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "jog_ring_brightness", name)?;
    assert_eq!(sett.get_jog_ring_brightness()?, value);
    Ok(())
}

fn assert_set_jog_ring_brightness(
    dir: &PathBuf,
    name: &str,
    value: JogRingBrightness,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "jog_ring_brightness", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_ring_brightness(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_jog_ring_indicator(
    dir: &PathBuf,
    name: &str,
    value: JogRingIndicator,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "jog_ring_indicator", name)?;
    assert_eq!(sett.get_jog_ring_indicator()?, value);
    Ok(())
}

fn assert_set_jog_ring_indicator(
    dir: &PathBuf,
    name: &str,
    value: JogRingIndicator,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "jog_ring_indicator", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_ring_indicator(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_language(dir: &PathBuf, name: &str, value: Language) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "language", name)?;
    assert_eq!(sett.get_language()?, value);
    Ok(())
}

fn assert_set_language(dir: &PathBuf, name: &str, value: Language) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "language", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_language(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_lcd_brightness(
    dir: &PathBuf,
    name: &str,
    value: LCDBrightness,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "lcd_brightness", name)?;
    assert_eq!(sett.get_lcd_brightness()?, value);
    Ok(())
}

fn assert_set_lcd_brightness(
    dir: &PathBuf,
    name: &str,
    value: LCDBrightness,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "lcd_brightness", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_lcd_brightness(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_master_tempo(dir: &PathBuf, name: &str, value: MasterTempo) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "master_tempo", name)?;
    assert_eq!(sett.get_master_tempo()?, value);
    Ok(())
}

fn assert_set_master_tempo(dir: &PathBuf, name: &str, value: MasterTempo) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "master_tempo", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_master_tempo(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_needle_lock(dir: &PathBuf, name: &str, value: NeedleLock) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "needle_lock", name)?;
    assert_eq!(sett.get_needle_lock()?, value);
    Ok(())
}

fn assert_set_needle_lock(dir: &PathBuf, name: &str, value: NeedleLock) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "needle_lock", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_needle_lock(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_on_air_display(dir: &PathBuf, name: &str, value: OnAirDisplay) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "on_air_display", name)?;
    assert_eq!(sett.get_on_air_display()?, value);
    Ok(())
}

fn assert_set_on_air_display(dir: &PathBuf, name: &str, value: OnAirDisplay) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "on_air_display", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_on_air_display(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_phase_meter(dir: &PathBuf, name: &str, value: PhaseMeter) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "phase_meter", name)?;
    assert_eq!(sett.get_phase_meter()?, value);
    Ok(())
}

fn assert_set_phase_meter(dir: &PathBuf, name: &str, value: PhaseMeter) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "phase_meter", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_phase_meter(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_play_mode(dir: &PathBuf, name: &str, value: PlayMode) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "play_mode", name)?;
    assert_eq!(sett.get_play_mode()?, value);
    Ok(())
}

fn assert_set_play_mode(dir: &PathBuf, name: &str, value: PlayMode) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "play_mode", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_play_mode(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_quantize(dir: &PathBuf, name: &str, value: Quantize) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "quantize", name)?;
    assert_eq!(sett.get_quantize()?, value);
    Ok(())
}

fn assert_set_quantize(dir: &PathBuf, name: &str, value: Quantize) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "quantize", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_quantize(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_quantize_beat_value(
    dir: &PathBuf,
    name: &str,
    value: QuantizeBeatValue,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "quantize_beat_value", name)?;
    assert_eq!(sett.get_quantize_beat_value()?, value);
    Ok(())
}

fn assert_set_quantize_beat_value(
    dir: &PathBuf,
    name: &str,
    value: QuantizeBeatValue,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "quantize_beat_value", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_quantize_beat_value(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_slip_flashing(dir: &PathBuf, name: &str, value: SlipFlashing) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "slip_flashing", name)?;
    assert_eq!(sett.get_slip_flashing()?, value);
    Ok(())
}

fn assert_set_slip_flashing(dir: &PathBuf, name: &str, value: SlipFlashing) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "slip_flashing", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_slip_flashing(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_sync(dir: &PathBuf, name: &str, value: Sync) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "sync", name)?;
    assert_eq!(sett.get_sync()?, value);
    Ok(())
}

fn assert_set_sync(dir: &PathBuf, name: &str, value: Sync) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "sync", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_sync(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_tempo_range(dir: &PathBuf, name: &str, value: TempoRange) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "tempo_range", name)?;
    assert_eq!(sett.get_tempo_range()?, value);
    Ok(())
}

fn assert_set_tempo_range(dir: &PathBuf, name: &str, value: TempoRange) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "tempo_range", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_tempo_range(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_time_mode(dir: &PathBuf, name: &str, value: TimeMode) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING.DAT", "time_mode", name)?;
    assert_eq!(sett.get_time_mode()?, value);
    Ok(())
}

fn assert_set_time_mode(dir: &PathBuf, name: &str, value: TimeMode) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING.DAT", "time_mode", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_time_mode(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_beat_jump_beat_value(
    dir: &PathBuf,
    name: &str,
    value: BeatJumpBeatValue,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING2.DAT", "beat_jump_beat_value", name)?;
    assert_eq!(sett.get_beat_jump_beat_value()?, value);
    Ok(())
}

fn assert_set_beat_jump_beat_value(
    dir: &PathBuf,
    name: &str,
    value: BeatJumpBeatValue,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING2.DAT", "beat_jump_beat_value", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING2.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_beat_jump_beat_value(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_jog_display_mode(
    dir: &PathBuf,
    name: &str,
    value: JogDisplayMode,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING2.DAT", "jog_display_mode", name)?;
    assert_eq!(sett.get_jog_display_mode()?, value);
    Ok(())
}

fn assert_set_jog_display_mode(
    dir: &PathBuf,
    name: &str,
    value: JogDisplayMode,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING2.DAT", "jog_display_mode", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING2.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_display_mode(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_jog_lcd_brightness(
    dir: &PathBuf,
    name: &str,
    value: JogLCDBrightness,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING2.DAT", "jog_lcd_brightness", name)?;
    assert_eq!(sett.get_jog_lcd_brightness()?, value);
    Ok(())
}

fn assert_set_jog_lcd_brightness(
    dir: &PathBuf,
    name: &str,
    value: JogLCDBrightness,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING2.DAT", "jog_lcd_brightness", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING2.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_lcd_brightness(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_pad_button_brightness(
    dir: &PathBuf,
    name: &str,
    value: PadButtonBrightness,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING2.DAT", "pad_button_brightness", name)?;
    assert_eq!(sett.get_pad_button_brightness()?, value);
    Ok(())
}

fn assert_set_pad_button_brightness(
    dir: &PathBuf,
    name: &str,
    value: PadButtonBrightness,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING2.DAT", "pad_button_brightness", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING2.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_pad_button_brightness(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_vinyl_speed_adjust(
    dir: &PathBuf,
    name: &str,
    value: VinylSpeedAdjust,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING2.DAT", "vinyl_speed_adjust", name)?;
    assert_eq!(sett.get_vinyl_speed_adjust()?, value);
    Ok(())
}

fn assert_set_vinyl_speed_adjust(
    dir: &PathBuf,
    name: &str,
    value: VinylSpeedAdjust,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING2.DAT", "vinyl_speed_adjust", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING2.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_vinyl_speed_adjust(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_waveform(dir: &PathBuf, name: &str, value: Waveform) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING2.DAT", "waveform", name)?;
    assert_eq!(sett.get_waveform()?, value);
    Ok(())
}

fn assert_set_waveform(dir: &PathBuf, name: &str, value: Waveform) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING2.DAT", "waveform", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING2.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_waveform(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_waveform_divisions(
    dir: &PathBuf,
    name: &str,
    value: WaveformDivisions,
) -> anyhow::Result<()> {
    let sett = load(dir, "MYSETTING2.DAT", "waveform_divisions", name)?;
    assert_eq!(sett.get_waveform_divisions()?, value);
    Ok(())
}

fn assert_set_waveform_divisions(
    dir: &PathBuf,
    name: &str,
    value: WaveformDivisions,
) -> anyhow::Result<()> {
    let in_file = path(dir, "MYSETTING2.DAT", "waveform_divisions", name);
    let out_file = dir.parent().unwrap().join("out").join("MYSETTING2.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_waveform_divisions(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_beat_fx_quantize(
    dir: &PathBuf,
    name: &str,
    value: BeatFXQuantize,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "beat_fx_quantize", name)?;
    assert_eq!(sett.get_beat_fx_quantize()?, value);
    Ok(())
}

fn assert_set_beat_fx_quantize(
    dir: &PathBuf,
    name: &str,
    value: BeatFXQuantize,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "beat_fx_quantize", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_beat_fx_quantize(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_channel_fader_curve(
    dir: &PathBuf,
    name: &str,
    value: ChannelFaderCurve,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "channel_fader_curve", name)?;
    assert_eq!(sett.get_channel_fader_curve()?, value);
    Ok(())
}

fn assert_set_channel_fader_curve(
    dir: &PathBuf,
    name: &str,
    value: ChannelFaderCurve,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "channel_fader_curve", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_channel_fader_curve(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_channel_fader_curve_long_fader(
    dir: &PathBuf,
    name: &str,
    value: ChannelFaderCurveLongFader,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "channel_fader_curve_long", name)?;
    assert_eq!(sett.get_channel_fader_curve_long_fader()?, value);
    Ok(())
}

fn assert_set_channel_fader_curve_long_fader(
    dir: &PathBuf,
    name: &str,
    value: ChannelFaderCurveLongFader,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "channel_fader_curve_long", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_channel_fader_curve_long_fader(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_cross_fader_curve(
    dir: &PathBuf,
    name: &str,
    value: CrossfaderCurve,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "cross_fader_curve", name)?;
    assert_eq!(sett.get_crossfader_curve()?, value);
    Ok(())
}

fn assert_set_cross_fader_curve(
    dir: &PathBuf,
    name: &str,
    value: CrossfaderCurve,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "cross_fader_curve", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_crossfader_curve(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_mixer_display_brightness(
    dir: &PathBuf,
    name: &str,
    value: MixerDisplayBrightness,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "display_brightness", name)?;
    assert_eq!(sett.get_mixer_display_brightness()?, value);
    Ok(())
}

fn assert_set_mixer_display_brightness(
    dir: &PathBuf,
    name: &str,
    value: MixerDisplayBrightness,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "display_brightness", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_mixer_display_brightness(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_headphones_mono_split(
    dir: &PathBuf,
    name: &str,
    value: HeadphonesMonoSplit,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "headphones_mono_split", name)?;
    assert_eq!(sett.get_headphones_mono_split()?, value);
    Ok(())
}

fn assert_set_headphones_mono_split(
    dir: &PathBuf,
    name: &str,
    value: HeadphonesMonoSplit,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "headphones_mono_split", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_headphones_mono_split(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_headphones_pre_eq(
    dir: &PathBuf,
    name: &str,
    value: HeadphonesPreEQ,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "headphones_pre_eq", name)?;
    assert_eq!(sett.get_headphones_pre_eq()?, value);
    Ok(())
}

fn assert_set_headphones_pre_eq(
    dir: &PathBuf,
    name: &str,
    value: HeadphonesPreEQ,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "headphones_pre_eq", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_headphones_pre_eq(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_mixer_indicator_brightness(
    dir: &PathBuf,
    name: &str,
    value: MixerIndicatorBrightness,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "indicator_brightness", name)?;
    assert_eq!(sett.get_mixer_indicator_brightness()?, value);
    Ok(())
}

fn assert_set_mixer_indicator_brightness(
    dir: &PathBuf,
    name: &str,
    value: MixerIndicatorBrightness,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "indicator_brightness", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_mixer_indicator_brightness(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_mic_low_cut(dir: &PathBuf, name: &str, value: MicLowCut) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "mic_low_cut", name)?;
    assert_eq!(sett.get_mic_low_cut()?, value);
    Ok(())
}

fn assert_set_mic_low_cut(dir: &PathBuf, name: &str, value: MicLowCut) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "mic_low_cut", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_mic_low_cut(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_midi_button_type(
    dir: &PathBuf,
    name: &str,
    value: MidiButtonType,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "midi_button_type", name)?;
    assert_eq!(sett.get_midi_button_type()?, value);
    Ok(())
}

fn assert_set_midi_button_type(
    dir: &PathBuf,
    name: &str,
    value: MidiButtonType,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "midi_button_type", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_midi_button_type(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_midi_channel(dir: &PathBuf, name: &str, value: MidiChannel) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "midi_channel", name)?;
    assert_eq!(sett.get_midi_channel()?, value);
    Ok(())
}

fn assert_set_midi_channel(dir: &PathBuf, name: &str, value: MidiChannel) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "midi_channel", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_midi_channel(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_talk_over_level(
    dir: &PathBuf,
    name: &str,
    value: TalkOverLevel,
) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "talk_over_level", name)?;
    assert_eq!(sett.get_talk_over_level()?, value);
    Ok(())
}

fn assert_set_talk_over_level(
    dir: &PathBuf,
    name: &str,
    value: TalkOverLevel,
) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "talk_over_level", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_talk_over_level(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

fn assert_get_talk_over_mode(dir: &PathBuf, name: &str, value: TalkOverMode) -> anyhow::Result<()> {
    let sett = load(dir, "DJMMYSETTING.DAT", "talk_over_mode", name)?;
    assert_eq!(sett.get_talk_over_mode()?, value);
    Ok(())
}

fn assert_set_talk_over_mode(dir: &PathBuf, name: &str, value: TalkOverMode) -> anyhow::Result<()> {
    let in_file = path(dir, "DJMMYSETTING.DAT", "talk_over_mode", name);
    let out_file = dir.parent().unwrap().join("out").join("DJMMYSETTING.DAT");
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_talk_over_mode(value)?;
    sett.dump_copy(&out_file)?;
    assert_file(&out_file, &in_file);
    Ok(())
}

// ---- MySetting ----------------------------------------------------------------------------------

#[test]
fn test_get_mysetting() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let dir = root.join("mysetting");

    assert_get_auto_cue(&dir, "on", AutoCue::On)?;
    assert_get_auto_cue(&dir, "off", AutoCue::Off)?;
    assert_get_auto_cue_level(&dir, "memory", AutoCueLevel::Memory)?;
    assert_get_auto_cue_level(&dir, "minus_36db", AutoCueLevel::Minus36dB)?;
    assert_get_auto_cue_level(&dir, "minus_42db", AutoCueLevel::Minus42dB)?;
    assert_get_auto_cue_level(&dir, "minus_48db", AutoCueLevel::Minus48dB)?;
    assert_get_auto_cue_level(&dir, "minus_54db", AutoCueLevel::Minus54dB)?;
    assert_get_auto_cue_level(&dir, "minus_60db", AutoCueLevel::Minus60dB)?;
    assert_get_auto_cue_level(&dir, "minus_66db", AutoCueLevel::Minus66dB)?;
    assert_get_auto_cue_level(&dir, "minus_72db", AutoCueLevel::Minus72dB)?;
    assert_get_auto_cue_level(&dir, "minus_78db", AutoCueLevel::Minus78dB)?;
    assert_get_disc_slot_illumination(&dir, "bright", DiscSlotIllumination::Bright)?;
    assert_get_disc_slot_illumination(&dir, "dark", DiscSlotIllumination::Dark)?;
    assert_get_disc_slot_illumination(&dir, "off", DiscSlotIllumination::Off)?;
    assert_get_eject_lock(&dir, "lock", EjectLock::Lock)?;
    assert_get_eject_lock(&dir, "unlock", EjectLock::Unlock)?;
    assert_get_hotcue_autoload(&dir, "off", HotCueAutoLoad::Off)?;
    assert_get_hotcue_autoload(&dir, "on", HotCueAutoLoad::On)?;
    assert_get_hotcue_autoload(&dir, "rekordbox", HotCueAutoLoad::RekordboxSetting)?;
    assert_get_hotcue_color(&dir, "off", HotCueColor::Off)?;
    assert_get_hotcue_color(&dir, "on", HotCueColor::On)?;
    assert_get_jog_mode(&dir, "cdj", JogMode::CDJ)?;
    assert_get_jog_mode(&dir, "vinyl", JogMode::Vinyl)?;
    assert_get_jog_ring_brightness(&dir, "bright", JogRingBrightness::Bright)?;
    assert_get_jog_ring_brightness(&dir, "dark", JogRingBrightness::Dark)?;
    assert_get_jog_ring_brightness(&dir, "off", JogRingBrightness::Off)?;
    assert_get_jog_ring_indicator(&dir, "off", JogRingIndicator::Off)?;
    assert_get_jog_ring_indicator(&dir, "on", JogRingIndicator::On)?;
    assert_get_language(&dir, "chinese_simplified", Language::ChineseSimplified)?;
    assert_get_language(&dir, "chinese_traditional", Language::ChineseTraditional)?;
    assert_get_language(&dir, "czech", Language::Czech)?;
    assert_get_language(&dir, "danish", Language::Danish)?;
    assert_get_language(&dir, "dutch", Language::Dutch)?;
    assert_get_language(&dir, "english", Language::English)?;
    assert_get_language(&dir, "french", Language::French)?;
    assert_get_language(&dir, "german", Language::German)?;
    assert_get_language(&dir, "greek", Language::Greek)?;
    assert_get_language(&dir, "hungarian", Language::Hungarian)?;
    assert_get_language(&dir, "italian", Language::Italian)?;
    assert_get_language(&dir, "japanese", Language::Japanese)?;
    assert_get_language(&dir, "korean", Language::Korean)?;
    assert_get_language(&dir, "portuguese", Language::Portuguese)?;
    assert_get_language(&dir, "russian", Language::Russian)?;
    assert_get_language(&dir, "spanish", Language::Spanish)?;
    assert_get_language(&dir, "swedish", Language::Swedish)?;
    assert_get_language(&dir, "turkish", Language::Turkish)?;
    assert_get_lcd_brightness(&dir, "five", LCDBrightness::Five)?;
    assert_get_lcd_brightness(&dir, "four", LCDBrightness::Four)?;
    assert_get_lcd_brightness(&dir, "three", LCDBrightness::Three)?;
    assert_get_lcd_brightness(&dir, "two", LCDBrightness::Two)?;
    assert_get_lcd_brightness(&dir, "one", LCDBrightness::One)?;
    assert_get_master_tempo(&dir, "off", MasterTempo::Off)?;
    assert_get_master_tempo(&dir, "on", MasterTempo::On)?;
    assert_get_needle_lock(&dir, "lock", NeedleLock::Lock)?;
    assert_get_needle_lock(&dir, "unlock", NeedleLock::Unlock)?;
    assert_get_on_air_display(&dir, "off", OnAirDisplay::Off)?;
    assert_get_on_air_display(&dir, "on", OnAirDisplay::On)?;
    assert_get_phase_meter(&dir, "type1", PhaseMeter::Type1)?;
    assert_get_phase_meter(&dir, "type2", PhaseMeter::Type2)?;
    assert_get_play_mode(&dir, "continue", PlayMode::Continue)?;
    assert_get_play_mode(&dir, "single", PlayMode::Single)?;
    assert_get_quantize(&dir, "off", Quantize::Off)?;
    assert_get_quantize(&dir, "on", Quantize::On)?;
    assert_get_quantize_beat_value(&dir, "eighth", QuantizeBeatValue::EighthBeat)?;
    assert_get_quantize_beat_value(&dir, "quarter", QuantizeBeatValue::QuarterBeat)?;
    assert_get_quantize_beat_value(&dir, "half", QuantizeBeatValue::HalfBeat)?;
    assert_get_quantize_beat_value(&dir, "one", QuantizeBeatValue::FullBeat)?;
    assert_get_slip_flashing(&dir, "off", SlipFlashing::Off)?;
    assert_get_slip_flashing(&dir, "on", SlipFlashing::On)?;
    assert_get_sync(&dir, "off", Sync::Off)?;
    assert_get_sync(&dir, "on", Sync::On)?;
    assert_get_tempo_range(&dir, "wide", TempoRange::Wide)?;
    assert_get_tempo_range(&dir, "sixteen", TempoRange::SixteenPercent)?;
    assert_get_tempo_range(&dir, "ten", TempoRange::TenPercent)?;
    assert_get_tempo_range(&dir, "six", TempoRange::SixPercent)?;
    assert_get_time_mode(&dir, "elapsed", TimeMode::Elapsed)?;
    assert_get_time_mode(&dir, "remain", TimeMode::Remain)?;

    Ok(())
}

#[test]
fn test_set_mysetting() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let name = "MYSETTING.DAT";
    let dir = root.join("mysetting");
    let out_file = root.join("out").join(name);
    std::fs::create_dir_all(out_file.parent().unwrap())?;

    assert_set_auto_cue(&dir, "on", AutoCue::On)?;
    assert_set_auto_cue(&dir, "off", AutoCue::Off)?;
    assert_set_auto_cue_level(&dir, "memory", AutoCueLevel::Memory)?;
    assert_set_auto_cue_level(&dir, "minus_36db", AutoCueLevel::Minus36dB)?;
    assert_set_auto_cue_level(&dir, "minus_42db", AutoCueLevel::Minus42dB)?;
    assert_set_auto_cue_level(&dir, "minus_48db", AutoCueLevel::Minus48dB)?;
    assert_set_auto_cue_level(&dir, "minus_54db", AutoCueLevel::Minus54dB)?;
    assert_set_auto_cue_level(&dir, "minus_60db", AutoCueLevel::Minus60dB)?;
    assert_set_auto_cue_level(&dir, "minus_66db", AutoCueLevel::Minus66dB)?;
    assert_set_auto_cue_level(&dir, "minus_72db", AutoCueLevel::Minus72dB)?;
    assert_set_auto_cue_level(&dir, "minus_78db", AutoCueLevel::Minus78dB)?;
    assert_set_disc_slot_illumination(&dir, "bright", DiscSlotIllumination::Bright)?;
    assert_set_disc_slot_illumination(&dir, "dark", DiscSlotIllumination::Dark)?;
    assert_set_disc_slot_illumination(&dir, "off", DiscSlotIllumination::Off)?;
    assert_set_eject_lock(&dir, "lock", EjectLock::Lock)?;
    assert_set_eject_lock(&dir, "unlock", EjectLock::Unlock)?;
    assert_set_hotcue_autoload(&dir, "off", HotCueAutoLoad::Off)?;
    assert_set_hotcue_autoload(&dir, "on", HotCueAutoLoad::On)?;
    assert_set_hotcue_autoload(&dir, "rekordbox", HotCueAutoLoad::RekordboxSetting)?;
    assert_set_hotcue_color(&dir, "off", HotCueColor::Off)?;
    assert_set_hotcue_color(&dir, "on", HotCueColor::On)?;
    assert_set_jog_mode(&dir, "cdj", JogMode::CDJ)?;
    assert_set_jog_mode(&dir, "vinyl", JogMode::Vinyl)?;
    assert_set_jog_ring_brightness(&dir, "bright", JogRingBrightness::Bright)?;
    assert_set_jog_ring_brightness(&dir, "dark", JogRingBrightness::Dark)?;
    assert_set_jog_ring_brightness(&dir, "off", JogRingBrightness::Off)?;
    assert_set_jog_ring_indicator(&dir, "off", JogRingIndicator::Off)?;
    assert_set_jog_ring_indicator(&dir, "on", JogRingIndicator::On)?;
    assert_set_language(&dir, "chinese_simplified", Language::ChineseSimplified)?;
    assert_set_language(&dir, "chinese_traditional", Language::ChineseTraditional)?;
    assert_set_language(&dir, "czech", Language::Czech)?;
    assert_set_language(&dir, "danish", Language::Danish)?;
    assert_set_language(&dir, "dutch", Language::Dutch)?;
    assert_set_language(&dir, "english", Language::English)?;
    assert_set_language(&dir, "french", Language::French)?;
    assert_set_language(&dir, "german", Language::German)?;
    assert_set_language(&dir, "greek", Language::Greek)?;
    assert_set_language(&dir, "hungarian", Language::Hungarian)?;
    assert_set_language(&dir, "italian", Language::Italian)?;
    assert_set_language(&dir, "japanese", Language::Japanese)?;
    assert_set_language(&dir, "korean", Language::Korean)?;
    assert_set_language(&dir, "portuguese", Language::Portuguese)?;
    assert_set_language(&dir, "russian", Language::Russian)?;
    assert_set_language(&dir, "spanish", Language::Spanish)?;
    assert_set_language(&dir, "swedish", Language::Swedish)?;
    assert_set_language(&dir, "turkish", Language::Turkish)?;
    assert_set_lcd_brightness(&dir, "five", LCDBrightness::Five)?;
    assert_set_lcd_brightness(&dir, "four", LCDBrightness::Four)?;
    assert_set_lcd_brightness(&dir, "three", LCDBrightness::Three)?;
    assert_set_lcd_brightness(&dir, "two", LCDBrightness::Two)?;
    assert_set_lcd_brightness(&dir, "one", LCDBrightness::One)?;
    assert_set_master_tempo(&dir, "off", MasterTempo::Off)?;
    assert_set_master_tempo(&dir, "on", MasterTempo::On)?;
    assert_set_needle_lock(&dir, "lock", NeedleLock::Lock)?;
    assert_set_needle_lock(&dir, "unlock", NeedleLock::Unlock)?;
    assert_set_on_air_display(&dir, "off", OnAirDisplay::Off)?;
    assert_set_on_air_display(&dir, "on", OnAirDisplay::On)?;
    assert_set_phase_meter(&dir, "type1", PhaseMeter::Type1)?;
    assert_set_phase_meter(&dir, "type2", PhaseMeter::Type2)?;
    assert_set_play_mode(&dir, "continue", PlayMode::Continue)?;
    assert_set_play_mode(&dir, "single", PlayMode::Single)?;
    assert_set_quantize(&dir, "off", Quantize::Off)?;
    assert_set_quantize(&dir, "on", Quantize::On)?;
    assert_set_quantize_beat_value(&dir, "eighth", QuantizeBeatValue::EighthBeat)?;
    assert_set_quantize_beat_value(&dir, "quarter", QuantizeBeatValue::QuarterBeat)?;
    assert_set_quantize_beat_value(&dir, "half", QuantizeBeatValue::HalfBeat)?;
    assert_set_quantize_beat_value(&dir, "one", QuantizeBeatValue::FullBeat)?;
    assert_set_slip_flashing(&dir, "off", SlipFlashing::Off)?;
    assert_set_slip_flashing(&dir, "on", SlipFlashing::On)?;
    assert_set_sync(&dir, "off", Sync::Off)?;
    assert_set_sync(&dir, "on", Sync::On)?;
    assert_set_tempo_range(&dir, "wide", TempoRange::Wide)?;
    assert_set_tempo_range(&dir, "sixteen", TempoRange::SixteenPercent)?;
    assert_set_tempo_range(&dir, "ten", TempoRange::TenPercent)?;
    assert_set_tempo_range(&dir, "six", TempoRange::SixPercent)?;
    assert_set_time_mode(&dir, "elapsed", TimeMode::Elapsed)?;
    assert_set_time_mode(&dir, "remain", TimeMode::Remain)?;

    Ok(())
}

#[test]
fn test_mysetting_defaults() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let sett = Setting::load(root.join("MYSETTING.DAT"))?;
    let default = Setting::new_mysetting("MYSETTING.DAT")?;

    assert_eq!(sett.get_auto_cue()?, default.get_auto_cue()?);
    assert_eq!(sett.get_auto_cue_level()?, default.get_auto_cue_level()?);
    assert_eq!(
        sett.get_disc_slot_illumination()?,
        default.get_disc_slot_illumination()?
    );
    assert_eq!(sett.get_eject_lock()?, default.get_eject_lock()?);
    assert_eq!(sett.get_hotcue_autoload()?, default.get_hotcue_autoload()?);
    assert_eq!(sett.get_hotcue_color()?, default.get_hotcue_color()?);
    assert_eq!(sett.get_jog_mode()?, default.get_jog_mode()?);
    assert_eq!(
        sett.get_jog_ring_brightness()?,
        default.get_jog_ring_brightness()?
    );
    assert_eq!(
        sett.get_jog_ring_indicator()?,
        default.get_jog_ring_indicator()?
    );
    assert_eq!(sett.get_language()?, default.get_language()?);
    assert_eq!(sett.get_lcd_brightness()?, default.get_lcd_brightness()?);
    assert_eq!(sett.get_master_tempo()?, default.get_master_tempo()?);
    assert_eq!(sett.get_needle_lock()?, default.get_needle_lock()?);
    assert_eq!(sett.get_on_air_display()?, default.get_on_air_display()?);
    assert_eq!(sett.get_phase_meter()?, default.get_phase_meter()?);
    assert_eq!(sett.get_play_mode()?, default.get_play_mode()?);
    assert_eq!(sett.get_quantize()?, default.get_quantize()?);
    assert_eq!(
        sett.get_quantize_beat_value()?,
        default.get_quantize_beat_value()?
    );
    assert_eq!(sett.get_slip_flashing()?, default.get_slip_flashing()?);
    assert_eq!(sett.get_sync()?, default.get_sync()?);
    assert_eq!(sett.get_tempo_range()?, default.get_tempo_range()?);
    assert_eq!(sett.get_time_mode()?, default.get_time_mode()?);

    Ok(())
}

// ---- MySetting2 ---------------------------------------------------------------------------------

#[test]
fn test_get_mysetting2() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let dir = root.join("mysetting2");

    assert_get_beat_jump_beat_value(&dir, "half", BeatJumpBeatValue::HalfBeat)?;
    assert_get_beat_jump_beat_value(&dir, "one", BeatJumpBeatValue::OneBeat)?;
    assert_get_beat_jump_beat_value(&dir, "two", BeatJumpBeatValue::TwoBeat)?;
    assert_get_beat_jump_beat_value(&dir, "four", BeatJumpBeatValue::FourBeat)?;
    assert_get_beat_jump_beat_value(&dir, "eight", BeatJumpBeatValue::EightBeat)?;
    assert_get_beat_jump_beat_value(&dir, "sixteen", BeatJumpBeatValue::SixteenBeat)?;
    assert_get_beat_jump_beat_value(&dir, "thirtytwo", BeatJumpBeatValue::ThirtytwoBeat)?;
    assert_get_beat_jump_beat_value(&dir, "sixtyfour", BeatJumpBeatValue::SixtyfourBeat)?;
    assert_get_jog_display_mode(&dir, "artwork", JogDisplayMode::Artwork)?;
    assert_get_jog_display_mode(&dir, "auto", JogDisplayMode::Auto)?;
    assert_get_jog_display_mode(&dir, "info", JogDisplayMode::Info)?;
    assert_get_jog_display_mode(&dir, "simple", JogDisplayMode::Simple)?;
    assert_get_jog_lcd_brightness(&dir, "one", JogLCDBrightness::One)?;
    assert_get_jog_lcd_brightness(&dir, "two", JogLCDBrightness::Two)?;
    assert_get_jog_lcd_brightness(&dir, "three", JogLCDBrightness::Three)?;
    assert_get_jog_lcd_brightness(&dir, "four", JogLCDBrightness::Four)?;
    assert_get_jog_lcd_brightness(&dir, "five", JogLCDBrightness::Five)?;
    assert_get_pad_button_brightness(&dir, "one", PadButtonBrightness::One)?;
    assert_get_pad_button_brightness(&dir, "two", PadButtonBrightness::Two)?;
    assert_get_pad_button_brightness(&dir, "three", PadButtonBrightness::Three)?;
    assert_get_pad_button_brightness(&dir, "four", PadButtonBrightness::Four)?;
    assert_get_vinyl_speed_adjust(&dir, "release", VinylSpeedAdjust::Release)?;
    assert_get_vinyl_speed_adjust(&dir, "touch", VinylSpeedAdjust::Touch)?;
    assert_get_vinyl_speed_adjust(&dir, "touch_release", VinylSpeedAdjust::TouchRelease)?;
    assert_get_waveform(&dir, "phase_meter", Waveform::PhaseMeter)?;
    assert_get_waveform(&dir, "waveform", Waveform::Waveform)?;
    assert_get_waveform_divisions(&dir, "phrase", WaveformDivisions::Phrase)?;
    assert_get_waveform_divisions(&dir, "time_scale", WaveformDivisions::TimeScale)?;

    Ok(())
}

#[test]
fn test_set_mysetting2() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let name = "MYSETTING2.DAT";
    let dir = root.join("mysetting2");
    let out_file = root.join("out").join(name);
    std::fs::create_dir_all(out_file.parent().unwrap())?;

    assert_set_beat_jump_beat_value(&dir, "half", BeatJumpBeatValue::HalfBeat)?;
    assert_set_beat_jump_beat_value(&dir, "one", BeatJumpBeatValue::OneBeat)?;
    assert_set_beat_jump_beat_value(&dir, "two", BeatJumpBeatValue::TwoBeat)?;
    assert_set_beat_jump_beat_value(&dir, "four", BeatJumpBeatValue::FourBeat)?;
    assert_set_beat_jump_beat_value(&dir, "eight", BeatJumpBeatValue::EightBeat)?;
    assert_set_beat_jump_beat_value(&dir, "sixteen", BeatJumpBeatValue::SixteenBeat)?;
    assert_set_beat_jump_beat_value(&dir, "thirtytwo", BeatJumpBeatValue::ThirtytwoBeat)?;
    assert_set_beat_jump_beat_value(&dir, "sixtyfour", BeatJumpBeatValue::SixtyfourBeat)?;
    assert_set_jog_display_mode(&dir, "artwork", JogDisplayMode::Artwork)?;
    assert_set_jog_display_mode(&dir, "auto", JogDisplayMode::Auto)?;
    assert_set_jog_display_mode(&dir, "info", JogDisplayMode::Info)?;
    assert_set_jog_display_mode(&dir, "simple", JogDisplayMode::Simple)?;
    assert_set_jog_lcd_brightness(&dir, "one", JogLCDBrightness::One)?;
    assert_set_jog_lcd_brightness(&dir, "two", JogLCDBrightness::Two)?;
    assert_set_jog_lcd_brightness(&dir, "three", JogLCDBrightness::Three)?;
    assert_set_jog_lcd_brightness(&dir, "four", JogLCDBrightness::Four)?;
    assert_set_jog_lcd_brightness(&dir, "five", JogLCDBrightness::Five)?;
    assert_set_pad_button_brightness(&dir, "one", PadButtonBrightness::One)?;
    assert_set_pad_button_brightness(&dir, "two", PadButtonBrightness::Two)?;
    assert_set_pad_button_brightness(&dir, "three", PadButtonBrightness::Three)?;
    assert_set_pad_button_brightness(&dir, "four", PadButtonBrightness::Four)?;
    assert_set_vinyl_speed_adjust(&dir, "release", VinylSpeedAdjust::Release)?;
    assert_set_vinyl_speed_adjust(&dir, "touch", VinylSpeedAdjust::Touch)?;
    assert_set_vinyl_speed_adjust(&dir, "touch_release", VinylSpeedAdjust::TouchRelease)?;
    assert_set_waveform(&dir, "phase_meter", Waveform::PhaseMeter)?;
    assert_set_waveform(&dir, "waveform", Waveform::Waveform)?;
    assert_set_waveform_divisions(&dir, "phrase", WaveformDivisions::Phrase)?;
    assert_set_waveform_divisions(&dir, "time_scale", WaveformDivisions::TimeScale)?;

    Ok(())
}

#[test]
fn test_mysetting2_defaults() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let sett = Setting::load(root.join("MYSETTING2.DAT"))?;
    let default = Setting::new_mysetting2("MYSETTING2.DAT")?;

    assert_eq!(
        sett.get_beat_jump_beat_value()?,
        default.get_beat_jump_beat_value()?
    );
    assert_eq!(
        sett.get_jog_display_mode()?,
        default.get_jog_display_mode()?
    );
    assert_eq!(
        sett.get_jog_lcd_brightness()?,
        default.get_jog_lcd_brightness()?
    );
    assert_eq!(
        sett.get_pad_button_brightness()?,
        default.get_pad_button_brightness()?
    );
    assert_eq!(
        sett.get_vinyl_speed_adjust()?,
        default.get_vinyl_speed_adjust()?
    );
    assert_eq!(sett.get_waveform()?, default.get_waveform()?);
    assert_eq!(
        sett.get_waveform_divisions()?,
        default.get_waveform_divisions()?
    );

    Ok(())
}

// ---- DjmdMySetting ------------------------------------------------------------------------------

#[test]
fn test_get_djmdmysetting() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let dir = root.join("djmmysetting");

    assert_get_beat_fx_quantize(&dir, "off", BeatFXQuantize::Off)?;
    assert_get_beat_fx_quantize(&dir, "on", BeatFXQuantize::On)?;
    assert_get_channel_fader_curve(&dir, "linear", ChannelFaderCurve::Linear)?;
    assert_get_channel_fader_curve(&dir, "steep_bottom", ChannelFaderCurve::SteepBottom)?;
    assert_get_channel_fader_curve(&dir, "steep_top", ChannelFaderCurve::SteepTop)?;
    assert_get_channel_fader_curve_long_fader(&dir, "linear", ChannelFaderCurveLongFader::Linear)?;
    assert_get_channel_fader_curve_long_fader(
        &dir,
        "exponential",
        ChannelFaderCurveLongFader::Exponential,
    )?;
    assert_get_channel_fader_curve_long_fader(&dir, "smooth", ChannelFaderCurveLongFader::Smooth)?;
    assert_get_cross_fader_curve(&dir, "constant", CrossfaderCurve::ConstantPower)?;
    assert_get_cross_fader_curve(&dir, "fast_cut", CrossfaderCurve::FastCut)?;
    assert_get_cross_fader_curve(&dir, "slow_cut", CrossfaderCurve::SlowCut)?;
    assert_get_mixer_display_brightness(&dir, "one", MixerDisplayBrightness::One)?;
    assert_get_mixer_display_brightness(&dir, "two", MixerDisplayBrightness::Two)?;
    assert_get_mixer_display_brightness(&dir, "three", MixerDisplayBrightness::Three)?;
    assert_get_mixer_display_brightness(&dir, "four", MixerDisplayBrightness::Four)?;
    assert_get_mixer_display_brightness(&dir, "five", MixerDisplayBrightness::Five)?;
    assert_get_mixer_display_brightness(&dir, "white", MixerDisplayBrightness::White)?;
    assert_get_headphones_mono_split(&dir, "mono_split", HeadphonesMonoSplit::MonoSplit)?;
    assert_get_headphones_mono_split(&dir, "stereo", HeadphonesMonoSplit::Stereo)?;
    assert_get_headphones_pre_eq(&dir, "pre_eq", HeadphonesPreEQ::PreEQ)?;
    assert_get_headphones_pre_eq(&dir, "post_eq", HeadphonesPreEQ::PostEQ)?;
    assert_get_mixer_indicator_brightness(&dir, "one", MixerIndicatorBrightness::One)?;
    assert_get_mixer_indicator_brightness(&dir, "two", MixerIndicatorBrightness::Two)?;
    assert_get_mixer_indicator_brightness(&dir, "three", MixerIndicatorBrightness::Three)?;
    assert_get_mic_low_cut(&dir, "off", MicLowCut::Off)?;
    assert_get_mic_low_cut(&dir, "on", MicLowCut::On)?;
    assert_get_midi_button_type(&dir, "toggle", MidiButtonType::Toggle)?;
    assert_get_midi_button_type(&dir, "trigger", MidiButtonType::Trigger)?;
    assert_get_midi_channel(&dir, "one", MidiChannel::One)?;
    assert_get_midi_channel(&dir, "two", MidiChannel::Two)?;
    assert_get_midi_channel(&dir, "three", MidiChannel::Three)?;
    assert_get_midi_channel(&dir, "four", MidiChannel::Four)?;
    assert_get_midi_channel(&dir, "five", MidiChannel::Five)?;
    assert_get_midi_channel(&dir, "six", MidiChannel::Six)?;
    assert_get_midi_channel(&dir, "seven", MidiChannel::Seven)?;
    assert_get_midi_channel(&dir, "eight", MidiChannel::Eight)?;
    assert_get_midi_channel(&dir, "nine", MidiChannel::Nine)?;
    assert_get_midi_channel(&dir, "ten", MidiChannel::Ten)?;
    assert_get_midi_channel(&dir, "eleven", MidiChannel::Eleven)?;
    assert_get_midi_channel(&dir, "twelve", MidiChannel::Twelve)?;
    assert_get_midi_channel(&dir, "thirteen", MidiChannel::Thirteen)?;
    assert_get_midi_channel(&dir, "fourteen", MidiChannel::Fourteen)?;
    assert_get_midi_channel(&dir, "fifteen", MidiChannel::Fifteen)?;
    assert_get_midi_channel(&dir, "sixteen", MidiChannel::Sixteen)?;
    assert_get_talk_over_level(&dir, "minus_6db", TalkOverLevel::Minus6dB)?;
    assert_get_talk_over_level(&dir, "minus_12db", TalkOverLevel::Minus12dB)?;
    assert_get_talk_over_level(&dir, "minus_18db", TalkOverLevel::Minus18dB)?;
    assert_get_talk_over_level(&dir, "minus_24db", TalkOverLevel::Minus24dB)?;
    assert_get_talk_over_mode(&dir, "advanced", TalkOverMode::Advanced)?;
    assert_get_talk_over_mode(&dir, "normal", TalkOverMode::Normal)?;

    Ok(())
}

#[test]
fn test_set_djmdmysetting() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let name = "DJMMYSETTING.DAT";
    let dir = root.join("djmmysetting");
    let out_file = root.join("out").join(name);
    std::fs::create_dir_all(out_file.parent().unwrap())?;

    assert_set_beat_fx_quantize(&dir, "off", BeatFXQuantize::Off)?;
    assert_set_beat_fx_quantize(&dir, "on", BeatFXQuantize::On)?;
    assert_set_channel_fader_curve(&dir, "linear", ChannelFaderCurve::Linear)?;
    assert_set_channel_fader_curve(&dir, "steep_bottom", ChannelFaderCurve::SteepBottom)?;
    assert_set_channel_fader_curve(&dir, "steep_top", ChannelFaderCurve::SteepTop)?;
    assert_set_channel_fader_curve_long_fader(&dir, "linear", ChannelFaderCurveLongFader::Linear)?;
    assert_set_channel_fader_curve_long_fader(
        &dir,
        "exponential",
        ChannelFaderCurveLongFader::Exponential,
    )?;
    assert_set_channel_fader_curve_long_fader(&dir, "smooth", ChannelFaderCurveLongFader::Smooth)?;
    assert_set_cross_fader_curve(&dir, "constant", CrossfaderCurve::ConstantPower)?;
    assert_set_cross_fader_curve(&dir, "fast_cut", CrossfaderCurve::FastCut)?;
    assert_set_cross_fader_curve(&dir, "slow_cut", CrossfaderCurve::SlowCut)?;
    assert_set_mixer_display_brightness(&dir, "one", MixerDisplayBrightness::One)?;
    assert_set_mixer_display_brightness(&dir, "two", MixerDisplayBrightness::Two)?;
    assert_set_mixer_display_brightness(&dir, "three", MixerDisplayBrightness::Three)?;
    assert_set_mixer_display_brightness(&dir, "four", MixerDisplayBrightness::Four)?;
    assert_set_mixer_display_brightness(&dir, "five", MixerDisplayBrightness::Five)?;
    assert_set_mixer_display_brightness(&dir, "white", MixerDisplayBrightness::White)?;
    assert_set_headphones_mono_split(&dir, "mono_split", HeadphonesMonoSplit::MonoSplit)?;
    assert_set_headphones_mono_split(&dir, "stereo", HeadphonesMonoSplit::Stereo)?;
    assert_set_headphones_pre_eq(&dir, "pre_eq", HeadphonesPreEQ::PreEQ)?;
    assert_set_headphones_pre_eq(&dir, "post_eq", HeadphonesPreEQ::PostEQ)?;
    assert_set_mixer_indicator_brightness(&dir, "one", MixerIndicatorBrightness::One)?;
    assert_set_mixer_indicator_brightness(&dir, "two", MixerIndicatorBrightness::Two)?;
    assert_set_mixer_indicator_brightness(&dir, "three", MixerIndicatorBrightness::Three)?;
    assert_set_mic_low_cut(&dir, "off", MicLowCut::Off)?;
    assert_set_mic_low_cut(&dir, "on", MicLowCut::On)?;
    assert_set_midi_button_type(&dir, "toggle", MidiButtonType::Toggle)?;
    assert_set_midi_button_type(&dir, "trigger", MidiButtonType::Trigger)?;
    assert_set_midi_channel(&dir, "one", MidiChannel::One)?;
    assert_set_midi_channel(&dir, "two", MidiChannel::Two)?;
    assert_set_midi_channel(&dir, "three", MidiChannel::Three)?;
    assert_set_midi_channel(&dir, "four", MidiChannel::Four)?;
    assert_set_midi_channel(&dir, "five", MidiChannel::Five)?;
    assert_set_midi_channel(&dir, "six", MidiChannel::Six)?;
    assert_set_midi_channel(&dir, "seven", MidiChannel::Seven)?;
    assert_set_midi_channel(&dir, "eight", MidiChannel::Eight)?;
    assert_set_midi_channel(&dir, "nine", MidiChannel::Nine)?;
    assert_set_midi_channel(&dir, "ten", MidiChannel::Ten)?;
    assert_set_midi_channel(&dir, "eleven", MidiChannel::Eleven)?;
    assert_set_midi_channel(&dir, "twelve", MidiChannel::Twelve)?;
    assert_set_midi_channel(&dir, "thirteen", MidiChannel::Thirteen)?;
    assert_set_midi_channel(&dir, "fourteen", MidiChannel::Fourteen)?;
    assert_set_midi_channel(&dir, "fifteen", MidiChannel::Fifteen)?;
    assert_set_midi_channel(&dir, "sixteen", MidiChannel::Sixteen)?;
    assert_set_talk_over_level(&dir, "minus_6db", TalkOverLevel::Minus6dB)?;
    assert_set_talk_over_level(&dir, "minus_12db", TalkOverLevel::Minus12dB)?;
    assert_set_talk_over_level(&dir, "minus_18db", TalkOverLevel::Minus18dB)?;
    assert_set_talk_over_level(&dir, "minus_24db", TalkOverLevel::Minus24dB)?;
    assert_set_talk_over_mode(&dir, "advanced", TalkOverMode::Advanced)?;
    assert_set_talk_over_mode(&dir, "normal", TalkOverMode::Normal)?;

    Ok(())
}

#[test]
fn test_djmdmysetting_defaults() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let sett = Setting::load(root.join("DJMMYSETTING.DAT"))?;
    let default = Setting::new_djmmysetting("DJMMYSETTING.DAT")?;

    assert_eq!(
        sett.get_beat_fx_quantize()?,
        default.get_beat_fx_quantize()?
    );
    assert_eq!(
        sett.get_channel_fader_curve()?,
        default.get_channel_fader_curve()?
    );
    assert_eq!(
        sett.get_channel_fader_curve_long_fader()?,
        default.get_channel_fader_curve_long_fader()?
    );
    assert_eq!(
        sett.get_crossfader_curve()?,
        default.get_crossfader_curve()?
    );
    assert_eq!(
        sett.get_mixer_display_brightness()?,
        default.get_mixer_display_brightness()?
    );
    assert_eq!(
        sett.get_headphones_mono_split()?,
        default.get_headphones_mono_split()?
    );
    assert_eq!(
        sett.get_headphones_pre_eq()?,
        default.get_headphones_pre_eq()?
    );
    assert_eq!(
        sett.get_mixer_indicator_brightness()?,
        default.get_mixer_indicator_brightness()?
    );
    assert_eq!(sett.get_mic_low_cut()?, default.get_mic_low_cut()?);
    assert_eq!(
        sett.get_midi_button_type()?,
        default.get_midi_button_type()?
    );
    assert_eq!(sett.get_midi_channel()?, default.get_midi_channel()?);
    assert_eq!(sett.get_talk_over_level()?, default.get_talk_over_level()?);
    assert_eq!(sett.get_talk_over_mode()?, default.get_talk_over_mode()?);

    Ok(())
}
