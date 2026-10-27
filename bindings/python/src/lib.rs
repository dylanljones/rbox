// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Python extension registration and shared utility bindings.

mod anlz;
mod errors;
mod locate;
mod masterdb;
mod one_library;
mod options;
mod settings;
mod share;
mod traits;
mod xml;

use pyo3::prelude::*;

#[pyfunction]
fn rekordbox_agent_app_dir() -> PyResult<String> {
    let path = rbox::core::rekordbox_agent_app_dir()
        .map_err(|e| PyErr::new::<errors::Error, _>(format!("{}", e)))?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn options_json_path() -> PyResult<String> {
    let path = rbox::core::options_json_path()
        .map_err(|e| PyErr::new::<errors::Error, _>(format!("{}", e)))?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn rekordbox_app_dir() -> PyResult<String> {
    let path = rbox::core::rekordbox_app_dir()
        .map_err(|e| PyErr::new::<errors::Error, _>(format!("{}", e)))?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn is_rekordbox_running() -> PyResult<bool> {
    Ok(rbox::core::is_rekordbox_running())
}

// Preserve the module's GIL requirement; free-threaded support needs a separate audit.
#[pymodule(gil_used = true)]
fn _rbox(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rekordbox_agent_app_dir, m)?)?;
    m.add_function(wrap_pyfunction!(options_json_path, m)?)?;
    m.add_function(wrap_pyfunction!(rekordbox_app_dir, m)?)?;
    m.add_function(wrap_pyfunction!(is_rekordbox_running, m)?)?;
    m.add_function(wrap_pyfunction!(share::anlz_dir_usb, m)?)?;
    m.add_function(wrap_pyfunction!(share::anlz_dir_local, m)?)?;
    m.add_function(wrap_pyfunction!(share::artwork_dir_local, m)?)?;
    m.add_function(wrap_pyfunction!(share::image_path_local, m)?)?;
    m.add_function(wrap_pyfunction!(share::artwork_dir_usb, m)?)?;

    m.add_class::<rbox::core::locate::LocationSource>()?;
    m.add_class::<locate::PyLibraryLocation>()?;
    m.add_class::<locate::PyProgramLocation>()?;
    m.add_class::<locate::PyLookupPaths>()?;
    m.add_function(wrap_pyfunction!(locate::locate_library, m)?)?;
    m.add_function(wrap_pyfunction!(locate::find_programs, m)?)?;
    m.add_function(wrap_pyfunction!(locate::find_programs_in, m)?)?;
    m.add_function(wrap_pyfunction!(locate::app_dir, m)?)?;
    m.add_function(wrap_pyfunction!(locate::default_master_db_path, m)?)?;
    m.add_function(wrap_pyfunction!(locate::master_db_path, m)?)?;

    // Shared track models
    m.add_class::<rbox::core::models::TrackMetadata>()?;
    m.add_class::<rbox::core::models::Track>()?;
    m.add_class::<rbox::core::TrackSource>()?;

    // Rekordbox Options
    m.add_class::<options::PyRekordboxOptions>()?;

    // Database enums
    m.add_class::<rbox::core::FileType>()?;
    m.add_class::<rbox::core::Analyzed>()?;
    m.add_class::<rbox::core::HistoryType>()?;
    m.add_class::<rbox::core::HotCueBanklistType>()?;
    m.add_class::<rbox::core::MyTagType>()?;
    m.add_class::<rbox::core::PlaylistType>()?;
    m.add_class::<rbox::core::RelatedTracksType>()?;
    m.add_class::<rbox::core::SamplerType>()?;

    // Master-db
    m.add_class::<masterdb::PyMasterDb>()?;
    m.add_class::<rbox::masterdb::models::AgentRegistry>()?;
    m.add_class::<rbox::masterdb::models::CloudAgentRegistry>()?;
    m.add_class::<rbox::masterdb::models::ContentActiveCensor>()?;
    m.add_class::<rbox::masterdb::models::ContentCue>()?;
    m.add_class::<rbox::masterdb::models::ContentFile>()?;
    m.add_class::<rbox::masterdb::models::DjmdActiveCensor>()?;
    m.add_class::<rbox::masterdb::models::DjmdAlbum>()?;
    m.add_class::<rbox::masterdb::models::DjmdAlbumView>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdAlbum>()?;
    m.add_class::<rbox::masterdb::models::DjmdArtist>()?;
    m.add_class::<rbox::masterdb::models::NewDjmdArtist>()?;
    m.add_class::<rbox::masterdb::models::DjmdCategory>()?;
    m.add_class::<rbox::masterdb::models::DjmdColor>()?;
    m.add_class::<rbox::masterdb::models::DjmdContent>()?;
    m.add_class::<rbox::masterdb::models::DjmdContentView>()?;
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

    // Device Library
    m.add_class::<rbox::devlib::models::DevAlbum>()?;
    m.add_class::<rbox::devlib::models::DevAlbumView>()?;
    m.add_class::<rbox::devlib::models::NewDevAlbum>()?;
    m.add_class::<rbox::devlib::models::DevArtist>()?;
    m.add_class::<rbox::devlib::models::NewDevArtist>()?;
    m.add_class::<rbox::devlib::models::DevArtwork>()?;
    m.add_class::<rbox::devlib::models::NewDevArtwork>()?;
    m.add_class::<rbox::devlib::models::DevBrowseColumn>()?;
    m.add_class::<rbox::devlib::models::DevBrowseCategory>()?;
    m.add_class::<rbox::devlib::models::DevBrowseSort>()?;
    m.add_class::<rbox::devlib::models::DevColor>()?;
    m.add_class::<rbox::devlib::models::DevContent>()?;
    m.add_class::<rbox::devlib::models::DevContentView>()?;
    m.add_class::<rbox::devlib::models::NewDevContent>()?;
    m.add_class::<rbox::devlib::models::DevGenre>()?;
    m.add_class::<rbox::devlib::models::NewDevGenre>()?;
    m.add_class::<rbox::devlib::models::DevHistory>()?;
    m.add_class::<rbox::devlib::models::NewDevHistory>()?;
    m.add_class::<rbox::devlib::models::DevHistoryContent>()?;
    m.add_class::<rbox::devlib::models::NewDevHistoryContent>()?;
    m.add_class::<rbox::devlib::models::DevKey>()?;
    m.add_class::<rbox::devlib::models::NewDevKey>()?;
    m.add_class::<rbox::devlib::models::DevLabel>()?;
    m.add_class::<rbox::devlib::models::NewDevLabel>()?;
    m.add_class::<rbox::devlib::models::DevMyTag>()?;
    m.add_class::<rbox::devlib::models::NewDevMyTag>()?;
    m.add_class::<rbox::devlib::models::DevMyTagContent>()?;
    m.add_class::<rbox::devlib::models::NewDevMyTagContent>()?;
    m.add_class::<rbox::devlib::models::playlist::DevPlaylistTreeNode>()?;
    m.add_class::<rbox::devlib::models::DevPlaylist>()?;
    m.add_class::<rbox::devlib::models::NewDevPlaylist>()?;
    m.add_class::<rbox::devlib::models::DevPlaylistContent>()?;
    m.add_class::<rbox::devlib::models::NewDevPlaylistContent>()?;
    m.add_class::<rbox::devlib::models::DevExportProperty>()?;
    m.add_class::<rbox::devlib::models::DevDatabaseProperty>()?;
    m.add_class::<rbox::devlib::models::DevRecommendedLike>()?;
    m.add_class::<rbox::devlib::models::NewDevRecommendedLike>()?;

    // Device Library Plus
    m.add_class::<one_library::PyOneLibrary>()?;
    m.add_class::<rbox::onelib::models::Album>()?;
    m.add_class::<rbox::onelib::models::AlbumView>()?;
    m.add_class::<rbox::onelib::models::NewAlbum>()?;
    m.add_class::<rbox::onelib::models::Artist>()?;
    m.add_class::<rbox::onelib::models::Composer>()?;
    m.add_class::<rbox::onelib::models::Lyricist>()?;
    m.add_class::<rbox::onelib::models::NewArtist>()?;
    m.add_class::<rbox::onelib::models::OriginalArtist>()?;
    m.add_class::<rbox::onelib::models::Remixer>()?;
    m.add_class::<rbox::onelib::models::Category>()?;
    m.add_class::<rbox::onelib::models::CategoryView>()?;
    m.add_class::<rbox::onelib::models::NewCategory>()?;
    m.add_class::<rbox::onelib::models::Color>()?;
    m.add_class::<rbox::onelib::models::NewColor>()?;
    m.add_class::<rbox::onelib::models::Content>()?;
    m.add_class::<rbox::onelib::models::ContentView>()?;
    m.add_class::<rbox::onelib::models::NewContent>()?;
    m.add_class::<rbox::onelib::models::Cue>()?;
    m.add_class::<rbox::onelib::models::Genre>()?;
    m.add_class::<rbox::onelib::models::NewGenre>()?;
    m.add_class::<rbox::onelib::models::History>()?;
    m.add_class::<rbox::onelib::models::HistoryContent>()?;
    m.add_class::<rbox::onelib::models::NewHistory>()?;
    m.add_class::<rbox::onelib::models::HotCueBankList>()?;
    m.add_class::<rbox::onelib::models::HotCueBankListCue>()?;
    m.add_class::<rbox::onelib::models::Image>()?;
    m.add_class::<rbox::onelib::models::NewImage>()?;
    m.add_class::<rbox::onelib::models::Key>()?;
    m.add_class::<rbox::onelib::models::NewKey>()?;
    m.add_class::<rbox::onelib::models::Label>()?;
    m.add_class::<rbox::onelib::models::NewLabel>()?;
    m.add_class::<rbox::onelib::models::MenuItem>()?;
    m.add_class::<rbox::onelib::models::NewMenuItem>()?;
    m.add_class::<rbox::onelib::models::MyTag>()?;
    m.add_class::<rbox::onelib::models::MyTagContent>()?;
    m.add_class::<rbox::onelib::models::NewMyTag>()?;
    m.add_class::<rbox::onelib::models::NewPlaylist>()?;
    m.add_class::<rbox::onelib::models::Playlist>()?;
    m.add_class::<rbox::onelib::models::PlaylistContent>()?;
    m.add_class::<rbox::onelib::models::Property>()?;
    m.add_class::<rbox::onelib::models::RecommendedLike>()?;
    m.add_class::<rbox::onelib::models::NewSort>()?;
    m.add_class::<rbox::onelib::models::Sort>()?;

    // Anlz
    m.add_class::<anlz::PySongStructure>()?;
    m.add_class::<anlz::PyUnknownSection>()?;
    m.add_class::<anlz::PyVBR>()?;
    m.add_class::<anlz::PySeekPoint>()?;
    m.add_class::<anlz::PySeekIndex>()?;
    m.add_class::<anlz::PyWaveform3BandCalibration>()?;
    m.add_class::<anlz::PyVocalDetection>()?;
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
    m.add_class::<xml::PyXmlTempo>()?;
    m.add("Tempo", m.getattr("XmlTempo")?)?;
    m.add_class::<xml::PyXmlPositionMark>()?;
    m.add("PositionMark", m.getattr("XmlPositionMark")?)?;
    m.add_class::<xml::PyXmlTrack>()?;

    m.add_class::<xml::PyXmlPlaylistTrack>()?;
    m.add("PlaylistTrack", m.getattr("XmlPlaylistTrack")?)?;
    m.add_class::<xml::PyXmlPlaylistNode>()?;
    m.add("PlaylistNode", m.getattr("XmlPlaylistNode")?)?;
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
