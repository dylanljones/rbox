// Author: Dylan Jones
// Date:   2025-05-24

use napi_derive::napi;
use rbox;

#[napi]
fn is_rekordbox_running() -> bool {
    rbox::util::is_rekordbox_running()
}
