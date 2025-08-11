// Author: Dylan Jones
// Date:   2025-08-11

use rbox::settings::*;
use std::path::PathBuf;

mod common;

fn load(dir: &PathBuf, file_name: &str, key: &str, value: &str) -> anyhow::Result<Setting> {
    let file_path = dir.join(key).join(value).join(file_name);
    Setting::load(file_path)
}

// ---- MySetting ----------------------------------------------------------------------------------

#[test]
fn test_get_mysetting() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let dir = root.join("mysetting");
    let name = "MYSETTING.DAT";

    let key = "auto_cue";
    assert_eq!(load(&dir, name, key, "on")?.get_auto_cue()?, AutoCue::On);
    assert_eq!(load(&dir, name, key, "off")?.get_auto_cue()?, AutoCue::Off);

    let key = "auto_cue_level";
    assert_eq!(
        load(&dir, name, key, "memory")?.get_auto_cue_level()?,
        AutoCueLevel::Memory
    );
    assert_eq!(
        load(&dir, name, key, "minus_36db")?.get_auto_cue_level()?,
        AutoCueLevel::Minus36dB
    );
    assert_eq!(
        load(&dir, name, key, "minus_42db")?.get_auto_cue_level()?,
        AutoCueLevel::Minus42dB
    );
    assert_eq!(
        load(&dir, name, key, "minus_48db")?.get_auto_cue_level()?,
        AutoCueLevel::Minus48dB
    );
    assert_eq!(
        load(&dir, name, key, "minus_54db")?.get_auto_cue_level()?,
        AutoCueLevel::Minus54dB
    );
    assert_eq!(
        load(&dir, name, key, "minus_60db")?.get_auto_cue_level()?,
        AutoCueLevel::Minus60dB
    );
    assert_eq!(
        load(&dir, name, key, "minus_66db")?.get_auto_cue_level()?,
        AutoCueLevel::Minus66dB
    );
    assert_eq!(
        load(&dir, name, key, "minus_72db")?.get_auto_cue_level()?,
        AutoCueLevel::Minus72dB
    );
    assert_eq!(
        load(&dir, name, key, "minus_78db")?.get_auto_cue_level()?,
        AutoCueLevel::Minus78dB
    );

    let key = "disc_slot_illumination";
    assert_eq!(
        load(&dir, name, key, "bright")?.get_disc_slot_illumination()?,
        DiscSlotIllumination::Bright
    );
    assert_eq!(
        load(&dir, name, key, "dark")?.get_disc_slot_illumination()?,
        DiscSlotIllumination::Dark
    );
    assert_eq!(
        load(&dir, name, key, "off")?.get_disc_slot_illumination()?,
        DiscSlotIllumination::Off
    );

    let key = "eject_lock";
    assert_eq!(
        load(&dir, name, key, "lock")?.get_eject_lock()?,
        EjectLock::Lock
    );
    assert_eq!(
        load(&dir, name, key, "unlock")?.get_eject_lock()?,
        EjectLock::Unlock
    );

    let key = "hotcue_autoload";
    assert_eq!(
        load(&dir, name, key, "off")?.get_hotcue_autoload()?,
        HotCueAutoLoad::Off
    );
    assert_eq!(
        load(&dir, name, key, "on")?.get_hotcue_autoload()?,
        HotCueAutoLoad::On
    );
    assert_eq!(
        load(&dir, name, key, "rekordbox")?.get_hotcue_autoload()?,
        HotCueAutoLoad::RekordboxSetting
    );

    let key = "hotcue_color";
    assert_eq!(
        load(&dir, name, key, "off")?.get_hotcue_color()?,
        HotCueColor::Off
    );
    assert_eq!(
        load(&dir, name, key, "on")?.get_hotcue_color()?,
        HotCueColor::On
    );

    let key = "jog_mode";
    assert_eq!(load(&dir, name, key, "cdj")?.get_jog_mode()?, JogMode::CDJ);
    assert_eq!(
        load(&dir, name, key, "vinyl")?.get_jog_mode()?,
        JogMode::Vinyl
    );

    let key = "jog_ring_brightness";
    assert_eq!(
        load(&dir, name, key, "bright")?.get_jog_ring_brightness()?,
        JogRingBrightness::Bright
    );
    assert_eq!(
        load(&dir, name, key, "dark")?.get_jog_ring_brightness()?,
        JogRingBrightness::Dark
    );
    assert_eq!(
        load(&dir, name, key, "off")?.get_jog_ring_brightness()?,
        JogRingBrightness::Off
    );

    let key = "jog_ring_indicator";
    assert_eq!(
        load(&dir, name, key, "off")?.get_jog_ring_indicator()?,
        JogRingIndicator::Off
    );
    assert_eq!(
        load(&dir, name, key, "on")?.get_jog_ring_indicator()?,
        JogRingIndicator::On
    );

    let key = "language";
    assert_eq!(
        load(&dir, name, key, "chinese_simplified")?.get_language()?,
        Language::ChineseSimplified
    );
    assert_eq!(
        load(&dir, name, key, "chinese_traditional")?.get_language()?,
        Language::ChineseTraditional
    );
    assert_eq!(
        load(&dir, name, key, "czech")?.get_language()?,
        Language::Czech
    );
    assert_eq!(
        load(&dir, name, key, "danish")?.get_language()?,
        Language::Danish
    );
    assert_eq!(
        load(&dir, name, key, "dutch")?.get_language()?,
        Language::Dutch
    );
    assert_eq!(
        load(&dir, name, key, "english")?.get_language()?,
        Language::English
    );
    assert_eq!(
        load(&dir, name, key, "french")?.get_language()?,
        Language::French
    );
    assert_eq!(
        load(&dir, name, key, "german")?.get_language()?,
        Language::German
    );
    assert_eq!(
        load(&dir, name, key, "greek")?.get_language()?,
        Language::Greek
    );
    assert_eq!(
        load(&dir, name, key, "hungarian")?.get_language()?,
        Language::Hungarian
    );
    assert_eq!(
        load(&dir, name, key, "italian")?.get_language()?,
        Language::Italian
    );
    assert_eq!(
        load(&dir, name, key, "japanese")?.get_language()?,
        Language::Japanese
    );
    assert_eq!(
        load(&dir, name, key, "korean")?.get_language()?,
        Language::Korean
    );
    assert_eq!(
        load(&dir, name, key, "portuguese")?.get_language()?,
        Language::Portuguese
    );
    assert_eq!(
        load(&dir, name, key, "russian")?.get_language()?,
        Language::Russian
    );
    assert_eq!(
        load(&dir, name, key, "spanish")?.get_language()?,
        Language::Spanish
    );
    assert_eq!(
        load(&dir, name, key, "swedish")?.get_language()?,
        Language::Swedish
    );
    assert_eq!(
        load(&dir, name, key, "turkish")?.get_language()?,
        Language::Turkish
    );

    let key = "lcd_brightness";
    assert_eq!(
        load(&dir, name, key, "five")?.get_lcd_brightness()?,
        LCDBrightness::Five
    );
    assert_eq!(
        load(&dir, name, key, "four")?.get_lcd_brightness()?,
        LCDBrightness::Four
    );
    assert_eq!(
        load(&dir, name, key, "three")?.get_lcd_brightness()?,
        LCDBrightness::Three
    );
    assert_eq!(
        load(&dir, name, key, "two")?.get_lcd_brightness()?,
        LCDBrightness::Two
    );
    assert_eq!(
        load(&dir, name, key, "one")?.get_lcd_brightness()?,
        LCDBrightness::One
    );

    let key = "master_tempo";
    assert_eq!(
        load(&dir, name, key, "off")?.get_master_tempo()?,
        MasterTempo::Off
    );
    assert_eq!(
        load(&dir, name, key, "on")?.get_master_tempo()?,
        MasterTempo::On
    );

    let key = "needle_lock";
    assert_eq!(
        load(&dir, name, key, "lock")?.get_needle_lock()?,
        NeedleLock::Lock
    );
    assert_eq!(
        load(&dir, name, key, "unlock")?.get_needle_lock()?,
        NeedleLock::Unlock
    );

    let key = "on_air_display";
    assert_eq!(
        load(&dir, name, key, "off")?.get_on_air_display()?,
        OnAirDisplay::Off
    );
    assert_eq!(
        load(&dir, name, key, "on")?.get_on_air_display()?,
        OnAirDisplay::On
    );

    let key = "phase_meter";
    assert_eq!(
        load(&dir, name, key, "type1")?.get_phase_meter()?,
        PhaseMeter::Type1
    );
    assert_eq!(
        load(&dir, name, key, "type2")?.get_phase_meter()?,
        PhaseMeter::Type2
    );

    let key = "play_mode";
    assert_eq!(
        load(&dir, name, key, "continue")?.get_play_mode()?,
        PlayMode::Continue
    );
    assert_eq!(
        load(&dir, name, key, "single")?.get_play_mode()?,
        PlayMode::Single
    );

    let key = "quantize";
    assert_eq!(load(&dir, name, key, "off")?.get_quantize()?, Quantize::Off);
    assert_eq!(load(&dir, name, key, "on")?.get_quantize()?, Quantize::On);

    let key = "quantize_beat_value";
    assert_eq!(
        load(&dir, name, key, "eighth")?.get_quantize_beat_value()?,
        QuantizeBeatValue::EighthBeat
    );
    assert_eq!(
        load(&dir, name, key, "quarter")?.get_quantize_beat_value()?,
        QuantizeBeatValue::QuarterBeat
    );
    assert_eq!(
        load(&dir, name, key, "half")?.get_quantize_beat_value()?,
        QuantizeBeatValue::HalfBeat
    );
    assert_eq!(
        load(&dir, name, key, "one")?.get_quantize_beat_value()?,
        QuantizeBeatValue::FullBeat
    );

    let key = "slip_flashing";
    assert_eq!(
        load(&dir, name, key, "off")?.get_slip_flashing()?,
        SlipFlashing::Off
    );
    assert_eq!(
        load(&dir, name, key, "on")?.get_slip_flashing()?,
        SlipFlashing::On
    );

    let key = "sync";
    assert_eq!(load(&dir, name, key, "off")?.get_sync()?, Sync::Off);
    assert_eq!(load(&dir, name, key, "on")?.get_sync()?, Sync::On);

    let key = "tempo_range";
    assert_eq!(
        load(&dir, name, key, "wide")?.get_tempo_range()?,
        TempoRange::Wide
    );
    assert_eq!(
        load(&dir, name, key, "sixteen")?.get_tempo_range()?,
        TempoRange::SixteenPercent
    );
    assert_eq!(
        load(&dir, name, key, "ten")?.get_tempo_range()?,
        TempoRange::TenPercent
    );
    assert_eq!(
        load(&dir, name, key, "six")?.get_tempo_range()?,
        TempoRange::SixPercent
    );

    let key = "time_mode";
    assert_eq!(
        load(&dir, name, key, "elapsed")?.get_time_mode()?,
        TimeMode::Elapsed
    );
    assert_eq!(
        load(&dir, name, key, "remain")?.get_time_mode()?,
        TimeMode::Remain
    );

    Ok(())
}

#[test]
fn test_mysetting_defaults() -> anyhow::Result<()> {
    let root = common::testdata_settings_dir()?;
    let file = root.join("MYSETTING.DAT");
    let sett = Setting::load(file)?;
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
