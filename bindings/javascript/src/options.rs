// Author: Dylan Jones
// Date:   2025-05-30

use napi_derive::napi;
use rbox::RekordboxOptions;

#[napi(js_name = "RekordboxOptions")]
pub struct JsRekordboxOptions {
    opts: RekordboxOptions,
}

#[napi]
impl JsRekordboxOptions {
    #[napi(constructor)]
    pub fn new(db_path: String, analysis_root: String, settings_root: String) -> Self {
        let opts = RekordboxOptions::new(db_path, analysis_root, settings_root)
            .expect("Failed to create RekordboxOptions");
        JsRekordboxOptions { opts }
    }

    #[napi(factory)]
    pub fn load(path: String) -> Self {
        let opts = RekordboxOptions::load(path).expect("Failed to load RekordboxOptions from file");
        JsRekordboxOptions { opts }
    }

    #[napi(factory)]
    pub fn open() -> Self {
        let opts = RekordboxOptions::open().expect("Failed to load default RekordboxOptions");
        JsRekordboxOptions { opts }
    }

    #[napi(getter)]
    pub fn db_path(&self) -> String {
        self.opts.db_path.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn analysis_root(&self) -> String {
        self.opts.analysis_root.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn settings_root(&self) -> String {
        self.opts.settings_root.to_string_lossy().to_string()
    }
}
