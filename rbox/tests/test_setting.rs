// Author: Dylan Jones
// Date:   2025-08-11

use anyhow::Result;
use rbox::settings::*;
use rstest::*;
use std::path::PathBuf;

mod common;

fn assert_file(sett: &mut Setting, expected: &PathBuf, name: &str) -> Result<()> {
    let tmp_dir = common::create_temp_dir();
    let out_file = tmp_dir.path().join(name);
    sett.dump_copy(&out_file)?;
    let expected_contents = std::fs::read(expected).expect("Failed to read expected file");
    let actual_contents = std::fs::read(out_file).expect("Failed to read actual file");
    assert_eq!(
        expected_contents, actual_contents,
        "File contents do not match"
    );
    Ok(())
}

fn mysetting_path(key: &str, value: &str) -> PathBuf {
    let root = common::settings_dir();
    root.join("mysetting")
        .join(key)
        .join(value)
        .join("MYSETTING.DAT")
}

fn assert_mysetting_file(sett: &mut Setting, expected: &PathBuf) -> Result<()> {
    assert_file(sett, &expected, "MYSETTING.DAT")
}

fn mysetting2_path(key: &str, value: &str) -> PathBuf {
    let root = common::settings_dir();
    root.join("mysetting2")
        .join(key)
        .join(value)
        .join("MYSETTING2.DAT")
}

fn assert_mysetting2_file(sett: &mut Setting, expected: &PathBuf) -> Result<()> {
    assert_file(sett, &expected, "MYSETTING2.DAT")
}

fn djmdmysetting_path(key: &str, value: &str) -> PathBuf {
    let root = common::settings_dir();
    root.join("djmmysetting")
        .join(key)
        .join(value)
        .join("DJMMYSETTING.DAT")
}

fn assert_djmdmysetting_file(sett: &mut Setting, expected: &PathBuf) -> Result<()> {
    assert_file(sett, &expected, "DJMMYSETTING.DAT")
}

fn devmysetting_path(key: &str, value: &str) -> PathBuf {
    let root = common::settings_dir();
    root.join("devsetting")
        .join(key)
        .join(value)
        .join("DEVSETTING.DAT")
}

fn assert_devmysetting_file(sett: &mut Setting, expected: &PathBuf) -> Result<()> {
    assert_file(sett, &expected, "DEVSETTING.DAT")
}

// ---- MySetting ----------------------------------------------------------------------------------

#[rstest]
#[case("on", AutoCue::On)]
#[case("off", AutoCue::Off)]
fn test_get_auto_cue(#[case] name: &str, #[case] value: AutoCue) -> Result<()> {
    let sett = Setting::load(&mysetting_path("auto_cue", name))?;
    assert_eq!(sett.get_auto_cue()?, value);
    Ok(())
}

#[rstest]
#[case("on", AutoCue::On)]
#[case("off", AutoCue::Off)]
fn test_set_auto_cue(#[case] name: &str, #[case] value: AutoCue) -> Result<()> {
    let in_file = mysetting_path("auto_cue", name);
    let mut sett = Setting::load(&in_file)?;
    sett.set_auto_cue(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("memory", AutoCueLevel::Memory)]
#[case("minus_36db", AutoCueLevel::Minus36dB)]
#[case("minus_42db", AutoCueLevel::Minus42dB)]
#[case("minus_48db", AutoCueLevel::Minus48dB)]
#[case("minus_54db", AutoCueLevel::Minus54dB)]
#[case("minus_60db", AutoCueLevel::Minus60dB)]
#[case("minus_66db", AutoCueLevel::Minus66dB)]
#[case("minus_72db", AutoCueLevel::Minus72dB)]
#[case("minus_78db", AutoCueLevel::Minus78dB)]
fn test_get_auto_cue_level(#[case] name: &str, #[case] value: AutoCueLevel) -> Result<()> {
    let sett = Setting::load(&mysetting_path("auto_cue_level", name))?;
    assert_eq!(sett.get_auto_cue_level()?, value);
    Ok(())
}

#[rstest]
#[case("memory", AutoCueLevel::Memory)]
#[case("minus_36db", AutoCueLevel::Minus36dB)]
#[case("minus_42db", AutoCueLevel::Minus42dB)]
#[case("minus_48db", AutoCueLevel::Minus48dB)]
#[case("minus_54db", AutoCueLevel::Minus54dB)]
#[case("minus_60db", AutoCueLevel::Minus60dB)]
#[case("minus_66db", AutoCueLevel::Minus66dB)]
#[case("minus_72db", AutoCueLevel::Minus72dB)]
#[case("minus_78db", AutoCueLevel::Minus78dB)]
fn test_set_auto_cue_level(#[case] name: &str, #[case] value: AutoCueLevel) -> Result<()> {
    let in_file = mysetting_path("auto_cue_level", name);
    let mut sett = Setting::load(&in_file)?;
    sett.set_auto_cue_level(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("bright", DiscSlotIllumination::Bright)]
#[case("dark", DiscSlotIllumination::Dark)]
#[case("off", DiscSlotIllumination::Off)]
fn test_get_disc_slot_illumination(
    #[case] name: &str,
    #[case] value: DiscSlotIllumination,
) -> Result<()> {
    let sett = Setting::load(&mysetting_path("disc_slot_illumination", name))?;
    assert_eq!(sett.get_disc_slot_illumination()?, value);
    Ok(())
}

#[rstest]
#[case("bright", DiscSlotIllumination::Bright)]
#[case("dark", DiscSlotIllumination::Dark)]
#[case("off", DiscSlotIllumination::Off)]
fn test_set_disc_slot_illumination(
    #[case] name: &str,
    #[case] value: DiscSlotIllumination,
) -> Result<()> {
    let in_file = mysetting_path("disc_slot_illumination", name);
    let mut sett = Setting::load(&in_file)?;
    sett.set_disc_slot_illumination(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("lock", EjectLock::Lock)]
#[case("unlock", EjectLock::Unlock)]
fn test_get_eject_lock(#[case] name: &str, #[case] value: EjectLock) -> Result<()> {
    let sett = Setting::load(&mysetting_path("eject_lock", name))?;
    assert_eq!(sett.get_eject_lock()?, value);
    Ok(())
}

#[rstest]
#[case("lock", EjectLock::Lock)]
#[case("unlock", EjectLock::Unlock)]
fn test_set_eject_lock(#[case] name: &str, #[case] value: EjectLock) -> Result<()> {
    let in_file = mysetting_path("eject_lock", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_eject_lock(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("off", HotCueAutoLoad::Off)]
#[case("on", HotCueAutoLoad::On)]
#[case("rekordbox", HotCueAutoLoad::RekordboxSetting)]
fn test_get_hotcue_autoload(#[case] name: &str, #[case] value: HotCueAutoLoad) -> Result<()> {
    let sett = Setting::load(&mysetting_path("hotcue_autoload", name))?;
    assert_eq!(sett.get_hotcue_autoload()?, value);
    Ok(())
}

#[rstest]
#[case("off", HotCueAutoLoad::Off)]
#[case("on", HotCueAutoLoad::On)]
#[case("rekordbox", HotCueAutoLoad::RekordboxSetting)]
fn test_set_hotcue_autoload(#[case] name: &str, #[case] value: HotCueAutoLoad) -> Result<()> {
    let in_file = mysetting_path("hotcue_autoload", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_hotcue_autoload(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("off", HotCueColor::Off)]
#[case("on", HotCueColor::On)]
fn test_get_hotcue_color(#[case] name: &str, #[case] value: HotCueColor) -> Result<()> {
    let sett = Setting::load(&mysetting_path("hotcue_color", name))?;
    assert_eq!(sett.get_hotcue_color()?, value);
    Ok(())
}

#[rstest]
#[case("off", HotCueColor::Off)]
#[case("on", HotCueColor::On)]
fn test_set_hotcue_color(#[case] name: &str, #[case] value: HotCueColor) -> Result<()> {
    let in_file = mysetting_path("hotcue_color", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_hotcue_color(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("cdj", JogMode::CDJ)]
#[case("vinyl", JogMode::Vinyl)]
fn test_get_jog_mode(#[case] name: &str, #[case] value: JogMode) -> Result<()> {
    let sett = Setting::load(&mysetting_path("jog_mode", name))?;
    assert_eq!(sett.get_jog_mode()?, value);
    Ok(())
}

#[rstest]
#[case("cdj", JogMode::CDJ)]
#[case("vinyl", JogMode::Vinyl)]
fn test_set_jog_mode(#[case] name: &str, #[case] value: JogMode) -> Result<()> {
    let in_file = mysetting_path("jog_mode", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_mode(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("bright", JogRingBrightness::Bright)]
#[case("dark", JogRingBrightness::Dark)]
#[case("off", JogRingBrightness::Off)]
fn test_get_jog_ring_brightness(
    #[case] name: &str,
    #[case] value: JogRingBrightness,
) -> Result<()> {
    let sett = Setting::load(&mysetting_path("jog_ring_brightness", name))?;
    assert_eq!(sett.get_jog_ring_brightness()?, value);
    Ok(())
}

#[rstest]
#[case("bright", JogRingBrightness::Bright)]
#[case("dark", JogRingBrightness::Dark)]
#[case("off", JogRingBrightness::Off)]
fn test_set_jog_ring_brightness(
    #[case] name: &str,
    #[case] value: JogRingBrightness,
) -> Result<()> {
    let in_file = mysetting_path("jog_ring_brightness", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_ring_brightness(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("off", JogRingIndicator::Off)]
#[case("on", JogRingIndicator::On)]
fn test_get_jog_ring_indicator(#[case] name: &str, #[case] value: JogRingIndicator) -> Result<()> {
    let sett = Setting::load(&mysetting_path("jog_ring_indicator", name))?;
    assert_eq!(sett.get_jog_ring_indicator()?, value);
    Ok(())
}

#[rstest]
#[case("off", JogRingIndicator::Off)]
#[case("on", JogRingIndicator::On)]
fn test_set_jog_ring_indicator(#[case] name: &str, #[case] value: JogRingIndicator) -> Result<()> {
    let in_file = mysetting_path("jog_ring_indicator", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_ring_indicator(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("chinese_simplified", Language::ChineseSimplified)]
#[case("chinese_traditional", Language::ChineseTraditional)]
#[case("czech", Language::Czech)]
#[case("danish", Language::Danish)]
#[case("dutch", Language::Dutch)]
#[case("english", Language::English)]
#[case("french", Language::French)]
#[case("german", Language::German)]
#[case("greek", Language::Greek)]
#[case("hungarian", Language::Hungarian)]
#[case("italian", Language::Italian)]
#[case("japanese", Language::Japanese)]
#[case("korean", Language::Korean)]
#[case("portuguese", Language::Portuguese)]
#[case("russian", Language::Russian)]
#[case("spanish", Language::Spanish)]
#[case("swedish", Language::Swedish)]
#[case("turkish", Language::Turkish)]
fn test_get_language(#[case] name: &str, #[case] value: Language) -> Result<()> {
    let sett = Setting::load(&mysetting_path("language", name))?;
    assert_eq!(sett.get_language()?, value);
    Ok(())
}

#[rstest]
#[case("chinese_simplified", Language::ChineseSimplified)]
#[case("chinese_traditional", Language::ChineseTraditional)]
#[case("czech", Language::Czech)]
#[case("danish", Language::Danish)]
#[case("dutch", Language::Dutch)]
#[case("english", Language::English)]
#[case("french", Language::French)]
#[case("german", Language::German)]
#[case("greek", Language::Greek)]
#[case("hungarian", Language::Hungarian)]
#[case("italian", Language::Italian)]
#[case("japanese", Language::Japanese)]
#[case("korean", Language::Korean)]
#[case("portuguese", Language::Portuguese)]
#[case("russian", Language::Russian)]
#[case("spanish", Language::Spanish)]
#[case("swedish", Language::Swedish)]
#[case("turkish", Language::Turkish)]
fn test_set_language(#[case] name: &str, #[case] value: Language) -> Result<()> {
    let in_file = mysetting_path("language", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_language(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("five", LCDBrightness::Five)]
#[case("four", LCDBrightness::Four)]
#[case("three", LCDBrightness::Three)]
#[case("two", LCDBrightness::Two)]
#[case("one", LCDBrightness::One)]
fn test_get_lcd_brightness(#[case] name: &str, #[case] value: LCDBrightness) -> Result<()> {
    let sett = Setting::load(&mysetting_path("lcd_brightness", name))?;
    assert_eq!(sett.get_lcd_brightness()?, value);
    Ok(())
}

#[rstest]
#[case("five", LCDBrightness::Five)]
#[case("four", LCDBrightness::Four)]
#[case("three", LCDBrightness::Three)]
#[case("two", LCDBrightness::Two)]
#[case("one", LCDBrightness::One)]
fn test_set_lcd_brightness(#[case] name: &str, #[case] value: LCDBrightness) -> Result<()> {
    let in_file = mysetting_path("lcd_brightness", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_lcd_brightness(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("off", MasterTempo::Off)]
#[case("on", MasterTempo::On)]
fn test_get_master_tempo(#[case] name: &str, #[case] value: MasterTempo) -> Result<()> {
    let sett = Setting::load(&mysetting_path("master_tempo", name))?;
    assert_eq!(sett.get_master_tempo()?, value);
    Ok(())
}

#[rstest]
#[case("off", MasterTempo::Off)]
#[case("on", MasterTempo::On)]
fn test_set_master_tempo(#[case] name: &str, #[case] value: MasterTempo) -> Result<()> {
    let in_file = mysetting_path("master_tempo", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_master_tempo(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("lock", NeedleLock::Lock)]
#[case("unlock", NeedleLock::Unlock)]
fn test_get_needle_lock(#[case] name: &str, #[case] value: NeedleLock) -> Result<()> {
    let sett = Setting::load(&mysetting_path("needle_lock", name))?;
    assert_eq!(sett.get_needle_lock()?, value);
    Ok(())
}

#[rstest]
#[case("lock", NeedleLock::Lock)]
#[case("unlock", NeedleLock::Unlock)]
fn test_set_needle_lock(#[case] name: &str, #[case] value: NeedleLock) -> Result<()> {
    let in_file = mysetting_path("needle_lock", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_needle_lock(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("off", OnAirDisplay::Off)]
#[case("on", OnAirDisplay::On)]
fn test_get_on_air_display(#[case] name: &str, #[case] value: OnAirDisplay) -> Result<()> {
    let sett = Setting::load(&mysetting_path("on_air_display", name))?;
    assert_eq!(sett.get_on_air_display()?, value);
    Ok(())
}

#[rstest]
#[case("off", OnAirDisplay::Off)]
#[case("on", OnAirDisplay::On)]
fn test_set_on_air_display(#[case] name: &str, #[case] value: OnAirDisplay) -> Result<()> {
    let in_file = mysetting_path("on_air_display", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_on_air_display(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("type1", PhaseMeter::Type1)]
#[case("type2", PhaseMeter::Type2)]
fn test_get_phase_meter(#[case] name: &str, #[case] value: PhaseMeter) -> Result<()> {
    let sett = Setting::load(&mysetting_path("phase_meter", name))?;
    assert_eq!(sett.get_phase_meter()?, value);
    Ok(())
}

#[rstest]
#[case("type1", PhaseMeter::Type1)]
#[case("type2", PhaseMeter::Type2)]
fn test_set_phase_meter(#[case] name: &str, #[case] value: PhaseMeter) -> Result<()> {
    let in_file = mysetting_path("phase_meter", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_phase_meter(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("continue", PlayMode::Continue)]
#[case("single", PlayMode::Single)]
fn test_get_play_mode(#[case] name: &str, #[case] value: PlayMode) -> Result<()> {
    let sett = Setting::load(&mysetting_path("play_mode", name))?;
    assert_eq!(sett.get_play_mode()?, value);
    Ok(())
}

#[rstest]
#[case("continue", PlayMode::Continue)]
#[case("single", PlayMode::Single)]
fn test_set_play_mode(#[case] name: &str, #[case] value: PlayMode) -> Result<()> {
    let in_file = mysetting_path("play_mode", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_play_mode(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("off", Quantize::Off)]
#[case("on", Quantize::On)]
fn test_get_quantize(#[case] name: &str, #[case] value: Quantize) -> Result<()> {
    let sett = Setting::load(&mysetting_path("quantize", name))?;
    assert_eq!(sett.get_quantize()?, value);
    Ok(())
}

#[rstest]
#[case("off", Quantize::Off)]
#[case("on", Quantize::On)]
fn test_set_quantize(#[case] name: &str, #[case] value: Quantize) -> Result<()> {
    let in_file = mysetting_path("quantize", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_quantize(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("eighth", QuantizeBeatValue::EighthBeat)]
#[case("quarter", QuantizeBeatValue::QuarterBeat)]
#[case("half", QuantizeBeatValue::HalfBeat)]
#[case("one", QuantizeBeatValue::FullBeat)]
fn test_get_quantize_beat_value(
    #[case] name: &str,
    #[case] value: QuantizeBeatValue,
) -> Result<()> {
    let sett = Setting::load(&mysetting_path("quantize_beat_value", name))?;
    assert_eq!(sett.get_quantize_beat_value()?, value);
    Ok(())
}

#[rstest]
#[case("eighth", QuantizeBeatValue::EighthBeat)]
#[case("quarter", QuantizeBeatValue::QuarterBeat)]
#[case("half", QuantizeBeatValue::HalfBeat)]
#[case("one", QuantizeBeatValue::FullBeat)]
fn test_set_quantize_beat_value(
    #[case] name: &str,
    #[case] value: QuantizeBeatValue,
) -> Result<()> {
    let in_file = mysetting_path("quantize_beat_value", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_quantize_beat_value(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("off", SlipFlashing::Off)]
#[case("on", SlipFlashing::On)]
fn test_get_slip_flashing(#[case] name: &str, #[case] value: SlipFlashing) -> Result<()> {
    let sett = Setting::load(&mysetting_path("slip_flashing", name))?;
    assert_eq!(sett.get_slip_flashing()?, value);
    Ok(())
}

#[rstest]
#[case("off", SlipFlashing::Off)]
#[case("on", SlipFlashing::On)]
fn test_set_slip_flashing(#[case] name: &str, #[case] value: SlipFlashing) -> Result<()> {
    let in_file = mysetting_path("slip_flashing", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_slip_flashing(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("off", Sync::Off)]
#[case("on", Sync::On)]
fn test_get_sync(#[case] name: &str, #[case] value: Sync) -> Result<()> {
    let sett = Setting::load(&mysetting_path("sync", name))?;
    assert_eq!(sett.get_sync()?, value);
    Ok(())
}

#[rstest]
#[case("off", Sync::Off)]
#[case("on", Sync::On)]
fn test_set_sync(#[case] name: &str, #[case] value: Sync) -> Result<()> {
    let in_file = mysetting_path("sync", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_sync(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("wide", TempoRange::Wide)]
#[case("sixteen", TempoRange::SixteenPercent)]
#[case("ten", TempoRange::TenPercent)]
#[case("six", TempoRange::SixPercent)]
fn test_get_tempo_range(#[case] name: &str, #[case] value: TempoRange) -> Result<()> {
    let sett = Setting::load(&mysetting_path("tempo_range", name))?;
    assert_eq!(sett.get_tempo_range()?, value);
    Ok(())
}

#[rstest]
#[case("wide", TempoRange::Wide)]
#[case("sixteen", TempoRange::SixteenPercent)]
#[case("ten", TempoRange::TenPercent)]
#[case("six", TempoRange::SixPercent)]
fn test_set_tempo_range(#[case] name: &str, #[case] value: TempoRange) -> Result<()> {
    let in_file = mysetting_path("tempo_range", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_tempo_range(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("elapsed", TimeMode::Elapsed)]
#[case("remain", TimeMode::Remain)]
fn test_get_time_mode(#[case] name: &str, #[case] value: TimeMode) -> Result<()> {
    let sett = Setting::load(&mysetting_path("time_mode", name))?;
    assert_eq!(sett.get_time_mode()?, value);
    Ok(())
}

#[rstest]
#[case("elapsed", TimeMode::Elapsed)]
#[case("remain", TimeMode::Remain)]
fn test_set_time_mode(#[case] name: &str, #[case] value: TimeMode) -> Result<()> {
    let in_file = mysetting_path("time_mode", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_time_mode(value)?;
    assert_mysetting_file(&mut sett, &in_file)
}

#[test]
fn test_mysetting_defaults() -> Result<()> {
    let root = common::settings_dir();
    let sett = Setting::load(root.join("MYSETTING.DAT"))?;
    let default = Setting::new_mysetting("MYSETTING.DAT")?;
    assert_eq!(sett.data.content, default.data.content);
    Ok(())
}

// ---- MySetting2 ---------------------------------------------------------------------------------

#[rstest]
#[case("half", BeatJumpBeatValue::HalfBeat)]
#[case("one", BeatJumpBeatValue::OneBeat)]
#[case("two", BeatJumpBeatValue::TwoBeat)]
#[case("four", BeatJumpBeatValue::FourBeat)]
#[case("eight", BeatJumpBeatValue::EightBeat)]
#[case("sixteen", BeatJumpBeatValue::SixteenBeat)]
#[case("thirtytwo", BeatJumpBeatValue::ThirtytwoBeat)]
#[case("sixtyfour", BeatJumpBeatValue::SixtyfourBeat)]
fn test_get_beat_jump_beat_value(
    #[case] name: &str,
    #[case] value: BeatJumpBeatValue,
) -> Result<()> {
    let sett = Setting::load(&mysetting2_path("beat_jump_beat_value", name))?;
    assert_eq!(sett.get_beat_jump_beat_value()?, value);
    Ok(())
}

#[rstest]
#[case("half", BeatJumpBeatValue::HalfBeat)]
#[case("one", BeatJumpBeatValue::OneBeat)]
#[case("two", BeatJumpBeatValue::TwoBeat)]
#[case("four", BeatJumpBeatValue::FourBeat)]
#[case("eight", BeatJumpBeatValue::EightBeat)]
#[case("sixteen", BeatJumpBeatValue::SixteenBeat)]
#[case("thirtytwo", BeatJumpBeatValue::ThirtytwoBeat)]
#[case("sixtyfour", BeatJumpBeatValue::SixtyfourBeat)]
fn test_set_beat_jump_beat_value(
    #[case] name: &str,
    #[case] value: BeatJumpBeatValue,
) -> Result<()> {
    let in_file = mysetting2_path("beat_jump_beat_value", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_beat_jump_beat_value(value)?;
    assert_mysetting2_file(&mut sett, &in_file)
}

#[rstest]
#[case("artwork", JogDisplayMode::Artwork)]
#[case("auto", JogDisplayMode::Auto)]
#[case("info", JogDisplayMode::Info)]
#[case("simple", JogDisplayMode::Simple)]
fn test_get_jog_display_mode(#[case] name: &str, #[case] value: JogDisplayMode) -> Result<()> {
    let sett = Setting::load(&mysetting2_path("jog_display_mode", name))?;
    assert_eq!(sett.get_jog_display_mode()?, value);
    Ok(())
}

#[rstest]
#[case("artwork", JogDisplayMode::Artwork)]
#[case("auto", JogDisplayMode::Auto)]
#[case("info", JogDisplayMode::Info)]
#[case("simple", JogDisplayMode::Simple)]
fn test_set_jog_display_mode(#[case] name: &str, #[case] value: JogDisplayMode) -> Result<()> {
    let in_file = mysetting2_path("jog_display_mode", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_display_mode(value)?;
    assert_mysetting2_file(&mut sett, &in_file)
}

#[rstest]
#[case("one", JogLCDBrightness::One)]
#[case("two", JogLCDBrightness::Two)]
#[case("three", JogLCDBrightness::Three)]
#[case("four", JogLCDBrightness::Four)]
#[case("five", JogLCDBrightness::Five)]
fn test_get_jog_lcd_brightness(#[case] name: &str, #[case] value: JogLCDBrightness) -> Result<()> {
    let sett = Setting::load(&mysetting2_path("jog_lcd_brightness", name))?;
    assert_eq!(sett.get_jog_lcd_brightness()?, value);
    Ok(())
}

#[rstest]
#[case("one", JogLCDBrightness::One)]
#[case("two", JogLCDBrightness::Two)]
#[case("three", JogLCDBrightness::Three)]
#[case("four", JogLCDBrightness::Four)]
#[case("five", JogLCDBrightness::Five)]
fn test_set_jog_lcd_brightness(#[case] name: &str, #[case] value: JogLCDBrightness) -> Result<()> {
    let in_file = mysetting2_path("jog_lcd_brightness", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_jog_lcd_brightness(value)?;
    assert_mysetting2_file(&mut sett, &in_file)
}

#[rstest]
#[case("one", PadButtonBrightness::One)]
#[case("two", PadButtonBrightness::Two)]
#[case("three", PadButtonBrightness::Three)]
#[case("four", PadButtonBrightness::Four)]
fn test_get_pad_button_brightness(
    #[case] name: &str,
    #[case] value: PadButtonBrightness,
) -> Result<()> {
    let sett = Setting::load(&mysetting2_path("pad_button_brightness", name))?;
    assert_eq!(sett.get_pad_button_brightness()?, value);
    Ok(())
}

#[rstest]
#[case("one", PadButtonBrightness::One)]
#[case("two", PadButtonBrightness::Two)]
#[case("three", PadButtonBrightness::Three)]
#[case("four", PadButtonBrightness::Four)]
fn test_set_pad_button_brightness(
    #[case] name: &str,
    #[case] value: PadButtonBrightness,
) -> Result<()> {
    let in_file = mysetting2_path("pad_button_brightness", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_pad_button_brightness(value)?;
    assert_mysetting2_file(&mut sett, &in_file)
}

#[rstest]
#[case("release", VinylSpeedAdjust::Release)]
#[case("touch", VinylSpeedAdjust::Touch)]
#[case("touch_release", VinylSpeedAdjust::TouchRelease)]
fn test_get_vinyl_speed_adjust(#[case] name: &str, #[case] value: VinylSpeedAdjust) -> Result<()> {
    let sett = Setting::load(&mysetting2_path("vinyl_speed_adjust", name))?;
    assert_eq!(sett.get_vinyl_speed_adjust()?, value);
    Ok(())
}

#[rstest]
#[case("release", VinylSpeedAdjust::Release)]
#[case("touch", VinylSpeedAdjust::Touch)]
#[case("touch_release", VinylSpeedAdjust::TouchRelease)]
fn test_set_vinyl_speed_adjust(#[case] name: &str, #[case] value: VinylSpeedAdjust) -> Result<()> {
    let in_file = mysetting2_path("vinyl_speed_adjust", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_vinyl_speed_adjust(value)?;
    assert_mysetting2_file(&mut sett, &in_file)
}

#[rstest]
#[case("phase_meter", Waveform::PhaseMeter)]
#[case("waveform", Waveform::Waveform)]
fn test_get_waveform(#[case] name: &str, #[case] value: Waveform) -> Result<()> {
    let sett = Setting::load(&mysetting2_path("waveform", name))?;
    assert_eq!(sett.get_waveform()?, value);
    Ok(())
}

#[rstest]
#[case("phase_meter", Waveform::PhaseMeter)]
#[case("waveform", Waveform::Waveform)]
fn test_set_waveform(#[case] name: &str, #[case] value: Waveform) -> Result<()> {
    let in_file = mysetting2_path("waveform", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_waveform(value)?;
    assert_mysetting2_file(&mut sett, &in_file)
}

#[rstest]
#[case("phrase", WaveformDivisions::Phrase)]
#[case("time_scale", WaveformDivisions::TimeScale)]
fn test_get_waveform_divisions(#[case] name: &str, #[case] value: WaveformDivisions) -> Result<()> {
    let sett = Setting::load(&mysetting2_path("waveform_divisions", name))?;
    assert_eq!(sett.get_waveform_divisions()?, value);
    Ok(())
}

#[rstest]
#[case("phrase", WaveformDivisions::Phrase)]
#[case("time_scale", WaveformDivisions::TimeScale)]
fn test_set_waveform_divisions(#[case] name: &str, #[case] value: WaveformDivisions) -> Result<()> {
    let in_file = mysetting2_path("waveform_divisions", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_waveform_divisions(value)?;
    assert_mysetting2_file(&mut sett, &in_file)
}

#[test]
fn test_mysetting2_defaults() -> Result<()> {
    let root = common::settings_dir();
    let sett = Setting::load(root.join("MYSETTING2.DAT"))?;
    let default = Setting::new_mysetting2("MYSETTING2.DAT")?;
    assert_eq!(sett.data.content, default.data.content);
    Ok(())
}

// ---- DjmdMySetting ------------------------------------------------------------------------------

#[rstest]
#[case("off", BeatFXQuantize::Off)]
#[case("on", BeatFXQuantize::On)]
fn test_get_beat_fx_quantize(#[case] name: &str, #[case] value: BeatFXQuantize) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("beat_fx_quantize", name))?;
    assert_eq!(sett.get_beat_fx_quantize()?, value);
    Ok(())
}

#[rstest]
#[case("off", BeatFXQuantize::Off)]
#[case("on", BeatFXQuantize::On)]
fn test_set_beat_fx_quantize(#[case] name: &str, #[case] value: BeatFXQuantize) -> Result<()> {
    let in_file = djmdmysetting_path("beat_fx_quantize", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_beat_fx_quantize(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("linear", ChannelFaderCurve::Linear)]
#[case("steep_bottom", ChannelFaderCurve::SteepBottom)]
#[case("steep_top", ChannelFaderCurve::SteepTop)]
fn test_get_channel_fader_curve(
    #[case] name: &str,
    #[case] value: ChannelFaderCurve,
) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("channel_fader_curve", name))?;
    assert_eq!(sett.get_channel_fader_curve()?, value);
    Ok(())
}

#[rstest]
#[case("linear", ChannelFaderCurve::Linear)]
#[case("steep_bottom", ChannelFaderCurve::SteepBottom)]
#[case("steep_top", ChannelFaderCurve::SteepTop)]
fn test_set_channel_fader_curve(
    #[case] name: &str,
    #[case] value: ChannelFaderCurve,
) -> Result<()> {
    let in_file = djmdmysetting_path("channel_fader_curve", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_channel_fader_curve(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("linear", ChannelFaderCurveLongFader::Linear)]
#[case("exponential", ChannelFaderCurveLongFader::Exponential)]
#[case("smooth", ChannelFaderCurveLongFader::Smooth)]
fn test_get_channel_fader_curve_long(
    #[case] name: &str,
    #[case] value: ChannelFaderCurveLongFader,
) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("channel_fader_curve_long", name))?;
    assert_eq!(sett.get_channel_fader_curve_long_fader()?, value);
    Ok(())
}

#[rstest]
#[case("linear", ChannelFaderCurveLongFader::Linear)]
#[case("exponential", ChannelFaderCurveLongFader::Exponential)]
#[case("smooth", ChannelFaderCurveLongFader::Smooth)]
fn test_set_channel_fader_curve_long(
    #[case] name: &str,
    #[case] value: ChannelFaderCurveLongFader,
) -> Result<()> {
    let in_file = djmdmysetting_path("channel_fader_curve_long", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_channel_fader_curve_long_fader(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("constant", CrossfaderCurve::ConstantPower)]
#[case("fast_cut", CrossfaderCurve::FastCut)]
#[case("slow_cut", CrossfaderCurve::SlowCut)]
fn test_get_cross_fader_curve(#[case] name: &str, #[case] value: CrossfaderCurve) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("cross_fader_curve", name))?;
    assert_eq!(sett.get_crossfader_curve()?, value);
    Ok(())
}

#[rstest]
#[case("constant", CrossfaderCurve::ConstantPower)]
#[case("fast_cut", CrossfaderCurve::FastCut)]
#[case("slow_cut", CrossfaderCurve::SlowCut)]
fn test_set_cross_fader_curve(#[case] name: &str, #[case] value: CrossfaderCurve) -> Result<()> {
    let in_file = djmdmysetting_path("cross_fader_curve", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_crossfader_curve(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("one", MixerDisplayBrightness::One)]
#[case("two", MixerDisplayBrightness::Two)]
#[case("three", MixerDisplayBrightness::Three)]
#[case("four", MixerDisplayBrightness::Four)]
#[case("five", MixerDisplayBrightness::Five)]
#[case("white", MixerDisplayBrightness::White)]
fn test_get_display_brightness(
    #[case] name: &str,
    #[case] value: MixerDisplayBrightness,
) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("display_brightness", name))?;
    assert_eq!(sett.get_mixer_display_brightness()?, value);
    Ok(())
}

#[rstest]
#[case("one", MixerDisplayBrightness::One)]
#[case("two", MixerDisplayBrightness::Two)]
#[case("three", MixerDisplayBrightness::Three)]
#[case("four", MixerDisplayBrightness::Four)]
#[case("five", MixerDisplayBrightness::Five)]
#[case("white", MixerDisplayBrightness::White)]
fn test_set_display_brightness(
    #[case] name: &str,
    #[case] value: MixerDisplayBrightness,
) -> Result<()> {
    let in_file = djmdmysetting_path("display_brightness", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_mixer_display_brightness(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("mono_split", HeadphonesMonoSplit::MonoSplit)]
#[case("stereo", HeadphonesMonoSplit::Stereo)]
fn test_get_headphones_mono_split(
    #[case] name: &str,
    #[case] value: HeadphonesMonoSplit,
) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("headphones_mono_split", name))?;
    assert_eq!(sett.get_headphones_mono_split()?, value);
    Ok(())
}

#[rstest]
#[case("mono_split", HeadphonesMonoSplit::MonoSplit)]
#[case("stereo", HeadphonesMonoSplit::Stereo)]
fn test_set_headphones_mono_split(
    #[case] name: &str,
    #[case] value: HeadphonesMonoSplit,
) -> Result<()> {
    let in_file = djmdmysetting_path("headphones_mono_split", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_headphones_mono_split(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("pre_eq", HeadphonesPreEQ::PreEQ)]
#[case("post_eq", HeadphonesPreEQ::PostEQ)]
fn test_get_headphones_pre_eq(#[case] name: &str, #[case] value: HeadphonesPreEQ) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("headphones_pre_eq", name))?;
    assert_eq!(sett.get_headphones_pre_eq()?, value);
    Ok(())
}

#[rstest]
#[case("pre_eq", HeadphonesPreEQ::PreEQ)]
#[case("post_eq", HeadphonesPreEQ::PostEQ)]
fn test_set_headphones_pre_eq(#[case] name: &str, #[case] value: HeadphonesPreEQ) -> Result<()> {
    let in_file = djmdmysetting_path("headphones_pre_eq", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_headphones_pre_eq(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("one", MixerIndicatorBrightness::One)]
#[case("two", MixerIndicatorBrightness::Two)]
#[case("three", MixerIndicatorBrightness::Three)]
fn test_get_indicator_brightness(
    #[case] name: &str,
    #[case] value: MixerIndicatorBrightness,
) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("indicator_brightness", name))?;
    assert_eq!(sett.get_mixer_indicator_brightness()?, value);
    Ok(())
}

#[rstest]
#[case("one", MixerIndicatorBrightness::One)]
#[case("two", MixerIndicatorBrightness::Two)]
#[case("three", MixerIndicatorBrightness::Three)]
fn test_set_indicator_brightness(
    #[case] name: &str,
    #[case] value: MixerIndicatorBrightness,
) -> Result<()> {
    let in_file = djmdmysetting_path("indicator_brightness", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_mixer_indicator_brightness(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("off", MicLowCut::Off)]
#[case("on", MicLowCut::On)]
fn test_get_mic_low_cut(#[case] name: &str, #[case] value: MicLowCut) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("mic_low_cut", name))?;
    assert_eq!(sett.get_mic_low_cut()?, value);
    Ok(())
}

#[rstest]
#[case("off", MicLowCut::Off)]
#[case("on", MicLowCut::On)]
fn test_set_mic_low_cut(#[case] name: &str, #[case] value: MicLowCut) -> Result<()> {
    let in_file = djmdmysetting_path("mic_low_cut", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_mic_low_cut(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("toggle", MidiButtonType::Toggle)]
#[case("trigger", MidiButtonType::Trigger)]
fn test_get_midi_button_type(#[case] name: &str, #[case] value: MidiButtonType) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("midi_button_type", name))?;
    assert_eq!(sett.get_midi_button_type()?, value);
    Ok(())
}

#[rstest]
#[case("toggle", MidiButtonType::Toggle)]
#[case("trigger", MidiButtonType::Trigger)]
fn test_set_midi_button_type(#[case] name: &str, #[case] value: MidiButtonType) -> Result<()> {
    let in_file = djmdmysetting_path("midi_button_type", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_midi_button_type(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("one", MidiChannel::One)]
#[case("two", MidiChannel::Two)]
#[case("three", MidiChannel::Three)]
#[case("four", MidiChannel::Four)]
#[case("five", MidiChannel::Five)]
#[case("six", MidiChannel::Six)]
#[case("seven", MidiChannel::Seven)]
#[case("eight", MidiChannel::Eight)]
#[case("nine", MidiChannel::Nine)]
#[case("ten", MidiChannel::Ten)]
#[case("eleven", MidiChannel::Eleven)]
#[case("twelve", MidiChannel::Twelve)]
#[case("thirteen", MidiChannel::Thirteen)]
#[case("fourteen", MidiChannel::Fourteen)]
#[case("fifteen", MidiChannel::Fifteen)]
#[case("sixteen", MidiChannel::Sixteen)]
fn test_get_midi_channel(#[case] name: &str, #[case] value: MidiChannel) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("midi_channel", name))?;
    assert_eq!(sett.get_midi_channel()?, value);
    Ok(())
}

#[rstest]
#[case("one", MidiChannel::One)]
#[case("two", MidiChannel::Two)]
#[case("three", MidiChannel::Three)]
#[case("four", MidiChannel::Four)]
#[case("five", MidiChannel::Five)]
#[case("six", MidiChannel::Six)]
#[case("seven", MidiChannel::Seven)]
#[case("eight", MidiChannel::Eight)]
#[case("nine", MidiChannel::Nine)]
#[case("ten", MidiChannel::Ten)]
#[case("eleven", MidiChannel::Eleven)]
#[case("twelve", MidiChannel::Twelve)]
#[case("thirteen", MidiChannel::Thirteen)]
#[case("fourteen", MidiChannel::Fourteen)]
#[case("fifteen", MidiChannel::Fifteen)]
#[case("sixteen", MidiChannel::Sixteen)]
fn test_set_midi_channel(#[case] name: &str, #[case] value: MidiChannel) -> Result<()> {
    let in_file = djmdmysetting_path("midi_channel", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_midi_channel(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("minus_6db", TalkOverLevel::Minus6dB)]
#[case("minus_12db", TalkOverLevel::Minus12dB)]
#[case("minus_18db", TalkOverLevel::Minus18dB)]
#[case("minus_24db", TalkOverLevel::Minus24dB)]
fn test_get_talk_over_level(#[case] name: &str, #[case] value: TalkOverLevel) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("talk_over_level", name))?;
    assert_eq!(sett.get_talk_over_level()?, value);
    Ok(())
}

#[rstest]
#[case("minus_6db", TalkOverLevel::Minus6dB)]
#[case("minus_12db", TalkOverLevel::Minus12dB)]
#[case("minus_18db", TalkOverLevel::Minus18dB)]
#[case("minus_24db", TalkOverLevel::Minus24dB)]
fn test_set_talk_over_level(#[case] name: &str, #[case] value: TalkOverLevel) -> Result<()> {
    let in_file = djmdmysetting_path("talk_over_level", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_talk_over_level(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("advanced", TalkOverMode::Advanced)]
#[case("normal", TalkOverMode::Normal)]
fn test_get_talk_over_mode(#[case] name: &str, #[case] value: TalkOverMode) -> Result<()> {
    let sett = Setting::load(&djmdmysetting_path("talk_over_mode", name))?;
    assert_eq!(sett.get_talk_over_mode()?, value);
    Ok(())
}

#[rstest]
#[case("advanced", TalkOverMode::Advanced)]
#[case("normal", TalkOverMode::Normal)]
fn test_set_talk_over_mode(#[case] name: &str, #[case] value: TalkOverMode) -> Result<()> {
    let in_file = djmdmysetting_path("talk_over_mode", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_talk_over_mode(value)?;
    assert_djmdmysetting_file(&mut sett, &in_file)
}

#[test]
fn test_djmdsetting_defaults() -> Result<()> {
    let root = common::settings_dir();
    let sett = Setting::load(root.join("DJMMYSETTING.DAT"))?;
    let default = Setting::new_djmmysetting("DJMMYSETTING.DAT")?;
    assert_eq!(sett.data.content, default.data.content);
    Ok(())
}

// ---- DevSetting ---------------------------------------------------------------------------------

#[rstest]
#[case("half", OverviewWaveformType::HalfWaveform)]
#[case("full", OverviewWaveformType::FullWaveform)]
fn test_get_overview_waveform_type(
    #[case] name: &str,
    #[case] value: OverviewWaveformType,
) -> Result<()> {
    let sett = Setting::load(&devmysetting_path("overview_waveform_type", name))?;
    assert_eq!(sett.get_overview_waveform_type()?, value);
    Ok(())
}

#[rstest]
#[case("half", OverviewWaveformType::HalfWaveform)]
#[case("full", OverviewWaveformType::FullWaveform)]
fn test_set_overview_waveform_type(
    #[case] name: &str,
    #[case] value: OverviewWaveformType,
) -> Result<()> {
    let in_file = devmysetting_path("overview_waveform_type", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_overview_waveform_type(value)?;
    assert_devmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("blue", WaveformColor::Blue)]
#[case("rgb", WaveformColor::Rgb)]
#[case("tri_band", WaveformColor::TriBand)]
fn test_get_waveform_color(#[case] name: &str, #[case] value: WaveformColor) -> Result<()> {
    let sett = Setting::load(&devmysetting_path("waveform_color", name))?;
    assert_eq!(sett.get_waveform_color()?, value);
    Ok(())
}

#[rstest]
#[case("blue", WaveformColor::Blue)]
#[case("rgb", WaveformColor::Rgb)]
#[case("tri_band", WaveformColor::TriBand)]
fn test_set_waveform_color(#[case] name: &str, #[case] value: WaveformColor) -> Result<()> {
    let in_file = devmysetting_path("waveform_color", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_waveform_color(value)?;
    assert_devmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("classic", KeyDisplayFormat::Classic)]
#[case("alpha_numeric", KeyDisplayFormat::Alphanumeric)]
fn test_get_key_display_format(#[case] name: &str, #[case] value: KeyDisplayFormat) -> Result<()> {
    let sett = Setting::load(&devmysetting_path("key_display_format", name))?;
    assert_eq!(sett.get_key_display_format()?, value);
    Ok(())
}

#[rstest]
#[case("classic", KeyDisplayFormat::Classic)]
#[case("alpha_numeric", KeyDisplayFormat::Alphanumeric)]
fn test_set_key_display_format(#[case] name: &str, #[case] value: KeyDisplayFormat) -> Result<()> {
    let in_file = devmysetting_path("key_display_format", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_key_display_format(value)?;
    assert_devmysetting_file(&mut sett, &in_file)
}

#[rstest]
#[case("center", WaveformCurrentPosition::Center)]
#[case("left", WaveformCurrentPosition::Left)]
fn test_get_waveform_current_pos(
    #[case] name: &str,
    #[case] value: WaveformCurrentPosition,
) -> Result<()> {
    let sett = Setting::load(&devmysetting_path("waveform_current_pos", name))?;
    assert_eq!(sett.get_waveform_current_position()?, value);
    Ok(())
}

#[rstest]
#[case("center", WaveformCurrentPosition::Center)]
#[case("left", WaveformCurrentPosition::Left)]
fn test_set_waveform_current_pos(
    #[case] name: &str,
    #[case] value: WaveformCurrentPosition,
) -> Result<()> {
    let in_file = devmysetting_path("waveform_current_pos", name);
    let mut sett = Setting::load(in_file.clone())?;
    sett.set_waveform_current_position(value)?;
    assert_devmysetting_file(&mut sett, &in_file)
}

#[test]
fn test_devsetting_defaults() -> Result<()> {
    let root = common::settings_dir();
    let sett = Setting::load(root.join("DEVSETTING.DAT"))?;
    let default = Setting::new_devsetting("DEVSETTING.DAT")?;
    assert_eq!(sett.data.content, default.data.content);
    Ok(())
}
