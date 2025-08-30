// Author: Dylan Jones
// Date:   2025-05-10

use napi::Result;
use napi_derive::napi;
use rbox::anlz::anlz::*;

#[napi(object, js_name = "Beat")]
pub struct JsBeat {
    pub beat_number: u16,
    pub tempo: u16,
    pub time: u32,
}

impl Into<JsBeat> for Beat {
    fn into(self) -> JsBeat {
        JsBeat {
            beat_number: self.beat_number,
            tempo: self.tempo,
            time: self.time,
        }
    }
}

#[napi(object, js_name = "ExtBeat")]
pub struct JsExtBeat {
    pub beat_number: u8,
}

impl Into<JsExtBeat> for ExtBeat {
    fn into(self) -> JsExtBeat {
        JsExtBeat {
            beat_number: self.beat_number,
        }
    }
}

#[napi(object, js_name = "BeatGrid")]
pub struct JsBeatGrid {
    pub beats: Vec<JsBeat>,
}

impl Into<JsBeatGrid> for BeatGrid {
    fn into(self) -> JsBeatGrid {
        JsBeatGrid {
            beats: self.beats.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(object, js_name = "ExtendedBeatGrid")]
pub struct JsExtendedBeatGrid {
    pub bpm: Vec<JsBeat>,
    pub beats: Vec<JsExtBeat>,
}

impl Into<JsExtendedBeatGrid> for ExtendedBeatGrid {
    fn into(self) -> JsExtendedBeatGrid {
        JsExtendedBeatGrid {
            bpm: self.bpm.iter().map(|item| item.clone().into()).collect(),
            beats: self.beats.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(js_name = "CueType")]
pub enum JsCueType {
    Point = 1,
    Loop = 2,
}

impl Into<JsCueType> for CueType {
    fn into(self) -> JsCueType {
        match self {
            CueType::Point => JsCueType::Point,
            CueType::Loop => JsCueType::Loop,
        }
    }
}

#[napi(js_name = "CueListType")]
pub enum JsCueListType {
    MemoryCues = 0,
    HotCues = 1,
}

impl Into<JsCueListType> for CueListType {
    fn into(self) -> JsCueListType {
        match self {
            CueListType::MemoryCues => JsCueListType::MemoryCues,
            CueListType::HotCues => JsCueListType::HotCues,
        }
    }
}

#[napi(js_name = "CueStatus")]
pub enum JsCueStatus {
    Disabled = 0,
    Active = 4,
}

impl Into<JsCueStatus> for CueStatus {
    fn into(self) -> JsCueStatus {
        match self {
            CueStatus::Disabled => JsCueStatus::Disabled,
            CueStatus::Active => JsCueStatus::Active,
        }
    }
}

#[napi(object, js_name = "Cue")]
pub struct JsCue {
    pub hot_cue: u32,
    pub status: JsCueStatus,
    pub order_first: u16,
    pub order_last: u16,
    pub cue_type: JsCueType,
    pub time: u32,
    pub loop_time: u32,
}

impl Into<JsCue> for Cue {
    fn into(self) -> JsCue {
        JsCue {
            hot_cue: self.hot_cue,
            status: self.status.into(),
            order_first: self.order_first,
            order_last: self.order_last,
            cue_type: self.cue_type.into(),
            time: self.time,
            loop_time: self.loop_time,
        }
    }
}

#[napi(object, js_name = "ExtendedCue")]
pub struct JsExtendedCue {
    pub hot_cue: u32,
    pub cue_type: JsCueType,
    pub time: u32,
    pub loop_time: u32,
    pub color: u8,
    pub loop_numerator: u16,
    pub loop_denominator: u16,
    pub comment: String,
    pub hot_cue_color_index: u8,
    pub hot_cue_color_red: u8,
    pub hot_cue_color_green: u8,
    pub hot_cue_color_blue: u8,
}

impl Into<JsExtendedCue> for ExtendedCue {
    fn into(self) -> JsExtendedCue {
        JsExtendedCue {
            hot_cue: self.hot_cue,
            cue_type: self.cue_type.into(),
            time: self.time,
            loop_time: self.loop_time,
            color: self.color,
            loop_numerator: self.loop_numerator,
            loop_denominator: self.loop_denominator,
            comment: self.comment.to_string(),
            hot_cue_color_index: self.hot_cue_color_index,
            hot_cue_color_red: self.hot_cue_color_rgb.0,
            hot_cue_color_green: self.hot_cue_color_rgb.1,
            hot_cue_color_blue: self.hot_cue_color_rgb.2,
        }
    }
}

#[napi(object, js_name = "CueList")]
pub struct JsCueList {
    pub list_type: JsCueListType,
    pub cues: Vec<JsCue>,
}

impl Into<JsCueList> for CueList {
    fn into(self) -> JsCueList {
        JsCueList {
            list_type: self.list_type.into(),
            cues: self.cues.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(object, js_name = "ExtendedCueList")]
pub struct JsExtendedCueList {
    pub list_type: JsCueListType,
    pub cues: Vec<JsExtendedCue>,
}

impl Into<JsExtendedCueList> for ExtendedCueList {
    fn into(self) -> JsExtendedCueList {
        JsExtendedCueList {
            list_type: self.list_type.into(),
            cues: self.cues.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(object, js_name = "TinyWaveformColumn")]
pub struct JsTinyWaveformColumn {
    pub height: u8,
}

impl Into<JsTinyWaveformColumn> for TinyWaveformColumn {
    fn into(self) -> JsTinyWaveformColumn {
        JsTinyWaveformColumn {
            height: self.height(),
        }
    }
}

#[napi(object, js_name = "TinyWaveformPreview")]
pub struct JsTinyWaveformPreview {
    pub data: Vec<JsTinyWaveformColumn>,
}

impl Into<JsTinyWaveformPreview> for TinyWaveformPreview {
    fn into(self) -> JsTinyWaveformPreview {
        JsTinyWaveformPreview {
            data: self.data.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(object, js_name = "WaveformColumn")]
pub struct JsWaveformColumn {
    pub height: u8,
    pub whiteness: u8,
}

impl Into<JsWaveformColumn> for WaveformColumn {
    fn into(self) -> JsWaveformColumn {
        JsWaveformColumn {
            height: self.height(),
            whiteness: self.whiteness(),
        }
    }
}

#[napi(object, js_name = "WaveformPreview")]
pub struct JsWaveformPreview {
    pub data: Vec<JsWaveformColumn>,
}

impl Into<JsWaveformPreview> for WaveformPreview {
    fn into(self) -> JsWaveformPreview {
        JsWaveformPreview {
            data: self.data.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(object, js_name = "WaveformDetail")]
pub struct JsWaveformDetail {
    pub data: Vec<JsWaveformColumn>,
}

impl Into<JsWaveformDetail> for WaveformDetail {
    fn into(self) -> JsWaveformDetail {
        JsWaveformDetail {
            data: self.data.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(object, js_name = "WaveformColorPreviewColumn")]
pub struct JsWaveformColorPreviewColumn {
    pub energy_bottom_half_freq: u8,
    pub energy_bottom_third_freq: u8,
    pub energy_mid_third_freq: u8,
    pub energy_top_third_freq: u8,
}

impl Into<JsWaveformColorPreviewColumn> for WaveformColorPreviewColumn {
    fn into(self) -> JsWaveformColorPreviewColumn {
        JsWaveformColorPreviewColumn {
            energy_bottom_half_freq: self.energy_bottom_half_freq,
            energy_bottom_third_freq: self.energy_bottom_third_freq,
            energy_mid_third_freq: self.energy_mid_third_freq,
            energy_top_third_freq: self.energy_top_third_freq,
        }
    }
}

#[napi(object, js_name = "WaveformColorPreview")]
pub struct JsWaveformColorPreview {
    pub data: Vec<JsWaveformColorPreviewColumn>,
}

impl Into<JsWaveformColorPreview> for WaveformColorPreview {
    fn into(self) -> JsWaveformColorPreview {
        JsWaveformColorPreview {
            data: self.data.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(object, js_name = "WaveformColorDetailColumn")]
pub struct JsWaveformColorDetailColumn {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub height: u8,
}

impl Into<JsWaveformColorDetailColumn> for WaveformColorDetailColumn {
    fn into(self) -> JsWaveformColorDetailColumn {
        JsWaveformColorDetailColumn {
            red: self.red(),
            green: self.green(),
            blue: self.blue(),
            height: self.height(),
        }
    }
}

#[napi(object, js_name = "WaveformColorDetail")]
pub struct JsWaveformColorDetail {
    pub data: Vec<JsWaveformColorDetailColumn>,
}

impl Into<JsWaveformColorDetail> for WaveformColorDetail {
    fn into(self) -> JsWaveformColorDetail {
        JsWaveformColorDetail {
            data: self.data.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(object, js_name = "Waveform3BandColumn")]
pub struct JsWaveform3BandColumn {
    pub mid: u8,
    pub high: u8,
    pub low: u8,
}

impl Into<JsWaveform3BandColumn> for Waveform3BandColumn {
    fn into(self) -> JsWaveform3BandColumn {
        JsWaveform3BandColumn {
            mid: self.mid(),
            high: self.high(),
            low: self.low(),
        }
    }
}

#[napi(object, js_name = "Waveform3BandPreview")]
pub struct JsWaveform3BandPreview {
    pub data: Vec<JsWaveform3BandColumn>,
}

impl Into<JsWaveform3BandPreview> for Waveform3BandPreview {
    fn into(self) -> JsWaveform3BandPreview {
        JsWaveform3BandPreview {
            data: self.data.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(object, js_name = "Waveform3BandDetail")]
pub struct JsWaveform3BandDetail {
    pub data: Vec<JsWaveform3BandColumn>,
}

impl Into<JsWaveform3BandDetail> for Waveform3BandDetail {
    fn into(self) -> JsWaveform3BandDetail {
        JsWaveform3BandDetail {
            data: self.data.iter().map(|item| item.clone().into()).collect(),
        }
    }
}

#[napi(js_name = "Mood")]
pub enum JsMood {
    High = 1,
    Mid,
    Low,
}

impl Into<JsMood> for Mood {
    fn into(self) -> JsMood {
        match self {
            Mood::High => JsMood::High,
            Mood::Mid => JsMood::Mid,
            Mood::Low => JsMood::Low,
        }
    }
}

#[napi(js_name = "Bank")]
pub enum JsBank {
    Default = 0,
    Cool,
    Natural,
    Hot,
    Subtle,
    Warm,
    Vivid,
    Club1,
    Club2,
}

impl Into<JsBank> for Bank {
    fn into(self) -> JsBank {
        match self {
            Bank::Default => JsBank::Default,
            Bank::Cool => JsBank::Cool,
            Bank::Natural => JsBank::Natural,
            Bank::Hot => JsBank::Hot,
            Bank::Subtle => JsBank::Subtle,
            Bank::Warm => JsBank::Warm,
            Bank::Vivid => JsBank::Vivid,
            Bank::Club1 => JsBank::Club1,
            Bank::Club2 => JsBank::Club2,
        }
    }
}

#[napi(object, js_name = "Phrase")]
pub struct JsPhrase {
    pub index: u16,
    pub beat: u16,
    pub kind: u16,
    pub k1: u8,
    pub k2: u8,
    pub b: u8,
    pub beat2: u16,
    pub beat3: u16,
    pub beat4: u16,
    pub k3: u8,
    pub fill: u8,
    pub beat_fill: u16,
}

impl Into<JsPhrase> for Phrase {
    fn into(self) -> JsPhrase {
        JsPhrase {
            index: self.index,
            beat: self.beat,
            kind: self.kind,
            k1: self.k1,
            k2: self.k2,
            b: self.b,
            beat2: self.beat2,
            beat3: self.beat3,
            beat4: self.beat4,
            k3: self.k3,
            fill: self.fill,
            beat_fill: self.beat_fill,
        }
    }
}

#[napi(object, js_name = "SongStructureData")]
pub struct JsSongStructureData {
    pub mood: JsMood,
    pub end_beat: u16,
    pub bank: JsBank,
    pub phrases: Vec<JsPhrase>,
}

impl Into<JsSongStructureData> for SongStructureData {
    fn into(self) -> JsSongStructureData {
        JsSongStructureData {
            mood: self.mood.into(),
            end_beat: self.end_beat,
            bank: self.bank.into(),
            phrases: self
                .phrases
                .iter()
                .map(|item| item.clone().into())
                .collect(),
        }
    }
}

#[napi(js_name = "Anlz")]
pub struct JsAnlz {
    anlz: Anlz,
}

#[napi]
impl JsAnlz {
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        let anlz = Anlz::load(path)?;
        Ok(JsAnlz { anlz })
    }

    #[napi]
    pub fn dump_copy(&mut self, path: String) -> Result<()> {
        self.anlz.dump_copy(path)?;
        Ok(())
    }

    #[napi]
    pub fn dump(&mut self) -> Result<()> {
        self.anlz.dump()?;
        Ok(())
    }

    #[napi]
    pub fn get_tags(&mut self) -> Result<Vec<String>> {
        let tags = self.anlz.get_tags()?;
        Ok(tags)
    }

    #[napi]
    pub fn get_beat_grid(&mut self) -> Option<JsBeatGrid> {
        let data = self.anlz.get_beat_grid();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_extended_beat_grid(&mut self) -> Option<JsExtendedBeatGrid> {
        let data = self.anlz.get_extended_beat_grid();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_hot_cues(&mut self) -> Option<JsCueList> {
        let data = self.anlz.get_hot_cues();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_memory_cues(&mut self) -> Option<JsCueList> {
        let data = self.anlz.get_memory_cues();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_extended_hot_cues(&mut self) -> Option<JsExtendedCueList> {
        let data = self.anlz.get_extended_hot_cues();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_extended_memory_cues(&mut self) -> Option<JsExtendedCueList> {
        let data = self.anlz.get_extended_memory_cues();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_path(&mut self) -> String {
        self.anlz.get_path().expect("Failed to get Anlz path")
    }

    #[napi]
    pub fn set_path(&mut self, path: String) -> Result<()> {
        self.anlz.set_path(path)?;
        Ok(())
    }

    #[napi]
    pub fn get_vbr_data(&mut self) -> Option<Vec<u8>> {
        self.anlz.get_vbr_data()
    }

    #[napi]
    pub fn get_tiny_waveform_preview(&mut self) -> Option<JsTinyWaveformPreview> {
        let data = self.anlz.get_tiny_waveform_preview();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_waveform_preview(&mut self) -> Option<JsWaveformPreview> {
        let data = self.anlz.get_waveform_preview();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_waveform_detail(&mut self) -> Option<JsWaveformDetail> {
        let data = self.anlz.get_waveform_detail();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_waveform_color_preview(&mut self) -> Option<JsWaveformColorPreview> {
        let data = self.anlz.get_waveform_color_preview();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_waveform_color_detail(&mut self) -> Option<JsWaveformColorDetail> {
        let data = self.anlz.get_waveform_color_detail();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_waveform_3band_preview(&mut self) -> Option<JsWaveform3BandPreview> {
        let data = self.anlz.get_waveform_3band_preview();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_waveform_3band_detail(&mut self) -> Option<JsWaveform3BandDetail> {
        let data = self.anlz.get_waveform_3band_detail();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }

    #[napi]
    pub fn get_song_structure(&mut self) -> Option<JsSongStructureData> {
        let data = self.anlz.get_song_structure();
        if let Some(data) = data {
            return Some(data.clone().into());
        }
        None
    }
}

#[napi(object, js_name = "AnlzPaths")]
pub struct JsAnlzPaths {
    pub dat: String,
    pub ext: Option<String>,
    pub ex2: Option<String>,
}

impl From<AnlzPaths> for JsAnlzPaths {
    fn from(model: AnlzPaths) -> Self {
        JsAnlzPaths {
            dat: model.dat.as_os_str().to_str().unwrap().to_string(),
            ext: model.ext.as_ref().map(|s| s.to_str().unwrap().to_string()),
            ex2: model.ex2.as_ref().map(|s| s.to_str().unwrap().to_string()),
        }
    }
}
