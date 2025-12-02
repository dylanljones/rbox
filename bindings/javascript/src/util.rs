// Author: Dylan Jones
// Date:   2025-05-24

use napi::Result;
use napi_derive::napi;
use rbox;

#[napi]
fn rekordbox_agent_app_dir() -> Result<String> {
    let path = rbox::util::rekordbox_agent_app_dir()?;
    Ok(path.to_str().unwrap().to_string())
}

#[napi]
fn options_json_path() -> Result<String> {
    let path = rbox::util::options_json_path()?;
    Ok(path.to_str().unwrap().to_string())
}

#[napi]
fn rekordbox_app_dir() -> Result<String> {
    let path = rbox::util::rekordbox_app_dir()?;
    Ok(path.to_str().unwrap().to_string())
}

#[napi]
fn is_rekordbox_running() -> bool {
    rbox::util::is_rekordbox_running()
}
