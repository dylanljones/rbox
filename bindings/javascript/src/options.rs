// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use napi::Result;
use napi_derive::napi;

use rbox::RekordboxOptions;

#[derive(Clone)]
#[napi(js_name = "RekordboxOptions")]
pub struct JsRekordboxOptions {
    pub(crate) inner: RekordboxOptions,
}

#[napi]
impl JsRekordboxOptions {
    #[napi(constructor)]
    pub fn new(db_path: String, analysis_root: String, settings_root: String) -> Result<Self> {
        let inner = RekordboxOptions::new(db_path, analysis_root, settings_root)?;
        Ok(JsRekordboxOptions { inner })
    }

    #[napi(factory)]
    pub fn load(path: String) -> Result<Self> {
        let inner = RekordboxOptions::load(path)?;
        Ok(JsRekordboxOptions { inner })
    }

    #[napi(factory)]
    pub fn open() -> Result<Self> {
        let inner = RekordboxOptions::open()?;
        Ok(JsRekordboxOptions { inner })
    }

    #[napi(getter)]
    pub fn db_path(&self) -> String {
        self.inner.db_path.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn analysis_root(&self) -> String {
        self.inner.analysis_root.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn settings_root(&self) -> String {
        self.inner.settings_root.to_string_lossy().to_string()
    }
}
