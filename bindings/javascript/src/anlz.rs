// Author: Dylan Jones
// Date:   2025-05-10

use napi_derive::napi;
use rbox::anlz::{Anlz, AnlzPaths};

#[napi(js_name = "Anlz")]
pub struct JsAnlz {
    anlz: Anlz,
}

#[napi]
impl JsAnlz {
    #[napi(constructor)]
    pub fn new(path: String) -> Self {
        JsAnlz {
            anlz: Anlz::load(path).expect("Failed to create Anlz"),
        }
    }

    #[napi]
    pub fn dump_copy(&mut self, path: String) {
        self.anlz.dump_copy(path).expect("Failed to dump Anlz");
    }

    #[napi]
    pub fn dump(&mut self) {
        self.anlz.dump().expect("Failed to dump Anlz");
    }

    #[napi]
    pub fn get_tags(&mut self) -> Vec<String> {
        self.anlz.get_tags().expect("Failed to get Anlz tags")
    }

    #[napi]
    pub fn get_path(&mut self) -> String {
        self.anlz.get_path().expect("Failed to get Anlz path")
    }

    #[napi]
    pub fn set_path(&mut self, path: String) {
        self.anlz.set_path(path).expect("Failed to set Anlz path");
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
