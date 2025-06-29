// Author: Dylan Jones
// Date:   2025-06-09

//! Rekordbox `*SETTING.DAT` file handler.
//!
//! These files are either present in the `%APPDATA%\Pioneer\rekordbox6` directory on the machine
//! running Rekordbox, or in the `PIONEER` directory of a USB drive (device exports).
//!
//! Rekordbox stores the user settings in `*SETTING.DAT` files, which also get exported to USB
//! devices. These files are either present in the `%APPDATA%\Pioneer\rekordbox6` directory on
//! the machine running Rekordbox, or in the `PIONEER` directory of a USB drive (device exports).
//!
//! The setting files store the settings found on the "DJ System" > "My Settings" page of the
//! Rekordbox preferences. These include language, LCD brightness, tempo fader range, crossfader
//! curve and other settings for Pioneer professional DJ equipment.

use anyhow::anyhow;
use binrw::{binrw, io::Cursor, BinRead, BinWrite, Endian, NullString};
use parse_display::Display;
use std::ffi::OsStr;
use std::path::Path;

// -- MySetting ------------------------------------------------------------------------------------

/// Found at "PLAYER > DISPLAY(INDICATOR) > ON AIR DISPLAY" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum OnAirDisplay {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

impl TryFrom<String> for OnAirDisplay {
    type Error = String;

    fn try_from(value: String) -> Result<OnAirDisplay, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid OnAirDisplay value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DISPLAY(LCD) > LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum LCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    #[display("1")]
    One = 0x81,
    /// Named "2" in the Rekordbox preferences.
    #[display("2")]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[display("3")]
    #[default]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[display("4")]
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[display("5")]
    Five,
}

impl TryFrom<String> for LCDBrightness {
    type Error = String;

    fn try_from(value: String) -> Result<LCDBrightness, Self::Error> {
        match value.as_str() {
            "One" => Ok(Self::One),
            "Two" => Ok(Self::Two),
            "Three" => Ok(Self::Three),
            "Four" => Ok(Self::Four),
            "Five" => Ok(Self::Five),
            _ => Err(format!("Invalid LCDBrightness value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum Quantize {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

impl TryFrom<String> for Quantize {
    type Error = String;

    fn try_from(value: String) -> Result<Quantize, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid Quantize value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum AutoCueLevel {
    /// Named "-78dB" in the Rekordbox preferences.
    #[display("-78dB")]
    Minus78dB = 0x87,
    /// Named "-72dB" in the Rekordbox preferences.
    #[display("-72dB")]
    Minus72dB = 0x86,
    /// Named "-66dB" in the Rekordbox preferences.
    #[display("-66dB")]
    Minus66dB = 0x85,
    /// Named "-60dB" in the Rekordbox preferences.
    #[display("-60dB")]
    Minus60dB = 0x84,
    /// Named "-54dB" in the Rekordbox preferences.
    #[display("-54dB")]
    Minus54dB = 0x83,
    /// Named "-48dB" in the Rekordbox preferences.
    #[display("-48dB")]
    Minus48dB = 0x82,
    /// Named "-42dB" in the Rekordbox preferences.
    #[display("-42dB")]
    Minus42dB = 0x81,
    /// Named "-36dB" in the Rekordbox preferences.
    #[display("-36dB")]
    Minus36dB = 0x80,
    /// Named "MEMORY" in the Rekordbox preferences.
    #[default]
    Memory = 0x88,
}

impl TryFrom<String> for AutoCue {
    type Error = String;

    fn try_from(value: String) -> Result<AutoCue, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid AutoCue value: {}", value)),
        }
    }
}

impl TryFrom<String> for AutoCueLevel {
    type Error = String;

    fn try_from(value: String) -> Result<AutoCueLevel, Self::Error> {
        match value.as_str() {
            "Minus78dB" => Ok(Self::Minus78dB),
            "Minus72dB" => Ok(Self::Minus72dB),
            "Minus66dB" => Ok(Self::Minus66dB),
            "Minus60dB" => Ok(Self::Minus60dB),
            "Minus54dB" => Ok(Self::Minus54dB),
            "Minus48dB" => Ok(Self::Minus48dB),
            "Minus42dB" => Ok(Self::Minus42dB),
            "Minus36dB" => Ok(Self::Minus36dB),
            "Memory" => Ok(Self::Memory),
            _ => Err(format!("Invalid AutoCueLevel value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DISPLAY(LCD) > LANGUAGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum Language {
    /// Named "English" in the Rekordbox preferences.
    #[default]
    English = 0x81,
    /// Named "Français" in the Rekordbox preferences.
    #[display("Français")]
    French,
    /// Named "Deutsch" in the Rekordbox preferences.
    #[display("Deutsch")]
    German,
    /// Named "Italiano" in the Rekordbox preferences.
    #[display("Italiano")]
    Italian,
    /// Named "Nederlands" in the Rekordbox preferences.
    #[display("Nederlands")]
    Dutch,
    /// Named "Español" in the Rekordbox preferences.
    #[display("Español")]
    Spanish,
    /// Named "Русский" in the Rekordbox preferences.
    #[display("Русский")]
    Russian,
    /// Named "한국어" in the Rekordbox preferences.
    #[display("한국어")]
    Korean,
    /// Named "简体中文" in the Rekordbox preferences.
    ChineseSimplified,
    #[display("简体中文")]
    /// Named "繁體中文" in the Rekordbox preferences.
    #[display("繁體中文")]
    ChineseTraditional,
    /// Named "日本語" in the Rekordbox preferences.
    #[display("日本語")]
    Japanese,
    /// Named "Português" in the Rekordbox preferences.
    #[display("Português")]
    Portuguese,
    /// Named "Svenska" in the Rekordbox preferences.
    #[display("Svenska")]
    Swedish,
    /// Named "Čeština" in the Rekordbox preferences.
    #[display("Čeština")]
    Czech,
    /// Named "Magyar" in the Rekordbox preferences.
    #[display("Magyar")]
    Hungarian,
    /// Named "Dansk" in the Rekordbox preferences.
    #[display("Dansk")]
    Danish,
    /// Named "Ελληνικά" in the Rekordbox preferences.
    #[display("Ελληνικά")]
    Greek,
    /// Named "Türkçe" in the Rekordbox preferences.
    #[display("Türkçe")]
    Turkish,
}

impl TryFrom<String> for Language {
    type Error = String;

    fn try_from(value: String) -> Result<Language, Self::Error> {
        match value.as_str() {
            "English" => Ok(Self::English),
            "French" => Ok(Self::French),
            "German" => Ok(Self::German),
            "Italian" => Ok(Self::Italian),
            "Dutch" => Ok(Self::Dutch),
            "Spanish" => Ok(Self::Spanish),
            "Russian" => Ok(Self::Russian),
            "Korean" => Ok(Self::Korean),
            "ChineseSimplified" => Ok(Self::ChineseSimplified),
            "ChineseTraditional" => Ok(Self::ChineseTraditional),
            "Japanese" => Ok(Self::Japanese),
            "Portuguese" => Ok(Self::Portuguese),
            "Swedish" => Ok(Self::Swedish),
            "Czech" => Ok(Self::Czech),
            "Hungarian" => Ok(Self::Hungarian),
            "Danish" => Ok(Self::Danish),
            "Greek" => Ok(Self::Greek),
            "Turkish" => Ok(Self::Turkish),
            _ => Err(format!("Invalid Language value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING BRIGHTNESS" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum JogRingBrightness {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    #[display("1 (Dark)")]
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    #[display("2 (Bright)")]
    #[default]
    Bright,
}

impl TryFrom<String> for JogRingBrightness {
    type Error = String;

    fn try_from(value: String) -> Result<JogRingBrightness, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "Dark" => Ok(Self::Dark),
            "Bright" => Ok(Self::Bright),
            _ => Err(format!("Invalid JogRingBrightness value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > JOG RING INDICATOR" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum JogRingIndicator {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

impl TryFrom<String> for JogRingIndicator {
    type Error = String;

    fn try_from(value: String) -> Result<JogRingIndicator, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid JogRingIndicator value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > SLIP FLASHING" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum SlipFlashing {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

impl TryFrom<String> for SlipFlashing {
    type Error = String;

    fn try_from(value: String) -> Result<SlipFlashing, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid SlipFlashing value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > DISC SLOT ILLUMINATION" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum DiscSlotIllumination {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "1 (Dark)" in the Rekordbox preferences.
    #[display("1 (Dark)")]
    Dark,
    /// Named "2 (Bright)" in the Rekordbox preferences.
    #[display("2 (Bright)")]
    #[default]
    Bright,
}

impl TryFrom<String> for DiscSlotIllumination {
    type Error = String;

    fn try_from(value: String) -> Result<DiscSlotIllumination, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "Dark" => Ok(Self::Dark),
            "Bright" => Ok(Self::Bright),
            _ => Err(format!("Invalid DiscSlotIllumination value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > EJECT/LOAD LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum EjectLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    #[default]
    Unlock = 0x80,
    /// Named "LOCK" in the Rekordbox preferences.
    Lock,
}

impl TryFrom<String> for EjectLock {
    type Error = String;

    fn try_from(value: String) -> Result<EjectLock, Self::Error> {
        match value.as_str() {
            "Unlock" => Ok(Self::Unlock),
            "Lock" => Ok(Self::Lock),
            _ => Err(format!("Invalid EjectLock value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > SYNC" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum Sync {
    /// Named "OFF" in the Rekordbox preferences.
    #[default]
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

impl TryFrom<String> for Sync {
    type Error = String;

    fn try_from(value: String) -> Result<Sync, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid Sync value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > PLAY MODE / AUTO PLAY MODE" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum PlayMode {
    /// Named "CONTINUE / ON" in the Rekordbox preferences.
    #[display("Continue / On")]
    Continue = 0x80,
    /// Named "SINGLE / OFF" in the Rekordbox preferences.
    #[display("Single / Off")]
    #[default]
    Single,
}

impl TryFrom<String> for PlayMode {
    type Error = String;

    fn try_from(value: String) -> Result<PlayMode, Self::Error> {
        match value.as_str() {
            "Continue" => Ok(Self::Continue),
            "Single" => Ok(Self::Single),
            _ => Err(format!("Invalid PlayMode value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > QUANTIZE BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum QuantizeBeatValue {
    /// Named "1/8 Beat" in the Rekordbox preferences.
    #[display("1/8 Beat")]
    EighthBeat = 0x83,
    /// Named "1/4 Beat" in the Rekordbox preferences.
    #[display("1/4 Beat")]
    QuarterBeat = 0x82,
    /// Named "1/2 Beat" in the Rekordbox preferences.
    #[display("1/2 Beat")]
    HalfBeat = 0x81,
    /// Named "1 Beat" in the Rekordbox preferences.
    #[default]
    #[display("1 Beat")]
    FullBeat = 0x80,
}

impl TryFrom<String> for QuantizeBeatValue {
    type Error = String;

    fn try_from(value: String) -> Result<QuantizeBeatValue, Self::Error> {
        match value.as_str() {
            "EighthBeat" => Ok(Self::EighthBeat),
            "QuarterBeat" => Ok(Self::QuarterBeat),
            "HalfBeat" => Ok(Self::HalfBeat),
            "FullBeat" => Ok(Self::FullBeat),
            _ => Err(format!("Invalid QuantizeBeatValue value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > HOT CUE AUTO LOAD" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum HotCueAutoLoad {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "rekordbox SETTING" in the Rekordbox preferences.
    #[display("rekordbox SETTING")]
    RekordboxSetting = 0x82,
    /// Named "On" in the Rekordbox preferences.
    #[default]
    On = 0x81,
}

impl TryFrom<String> for HotCueAutoLoad {
    type Error = String;

    fn try_from(value: String) -> Result<HotCueAutoLoad, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "RekordboxSetting" => Ok(Self::RekordboxSetting),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid HotCueAutoLoad value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > HOT CUE COLOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum HotCueColor {
    /// Named "OFF" in the Rekordbox preferences.
    #[default]
    Off = 0x80,
    /// Named "On" in the Rekordbox preferences.
    On,
}

impl TryFrom<String> for HotCueColor {
    type Error = String;

    fn try_from(value: String) -> Result<HotCueColor, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid HotCueColor value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > NEEDLE LOCK" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum NeedleLock {
    /// Named "UNLOCK" in the Rekordbox preferences.
    Unlock = 0x80,
    /// Named "LOCK" in the Rekordbox preferences.
    #[default]
    Lock,
}

impl TryFrom<String> for NeedleLock {
    type Error = String;

    fn try_from(value: String) -> Result<NeedleLock, Self::Error> {
        match value.as_str() {
            "Unlock" => Ok(Self::Unlock),
            "Lock" => Ok(Self::Lock),
            _ => Err(format!("Invalid NeedleLock value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > TIME MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum TimeMode {
    /// Named "Elapsed" in the Rekordbox preferences.
    Elapsed = 0x80,
    /// Named "REMAIN" in the Rekordbox preferences.
    #[default]
    Remain,
}

impl TryFrom<String> for TimeMode {
    type Error = String;

    fn try_from(value: String) -> Result<TimeMode, Self::Error> {
        match value.as_str() {
            "Elapsed" => Ok(Self::Elapsed),
            "Remain" => Ok(Self::Remain),
            _ => Err(format!("Invalid TimeMode value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > JOG MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum JogMode {
    /// Named "VINYL" in the Rekordbox preferences.
    #[default]
    Vinyl = 0x81,
    /// Named "CDJ" in the Rekordbox preferences.
    CDJ = 0x80,
}

impl TryFrom<String> for JogMode {
    type Error = String;

    fn try_from(value: String) -> Result<JogMode, Self::Error> {
        match value.as_str() {
            "Vinyl" => Ok(Self::Vinyl),
            "CDJ" => Ok(Self::CDJ),
            _ => Err(format!("Invalid JogMode value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > AUTO CUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum AutoCue {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

/// Found at "PLAYER > DJ SETTING > MASTER TEMPO" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum MasterTempo {
    /// Named "OFF" in the Rekordbox preferences.
    #[default]
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    On,
}

impl TryFrom<String> for MasterTempo {
    type Error = String;

    fn try_from(value: String) -> Result<MasterTempo, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid MasterTempo value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > TEMPO RANGE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum TempoRange {
    /// Named "±6" in the Rekordbox preferences.
    #[display("±6%")]
    SixPercent = 0x80,
    /// Named "±10" in the Rekordbox preferences.
    #[display("±10%")]
    #[default]
    TenPercent,
    /// Named "±16" in the Rekordbox preferences.
    #[display("±16%")]
    SixteenPercent,
    /// Named "WIDE" in the Rekordbox preferences.
    Wide,
}

impl TryFrom<String> for TempoRange {
    type Error = String;

    fn try_from(value: String) -> Result<TempoRange, Self::Error> {
        match value.as_str() {
            "SixPercent" => Ok(Self::SixPercent),
            "TenPercent" => Ok(Self::TenPercent),
            "SixteenPercent" => Ok(Self::SixteenPercent),
            "Wide" => Ok(Self::Wide),
            _ => Err(format!("Invalid TempoRange value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum PhaseMeter {
    /// Named "TYPE 1" in the Rekordbox preferences.
    #[default]
    #[display("Type 1")]
    Type1 = 0x80,
    /// Named "TYPE 2" in the Rekordbox preferences.
    #[display("Type 2")]
    Type2,
}

impl TryFrom<String> for PhaseMeter {
    type Error = String;

    fn try_from(value: String) -> Result<PhaseMeter, Self::Error> {
        match value.as_str() {
            "Type1" => Ok(Self::Type1),
            "Type2" => Ok(Self::Type2),
            _ => Err(format!("Invalid PhaseMeter value: {}", value)),
        }
    }
}

/// Payload of a `MYSETTING.DAT` file (40 bytes).
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct MySetting {
    /// Unknown field.
    unknown1: [u8; 8],
    /// "ON AIR DISPLAY" setting.
    pub on_air_display: OnAirDisplay,
    /// "LCD BRIGHTNESS" setting.
    pub lcd_brightness: LCDBrightness,
    /// "QUANTIZE" setting.
    pub quantize: Quantize,
    /// "AUTO CUE LEVEL" setting.
    pub auto_cue_level: AutoCueLevel,
    /// "LANGUAGE" setting.
    pub language: Language,
    /// Unknown field.
    unknown2: u8,
    /// "JOG RING BRIGHTNESS" setting.
    pub jog_ring_brightness: JogRingBrightness,
    /// "JOG RING INDICATOR" setting.
    pub jog_ring_indicator: JogRingIndicator,
    /// "SLIP FLASHING" setting.
    pub slip_flashing: SlipFlashing,
    /// Unknown field.
    unknown3: [u8; 3],
    /// "DISC SLOT ILLUMINATION" setting.
    pub disc_slot_illumination: DiscSlotIllumination,
    /// "EJECT/LOAD LOCK" setting.
    pub eject_lock: EjectLock,
    /// "SYNC" setting.
    pub sync: Sync,
    /// "PLAY MODE / AUTO PLAY MODE" setting.
    pub play_mode: PlayMode,
    /// Quantize Beat Value setting.
    pub quantize_beat_value: QuantizeBeatValue,
    /// "HOT CUE AUTO LOAD" setting.
    pub hotcue_autoload: HotCueAutoLoad,
    /// "HOT CUE COLOR" setting.
    pub hotcue_color: HotCueColor,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown4 == 0))]
    unknown4: u16,
    /// "NEEDLE LOCK" setting.
    pub needle_lock: NeedleLock,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown5 == 0))]
    unknown5: u16,
    /// "TIME MODE" setting.
    pub time_mode: TimeMode,
    /// "TIME MODE" setting.
    pub jog_mode: JogMode,
    /// "AUTO CUE" setting.
    pub auto_cue: AutoCue,
    /// "MASTER TEMPO" setting.
    pub master_tempo: MasterTempo,
    /// "TEMPO RANGE" setting.
    pub tempo_range: TempoRange,
    /// "PHASE METER" setting.
    pub phase_meter: PhaseMeter,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown6 == 0))]
    unknown6: u16,
}

impl Default for MySetting {
    fn default() -> Self {
        Self {
            unknown1: [0x78, 0x56, 0x34, 0x12, 0x02, 0x00, 0x00, 0x00],
            on_air_display: OnAirDisplay::default(),
            lcd_brightness: LCDBrightness::default(),
            quantize: Quantize::default(),
            auto_cue_level: AutoCueLevel::default(),
            language: Language::default(),
            unknown2: 0x01,
            jog_ring_brightness: JogRingBrightness::default(),
            jog_ring_indicator: JogRingIndicator::default(),
            slip_flashing: SlipFlashing::default(),
            unknown3: [0x01, 0x01, 0x01],
            disc_slot_illumination: DiscSlotIllumination::default(),
            eject_lock: EjectLock::default(),
            sync: Sync::default(),
            play_mode: PlayMode::default(),
            quantize_beat_value: QuantizeBeatValue::default(),
            hotcue_autoload: HotCueAutoLoad::default(),
            hotcue_color: HotCueColor::default(),
            unknown4: 0x0000,
            needle_lock: NeedleLock::default(),
            unknown5: 0x0000,
            time_mode: TimeMode::default(),
            jog_mode: JogMode::default(),
            auto_cue: AutoCue::default(),
            master_tempo: MasterTempo::default(),
            tempo_range: TempoRange::default(),
            phase_meter: PhaseMeter::default(),
            unknown6: 0x0000,
        }
    }
}

impl MySetting {
    fn file_name() -> String {
        "MYSETTING.DAT".into()
    }
}

// -- MySetting2 -----------------------------------------------------------------------------------

/// Found at "PLAYER > DJ SETTING > VINYL SPEED ADJUST" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum VinylSpeedAdjust {
    /// Named "TOUCH & RELEASE" in the Rekordbox preferences.
    #[display("Touch & Release")]
    TouchRelease = 0x80,
    /// Named "TOUCH" in the Rekordbox preferences.
    #[default]
    Touch,
    /// Named "RELEASE" in the Rekordbox preferences.
    Release,
}

impl TryFrom<String> for VinylSpeedAdjust {
    type Error = String;

    fn try_from(value: String) -> Result<VinylSpeedAdjust, Self::Error> {
        match value.as_str() {
            "TouchRelease" => Ok(Self::TouchRelease),
            "Touch" => Ok(Self::Touch),
            "Release" => Ok(Self::Release),
            _ => Err(format!("Invalid VinylSpeedAdjust value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG DISPLAY MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum JogDisplayMode {
    /// Named "AUTO" in the Rekordbox preferences.
    #[default]
    Auto = 0x80,
    /// Named "INFO" in the Rekordbox preferences.
    Info,
    /// Named "SIMPLE" in the Rekordbox preferences.
    Simple,
    /// Named "ARTWORK" in the Rekordbox preferences.
    Artwork,
}

impl TryFrom<String> for JogDisplayMode {
    type Error = String;

    fn try_from(value: String) -> Result<JogDisplayMode, Self::Error> {
        match value.as_str() {
            "Auto" => Ok(Self::Auto),
            "Info" => Ok(Self::Info),
            "Simple" => Ok(Self::Simple),
            "Artwork" => Ok(Self::Artwork),
            _ => Err(format!("Invalid JogDisplayMode value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DISPLAY(INDICATOR) > PAD/BUTTON BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum PadButtonBrightness {
    /// Named "1" in the Rekordbox preferences.
    #[display("1")]
    One = 0x81,
    /// Named "2" in the Rekordbox preferences.
    #[display("2")]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[display("3")]
    #[default]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[display("4")]
    Four,
}

impl TryFrom<String> for PadButtonBrightness {
    type Error = String;

    fn try_from(value: String) -> Result<PadButtonBrightness, Self::Error> {
        match value.as_str() {
            "One" => Ok(Self::One),
            "Two" => Ok(Self::Two),
            "Three" => Ok(Self::Three),
            "Four" => Ok(Self::Four),
            _ => Err(format!("Invalid PadButtonBrightness value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DISPLAY(LCD) > JOG LCD BRIGHTNESS" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum JogLCDBrightness {
    /// Named "1" in the Rekordbox preferences.
    #[display("1")]
    One = 0x81,
    /// Named "2" in the Rekordbox preferences.
    #[display("2")]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[display("3")]
    #[default]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[display("4")]
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[display("5")]
    Five,
}

impl TryFrom<String> for JogLCDBrightness {
    type Error = String;

    fn try_from(value: String) -> Result<JogLCDBrightness, Self::Error> {
        match value.as_str() {
            "One" => Ok(Self::One),
            "Two" => Ok(Self::Two),
            "Three" => Ok(Self::Three),
            "Four" => Ok(Self::Four),
            "Five" => Ok(Self::Five),
            _ => Err(format!("Invalid JogLCDBrightness value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM DIVISIONS" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum WaveformDivisions {
    /// Named "TIME SCALE" in the Rekordbox preferences.
    #[display("Time Scale")]
    TimeScale = 0x80,
    /// Named "PHRASE" in the Rekordbox preferences.
    #[default]
    Phrase,
}

impl TryFrom<String> for Waveform {
    type Error = String;

    fn try_from(value: String) -> Result<Waveform, Self::Error> {
        match value.as_str() {
            "Waveform" => Ok(Self::Waveform),
            "PhaseMeter" => Ok(Self::PhaseMeter),
            _ => Err(format!("Invalid Waveform value: {}", value)),
        }
    }
}

impl TryFrom<String> for WaveformDivisions {
    type Error = String;

    fn try_from(value: String) -> Result<WaveformDivisions, Self::Error> {
        match value.as_str() {
            "TimeScale" => Ok(Self::TimeScale),
            "Phrase" => Ok(Self::Phrase),
            _ => Err(format!("Invalid WaveformDivisions value: {}", value)),
        }
    }
}

/// Found at "PLAYER > DJ SETTING > WAVEFORM / PHASE METER" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum Waveform {
    /// Named "WAVEFORM" in the Rekordbox preferences.
    #[default]
    Waveform = 0x80,
    /// Named "PHASE METER" in the Rekordbox preferences.
    #[display("Phase Meter")]
    PhaseMeter,
}

/// Found at "PLAYER > DJ SETTING > BEAT JUMP BEAT VALUE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum BeatJumpBeatValue {
    /// Named "1/2 BEAT" in the Rekordbox preferences.
    #[display("1/2 Beat")]
    HalfBeat = 0x80,
    /// Named "1 BEAT" in the Rekordbox preferences.
    #[display("1 Beat")]
    OneBeat,
    /// Named "2 BEAT" in the Rekordbox preferences.
    #[display("2 Beat")]
    TwoBeat,
    /// Named "4 BEAT" in the Rekordbox preferences.
    #[display("4 Beat")]
    FourBeat,
    /// Named "8 BEAT" in the Rekordbox preferences.
    #[display("8 Beat")]
    EightBeat,
    /// Named "16 BEAT" in the Rekordbox preferences.
    #[display("16 Beat")]
    #[default]
    SixteenBeat,
    /// Named "32 BEAT" in the Rekordbox preferences.
    #[display("32 Beat")]
    ThirtytwoBeat,
    /// Named "64 BEAT" in the Rekordbox preferences.
    #[display("64 Beat")]
    SixtyfourBeat,
}

impl TryFrom<String> for BeatJumpBeatValue {
    type Error = String;

    fn try_from(value: String) -> Result<BeatJumpBeatValue, Self::Error> {
        match value.as_str() {
            "HalfBeat" => Ok(Self::HalfBeat),
            "OneBeat" => Ok(Self::OneBeat),
            "TwoBeat" => Ok(Self::TwoBeat),
            "FourBeat" => Ok(Self::FourBeat),
            "EightBeat" => Ok(Self::EightBeat),
            "SixteenBeat" => Ok(Self::SixteenBeat),
            "ThirtytwoBeat" => Ok(Self::ThirtytwoBeat),
            "SixtyfourBeat" => Ok(Self::SixtyfourBeat),
            _ => Err(format!("Invalid BeatJumpBeatValue value: {}", value)),
        }
    }
}

/// Payload of a `MYSETTING2.DAT` file (40 bytes).
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct MySetting2 {
    /// "VINYL SPEED ADJUST" setting.
    pub vinyl_speed_adjust: VinylSpeedAdjust,
    /// "JOG DISPLAY MODE" setting.
    pub jog_display_mode: JogDisplayMode,
    /// "PAD/BUTTON BRIGHTNESS" setting.
    pub pad_button_brightness: PadButtonBrightness,
    /// "JOG LCD BRIGHTNESS" setting.
    pub jog_lcd_brightness: JogLCDBrightness,
    /// "WAVEFORM DIVISIONS" setting.
    pub waveform_divisions: WaveformDivisions,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown1 == [0; 5]))]
    unknown1: [u8; 5],
    /// "WAVEFORM / PHASE METER" setting.
    pub waveform: Waveform,
    /// Unknown field.
    unknown2: u8,
    /// "BEAT JUMP BEAT VALUE" setting.
    pub beat_jump_beat_value: BeatJumpBeatValue,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown3 == [0; 27]))]
    unknown3: [u8; 27],
}

impl Default for MySetting2 {
    fn default() -> Self {
        Self {
            vinyl_speed_adjust: VinylSpeedAdjust::default(),
            jog_display_mode: JogDisplayMode::default(),
            pad_button_brightness: PadButtonBrightness::default(),
            jog_lcd_brightness: JogLCDBrightness::default(),
            waveform_divisions: WaveformDivisions::default(),
            unknown1: [0; 5],
            waveform: Waveform::default(),
            unknown2: 0x81,
            beat_jump_beat_value: BeatJumpBeatValue::default(),
            unknown3: [0; 27],
        }
    }
}

impl MySetting2 {
    fn file_name() -> String {
        "MYSETTING2.DAT".into()
    }
}

// -- DJMMySetting ---------------------------------------------------------------------------------

/// Found at "MIXER > DJ SETTING > CH FADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum ChannelFaderCurve {
    /// Steep volume raise when the fader is moved near the top.
    #[display("Steep Top")]
    SteepTop = 0x80,
    /// Linear volume raise when the fader is moved.
    #[display("Linear")]
    #[default]
    Linear,
    /// Steep volume raise when the fader is moved near the bottom.
    #[display("Steep Bottom")]
    SteepBottom,
}

impl TryFrom<String> for ChannelFaderCurve {
    type Error = String;

    fn try_from(value: String) -> Result<ChannelFaderCurve, Self::Error> {
        match value.as_str() {
            "SteepTop" => Ok(Self::SteepTop),
            "Linear" => Ok(Self::Linear),
            "SteepBottom" => Ok(Self::SteepBottom),
            _ => Err(format!("Invalid ChannelFaderCurve value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > CROSSFADER CURVE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum CrossfaderCurve {
    /// Logarithmic volume raise of the other channel near the edges of the fader.
    #[display("Constant Power")]
    ConstantPower = 0x80,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    #[display("Slow Cut")]
    SlowCut,
    /// Steep linear volume raise of the other channel near the edges of the fader, no volume
    /// change in the center.
    #[display("Fast Cut")]
    #[default]
    FastCut,
}

impl TryFrom<String> for CrossfaderCurve {
    type Error = String;

    fn try_from(value: String) -> Result<CrossfaderCurve, Self::Error> {
        match value.as_str() {
            "ConstantPower" => Ok(Self::ConstantPower),
            "SlowCut" => Ok(Self::SlowCut),
            "FastCut" => Ok(Self::FastCut),
            _ => Err(format!("Invalid CrossfaderCurve value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > HEADPHONES PRE EQ" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum HeadphonesPreEQ {
    /// Named "POST EQ" in the Rekordbox preferences.
    #[default]
    #[display("Post EQ")]
    PostEQ = 0x80,
    /// Named "PRE EQ" in the Rekordbox preferences.
    #[display("Pre EQ")]
    PreEQ,
}

impl TryFrom<String> for HeadphonesPreEQ {
    type Error = String;

    fn try_from(value: String) -> Result<HeadphonesPreEQ, Self::Error> {
        match value.as_str() {
            "PostEQ" => Ok(Self::PostEQ),
            "PreEQ" => Ok(Self::PreEQ),
            _ => Err(format!("Invalid HeadphonesPreEQ value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > HEADPHONES MONO SPLIT" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum HeadphonesMonoSplit {
    /// Named "MONO SPLIT" in the Rekordbox preferences.
    #[display("Mono Split")]
    MonoSplit = 0x81,
    /// Named "STEREO" in the Rekordbox preferences.
    #[default]
    Stereo = 0x80,
}

impl TryFrom<String> for HeadphonesMonoSplit {
    type Error = String;

    fn try_from(value: String) -> Result<HeadphonesMonoSplit, Self::Error> {
        match value.as_str() {
            "MonoSplit" => Ok(Self::MonoSplit),
            "Stereo" => Ok(Self::Stereo),
            _ => Err(format!("Invalid HeadphonesMonoSplit value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > BEAT FX QUANTIZE" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum BeatFXQuantize {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON" in the Rekordbox preferences.
    #[default]
    On,
}

impl TryFrom<String> for BeatFXQuantize {
    type Error = String;

    fn try_from(value: String) -> Result<BeatFXQuantize, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid BeatFXQuantize value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > MIC LOW CUT" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum MicLowCut {
    /// Named "OFF" in the Rekordbox preferences.
    Off = 0x80,
    /// Named "ON(for MC)" in the Rekordbox preferences.
    #[default]
    On,
}

impl TryFrom<String> for MicLowCut {
    type Error = String;

    fn try_from(value: String) -> Result<MicLowCut, Self::Error> {
        match value.as_str() {
            "Off" => Ok(Self::Off),
            "On" => Ok(Self::On),
            _ => Err(format!("Invalid MicLowCut value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > TALK OVER MODE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum TalkOverMode {
    /// Named "ADVANCED" in the Rekordbox preferences.
    #[default]
    Advanced = 0x80,
    /// Named "NORMAL" in the Rekordbox preferences.
    Normal,
}

impl TryFrom<String> for TalkOverMode {
    type Error = String;

    fn try_from(value: String) -> Result<TalkOverMode, Self::Error> {
        match value.as_str() {
            "Advanced" => Ok(Self::Advanced),
            "Normal" => Ok(Self::Normal),
            _ => Err(format!("Invalid TalkOverMode value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > TALK OVER LEVEL" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum TalkOverLevel {
    /// Named "-24dB" in the Rekordbox preferences.
    #[display("-24dB")]
    Minus24dB = 0x80,
    /// Named "-18dB" in the Rekordbox preferences.
    #[default]
    #[display("-18dB")]
    Minus18dB,
    /// Named "-12dB" in the Rekordbox preferences.
    #[display("-12dB")]
    Minus12dB,
    /// Named "-6dB" in the Rekordbox preferences.
    #[display("-6dB")]
    Minus6dB,
}

impl TryFrom<String> for TalkOverLevel {
    type Error = String;

    fn try_from(value: String) -> Result<TalkOverLevel, Self::Error> {
        match value.as_str() {
            "Minus24dB" => Ok(Self::Minus24dB),
            "Minus18dB" => Ok(Self::Minus18dB),
            "Minus12dB" => Ok(Self::Minus12dB),
            "Minus6dB" => Ok(Self::Minus6dB),
            _ => Err(format!("Invalid TalkOverLevel value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > MIDI CH" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum MidiChannel {
    /// Named "1" in the Rekordbox preferences.
    #[default]
    #[display("1")]
    One = 0x80,
    /// Named "2" in the Rekordbox preferences.
    #[display("2")]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[display("3")]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[display("4")]
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[display("5")]
    Five,
    /// Named "6" in the Rekordbox preferences.
    #[display("6")]
    Six,
    /// Named "7" in the Rekordbox preferences.
    #[display("7")]
    Seven,
    /// Named "8" in the Rekordbox preferences.
    #[display("8")]
    Eight,
    /// Named "9" in the Rekordbox preferences.
    #[display("9")]
    Nine,
    /// Named "10" in the Rekordbox preferences.
    #[display("10")]
    Ten,
    /// Named "11" in the Rekordbox preferences.
    #[display("11")]
    Eleven,
    /// Named "12" in the Rekordbox preferences.
    #[display("12")]
    Twelve,
    /// Named "13" in the Rekordbox preferences.
    #[display("13")]
    Thirteen,
    /// Named "14" in the Rekordbox preferences.
    #[display("14")]
    Fourteen,
    /// Named "15" in the Rekordbox preferences.
    #[display("15")]
    Fifteen,
    /// Named "16" in the Rekordbox preferences.
    #[display("16")]
    Sixteen,
}

impl TryFrom<String> for MidiChannel {
    type Error = String;

    fn try_from(value: String) -> Result<MidiChannel, Self::Error> {
        match value.as_str() {
            "One" => Ok(Self::One),
            "Two" => Ok(Self::Two),
            "Three" => Ok(Self::Three),
            "Four" => Ok(Self::Four),
            "Five" => Ok(Self::Five),
            "Six" => Ok(Self::Six),
            "Seven" => Ok(Self::Seven),
            "Eight" => Ok(Self::Eight),
            "Nine" => Ok(Self::Nine),
            "Ten" => Ok(Self::Ten),
            "Eleven" => Ok(Self::Eleven),
            "Twelve" => Ok(Self::Twelve),
            "Thirteen" => Ok(Self::Thirteen),
            "Fourteen" => Ok(Self::Fourteen),
            "Fifteen" => Ok(Self::Fifteen),
            "Sixteen" => Ok(Self::Sixteen),
            _ => Err(format!("Invalid MidiChannel value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > MIDI BUTTON TYPE" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum MidiButtonType {
    #[default]
    /// Named "TOGGLE" in the Rekordbox preferences.
    Toggle = 0x80,
    /// Named "TRIGGER" in the Rekordbox preferences.
    Trigger,
}

impl TryFrom<String> for MidiButtonType {
    type Error = String;

    fn try_from(value: String) -> Result<MidiButtonType, Self::Error> {
        match value.as_str() {
            "Toggle" => Ok(Self::Toggle),
            "Trigger" => Ok(Self::Trigger),
            _ => Err(format!("Invalid MidiButtonType value: {}", value)),
        }
    }
}

/// Found at "MIXER > BRIGHTNESS > DISPLAY" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum MixerDisplayBrightness {
    /// Named "WHITE" in the Rekordbox preferences.
    White = 0x80,
    /// Named "1" in the Rekordbox preferences.
    #[display("1")]
    One,
    /// Named "2" in the Rekordbox preferences.
    #[display("2")]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[display("3")]
    Three,
    /// Named "4" in the Rekordbox preferences.
    #[display("4")]
    Four,
    /// Named "5" in the Rekordbox preferences.
    #[default]
    #[display("5")]
    Five,
}

impl TryFrom<String> for MixerDisplayBrightness {
    type Error = String;

    fn try_from(value: String) -> Result<MixerDisplayBrightness, Self::Error> {
        match value.as_str() {
            "White" => Ok(Self::White),
            "One" => Ok(Self::One),
            "Two" => Ok(Self::Two),
            "Three" => Ok(Self::Three),
            "Four" => Ok(Self::Four),
            "Five" => Ok(Self::Five),
            _ => Err(format!("Invalid MixerDisplayBrightness value: {}", value)),
        }
    }
}

/// Found at "MIXER > BRIGHTNESS > INDICATOR" of the "My Settings" page in the Rekordbox
/// preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum MixerIndicatorBrightness {
    /// Named "1" in the Rekordbox preferences.
    #[display("1")]
    One = 0x80,
    /// Named "2" in the Rekordbox preferences.
    #[display("2")]
    Two,
    /// Named "3" in the Rekordbox preferences.
    #[display("3")]
    #[default]
    Three,
}

impl TryFrom<String> for MixerIndicatorBrightness {
    type Error = String;

    fn try_from(value: String) -> Result<MixerIndicatorBrightness, Self::Error> {
        match value.as_str() {
            "One" => Ok(Self::One),
            "Two" => Ok(Self::Two),
            "Three" => Ok(Self::Three),
            _ => Err(format!("Invalid MixerIndicatorBrightness value: {}", value)),
        }
    }
}

/// Found at "MIXER > DJ SETTING > CH FADER CURVE (LONG FADER)" of the "My Settings" page in the
/// Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum ChannelFaderCurveLongFader {
    /// Very steep volume raise when the fader is moved the near the top (e.g. y = x⁵).
    #[default]
    Exponential = 0x80,
    /// Steep volume raise when the fader is moved the near the top (e.g. y = x²).
    Smooth,
    /// Linear volume raise when the fader is moved (e.g. y = k * x).
    Linear,
}

impl TryFrom<String> for ChannelFaderCurveLongFader {
    type Error = String;

    fn try_from(value: String) -> Result<ChannelFaderCurveLongFader, Self::Error> {
        match value.as_str() {
            "Exponential" => Ok(Self::Exponential),
            "Smooth" => Ok(Self::Smooth),
            "Linear" => Ok(Self::Linear),
            _ => Err(format!(
                "Invalid ChannelFaderCurveLongFader value: {}",
                value
            )),
        }
    }
}

/// Payload of a `DJMMYSETTING.DAT` file (52 bytes).
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct DJMMySetting {
    /// Unknown field.
    unknown1: [u8; 12],
    /// "CH FADER CURVE" setting.
    pub channel_fader_curve: ChannelFaderCurve,
    /// "CROSSFADER CURVE" setting.
    pub crossfader_curve: CrossfaderCurve,
    /// "HEADPHONES PRE EQ" setting.
    pub headphones_pre_eq: HeadphonesPreEQ,
    /// "HEADPHONES MONO SPLIT" setting.
    pub headphones_mono_split: HeadphonesMonoSplit,
    /// "BEAT FX QUANTIZE" setting.
    pub beat_fx_quantize: BeatFXQuantize,
    /// "MIC LOW CUT" setting.
    pub mic_low_cut: MicLowCut,
    /// "TALK OVER MODE" setting.
    pub talk_over_mode: TalkOverMode,
    /// "TALK OVER LEVEL" setting.
    pub talk_over_level: TalkOverLevel,
    /// "MIDI CH" setting.
    pub midi_channel: MidiChannel,
    /// "MIDI BUTTON TYPE" setting.
    pub midi_button_type: MidiButtonType,
    /// "BRIGHTNESS > DISPLAY" setting.
    pub display_brightness: MixerDisplayBrightness,
    /// "BRIGHTNESS > INDICATOR" setting.
    pub indicator_brightness: MixerIndicatorBrightness,
    /// "CH FADER CURVE (LONG FADER)" setting.
    pub channel_fader_curve_long_fader: ChannelFaderCurveLongFader,
    /// Unknown field (apparently always 0).
    #[br(assert(unknown2 == [0; 27]))]
    unknown2: [u8; 27],
}

impl Default for DJMMySetting {
    fn default() -> Self {
        Self {
            unknown1: [
                0x78, 0x56, 0x34, 0x12, 0x01, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00,
            ],
            channel_fader_curve: ChannelFaderCurve::default(),
            crossfader_curve: CrossfaderCurve::default(),
            headphones_pre_eq: HeadphonesPreEQ::default(),
            headphones_mono_split: HeadphonesMonoSplit::default(),
            beat_fx_quantize: BeatFXQuantize::default(),
            mic_low_cut: MicLowCut::default(),
            talk_over_mode: TalkOverMode::default(),
            talk_over_level: TalkOverLevel::default(),
            midi_channel: MidiChannel::default(),
            midi_button_type: MidiButtonType::default(),
            display_brightness: MixerDisplayBrightness::default(),
            indicator_brightness: MixerIndicatorBrightness::default(),
            channel_fader_curve_long_fader: ChannelFaderCurveLongFader::default(),
            unknown2: [0; 27],
        }
    }
}

impl DJMMySetting {
    fn file_name() -> String {
        "DJMMYSETTING.DAT".into()
    }
}

// -- DevSetting -----------------------------------------------------------------------------------

/// Type of the Overview Waveform displayed on the CDJ.
///
/// Found on the "General" page in the Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum OverviewWaveformType {
    /// Named "Half Waveform" in the Rekordbox preferences.
    #[default]
    #[display("Half Waveform")]
    HalfWaveform = 0x01,
    /// Named "Full Waveform" in the Rekordbox preferences.
    #[display("Full Waveform")]
    FullWaveform,
}

impl TryFrom<String> for OverviewWaveformType {
    type Error = String;

    fn try_from(value: String) -> Result<OverviewWaveformType, Self::Error> {
        match value.as_str() {
            "HalfWaveform" => Ok(Self::HalfWaveform),
            "FullWaveform" => Ok(Self::FullWaveform),
            _ => Err(format!("Invalid OverviewWaveformType value: {}", value)),
        }
    }
}

/// Waveform color displayed on the CDJ.
///
/// Found on the "General" page in the Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum WaveformColor {
    /// Named "BLUE" in the Rekordbox preferences.
    #[default]
    Blue = 0x01,
    /// Named "RGB" in the Rekordbox preferences.
    #[display("RGB")]
    Rgb = 0x03,
    /// Named "3Band" in the Rekordbox preferences.
    #[display("3Band")]
    TriBand = 0x04,
}

impl TryFrom<String> for WaveformColor {
    type Error = String;

    fn try_from(value: String) -> Result<WaveformColor, Self::Error> {
        match value.as_str() {
            "Blue" => Ok(Self::Blue),
            "Rgb" => Ok(Self::Rgb),
            "TriBand" => Ok(Self::TriBand),
            _ => Err(format!("Invalid WaveformColor value: {}", value)),
        }
    }
}

/// The key display format displayed on the CDJ.
///
/// Found on the "General" page in the Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum KeyDisplayFormat {
    /// Named "Classic" in the Rekordbox preferences.
    #[default]
    Classic = 0x01,
    /// Named "Alphanumeric" in the Rekordbox preferences.
    Alphanumeric,
}

impl TryFrom<String> for KeyDisplayFormat {
    type Error = String;

    fn try_from(value: String) -> Result<KeyDisplayFormat, Self::Error> {
        match value.as_str() {
            "Classic" => Ok(Self::Classic),
            "Alphanumeric" => Ok(Self::Alphanumeric),
            _ => Err(format!("Invalid KeyDisplayFormat value: {}", value)),
        }
    }
}

/// Waveform Current Position displayed on the CDJ.
///
/// Found on the "General" page in the Rekordbox preferences.
#[binrw]
#[derive(Display, Debug, PartialEq, Eq, Default, Clone, Copy)]
#[brw(repr = u8)]
pub enum WaveformCurrentPosition {
    /// Named "LEFT" in the Rekordbox preferences.
    Left = 0x02,
    /// Named "CENTER" in the Rekordbox preferences.
    #[default]
    Center = 0x01,
}

impl TryFrom<String> for WaveformCurrentPosition {
    type Error = String;

    fn try_from(value: String) -> Result<WaveformCurrentPosition, Self::Error> {
        match value.as_str() {
            "Left" => Ok(Self::Left),
            "Center" => Ok(Self::Center),
            _ => Err(format!("Invalid WaveformCurrentPosition value: {}", value)),
        }
    }
}

/// Payload of a `DEVSETTING.DAT` file (32 bytes).
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
pub struct DevSetting {
    /// Unknown field.
    #[br(assert(unknown1 == [0x78, 0x56, 0x34, 0x12, 0x01, 0x00, 0x00, 0x00, 0x01]))]
    unknown1: [u8; 9],
    /// "Type of the overview Waveform" setting.
    pub overview_waveform_type: OverviewWaveformType,
    /// "Waveform color" setting.
    pub waveform_color: WaveformColor,
    /// Unknown field.
    #[br(assert(unknown2 == 0x01))]
    unknown2: u8,
    /// "Key display format" setting.
    pub key_display_format: KeyDisplayFormat,
    /// "Waveform Current Position" setting.
    pub waveform_current_position: WaveformCurrentPosition,
    /// Unknown field.
    #[br(assert(unknown3 == [0x00; 18]))]
    unknown3: [u8; 18],
}

impl Default for DevSetting {
    fn default() -> Self {
        Self {
            unknown1: [0x78, 0x56, 0x34, 0x12, 0x01, 0x00, 0x00, 0x00, 0x01],
            overview_waveform_type: OverviewWaveformType::default(),
            waveform_color: WaveformColor::default(),
            key_display_format: KeyDisplayFormat::default(),
            unknown2: 0x01,
            waveform_current_position: WaveformCurrentPosition::default(),
            unknown3: [0x00; 18],
        }
    }
}

impl DevSetting {
    fn file_name() -> String {
        "DEVSETTING.DAT".into()
    }
}

// -- Setting file data ----------------------------------------------------------------------------

/// Data section of a `*SETTING.DAT` file.
#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
#[br(import(len: u32))]
pub enum SettingContent {
    /// Payload of a `MYSETTING.DAT` file (40 bytes).
    #[br(pre_assert(len == 40))]
    MySetting(MySetting),
    // /// Payload of a `MYSETTING2.DAT` file (40 bytes).
    #[br(pre_assert(len == 40))]
    MySetting2(MySetting2),
    /// Payload of a `DJMMYSETTING.DAT` file (52 bytes).
    #[br(pre_assert(len == 52))]
    DJMMySetting(DJMMySetting),
    /// Payload of a `DEVSETTING.DAT` file (32 bytes).
    #[br(pre_assert(len == 32))]
    DevSetting(DevSetting),
}

impl SettingContent {
    fn size(&self) -> u32 {
        match &self {
            Self::MySetting(_) => 40,
            Self::MySetting2(_) => 40,
            Self::DJMMySetting(_) => 52,
            Self::DevSetting(_) => 32,
        }
    }
}

#[binrw]
#[derive(Debug, PartialEq, Eq, Clone)]
#[brw(little)]
#[bw(import(no_checksum: bool))]
/// Represents the data of a Setting file.
pub struct SettingData {
    /// Size of the string data field (should be always 96).
    #[br(temp, assert(len_stringdata == 0x60))]
    #[bw(calc = 0x60)]
    len_stringdata: u32,
    /// Name of the brand (depends on the kind of file).
    #[brw(pad_size_to = 0x20, assert(brand.len() <= (0x20 - 1)))]
    pub brand: NullString,
    /// Name of the software ("rekordbox").
    #[brw(pad_size_to = 0x20, assert(software.len() <= (0x20 - 1)))]
    pub software: NullString,
    /// Some kind of version number.
    #[brw(pad_size_to = 0x20, assert(version.len() <= (0x20 - 1)))]
    pub version: NullString,
    /// Size of the `data` data in bytes.
    #[br(temp)]
    #[bw(calc = content.size())]
    len_data: u32,
    /// The actual settings data.
    #[br(args(len_data))]
    pub content: SettingContent,
    /// CRC16 XMODEM checksum. The checksum is calculated over the contents of the `data`
    /// field, except for `DJMSETTING.DAT` files where the checksum is calculated over all
    /// preceding bytes including the length fields.
    ///
    /// See <https://reveng.sourceforge.io/crc-catalogue/all.htm#crc.cat.crc-16-xmodem> for details.
    #[br(temp)]
    #[bw(calc = no_checksum.then_some(0).unwrap_or_else(|| self.calculate_checksum()))]
    _checksum: u16,
    /// Unknown field (apparently always `0000`).
    #[br(temp)]
    #[br(assert(unknown == 0))]
    #[bw(calc = 0u16)]
    unknown: u16,
}

impl SettingData
where
    SettingData: BinWrite,
{
    /// Calculate the CRC16 checksum.
    ///
    /// This is horribly inefficient and basically serializes the whole data structure twice, but
    /// there seems to be no other way to achieve this.
    ///
    /// See https://github.com/jam1garner/binrw/issues/102
    fn calculate_checksum(&self) -> u16 {
        let mut data = Vec::<u8>::with_capacity(156);
        let mut writer = Cursor::new(&mut data);
        self.write_options(&mut writer, Endian::Little, (true,))
            .unwrap();
        let start = match self.content {
            // In `DJMMYSETTING.DAT`, the checksum is calculated over all previous bytes, including
            // the section lengths and string data.
            SettingContent::DJMMySetting(_) => 0,
            // In all other files`, the checksum is calculated just over the data section which
            // starts at offset 104,
            _ => 104,
        };

        let end = data.len() - 4;
        crc16::State::<crc16::XMODEM>::calculate(&data[start..end])
    }
}

impl SettingData {
    /// Create a new object containing with the given brand string and data.
    #[must_use]
    fn default_with_brand_and_data(brand: NullString, content: SettingContent) -> Self {
        Self {
            brand,
            software: "rekordbox".into(),
            version: "6.6.1".into(),
            content,
        }
    }

    /// Create a new object containing the default values of a `MYSETTING.DAT` file.
    #[must_use]
    pub fn default_mysetting() -> Self {
        Self::default_with_brand_and_data(
            "PIONEER".into(),
            SettingContent::MySetting(MySetting::default()),
        )
    }

    /// Create a new object containing the default values of a `MYSETTING2.DAT` file.
    #[must_use]
    pub fn default_mysetting2() -> Self {
        Self::default_with_brand_and_data(
            "PIONEER".into(),
            SettingContent::MySetting2(MySetting2::default()),
        )
    }

    /// Create a new object containing the default values of a `DJMMYSETTING.DAT` file.
    #[must_use]
    pub fn default_djmmysetting() -> Self {
        Self::default_with_brand_and_data(
            "PioneerDJ".into(),
            SettingContent::DJMMySetting(DJMMySetting::default()),
        )
    }

    /// Create a new object containing the default values of a `DEVSETTING.DAT` file.
    #[must_use]
    pub fn default_devsetting() -> Self {
        Self::default_with_brand_and_data(
            "PIONEER DJ".into(),
            SettingContent::DevSetting(DevSetting::default()),
        )
    }

    fn file_name(&self) -> &str {
        match &self.content {
            SettingContent::MySetting(_) => "MYSETTING.DAT",
            SettingContent::MySetting2(_) => "MYSETTING2.DAT",
            SettingContent::DJMMySetting(_) => "DJMMYSETTING.DAT",
            SettingContent::DevSetting(_) => "DEVSETTING.DAT",
        }
    }
}

// -- Setting file handler -------------------------------------------------------------------------

/// Rekordbox `*SETTING.DAT` file handler.
pub struct Setting {
    /// Setting file path.
    path: std::path::PathBuf,
    /// Setting file data.
    pub data: SettingData,
}

impl Setting {
    /// Create a new `MYSETTING.DAT` file filled with the default values.
    pub fn new_mysetting<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> anyhow::Result<Self> {
        let p = Path::new(&path).to_path_buf();
        let file_name = p.file_name().unwrap().to_str().unwrap();
        if file_name != &MySetting::file_name() {
            return Err(anyhow!(format!(
                "Invalid file name: expected {}, got {:?}",
                MySetting::file_name(),
                file_name
            )));
        }
        let data = SettingData::default_mysetting();
        Ok(Self { path: p, data })
    }

    /// Create a new `MYSETTING2.DAT` file filled with the default values.
    pub fn new_mysetting2<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> anyhow::Result<Self> {
        let p = Path::new(&path).to_path_buf();
        let file_name = p.file_name().unwrap().to_str().unwrap();
        if file_name != &MySetting2::file_name() {
            return Err(anyhow!(format!(
                "Invalid file name: expected {}, got {:?}",
                MySetting::file_name(),
                file_name
            )));
        }
        let data = SettingData::default_mysetting2();
        Ok(Self { path: p, data })
    }

    /// Create a new `DJMMYSETTING.DAT` file filled with the default values.
    pub fn new_djmmysetting<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> anyhow::Result<Self> {
        let p = Path::new(&path).to_path_buf();
        let file_name = p.file_name().unwrap().to_str().unwrap();
        if file_name != &DJMMySetting::file_name() {
            return Err(anyhow!(format!(
                "Invalid file name: expected {}, got {:?}",
                MySetting::file_name(),
                file_name
            )));
        }
        let data = SettingData::default_djmmysetting();
        Ok(Self { path: p, data })
    }

    /// Create a new `DEVSETTING.DAT` file filled with the default values.
    pub fn new_devsetting<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> anyhow::Result<Self> {
        let p = Path::new(&path).to_path_buf();
        let file_name = p.file_name().unwrap().to_str().unwrap();
        if file_name != &DevSetting::file_name() {
            return Err(anyhow!(format!(
                "Invalid file name: expected {}, got {:?}",
                MySetting::file_name(),
                file_name
            )));
        }
        let data = SettingData::default_devsetting();
        Ok(Self { path: p, data })
    }

    /// Read a Rekordbox Setting file.
    pub fn load<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> anyhow::Result<Self> {
        let p = Path::new(&path).to_path_buf();
        let mut file = std::fs::File::open(&p).expect("File not found");
        let data = SettingData::read(&mut file).expect("Can't read Setting file");
        Ok(Self { path: p, data })
    }

    /// Write the Setting file.
    pub fn dump_copy<P: AsRef<Path> + AsRef<OsStr>>(&mut self, path: P) -> anyhow::Result<()> {
        let p = Path::new(&path).to_path_buf();
        let file_name = p.file_name().unwrap().to_str().unwrap();
        if file_name != self.data.file_name() {
            return Err(anyhow!(format!(
                "Invalid file name: expected {}, got {:?}",
                self.data.file_name(),
                file_name
            )));
        }
        let mut file = std::fs::File::create(path).expect("Failed to create file");
        self.data.write(&mut file)?;
        Ok(())
    }

    /// Write the Setting file to the original file.
    pub fn dump(&mut self) -> anyhow::Result<()> {
        let path = &self.path.clone();
        self.dump_copy(path)?;
        Ok(())
    }

    // -- MySetting getters and setters ------------------------------------------------------------

    /// Gets the value of the "ON AIR DISPLAY" setting.
    ///
    /// # Returns
    /// - `Ok(OnAirDisplay)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_on_air_display(&self) -> anyhow::Result<OnAirDisplay> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.on_air_display)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "ON AIR DISPLAY" setting.
    ///
    /// # Parameters
    /// - `on_air_display`: The new value of the `OnAirDisplay` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_on_air_display(&mut self, on_air_display: OnAirDisplay) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.on_air_display = on_air_display;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "LCD BRIGHTNESS" setting.
    ///
    /// # Returns
    /// - `Ok(LCDBrightness)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_lcd_brightness(&self) -> anyhow::Result<LCDBrightness> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.lcd_brightness)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "LCD BRIGHTNESS" setting.
    ///
    /// # Parameters
    /// - `lcd_brightness`: The new value of the `LCDBrightness` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_lcd_brightness(&mut self, lcd_brightness: LCDBrightness) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.lcd_brightness = lcd_brightness;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "QUANTIZE" setting.
    ///
    /// # Returns
    /// - `Ok(Quantize)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_quantize(&self) -> anyhow::Result<Quantize> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.quantize)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "QUANTIZE" setting.
    ///
    /// # Parameters
    /// - `quantize`: The new value of the `Quantize` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_quantize(&mut self, quantize: Quantize) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.quantize = quantize;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "AUTO CUE LEVEL" setting.
    ///
    /// # Returns
    /// - `Ok(AutoCueLevel)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_auto_cue_level(&self) -> anyhow::Result<AutoCueLevel> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.auto_cue_level)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "AUTO CUE LEVEL" setting.
    ///
    /// # Parameters
    /// - `auto_cue_level`: The new value of the `AutoCueLevel` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_auto_cue_level(&mut self, auto_cue_level: AutoCueLevel) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.auto_cue_level = auto_cue_level;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "LANGUAGE" setting.
    ///
    /// # Returns
    /// - `Ok(Language)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_language(&self) -> anyhow::Result<Language> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.language)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "LANGUAGE" setting.
    ///
    /// # Parameters
    /// - `language`: The new value of the `Language` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_language(&mut self, language: Language) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.language = language;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "JOG RING BRIGHTNESS" setting.
    ///
    /// # Returns
    /// - `Ok(JogRingBrightness)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_jog_ring_brightness(&self) -> anyhow::Result<JogRingBrightness> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.jog_ring_brightness)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "JOG RING BRIGHTNESS" setting.
    ///
    /// # Parameters
    /// - `jog_ring_brightness`: The new value of the `JogRingBrightness` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_jog_ring_brightness(
        &mut self,
        jog_ring_brightness: JogRingBrightness,
    ) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.jog_ring_brightness = jog_ring_brightness;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "JOG RING INDICATOR" setting.
    ///
    /// # Returns
    /// - `Ok(JogRingIndicator)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_jog_ring_indicator(&self) -> anyhow::Result<JogRingIndicator> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.jog_ring_indicator)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "JOG RING INDICATOR" setting.
    ///
    /// # Parameters
    /// - `jog_ring_indicator`: The new value of the `JogRingIndicator` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_jog_ring_indicator(
        &mut self,
        jog_ring_indicator: JogRingIndicator,
    ) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.jog_ring_indicator = jog_ring_indicator;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }
    /// Gets the value of the "SLIP FLASHING" setting.
    ///
    /// # Returns
    /// - `Ok(SlipFlashing)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_slip_flashing(&self) -> anyhow::Result<SlipFlashing> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.slip_flashing)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "SLIP FLASHING" setting.
    ///
    /// # Parameters
    /// - `slip_flashing`: The new value of the `SlipFlashing` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_slip_flashing(&mut self, slip_flashing: SlipFlashing) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.slip_flashing = slip_flashing;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "DISC SLOT ILLUMINATION" setting.
    ///
    /// # Returns
    /// - `Ok(DiscSlotIllumination)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_disc_slot_illumination(&self) -> anyhow::Result<DiscSlotIllumination> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.disc_slot_illumination)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "DISC SLOT ILLUMINATION" setting.
    ///
    /// # Parameters
    /// - `disc_slot_illumination`: The new value of the `DiscSlotIllumination` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_disc_slot_illumination(
        &mut self,
        disc_slot_illumination: DiscSlotIllumination,
    ) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.disc_slot_illumination = disc_slot_illumination;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "EJECT/LOAD LOCK" setting.
    ///
    /// # Returns
    /// - `Ok(EjectLock)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_eject_lock(&self) -> anyhow::Result<EjectLock> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.eject_lock)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "EJECT/LOAD LOCK" setting.
    ///
    /// # Parameters
    /// - `eject_lock`: The new value of the `EjectLock` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_eject_lock(&mut self, eject_lock: EjectLock) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.eject_lock = eject_lock;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "SYNC" setting.
    ///
    /// # Returns
    /// - `Ok(Sync)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_sync(&self) -> anyhow::Result<Sync> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.sync)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "SYNC" setting.
    ///
    /// # Parameters
    /// - `sync`: The new value of the `Sync` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_sync(&mut self, sync: Sync) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.sync = sync;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }
    /// Gets the value of the "PLAY MODE" setting.
    ///
    /// # Returns
    /// - `Ok(PlayMode)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_play_mode(&self) -> anyhow::Result<PlayMode> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.play_mode)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "PLAY MODE" setting.
    ///
    /// # Parameters
    /// - `play_mode`: The new value of the `PlayMode` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_play_mode(&mut self, play_mode: PlayMode) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.play_mode = play_mode;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "QUANTIZE BEAT VALUE" setting.
    ///
    /// # Returns
    /// - `Ok(QuantizeBeatValue)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_quantize_beat_value(&self) -> anyhow::Result<QuantizeBeatValue> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.quantize_beat_value)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "QUANTIZE BEAT VALUE" setting.
    ///
    /// # Parameters
    /// - `quantize_beat_value`: The new value of the `QuantizeBeatValue` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_quantize_beat_value(
        &mut self,
        quantize_beat_value: QuantizeBeatValue,
    ) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.quantize_beat_value = quantize_beat_value;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "HOT CUE AUTO LOAD" setting.
    ///
    /// # Returns
    /// - `Ok(HotCueAutoLoad)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_hotcue_autoload(&self) -> anyhow::Result<HotCueAutoLoad> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.hotcue_autoload)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "HOT CUE AUTO LOAD" setting.
    ///
    /// # Parameters
    /// - `hot_cue_autoload`: The new value of the `HotCueAutoLoad` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_hotcue_autoload(&mut self, hotcue_autoload: HotCueAutoLoad) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.hotcue_autoload = hotcue_autoload;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "HOT CUE COLOR" setting.
    ///
    /// # Returns
    /// - `Ok(HotCueColor)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_hotcue_color(&self) -> anyhow::Result<HotCueColor> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.hotcue_color)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "HOT CUE COLOR" setting.
    ///
    /// # Parameters
    /// - `hot_cue_color`: The new value of the `HotCueColor` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_hotcue_color(&mut self, hotcue_color: HotCueColor) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.hotcue_color = hotcue_color;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "NEEDLE LOCK" setting.
    ///
    /// # Returns
    /// - `Ok(NeedleLock)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_needle_lock(&self) -> anyhow::Result<NeedleLock> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.needle_lock)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "NEEDLE LOCK" setting.
    ///
    /// # Parameters
    /// - `needle_lock`: The new value of the `NeedleLock` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_needle_lock(&mut self, needle_lock: NeedleLock) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.needle_lock = needle_lock;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "TIME MODE" setting.
    ///
    /// # Returns
    /// - `Ok(TimeMode)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_time_mode(&self) -> anyhow::Result<TimeMode> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.time_mode)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "TIME MODE" setting.
    ///
    /// # Parameters
    /// - `time_mode`: The new value of the `TimeMode` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_time_mode(&mut self, time_mode: TimeMode) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.time_mode = time_mode;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "JOG MODE" setting.
    ///
    /// # Returns
    /// - `Ok(JogMode)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_jog_mode(&self) -> anyhow::Result<JogMode> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.jog_mode)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "JOG MODE" setting.
    ///
    /// # Parameters
    /// - `jog_mode`: The new value of the `JogMode` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_jog_mode(&mut self, jog_mode: JogMode) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.jog_mode = jog_mode;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "AUTO CUE" setting.
    ///
    /// # Returns
    /// - `Ok(AutoCue)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_auto_cue(&self) -> anyhow::Result<AutoCue> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.auto_cue)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "AUTO CUE" setting.
    ///
    /// # Parameters
    /// - `auto_cue`: The new value of the `AutoCue` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_auto_cue(&mut self, auto_cue: AutoCue) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.auto_cue = auto_cue;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "MASTER TEMPO" setting.
    ///
    /// # Returns
    /// - `Ok(MasterTempo)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_master_tempo(&self) -> anyhow::Result<MasterTempo> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.master_tempo)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "MASTER TEMPO" setting.
    ///
    /// # Parameters
    /// - `master_tempo`: The new value of the `MasterTempo` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_master_tempo(&mut self, master_tempo: MasterTempo) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.master_tempo = master_tempo;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "TEMPO RANGE" setting.
    ///
    /// # Returns
    /// - `Ok(TempoRange)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_tempo_range(&self) -> anyhow::Result<TempoRange> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.tempo_range)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "TEMPO RANGE" setting.
    ///
    /// # Parameters
    /// - `tempo_range`: The new value of the `TempoRange` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_tempo_range(&mut self, tempo_range: TempoRange) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.tempo_range = tempo_range;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Gets the value of the "PHASE METER" setting.
    ///
    /// # Returns
    /// - `Ok(PhaseMeter)` if the `SettingContent` is of type `MySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn get_phase_meter(&self) -> anyhow::Result<PhaseMeter> {
        if let SettingContent::MySetting(content) = &self.data.content {
            Ok(content.phase_meter)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    /// Sets the value of the "PHASE METER" setting.
    ///
    /// # Parameters
    /// - `phase_meter`: The new value of the `PhaseMeter` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting` file.
    pub fn set_phase_meter(&mut self, phase_meter: PhaseMeter) -> anyhow::Result<()> {
        if let SettingContent::MySetting(content) = &mut self.data.content {
            content.phase_meter = phase_meter;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting::file_name())))
        }
    }

    // -- MySetting2 getters and setters --------------------------------------------------------------

    /// Gets the value of the "VINYL SPEED ADJUST" setting.
    ///
    /// # Returns
    /// - `Ok(VinylSpeedAdjust)` if the `SettingContent` is of type `MySetting2`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn get_vinyl_speed_adjust(&self) -> anyhow::Result<VinylSpeedAdjust> {
        if let SettingContent::MySetting2(content) = &self.data.content {
            Ok(content.vinyl_speed_adjust)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Sets the value of the "VINYL SPEED ADJUST" setting.
    ///
    /// # Parameters
    /// - `vinyl_speed_adjust`: The new value of the `VinylSpeedAdjust` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn set_vinyl_speed_adjust(
        &mut self,
        vinyl_speed_adjust: VinylSpeedAdjust,
    ) -> anyhow::Result<()> {
        if let SettingContent::MySetting2(content) = &mut self.data.content {
            content.vinyl_speed_adjust = vinyl_speed_adjust;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Gets the value of the "JOG DISPLAY MODE" setting.
    ///
    /// # Returns
    /// - `Ok(JogDisplayMode)` if the `SettingContent` is of type `MySetting2`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn get_jog_display_mode(&self) -> anyhow::Result<JogDisplayMode> {
        if let SettingContent::MySetting2(content) = &self.data.content {
            Ok(content.jog_display_mode)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Sets the value of the "JOG DISPLAY MODE" setting.
    ///
    /// # Parameters
    /// - `jog_display_mode`: The new value of the `JogDisplayMode` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn set_jog_display_mode(&mut self, jog_display_mode: JogDisplayMode) -> anyhow::Result<()> {
        if let SettingContent::MySetting2(content) = &mut self.data.content {
            content.jog_display_mode = jog_display_mode;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Gets the value of the "PAD/BUTTON BRIGHTNESS" setting.
    ///
    /// # Returns
    /// - `Ok(PadButtonBrightness)` if the `SettingContent` is of type `MySetting2`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn get_pad_button_brightness(&self) -> anyhow::Result<PadButtonBrightness> {
        if let SettingContent::MySetting2(content) = &self.data.content {
            Ok(content.pad_button_brightness)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Sets the value of the "PAD/BUTTON BRIGHTNESS" setting.
    ///
    /// # Parameters
    /// - `pad_button_brightness`: The new value of the `PadButtonBrightness` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn set_pad_button_brightness(
        &mut self,
        pad_button_brightness: PadButtonBrightness,
    ) -> anyhow::Result<()> {
        if let SettingContent::MySetting2(content) = &mut self.data.content {
            content.pad_button_brightness = pad_button_brightness;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Gets the value of the "JOG LCD BRIGHTNESS" setting.
    ///
    /// # Returns
    /// - `Ok(JogLCDBrightness)` if the `SettingContent` is of type `MySetting2`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn get_jog_lcd_brightness(&self) -> anyhow::Result<JogLCDBrightness> {
        if let SettingContent::MySetting2(content) = &self.data.content {
            Ok(content.jog_lcd_brightness)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Sets the value of the "JOG LCD BRIGHTNESS" setting.
    ///
    /// # Parameters
    /// - `jog_lcd_brightness`: The new value of the `JogLCDBrightness` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn set_jog_lcd_brightness(
        &mut self,
        jog_lcd_brightness: JogLCDBrightness,
    ) -> anyhow::Result<()> {
        if let SettingContent::MySetting2(content) = &mut self.data.content {
            content.jog_lcd_brightness = jog_lcd_brightness;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Gets the value of the "WAVEFORM DIVISIONS" setting.
    ///
    /// # Returns
    /// - `Ok(WaveformDivisions)` if the `SettingContent` is of type `MySetting2`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn get_waveform_divisions(&self) -> anyhow::Result<WaveformDivisions> {
        if let SettingContent::MySetting2(content) = &self.data.content {
            Ok(content.waveform_divisions)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Sets the value of the "WAVEFORM DIVISIONS" setting.
    ///
    /// # Parameters
    /// - `waveform_divisions`: The new value of the `WaveformDivisions` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn set_waveform_divisions(
        &mut self,
        waveform_divisions: WaveformDivisions,
    ) -> anyhow::Result<()> {
        if let SettingContent::MySetting2(content) = &mut self.data.content {
            content.waveform_divisions = waveform_divisions;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Gets the value of the "WAVEFORM / PHASE METER" setting.
    ///
    /// # Returns
    /// - `Ok(Waveform)` if the `SettingContent` is of type `MySetting2`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn get_waveform(&self) -> anyhow::Result<Waveform> {
        if let SettingContent::MySetting2(content) = &self.data.content {
            Ok(content.waveform)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Sets the value of the "WAVEFORM / PHASE METER" setting.
    ///
    /// # Parameters
    /// - `waveform`: The new value of the `Waveform` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn set_waveform(&mut self, waveform: Waveform) -> anyhow::Result<()> {
        if let SettingContent::MySetting2(content) = &mut self.data.content {
            content.waveform = waveform;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Gets the value of the "BEAT JUMP BEAT VALUE" setting.
    ///
    /// # Returns
    /// - `Ok(BeatJumpBeatValue)` if the `SettingContent` is of type `MySetting2`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn get_beat_jump_beat_value(&self) -> anyhow::Result<BeatJumpBeatValue> {
        if let SettingContent::MySetting2(content) = &self.data.content {
            Ok(content.beat_jump_beat_value)
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    /// Sets the value of the "BEAT JUMP BEAT VALUE" setting.
    ///
    /// # Parameters
    /// - `beat_jump_beat_value`: The new value of the `BeatJumpBeatValue` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `MySetting2`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `MySetting2` file.
    pub fn set_beat_jump_beat_value(
        &mut self,
        beat_jump_beat_value: BeatJumpBeatValue,
    ) -> anyhow::Result<()> {
        if let SettingContent::MySetting2(content) = &mut self.data.content {
            content.beat_jump_beat_value = beat_jump_beat_value;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", MySetting2::file_name())))
        }
    }

    // -- DJMMySetting getters and setters -----------------------------------------------------------

    /// Gets the value of the "CH FADER CURVE" setting.
    ///
    /// # Returns
    /// - `Ok(ChannelFaderCurve)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_channel_fader_curve(&self) -> anyhow::Result<ChannelFaderCurve> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.channel_fader_curve)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "CH FADER CURVE" setting.
    ///
    /// # Parameters
    /// - `channel_fader_curve`: The new value of the `ChannelFaderCurve` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_channel_fader_curve(
        &mut self,
        channel_fader_curve: ChannelFaderCurve,
    ) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.channel_fader_curve = channel_fader_curve;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "CROSSFADER CURVE" setting.
    ///
    /// # Returns
    /// - `Ok(CrossfaderCurve)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_crossfader_curve(&self) -> anyhow::Result<CrossfaderCurve> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.crossfader_curve)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "CROSSFADER CURVE" setting.
    ///
    /// # Parameters
    /// - `crossfader_curve`: The new value of the `CrossfaderCurve` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_crossfader_curve(
        &mut self,
        crossfader_curve: CrossfaderCurve,
    ) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.crossfader_curve = crossfader_curve;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "HEADPHONES PRE EQ" setting.
    ///
    /// # Returns
    /// - `Ok(HeadphonesPreEQ)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_headphones_pre_eq(&self) -> anyhow::Result<HeadphonesPreEQ> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.headphones_pre_eq)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "HEADPHONES PRE EQ" setting.
    ///
    /// # Parameters
    /// - `headphones_pre_eq`: The new value of the `HeadphonesPreEQ` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_headphones_pre_eq(
        &mut self,
        headphones_pre_eq: HeadphonesPreEQ,
    ) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.headphones_pre_eq = headphones_pre_eq;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "HEADPHONES MONO SPLIT" setting.
    ///
    /// # Returns
    /// - `Ok(HeadphonesMonoSplit)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_headphones_mono_split(&self) -> anyhow::Result<HeadphonesMonoSplit> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.headphones_mono_split)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "HEADPHONES MONO SPLIT" setting.
    ///
    /// # Parameters
    /// - `headphones_mono_split`: The new value of the `HeadphonesMonoSplit` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_headphones_mono_split(
        &mut self,
        headphones_mono_split: HeadphonesMonoSplit,
    ) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.headphones_mono_split = headphones_mono_split;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "BEAT FX QUANTIZE" setting.
    ///
    /// # Returns
    /// - `Ok(BeatFXQuantize)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_beat_fx_quantize(&self) -> anyhow::Result<BeatFXQuantize> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.beat_fx_quantize)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "BEAT FX QUANTIZE" setting.
    ///
    /// # Parameters
    /// - `beat_fx_quantize`: The new value of the `BeatFXQuantize` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_beat_fx_quantize(&mut self, beat_fx_quantize: BeatFXQuantize) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.beat_fx_quantize = beat_fx_quantize;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "MIC LOW CUT" setting.
    ///
    /// # Returns
    /// - `Ok(MicLowCut)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_mic_low_cut(&self) -> anyhow::Result<MicLowCut> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.mic_low_cut)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "MIC LOW CUT" setting.
    ///
    /// # Parameters
    /// - `mic_low_cut`: The new value of the `MicLowCut` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_mic_low_cut(&mut self, mic_low_cut: MicLowCut) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.mic_low_cut = mic_low_cut;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "TALK OVER MODE" setting.
    ///
    /// # Returns
    /// - `Ok(TalkOverMode)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_talk_over_mode(&self) -> anyhow::Result<TalkOverMode> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.talk_over_mode)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "TALK OVER MODE" setting.
    ///
    /// # Parameters
    /// - `talk_over_mode`: The new value of the `TalkOverMode` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_talk_over_mode(&mut self, talk_over_mode: TalkOverMode) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.talk_over_mode = talk_over_mode;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "TALK OVER LEVEL" setting.
    ///
    /// # Returns
    /// - `Ok(TalkOverLevel)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_talk_over_level(&self) -> anyhow::Result<TalkOverLevel> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.talk_over_level)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "TALK OVER LEVEL" setting.
    ///
    /// # Parameters
    /// - `talk_over_level`: The new value of the `TalkOverLevel` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_talk_over_level(&mut self, talk_over_level: TalkOverLevel) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.talk_over_level = talk_over_level;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "MIDI CH" setting.
    ///
    /// # Returns
    /// - `Ok(MidiChannel)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_midi_channel(&self) -> anyhow::Result<MidiChannel> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.midi_channel)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "MIDI CH" setting.
    ///
    /// # Parameters
    /// - `midi_channel`: The new value of the `MidiChannel` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_midi_channel(&mut self, midi_channel: MidiChannel) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.midi_channel = midi_channel;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "MIDI BUTTON TYPE" setting.
    ///
    /// # Returns
    /// - `Ok(MidiButtonType)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_midi_button_type(&self) -> anyhow::Result<MidiButtonType> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.midi_button_type)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "MIDI BUTTON TYPE" setting.
    ///
    /// # Parameters
    /// - `midi_button_type`: The new value of the `MidiButtonType` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_midi_button_type(&mut self, midi_button_type: MidiButtonType) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.midi_button_type = midi_button_type;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "MIXER DISPLAY BRIGHTNESS" setting.
    ///
    /// # Returns
    /// - `Ok(MixerDisplayBrightness)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_mixer_display_brightness(&self) -> anyhow::Result<MixerDisplayBrightness> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.display_brightness)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "MIXER DISPLAY BRIGHTNESS" setting.
    ///
    /// # Parameters
    /// - `mixer_display_brightness`: The new value of the `MixerDisplayBrightness` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_mixer_display_brightness(
        &mut self,
        mixer_display_brightness: MixerDisplayBrightness,
    ) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.display_brightness = mixer_display_brightness;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "MIXER INDICATOR BRIGHTNESS" setting.
    ///
    /// # Returns
    /// - `Ok(MixerIndicatorBrightness)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_mixer_indicator_brightness(&self) -> anyhow::Result<MixerIndicatorBrightness> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.indicator_brightness)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "MIXER INDICATOR BRIGHTNESS" setting.
    ///
    /// # Parameters
    /// - `mixer_indicator_brightness`: The new value of the `MixerIndicatorBrightness` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_mixer_indicator_brightness(
        &mut self,
        mixer_indicator_brightness: MixerIndicatorBrightness,
    ) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.indicator_brightness = mixer_indicator_brightness;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Gets the value of the "CH FADER CURVE (LONG FADER)" setting.
    ///
    /// # Returns
    /// - `Ok(ChannelFaderCurveLongFader)` if the `SettingContent` is of type `DJMMySetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn get_channel_fader_curve_long_fader(&self) -> anyhow::Result<ChannelFaderCurveLongFader> {
        if let SettingContent::DJMMySetting(content) = &self.data.content {
            Ok(content.channel_fader_curve_long_fader)
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    /// Sets the value of the "CH FADER CURVE (LONG FADER)" setting.
    ///
    /// # Parameters
    /// - `channel_fader_curve_long_fader`: The new value of the `ChannelFaderCurveLongFader` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DJMMySetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DJMMySetting` file.
    pub fn set_channel_fader_curve_long_fader(
        &mut self,
        channel_fader_curve_long_fader: ChannelFaderCurveLongFader,
    ) -> anyhow::Result<()> {
        if let SettingContent::DJMMySetting(content) = &mut self.data.content {
            content.channel_fader_curve_long_fader = channel_fader_curve_long_fader;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DJMMySetting::file_name())))
        }
    }

    // -- DevSetting getters and setters --------------------------------------------------------------

    /// Gets the value of the "Type of the overview Waveform" setting.
    ///
    /// # Returns
    /// - `Ok(OverviewWaveformType)` if the `SettingContent` is of type `DevSetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DevSetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DevSetting` file.
    pub fn get_overview_waveform_type(&self) -> anyhow::Result<OverviewWaveformType> {
        if let SettingContent::DevSetting(content) = &self.data.content {
            Ok(content.overview_waveform_type)
        } else {
            Err(anyhow!(format!("Not a {} file", DevSetting::file_name())))
        }
    }

    /// Sets the value of the "Type of the overview Waveform" setting.
    ///
    /// # Parameters
    /// - `overview_waveform_type`: The new value of the `OverviewWaveformType` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DevSetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DevSetting` file.
    pub fn set_overview_waveform_type(
        &mut self,
        overview_waveform_type: OverviewWaveformType,
    ) -> anyhow::Result<()> {
        if let SettingContent::DevSetting(content) = &mut self.data.content {
            content.overview_waveform_type = overview_waveform_type;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DevSetting::file_name())))
        }
    }

    /// Gets the value of the "Waveform color" setting.
    ///
    /// # Returns
    /// - `Ok(WaveformColor)` if the `SettingContent` is of type `DevSetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DevSetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DevSetting` file.
    pub fn get_waveform_color(&self) -> anyhow::Result<WaveformColor> {
        if let SettingContent::DevSetting(content) = &self.data.content {
            Ok(content.waveform_color)
        } else {
            Err(anyhow!(format!("Not a {} file", DevSetting::file_name())))
        }
    }

    /// Sets the value of the "Waveform color" setting.
    ///
    /// # Parameters
    /// - `waveform_color`: The new value of the `WaveformColor` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DevSetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DevSetting` file.
    pub fn set_waveform_color(&mut self, waveform_color: WaveformColor) -> anyhow::Result<()> {
        if let SettingContent::DevSetting(content) = &mut self.data.content {
            content.waveform_color = waveform_color;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DevSetting::file_name())))
        }
    }

    /// Gets the value of the "Key display format" setting.
    ///
    /// # Returns
    /// - `Ok(KeyDisplayFormat)` if the `SettingContent` is of type `DevSetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DevSetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DevSetting` file.
    pub fn get_key_display_format(&self) -> anyhow::Result<KeyDisplayFormat> {
        if let SettingContent::DevSetting(content) = &self.data.content {
            Ok(content.key_display_format)
        } else {
            Err(anyhow!(format!("Not a {} file", DevSetting::file_name())))
        }
    }

    /// Sets the value of the "Key display format" setting.
    ///
    /// # Parameters
    /// - `key_display_format`: The new value of the `KeyDisplayFormat` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DevSetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DevSetting` file.
    pub fn set_key_display_format(
        &mut self,
        key_display_format: KeyDisplayFormat,
    ) -> anyhow::Result<()> {
        if let SettingContent::DevSetting(content) = &mut self.data.content {
            content.key_display_format = key_display_format;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DevSetting::file_name())))
        }
    }

    /// Gets the value of the "Waveform Current Position" setting.
    ///
    /// # Returns
    /// - `Ok(WaveformCurrentPosition)` if the `SettingContent` is of type `DevSetting`.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DevSetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DevSetting` file.
    pub fn get_waveform_current_position(&self) -> anyhow::Result<WaveformCurrentPosition> {
        if let SettingContent::DevSetting(content) = &self.data.content {
            Ok(content.waveform_current_position)
        } else {
            Err(anyhow!(format!("Not a {} file", DevSetting::file_name())))
        }
    }

    /// Sets the value of the "Waveform Current Position" setting.
    ///
    /// # Parameters
    /// - `waveform_current_position`: The new value of the `WaveformCurrentPosition` setting to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the value is successfully set.
    /// - `Err(anyhow::Error)` if the `SettingContent` is not of type `DevSetting`.
    ///
    /// # Errors
    /// Returns an error if the `SettingContent` is not a `DevSetting` file.
    pub fn set_waveform_current_position(
        &mut self,
        waveform_current_position: WaveformCurrentPosition,
    ) -> anyhow::Result<()> {
        if let SettingContent::DevSetting(content) = &mut self.data.content {
            content.waveform_current_position = waveform_current_position;
            Ok(())
        } else {
            Err(anyhow!(format!("Not a {} file", DevSetting::file_name())))
        }
    }
}
