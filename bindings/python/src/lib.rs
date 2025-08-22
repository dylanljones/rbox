// Author: Dylan Jones
// Date:   2025-05-01

mod anlz;
mod errors;
mod iter;
mod masterdb;
mod py_models;
mod settings;
mod traits;
mod xml;

use pyo3::prelude::*;

#[pyfunction]
fn get_rekordbox_app_dir_path() -> PyResult<String> {
    let path = rbox::util::get_rekordbox_app_dir_path()?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn get_rekordbox_share_path() -> PyResult<String> {
    let path = rbox::util::get_rekordbox_share_path()?;
    Ok(path.to_str().unwrap().to_string())
}

#[pyfunction]
fn get_master_db_path() -> PyResult<String> {
    let path = rbox::util::get_master_db_path()?;
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
    m.add_class::<py_models::PyAgentRegistry>()?;
    m.add_class::<py_models::PyCloudAgentRegistry>()?;
    m.add_class::<py_models::PyContentActiveCensor>()?;
    m.add_class::<py_models::PyContentCue>()?;
    m.add_class::<py_models::PyContentFile>()?;
    m.add_class::<py_models::PyDjmdActiveCensor>()?;
    m.add_class::<py_models::PyDjmdAlbum>()?;
    m.add_class::<py_models::PyDjmdArtist>()?;
    m.add_class::<py_models::PyDjmdCategory>()?;
    m.add_class::<py_models::PyDjmdColor>()?;
    m.add_class::<py_models::PyDjmdContent>()?;
    m.add_class::<py_models::PyDjmdCue>()?;
    m.add_class::<py_models::PyDjmdDevice>()?;
    m.add_class::<py_models::PyDjmdGenre>()?;
    m.add_class::<py_models::PyDjmdHistory>()?;
    m.add_class::<py_models::PyDjmdSongHistory>()?;
    m.add_class::<py_models::PyDjmdHotCueBanklist>()?;
    m.add_class::<py_models::PyDjmdSongHotCueBanklist>()?;
    m.add_class::<py_models::PyHotCueBanklistCue>()?;
    m.add_class::<py_models::PyDjmdKey>()?;
    m.add_class::<py_models::PyDjmdLabel>()?;
    m.add_class::<py_models::PyDjmdMenuItems>()?;
    m.add_class::<py_models::PyDjmdMixerParam>()?;
    m.add_class::<py_models::PyDjmdMyTag>()?;
    m.add_class::<py_models::PyDjmdSongMyTag>()?;
    m.add_class::<py_models::PyDjmdPlaylist>()?;
    m.add_class::<py_models::PyDjmdPlaylistTreeItem>()?;
    m.add_class::<py_models::PyDjmdSongPlaylist>()?;
    m.add_class::<py_models::PyDjmdProperty>()?;
    m.add_class::<py_models::PyDjmdCloudProperty>()?;
    m.add_class::<py_models::PyDjmdRecommendLike>()?;
    m.add_class::<py_models::PyDjmdRelatedTracks>()?;
    m.add_class::<py_models::PyDjmdSongRelatedTracks>()?;
    m.add_class::<py_models::PyDjmdSampler>()?;
    m.add_class::<py_models::PyDjmdSongSampler>()?;
    m.add_class::<py_models::PyDjmdSongTagList>()?;
    m.add_class::<py_models::PyDjmdSort>()?;
    m.add_class::<py_models::PyImageFile>()?;
    m.add_class::<py_models::PySettingFile>()?;
    m.add_class::<py_models::PyUuidIDMap>()?;
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
