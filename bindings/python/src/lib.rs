// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

mod anlz;
mod errors;
mod masterdb;
mod one_library;
mod options;
mod settings;
mod traits;
mod xml;

use pyo3::prelude::*;

#[pyfunction]
fn rekordbox_agent_app_dir() -> PyResult<String> {
    let path = rbox::util::rekordbox_agent_app_dir()
        .map_err(|e| PyErr::new::<errors::Error, _>(format!("{}", e)))?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn options_json_path() -> PyResult<String> {
    let path = rbox::util::options_json_path()
        .map_err(|e| PyErr::new::<errors::Error, _>(format!("{}", e)))?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn rekordbox_app_dir() -> PyResult<String> {
    let path = rbox::util::rekordbox_app_dir()
        .map_err(|e| PyErr::new::<errors::Error, _>(format!("{}", e)))?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn is_rekordbox_running() -> PyResult<bool> {
    Ok(rbox::util::is_rekordbox_running())
}

#[pymodule]
fn _rbox(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rekordbox_agent_app_dir, m)?)?;
    m.add_function(wrap_pyfunction!(options_json_path, m)?)?;
    m.add_function(wrap_pyfunction!(rekordbox_app_dir, m)?)?;
    m.add_function(wrap_pyfunction!(is_rekordbox_running, m)?)?;

    // Rekordbox Options
    m.add_class::<options::PyRekordboxOptions>()?;

    // Database enums
    m.add_class::<rbox::enums::FileType>()?;
    m.add_class::<rbox::enums::Analyzed>()?;
    m.add_class::<rbox::enums::HistoryType>()?;
    m.add_class::<rbox::enums::HotCueBanklistType>()?;
    m.add_class::<rbox::enums::MyTagType>()?;
    m.add_class::<rbox::enums::PlaylistType>()?;
    m.add_class::<rbox::enums::RelatedTracksType>()?;
    m.add_class::<rbox::enums::SamplerType>()?;

    // Master-db
    m.add_class::<masterdb::PyMasterDb>()?;
    m.add_class::<rbox::masterdb::models::AgentRegistry>()?;
    m.add_class::<rbox::masterdb::models::CloudAgentRegistry>()?;
    m.add_class::<rbox::masterdb::models::ContentActiveCensor>()?;
    m.add_class::<rbox::masterdb::models::ContentCue>()?;
    m.add_class::<rbox::masterdb::models::ContentFile>()?;
    m.add_class::<rbox::masterdb::models::DjmdActiveCensor>()?;
    m.add_class::<rbox::masterdb::models::DjmdAlbum>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdAlbum>()?;
    m.add_class::<rbox::masterdb::models::DjmdArtist>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdArtist>()?;
    m.add_class::<rbox::masterdb::models::DjmdCategory>()?;
    m.add_class::<rbox::masterdb::models::DjmdColor>()?;
    m.add_class::<rbox::masterdb::models::DjmdContent>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdContent>()?;
    m.add_class::<rbox::masterdb::models::DjmdCue>()?;
    m.add_class::<rbox::masterdb::models::DjmdDevice>()?;
    m.add_class::<rbox::masterdb::models::DjmdGenre>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdGenre>()?;
    m.add_class::<rbox::masterdb::models::DjmdHistory>()?;
    m.add_class::<rbox::masterdb::models::DjmdSongHistory>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdHistory>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdSongHistory>()?;
    m.add_class::<rbox::masterdb::models::DjmdHotCueBanklist>()?;
    m.add_class::<rbox::masterdb::models::DjmdSongHotCueBanklist>()?;
    m.add_class::<rbox::masterdb::models::HotCueBanklistCue>()?;
    m.add_class::<rbox::masterdb::models::DjmdKey>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdKey>()?;
    m.add_class::<rbox::masterdb::models::DjmdLabel>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdLabel>()?;
    m.add_class::<rbox::masterdb::models::DjmdMenuItems>()?;
    m.add_class::<rbox::masterdb::models::DjmdMixerParam>()?;
    m.add_class::<rbox::masterdb::models::DjmdMyTag>()?;
    m.add_class::<rbox::masterdb::models::DjmdSongMyTag>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdMyTag>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdSongMyTag>()?;
    m.add_class::<rbox::masterdb::models::DjmdPlaylist>()?;
    m.add_class::<rbox::masterdb::models::DjmdSongPlaylist>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdPlaylist>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdSongPlaylist>()?;
    m.add_class::<masterdb::PyDjmdPlaylistTreeNode>()?;
    m.add_class::<rbox::masterdb::models::DjmdCloudProperty>()?;
    m.add_class::<rbox::masterdb::models::DjmdProperty>()?;
    m.add_class::<rbox::masterdb::models::DjmdRecommendLike>()?;
    m.add_class::<rbox::masterdb::models::DjmdRelatedTracks>()?;
    m.add_class::<rbox::masterdb::models::DjmdSongRelatedTracks>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdRelatedTracks>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdSongRelatedTracks>()?;
    m.add_class::<rbox::masterdb::models::DjmdSampler>()?;
    m.add_class::<rbox::masterdb::models::DjmdSongSampler>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdSampler>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdSongSampler>()?;
    m.add_class::<rbox::masterdb::models::DjmdSongTagList>()?;
    m.add_class::<rbox::masterdb::models::DjmdSort>()?;
    m.add_class::<rbox::masterdb::models::ImageFile>()?;
    m.add_class::<rbox::masterdb::models::SettingFile>()?;
    m.add_class::<rbox::masterdb::models::UuidIDMap>()?;

    // Device Library Plus
    m.add_class::<one_library::PyOneLibrary>()?;
    m.add_class::<rbox::one_library::models::Album>()?;
    m.add_class::<rbox::one_library::models::NewAlbum>()?;
    m.add_class::<rbox::one_library::models::Artist>()?;
    m.add_class::<rbox::one_library::models::Composer>()?;
    m.add_class::<rbox::one_library::models::Lyricist>()?;
    m.add_class::<rbox::one_library::models::NewArtist>()?;
    m.add_class::<rbox::one_library::models::OriginalArtist>()?;
    m.add_class::<rbox::one_library::models::Remixer>()?;
    m.add_class::<rbox::one_library::models::Category>()?;
    m.add_class::<rbox::one_library::models::NewCategory>()?;
    m.add_class::<rbox::one_library::models::Color>()?;
    m.add_class::<rbox::one_library::models::NewColor>()?;
    m.add_class::<rbox::one_library::models::Content>()?;
    m.add_class::<rbox::one_library::models::NewContent>()?;
    m.add_class::<rbox::one_library::models::Cue>()?;
    m.add_class::<rbox::one_library::models::Genre>()?;
    m.add_class::<rbox::one_library::models::NewGenre>()?;
    m.add_class::<rbox::one_library::models::History>()?;
    m.add_class::<rbox::one_library::models::HistoryContent>()?;
    m.add_class::<rbox::one_library::models::NewHistory>()?;
    m.add_class::<rbox::one_library::models::HotCueBankList>()?;
    m.add_class::<rbox::one_library::models::HotCueBankListCue>()?;
    m.add_class::<rbox::one_library::models::Image>()?;
    m.add_class::<rbox::one_library::models::NewImage>()?;
    m.add_class::<rbox::one_library::models::Key>()?;
    m.add_class::<rbox::one_library::models::NewKey>()?;
    m.add_class::<rbox::one_library::models::Label>()?;
    m.add_class::<rbox::one_library::models::NewLabel>()?;
    m.add_class::<rbox::one_library::models::MenuItem>()?;
    m.add_class::<rbox::one_library::models::NewMenuItem>()?;
    m.add_class::<rbox::one_library::models::MyTag>()?;
    m.add_class::<rbox::one_library::models::MyTagContent>()?;
    m.add_class::<rbox::one_library::models::NewMyTag>()?;
    m.add_class::<rbox::one_library::models::NewPlaylist>()?;
    m.add_class::<rbox::one_library::models::Playlist>()?;
    m.add_class::<rbox::one_library::models::PlaylistContent>()?;
    m.add_class::<rbox::one_library::models::Property>()?;
    m.add_class::<rbox::one_library::models::RecommendedLike>()?;
    m.add_class::<rbox::one_library::models::NewSort>()?;
    m.add_class::<rbox::one_library::models::Sort>()?;

    // Anlz
    m.add_class::<anlz::PyAnlz>()?;
    m.add_class::<anlz::PyBeat>()?;
    m.add_class::<anlz::PyExtBeat>()?;
    m.add_class::<anlz::PyBeatGrid>()?;
    m.add_class::<anlz::PyExtendedBeatGrid>()?;
    m.add_class::<anlz::PyCue>()?;
    m.add_class::<anlz::PyExtendedCue>()?;
    m.add_class::<anlz::PyCueList>()?;
    m.add_class::<anlz::PyExtendedCueList>()?;
    m.add_class::<anlz::PyTinyWaveformColumn>()?;
    m.add_class::<anlz::PyTinyWaveformPreview>()?;
    m.add_class::<anlz::PyWaveformColumn>()?;
    m.add_class::<anlz::PyWaveformPreview>()?;
    m.add_class::<anlz::PyWaveformDetail>()?;
    m.add_class::<anlz::PyWaveformColorPreviewColumn>()?;
    m.add_class::<anlz::PyWaveformColorPreview>()?;
    m.add_class::<anlz::PyWaveformColorDetailColumn>()?;
    m.add_class::<anlz::PyWaveformColorDetail>()?;
    m.add_class::<anlz::PyWaveform3BandColumn>()?;
    m.add_class::<anlz::PyWaveform3BandPreview>()?;
    m.add_class::<anlz::PyWaveform3BandDetail>()?;
    m.add_class::<anlz::PyPhrase>()?;
    m.add_class::<anlz::PySongStructureData>()?;

    // Rekordbox XML
    m.add_class::<xml::PyTempo>()?;
    m.add_class::<xml::PyPositionMark>()?;
    m.add_class::<xml::PyTrack>()?;
    m.add_class::<xml::PyPlaylistTrack>()?;
    m.add_class::<xml::PyPlaylistNode>()?;
    m.add_class::<xml::PyRekordboxXml>()?;

    // My-Settings
    m.add_class::<settings::PySetting>()?;

    // Errors
    m.add("Error", m.py().get_type::<errors::Error>())?;
    m.add("OptionsError", m.py().get_type::<errors::OptionsError>())?;
    m.add("MasterDbError", m.py().get_type::<errors::MasterDbError>())?;
    m.add(
        "OneLibraryError",
        m.py().get_type::<errors::OneLibraryError>(),
    )?;
    m.add("AnlzError", m.py().get_type::<errors::AnlzError>())?;
    m.add("XmlError", m.py().get_type::<errors::XmlError>())?;
    m.add("SettingError", m.py().get_type::<errors::SettingError>())?;

    Ok(())
}
