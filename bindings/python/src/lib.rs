// Author: Dylan Jones
// Date:   2025-05-01

mod anlz;
mod errors;
mod iter;
mod masterdb;
mod settings;
mod traits;
mod xml;

use pyo3::prelude::*;
use rbox::masterdb::models;

#[pyfunction]
fn get_rekordbox_app_dir_path() -> PyResult<String> {
    let path = rbox::util::rekordbox_app_dir()
        .map_err(|e| PyErr::new::<errors::Error, _>(format!("{}", e)))?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn get_rekordbox_share_path() -> PyResult<String> {
    let path = rbox::util::get_rekordbox_share_path()
        .map_err(|e| PyErr::new::<errors::Error, _>(format!("{}", e)))?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn get_master_db_path() -> PyResult<String> {
    let path = rbox::util::get_master_db_path()
        .map_err(|e| PyErr::new::<errors::Error, _>(format!("{}", e)))?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn is_rekordbox_running() -> PyResult<bool> {
    Ok(rbox::util::is_rekordbox_running())
}

#[pymodule]
fn _rbox(m: &Bound<PyModule>) -> PyResult<()> {
    let _ = m.add_function(wrap_pyfunction!(get_rekordbox_app_dir_path, m)?);
    let _ = m.add_function(wrap_pyfunction!(get_rekordbox_share_path, m)?);
    let _ = m.add_function(wrap_pyfunction!(get_master_db_path, m)?);
    let _ = m.add_function(wrap_pyfunction!(is_rekordbox_running, m)?);

    m.add_class::<masterdb::PyMasterDb>()?;
    m.add_class::<models::AgentRegistry>()?;
    m.add_class::<models::CloudAgentRegistry>()?;
    m.add_class::<models::ContentActiveCensor>()?;
    m.add_class::<models::ContentCue>()?;
    m.add_class::<models::ContentFile>()?;
    m.add_class::<models::DjmdActiveCensor>()?;
    m.add_class::<models::DjmdAlbum>()?;
    m.add_class::<models::DjmdArtist>()?;
    m.add_class::<models::DjmdCategory>()?;
    m.add_class::<models::DjmdColor>()?;
    m.add_class::<models::DjmdContent>()?;
    m.add_class::<models::DjmdCue>()?;
    m.add_class::<models::DjmdDevice>()?;
    m.add_class::<models::DjmdGenre>()?;
    m.add_class::<models::DjmdHistory>()?;
    m.add_class::<models::DjmdSongHistory>()?;
    m.add_class::<models::DjmdHotCueBanklist>()?;
    m.add_class::<models::DjmdSongHotCueBanklist>()?;
    m.add_class::<models::HotCueBanklistCue>()?;
    m.add_class::<models::DjmdKey>()?;
    m.add_class::<models::DjmdLabel>()?;
    m.add_class::<models::DjmdMenuItems>()?;
    m.add_class::<models::DjmdMixerParam>()?;
    m.add_class::<models::DjmdMyTag>()?;
    m.add_class::<models::DjmdSongMyTag>()?;
    m.add_class::<models::DjmdPlaylist>()?;
    m.add_class::<masterdb::PyDjmdPlaylistTreeItem>()?;
    m.add_class::<models::DjmdSongPlaylist>()?;
    m.add_class::<models::DjmdProperty>()?;
    m.add_class::<models::DjmdCloudProperty>()?;
    m.add_class::<models::DjmdRecommendLike>()?;
    m.add_class::<models::DjmdRelatedTracks>()?;
    m.add_class::<models::DjmdSongRelatedTracks>()?;
    m.add_class::<models::DjmdSampler>()?;
    m.add_class::<models::DjmdSongSampler>()?;
    m.add_class::<models::DjmdSongTagList>()?;
    m.add_class::<models::DjmdSort>()?;
    m.add_class::<models::ImageFile>()?;
    m.add_class::<models::SettingFile>()?;
    m.add_class::<models::UuidIDMap>()?;

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

    m.add_class::<xml::PyTempo>()?;
    m.add_class::<xml::PyPositionMark>()?;
    m.add_class::<xml::PyTrack>()?;
    m.add_class::<xml::PyPlaylistTrack>()?;
    m.add_class::<xml::PyPlaylistNode>()?;
    m.add_class::<xml::PyRekordboxXml>()?;

    m.add_class::<settings::PySetting>()?;

    m.add("Error", m.py().get_type::<errors::Error>())?;
    m.add("DatabaseError", m.py().get_type::<errors::DatabaseError>())?;
    m.add("AnlzError", m.py().get_type::<errors::AnlzError>())?;
    m.add("XmlError", m.py().get_type::<errors::XmlError>())?;
    m.add("SettingError", m.py().get_type::<errors::SettingError>())?;
    Ok(())
}
