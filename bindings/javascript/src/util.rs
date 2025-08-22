// Author: Dylan Jones
// Date:   2025-05-24

use napi_derive::napi;
use rbox;

#[napi]
fn get_rekordbox_app_dir_path() -> String {
    let path = rbox::util::get_rekordbox_app_dir_path()
        .expect("Failed to get Rekordbox app directory path");
    path.to_str().unwrap().to_string()
}

#[napi]
fn get_rekordbox_share_path() -> String {
    let path = rbox::util::get_rekordbox_share_path()
        .expect("Failed to get Rekordbox share directory path");
    path.to_str().unwrap().to_string()
}

#[napi]
fn get_master_db_path() -> String {
    let path = rbox::util::get_master_db_path().expect("Failed to get Rekordbox master.db path");
    path.to_str().unwrap().to_string()
}

#[napi]
fn is_rekordbox_running() -> bool {
    rbox::util::is_rekordbox_running()
}
