// Auto-generated code for JavaScript bindings

#![allow(non_snake_case)]

use chrono::{DateTime, Utc};
use napi::Env;
use napi::JsDate;
use napi_derive::napi;
use rbox::masterdb::models::*;

pub fn datetime_to_js_date(env: &Env, date: &DateTime<Utc>) -> JsDate {
    env.create_date(date.timestamp_millis() as f64).unwrap()
}

pub fn js_date_to_datetime(js_date: JsDate) -> DateTime<Utc> {
    match js_date.value_of() {
        Ok(timestamp_ms) => {
            let timestamp_ms = timestamp_ms as i64;
            DateTime::<Utc>::from_timestamp_millis(timestamp_ms).unwrap_or_else(|| Utc::now())
        }
        Err(_) => Utc::now(),
    }
}

pub trait IntoJs<T> {
    fn into_js(self, env: &Env) -> T;
}

#[napi(object, js_name = "AgentRegistry")]
pub struct JsAgentRegistry {
    pub registry_id: String,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub id_1: Option<String>,
    pub id_2: Option<String>,
    pub int_1: Option<i32>,
    pub int_2: Option<i32>,
    pub str_1: Option<String>,
    pub str_2: Option<String>,
    pub date_1: Option<String>,
    pub date_2: Option<String>,
    pub text_1: Option<String>,
    pub text_2: Option<String>,
}

impl IntoJs<JsAgentRegistry> for AgentRegistry {
    fn into_js(self, env: &Env) -> JsAgentRegistry {
        JsAgentRegistry {
            registry_id: self.registry_id,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            id_1: self.id_1,
            id_2: self.id_2,
            int_1: self.int_1,
            int_2: self.int_2,
            str_1: self.str_1,
            str_2: self.str_2,
            date_1: self.date_1,
            date_2: self.date_2,
            text_1: self.text_1,
            text_2: self.text_2,
        }
    }
}

impl From<JsAgentRegistry> for AgentRegistry {
    fn from(model: JsAgentRegistry) -> Self {
        AgentRegistry {
            registry_id: model.registry_id,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            id_1: model.id_1,
            id_2: model.id_2,
            int_1: model.int_1,
            int_2: model.int_2,
            str_1: model.str_1,
            str_2: model.str_2,
            date_1: model.date_1,
            date_2: model.date_2,
            text_1: model.text_1,
            text_2: model.text_2,
        }
    }
}

#[napi(object, js_name = "CloudAgentRegistry")]
pub struct JsCloudAgentRegistry {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub int_1: Option<i32>,
    pub int_2: Option<i32>,
    pub str_1: Option<String>,
    pub str_2: Option<String>,
    pub date_1: Option<String>,
    pub date_2: Option<String>,
    pub text_1: Option<String>,
    pub text_2: Option<String>,
}

impl IntoJs<JsCloudAgentRegistry> for CloudAgentRegistry {
    fn into_js(self, env: &Env) -> JsCloudAgentRegistry {
        JsCloudAgentRegistry {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            int_1: self.int_1,
            int_2: self.int_2,
            str_1: self.str_1,
            str_2: self.str_2,
            date_1: self.date_1,
            date_2: self.date_2,
            text_1: self.text_1,
            text_2: self.text_2,
        }
    }
}

impl From<JsCloudAgentRegistry> for CloudAgentRegistry {
    fn from(model: JsCloudAgentRegistry) -> Self {
        CloudAgentRegistry {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            int_1: model.int_1,
            int_2: model.int_2,
            str_1: model.str_1,
            str_2: model.str_2,
            date_1: model.date_1,
            date_2: model.date_2,
            text_1: model.text_1,
            text_2: model.text_2,
        }
    }
}

#[napi(object, js_name = "ContentActiveCensor")]
pub struct JsContentActiveCensor {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ContentID: Option<String>,
    pub ActiveCensors: Option<String>,
    pub rb_activecensor_count: Option<i32>,
}

impl IntoJs<JsContentActiveCensor> for ContentActiveCensor {
    fn into_js(self, env: &Env) -> JsContentActiveCensor {
        JsContentActiveCensor {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ContentID: self.ContentID,
            ActiveCensors: self.ActiveCensors,
            rb_activecensor_count: self.rb_activecensor_count,
        }
    }
}

impl From<JsContentActiveCensor> for ContentActiveCensor {
    fn from(model: JsContentActiveCensor) -> Self {
        ContentActiveCensor {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ContentID: model.ContentID,
            ActiveCensors: model.ActiveCensors,
            rb_activecensor_count: model.rb_activecensor_count,
        }
    }
}

#[napi(object, js_name = "ContentCue")]
pub struct JsContentCue {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ContentID: Option<String>,
    pub Cues: Option<String>,
    pub rb_cue_count: Option<i32>,
}

impl IntoJs<JsContentCue> for ContentCue {
    fn into_js(self, env: &Env) -> JsContentCue {
        JsContentCue {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ContentID: self.ContentID,
            Cues: self.Cues,
            rb_cue_count: self.rb_cue_count,
        }
    }
}

impl From<JsContentCue> for ContentCue {
    fn from(model: JsContentCue) -> Self {
        ContentCue {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ContentID: model.ContentID,
            Cues: model.Cues,
            rb_cue_count: model.rb_cue_count,
        }
    }
}

#[napi(object, js_name = "ContentFile")]
pub struct JsContentFile {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ContentID: Option<String>,
    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_local_file_status: Option<i32>,
    pub rb_in_progress: Option<i32>,
    pub rb_process_type: Option<i32>,
    pub rb_temp_path: Option<String>,
    pub rb_priority: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

impl IntoJs<JsContentFile> for ContentFile {
    fn into_js(self, env: &Env) -> JsContentFile {
        JsContentFile {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ContentID: self.ContentID,
            Path: self.Path,
            Hash: self.Hash,
            Size: self.Size,
            rb_local_path: self.rb_local_path,
            rb_insync_hash: self.rb_insync_hash,
            rb_insync_local_usn: self.rb_insync_local_usn,
            rb_file_hash_dirty: self.rb_file_hash_dirty,
            rb_local_file_status: self.rb_local_file_status,
            rb_in_progress: self.rb_in_progress,
            rb_process_type: self.rb_process_type,
            rb_temp_path: self.rb_temp_path,
            rb_priority: self.rb_priority,
            rb_file_size_dirty: self.rb_file_size_dirty,
        }
    }
}

impl From<JsContentFile> for ContentFile {
    fn from(model: JsContentFile) -> Self {
        ContentFile {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ContentID: model.ContentID,
            Path: model.Path,
            Hash: model.Hash,
            Size: model.Size,
            rb_local_path: model.rb_local_path,
            rb_insync_hash: model.rb_insync_hash,
            rb_insync_local_usn: model.rb_insync_local_usn,
            rb_file_hash_dirty: model.rb_file_hash_dirty,
            rb_local_file_status: model.rb_local_file_status,
            rb_in_progress: model.rb_in_progress,
            rb_process_type: model.rb_process_type,
            rb_temp_path: model.rb_temp_path,
            rb_priority: model.rb_priority,
            rb_file_size_dirty: model.rb_file_size_dirty,
        }
    }
}

#[napi(object, js_name = "DjmdActiveCensor")]
pub struct JsDjmdActiveCensor {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ContentID: Option<String>,
    pub InMsec: Option<i32>,
    pub OutMsec: Option<i32>,
    pub Info: Option<i32>,
    pub ParameterList: Option<String>,
    pub ContentUUID: Option<String>,
}

impl IntoJs<JsDjmdActiveCensor> for DjmdActiveCensor {
    fn into_js(self, env: &Env) -> JsDjmdActiveCensor {
        JsDjmdActiveCensor {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ContentID: self.ContentID,
            InMsec: self.InMsec,
            OutMsec: self.OutMsec,
            Info: self.Info,
            ParameterList: self.ParameterList,
            ContentUUID: self.ContentUUID,
        }
    }
}

impl From<JsDjmdActiveCensor> for DjmdActiveCensor {
    fn from(model: JsDjmdActiveCensor) -> Self {
        DjmdActiveCensor {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ContentID: model.ContentID,
            InMsec: model.InMsec,
            OutMsec: model.OutMsec,
            Info: model.Info,
            ParameterList: model.ParameterList,
            ContentUUID: model.ContentUUID,
        }
    }
}

#[napi(object, js_name = "DjmdAlbum")]
pub struct JsDjmdAlbum {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Name: Option<String>,
    pub AlbumArtistID: Option<String>,
    pub ImagePath: Option<String>,
    pub Compilation: Option<i32>,
    pub SearchStr: Option<String>,
}

impl IntoJs<JsDjmdAlbum> for DjmdAlbum {
    fn into_js(self, env: &Env) -> JsDjmdAlbum {
        JsDjmdAlbum {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Name: self.Name,
            AlbumArtistID: self.AlbumArtistID,
            ImagePath: self.ImagePath,
            Compilation: self.Compilation,
            SearchStr: self.SearchStr,
        }
    }
}

impl From<JsDjmdAlbum> for DjmdAlbum {
    fn from(model: JsDjmdAlbum) -> Self {
        DjmdAlbum {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Name: model.Name,
            AlbumArtistID: model.AlbumArtistID,
            ImagePath: model.ImagePath,
            Compilation: model.Compilation,
            SearchStr: model.SearchStr,
        }
    }
}

#[napi(object, js_name = "DjmdArtist")]
pub struct JsDjmdArtist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Name: Option<String>,
    pub SearchStr: Option<String>,
}

impl IntoJs<JsDjmdArtist> for DjmdArtist {
    fn into_js(self, env: &Env) -> JsDjmdArtist {
        JsDjmdArtist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Name: self.Name,
            SearchStr: self.SearchStr,
        }
    }
}

impl From<JsDjmdArtist> for DjmdArtist {
    fn from(model: JsDjmdArtist) -> Self {
        DjmdArtist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Name: model.Name,
            SearchStr: model.SearchStr,
        }
    }
}

#[napi(object, js_name = "DjmdCategory")]
pub struct JsDjmdCategory {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub MenuItemID: Option<String>,
    pub Seq: Option<i32>,
    pub Disable: Option<i32>,
    pub InfoOrder: Option<i32>,
}

impl IntoJs<JsDjmdCategory> for DjmdCategory {
    fn into_js(self, env: &Env) -> JsDjmdCategory {
        JsDjmdCategory {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            MenuItemID: self.MenuItemID,
            Seq: self.Seq,
            Disable: self.Disable,
            InfoOrder: self.InfoOrder,
        }
    }
}

impl From<JsDjmdCategory> for DjmdCategory {
    fn from(model: JsDjmdCategory) -> Self {
        DjmdCategory {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            MenuItemID: model.MenuItemID,
            Seq: model.Seq,
            Disable: model.Disable,
            InfoOrder: model.InfoOrder,
        }
    }
}

#[napi(object, js_name = "DjmdColor")]
pub struct JsDjmdColor {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ColorCode: Option<String>,
    pub SortKey: Option<i32>,
    pub Commnt: Option<String>,
}

impl IntoJs<JsDjmdColor> for DjmdColor {
    fn into_js(self, env: &Env) -> JsDjmdColor {
        JsDjmdColor {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ColorCode: self.ColorCode,
            SortKey: self.SortKey,
            Commnt: self.Commnt,
        }
    }
}

impl From<JsDjmdColor> for DjmdColor {
    fn from(model: JsDjmdColor) -> Self {
        DjmdColor {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ColorCode: model.ColorCode,
            SortKey: model.SortKey,
            Commnt: model.Commnt,
        }
    }
}

#[napi(object, js_name = "DjmdContent")]
pub struct JsDjmdContent {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub FolderPath: Option<String>,
    pub FileNameL: Option<String>,
    pub FileNameS: Option<String>,
    pub Title: Option<String>,
    pub ArtistID: Option<String>,
    pub AlbumID: Option<String>,
    pub GenreID: Option<String>,
    pub BPM: Option<i32>,
    pub Length: Option<i32>,
    pub TrackNo: Option<i32>,
    pub BitRate: Option<i32>,
    pub BitDepth: Option<i32>,
    pub Commnt: Option<String>,
    pub FileType: Option<i32>,
    pub Rating: Option<i32>,
    pub ReleaseYear: Option<i32>,
    pub RemixerID: Option<String>,
    pub LabelID: Option<String>,
    pub OrgArtistID: Option<String>,
    pub KeyID: Option<String>,
    pub StockDate: Option<String>,
    pub ColorID: Option<String>,
    pub DJPlayCount: Option<i32>,
    pub ImagePath: Option<String>,
    pub MasterDBID: Option<String>,
    pub MasterSongID: Option<String>,
    pub AnalysisDataPath: Option<String>,
    pub SearchStr: Option<String>,
    pub FileSize: Option<i32>,
    pub DiscNo: Option<i32>,
    pub ComposerID: Option<String>,
    pub Subtitle: Option<String>,
    pub SampleRate: Option<i32>,
    pub DisableQuantize: Option<i32>,
    pub Analysed: Option<i32>,
    pub ReleaseDate: Option<String>,
    pub DateCreated: Option<String>,
    pub ContentLink: Option<i32>,
    pub Tag: Option<String>,
    pub ModifiedByRBM: Option<String>,
    pub HotCueAutoLoad: Option<String>,
    pub DeliveryControl: Option<String>,
    pub DeliveryComment: Option<String>,
    pub CueUpdated: Option<String>,
    pub AnalysisUpdated: Option<String>,
    pub TrackInfoUpdated: Option<String>,
    pub Lyricist: Option<String>,
    pub ISRC: Option<String>,
    pub SamplerTrackInfo: Option<i32>,
    pub SamplerPlayOffset: Option<i32>,
    pub SamplerGain: Option<f64>,
    pub VideoAssociate: Option<String>,
    pub LyricStatus: Option<i32>,
    pub ServiceID: Option<i32>,
    pub OrgFolderPath: Option<String>,
    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub ExtInfo: Option<String>,
    pub rb_file_id: Option<String>,
    pub DeviceID: Option<String>,
    pub rb_LocalFolderPath: Option<String>,
    pub SrcID: Option<String>,
    pub SrcTitle: Option<String>,
    pub SrcArtistName: Option<String>,
    pub SrcAlbumName: Option<String>,
    pub SrcLength: Option<i32>,
}

impl IntoJs<JsDjmdContent> for DjmdContent {
    fn into_js(self, env: &Env) -> JsDjmdContent {
        JsDjmdContent {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            FolderPath: self.FolderPath,
            FileNameL: self.FileNameL,
            FileNameS: self.FileNameS,
            Title: self.Title,
            ArtistID: self.ArtistID,
            AlbumID: self.AlbumID,
            GenreID: self.GenreID,
            BPM: self.BPM,
            Length: self.Length,
            TrackNo: self.TrackNo,
            BitRate: self.BitRate,
            BitDepth: self.BitDepth,
            Commnt: self.Commnt,
            FileType: self.FileType,
            Rating: self.Rating,
            ReleaseYear: self.ReleaseYear,
            RemixerID: self.RemixerID,
            LabelID: self.LabelID,
            OrgArtistID: self.OrgArtistID,
            KeyID: self.KeyID,
            StockDate: self.StockDate,
            ColorID: self.ColorID,
            DJPlayCount: self.DJPlayCount,
            ImagePath: self.ImagePath,
            MasterDBID: self.MasterDBID,
            MasterSongID: self.MasterSongID,
            AnalysisDataPath: self.AnalysisDataPath,
            SearchStr: self.SearchStr,
            FileSize: self.FileSize,
            DiscNo: self.DiscNo,
            ComposerID: self.ComposerID,
            Subtitle: self.Subtitle,
            SampleRate: self.SampleRate,
            DisableQuantize: self.DisableQuantize,
            Analysed: self.Analysed,
            ReleaseDate: self.ReleaseDate,
            DateCreated: self.DateCreated,
            ContentLink: self.ContentLink,
            Tag: self.Tag,
            ModifiedByRBM: self.ModifiedByRBM,
            HotCueAutoLoad: self.HotCueAutoLoad,
            DeliveryControl: self.DeliveryControl,
            DeliveryComment: self.DeliveryComment,
            CueUpdated: self.CueUpdated,
            AnalysisUpdated: self.AnalysisUpdated,
            TrackInfoUpdated: self.TrackInfoUpdated,
            Lyricist: self.Lyricist,
            ISRC: self.ISRC,
            SamplerTrackInfo: self.SamplerTrackInfo,
            SamplerPlayOffset: self.SamplerPlayOffset,
            SamplerGain: self.SamplerGain,
            VideoAssociate: self.VideoAssociate,
            LyricStatus: self.LyricStatus,
            ServiceID: self.ServiceID,
            OrgFolderPath: self.OrgFolderPath,
            Reserved1: self.Reserved1,
            Reserved2: self.Reserved2,
            Reserved3: self.Reserved3,
            Reserved4: self.Reserved4,
            ExtInfo: self.ExtInfo,
            rb_file_id: self.rb_file_id,
            DeviceID: self.DeviceID,
            rb_LocalFolderPath: self.rb_LocalFolderPath,
            SrcID: self.SrcID,
            SrcTitle: self.SrcTitle,
            SrcArtistName: self.SrcArtistName,
            SrcAlbumName: self.SrcAlbumName,
            SrcLength: self.SrcLength,
        }
    }
}

impl From<JsDjmdContent> for DjmdContent {
    fn from(model: JsDjmdContent) -> Self {
        DjmdContent {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            FolderPath: model.FolderPath,
            FileNameL: model.FileNameL,
            FileNameS: model.FileNameS,
            Title: model.Title,
            ArtistID: model.ArtistID,
            AlbumID: model.AlbumID,
            GenreID: model.GenreID,
            BPM: model.BPM,
            Length: model.Length,
            TrackNo: model.TrackNo,
            BitRate: model.BitRate,
            BitDepth: model.BitDepth,
            Commnt: model.Commnt,
            FileType: model.FileType,
            Rating: model.Rating,
            ReleaseYear: model.ReleaseYear,
            RemixerID: model.RemixerID,
            LabelID: model.LabelID,
            OrgArtistID: model.OrgArtistID,
            KeyID: model.KeyID,
            StockDate: model.StockDate,
            ColorID: model.ColorID,
            DJPlayCount: model.DJPlayCount,
            ImagePath: model.ImagePath,
            MasterDBID: model.MasterDBID,
            MasterSongID: model.MasterSongID,
            AnalysisDataPath: model.AnalysisDataPath,
            SearchStr: model.SearchStr,
            FileSize: model.FileSize,
            DiscNo: model.DiscNo,
            ComposerID: model.ComposerID,
            Subtitle: model.Subtitle,
            SampleRate: model.SampleRate,
            DisableQuantize: model.DisableQuantize,
            Analysed: model.Analysed,
            ReleaseDate: model.ReleaseDate,
            DateCreated: model.DateCreated,
            ContentLink: model.ContentLink,
            Tag: model.Tag,
            ModifiedByRBM: model.ModifiedByRBM,
            HotCueAutoLoad: model.HotCueAutoLoad,
            DeliveryControl: model.DeliveryControl,
            DeliveryComment: model.DeliveryComment,
            CueUpdated: model.CueUpdated,
            AnalysisUpdated: model.AnalysisUpdated,
            TrackInfoUpdated: model.TrackInfoUpdated,
            Lyricist: model.Lyricist,
            ISRC: model.ISRC,
            SamplerTrackInfo: model.SamplerTrackInfo,
            SamplerPlayOffset: model.SamplerPlayOffset,
            SamplerGain: model.SamplerGain,
            VideoAssociate: model.VideoAssociate,
            LyricStatus: model.LyricStatus,
            ServiceID: model.ServiceID,
            OrgFolderPath: model.OrgFolderPath,
            Reserved1: model.Reserved1,
            Reserved2: model.Reserved2,
            Reserved3: model.Reserved3,
            Reserved4: model.Reserved4,
            ExtInfo: model.ExtInfo,
            rb_file_id: model.rb_file_id,
            DeviceID: model.DeviceID,
            rb_LocalFolderPath: model.rb_LocalFolderPath,
            SrcID: model.SrcID,
            SrcTitle: model.SrcTitle,
            SrcArtistName: model.SrcArtistName,
            SrcAlbumName: model.SrcAlbumName,
            SrcLength: model.SrcLength,
        }
    }
}

#[napi(object, js_name = "DjmdCue")]
pub struct JsDjmdCue {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ContentID: Option<String>,
    pub InMsec: Option<i32>,
    pub InFrame: Option<i32>,
    pub InMpegFrame: Option<i32>,
    pub InMpegAbs: Option<i32>,
    pub OutMsec: Option<i32>,
    pub OutFrame: Option<i32>,
    pub OutMpegFrame: Option<i32>,
    pub OutMpegAbs: Option<i32>,
    pub Kind: Option<i32>,
    pub Color: Option<i32>,
    pub ColorTableIndex: Option<i32>,
    pub ActiveLoop: Option<i32>,
    pub Comment: Option<String>,
    pub BeatLoopSize: Option<i32>,
    pub CueMicrosec: Option<i32>,
    pub InPointSeekInfo: Option<String>,
    pub OutPointSeekInfo: Option<String>,
    pub ContentUUID: Option<String>,
}

impl IntoJs<JsDjmdCue> for DjmdCue {
    fn into_js(self, env: &Env) -> JsDjmdCue {
        JsDjmdCue {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ContentID: self.ContentID,
            InMsec: self.InMsec,
            InFrame: self.InFrame,
            InMpegFrame: self.InMpegFrame,
            InMpegAbs: self.InMpegAbs,
            OutMsec: self.OutMsec,
            OutFrame: self.OutFrame,
            OutMpegFrame: self.OutMpegFrame,
            OutMpegAbs: self.OutMpegAbs,
            Kind: self.Kind,
            Color: self.Color,
            ColorTableIndex: self.ColorTableIndex,
            ActiveLoop: self.ActiveLoop,
            Comment: self.Comment,
            BeatLoopSize: self.BeatLoopSize,
            CueMicrosec: self.CueMicrosec,
            InPointSeekInfo: self.InPointSeekInfo,
            OutPointSeekInfo: self.OutPointSeekInfo,
            ContentUUID: self.ContentUUID,
        }
    }
}

impl From<JsDjmdCue> for DjmdCue {
    fn from(model: JsDjmdCue) -> Self {
        DjmdCue {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ContentID: model.ContentID,
            InMsec: model.InMsec,
            InFrame: model.InFrame,
            InMpegFrame: model.InMpegFrame,
            InMpegAbs: model.InMpegAbs,
            OutMsec: model.OutMsec,
            OutFrame: model.OutFrame,
            OutMpegFrame: model.OutMpegFrame,
            OutMpegAbs: model.OutMpegAbs,
            Kind: model.Kind,
            Color: model.Color,
            ColorTableIndex: model.ColorTableIndex,
            ActiveLoop: model.ActiveLoop,
            Comment: model.Comment,
            BeatLoopSize: model.BeatLoopSize,
            CueMicrosec: model.CueMicrosec,
            InPointSeekInfo: model.InPointSeekInfo,
            OutPointSeekInfo: model.OutPointSeekInfo,
            ContentUUID: model.ContentUUID,
        }
    }
}

#[napi(object, js_name = "DjmdDevice")]
pub struct JsDjmdDevice {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub MasterDBID: Option<String>,
    pub Name: Option<String>,
}

impl IntoJs<JsDjmdDevice> for DjmdDevice {
    fn into_js(self, env: &Env) -> JsDjmdDevice {
        JsDjmdDevice {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            MasterDBID: self.MasterDBID,
            Name: self.Name,
        }
    }
}

impl From<JsDjmdDevice> for DjmdDevice {
    fn from(model: JsDjmdDevice) -> Self {
        DjmdDevice {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            MasterDBID: model.MasterDBID,
            Name: model.Name,
        }
    }
}

#[napi(object, js_name = "DjmdGenre")]
pub struct JsDjmdGenre {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Name: Option<String>,
}

impl IntoJs<JsDjmdGenre> for DjmdGenre {
    fn into_js(self, env: &Env) -> JsDjmdGenre {
        JsDjmdGenre {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Name: self.Name,
        }
    }
}

impl From<JsDjmdGenre> for DjmdGenre {
    fn from(model: JsDjmdGenre) -> Self {
        DjmdGenre {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Name: model.Name,
        }
    }
}

#[napi(object, js_name = "DjmdHistory")]
pub struct JsDjmdHistory {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub DateCreated: Option<String>,
}

impl IntoJs<JsDjmdHistory> for DjmdHistory {
    fn into_js(self, env: &Env) -> JsDjmdHistory {
        JsDjmdHistory {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Seq: self.Seq,
            Name: self.Name,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
            DateCreated: self.DateCreated,
        }
    }
}

impl From<JsDjmdHistory> for DjmdHistory {
    fn from(model: JsDjmdHistory) -> Self {
        DjmdHistory {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Seq: model.Seq,
            Name: model.Name,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
            DateCreated: model.DateCreated,
        }
    }
}

#[napi(object, js_name = "DjmdSongHistory")]
pub struct JsDjmdSongHistory {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub HistoryID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoJs<JsDjmdSongHistory> for DjmdSongHistory {
    fn into_js(self, env: &Env) -> JsDjmdSongHistory {
        JsDjmdSongHistory {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            HistoryID: self.HistoryID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        }
    }
}

impl From<JsDjmdSongHistory> for DjmdSongHistory {
    fn from(model: JsDjmdSongHistory) -> Self {
        DjmdSongHistory {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            HistoryID: model.HistoryID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

#[napi(object, js_name = "DjmdHotCueBanklist")]
pub struct JsDjmdHotCueBanklist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

impl IntoJs<JsDjmdHotCueBanklist> for DjmdHotCueBanklist {
    fn into_js(self, env: &Env) -> JsDjmdHotCueBanklist {
        JsDjmdHotCueBanklist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Seq: self.Seq,
            Name: self.Name,
            ImagePath: self.ImagePath,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
        }
    }
}

impl From<JsDjmdHotCueBanklist> for DjmdHotCueBanklist {
    fn from(model: JsDjmdHotCueBanklist) -> Self {
        DjmdHotCueBanklist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Seq: model.Seq,
            Name: model.Name,
            ImagePath: model.ImagePath,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
        }
    }
}

#[napi(object, js_name = "DjmdSongHotCueBanklist")]
pub struct JsDjmdSongHotCueBanklist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
    pub CueID: Option<String>,
    pub InMsec: Option<i32>,
    pub InFrame: Option<i32>,
    pub InMpegFrame: Option<i32>,
    pub InMpegAbs: Option<i32>,
    pub OutMsec: Option<i32>,
    pub OutFrame: Option<i32>,
    pub OutMpegFrame: Option<i32>,
    pub OutMpegAbs: Option<i32>,
    pub Color: Option<i32>,
    pub ColorTableIndex: Option<i32>,
    pub ActiveLoop: Option<i32>,
    pub Comment: Option<String>,
    pub BeatLoopSize: Option<i32>,
    pub CueMicrosec: Option<i32>,
    pub InPointSeekInfo: Option<String>,
    pub OutPointSeekInfo: Option<String>,
    pub HotCueBanklistUUID: Option<String>,
}

impl IntoJs<JsDjmdSongHotCueBanklist> for DjmdSongHotCueBanklist {
    fn into_js(self, env: &Env) -> JsDjmdSongHotCueBanklist {
        JsDjmdSongHotCueBanklist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
            CueID: self.CueID,
            InMsec: self.InMsec,
            InFrame: self.InFrame,
            InMpegFrame: self.InMpegFrame,
            InMpegAbs: self.InMpegAbs,
            OutMsec: self.OutMsec,
            OutFrame: self.OutFrame,
            OutMpegFrame: self.OutMpegFrame,
            OutMpegAbs: self.OutMpegAbs,
            Color: self.Color,
            ColorTableIndex: self.ColorTableIndex,
            ActiveLoop: self.ActiveLoop,
            Comment: self.Comment,
            BeatLoopSize: self.BeatLoopSize,
            CueMicrosec: self.CueMicrosec,
            InPointSeekInfo: self.InPointSeekInfo,
            OutPointSeekInfo: self.OutPointSeekInfo,
            HotCueBanklistUUID: self.HotCueBanklistUUID,
        }
    }
}

impl From<JsDjmdSongHotCueBanklist> for DjmdSongHotCueBanklist {
    fn from(model: JsDjmdSongHotCueBanklist) -> Self {
        DjmdSongHotCueBanklist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
            CueID: model.CueID,
            InMsec: model.InMsec,
            InFrame: model.InFrame,
            InMpegFrame: model.InMpegFrame,
            InMpegAbs: model.InMpegAbs,
            OutMsec: model.OutMsec,
            OutFrame: model.OutFrame,
            OutMpegFrame: model.OutMpegFrame,
            OutMpegAbs: model.OutMpegAbs,
            Color: model.Color,
            ColorTableIndex: model.ColorTableIndex,
            ActiveLoop: model.ActiveLoop,
            Comment: model.Comment,
            BeatLoopSize: model.BeatLoopSize,
            CueMicrosec: model.CueMicrosec,
            InPointSeekInfo: model.InPointSeekInfo,
            OutPointSeekInfo: model.OutPointSeekInfo,
            HotCueBanklistUUID: model.HotCueBanklistUUID,
        }
    }
}

#[napi(object, js_name = "HotCueBanklistCue")]
pub struct JsHotCueBanklistCue {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub HotCueBanklistID: Option<String>,
    pub Cues: Option<String>,
    pub rb_cue_count: Option<i32>,
}

impl IntoJs<JsHotCueBanklistCue> for HotCueBanklistCue {
    fn into_js(self, env: &Env) -> JsHotCueBanklistCue {
        JsHotCueBanklistCue {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            HotCueBanklistID: self.HotCueBanklistID,
            Cues: self.Cues,
            rb_cue_count: self.rb_cue_count,
        }
    }
}

impl From<JsHotCueBanklistCue> for HotCueBanklistCue {
    fn from(model: JsHotCueBanklistCue) -> Self {
        HotCueBanklistCue {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            HotCueBanklistID: model.HotCueBanklistID,
            Cues: model.Cues,
            rb_cue_count: model.rb_cue_count,
        }
    }
}

#[napi(object, js_name = "DjmdKey")]
pub struct JsDjmdKey {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ScaleName: Option<String>,
    pub Seq: Option<i32>,
}

impl IntoJs<JsDjmdKey> for DjmdKey {
    fn into_js(self, env: &Env) -> JsDjmdKey {
        JsDjmdKey {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ScaleName: self.ScaleName,
            Seq: self.Seq,
        }
    }
}

impl From<JsDjmdKey> for DjmdKey {
    fn from(model: JsDjmdKey) -> Self {
        DjmdKey {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ScaleName: model.ScaleName,
            Seq: model.Seq,
        }
    }
}

#[napi(object, js_name = "DjmdLabel")]
pub struct JsDjmdLabel {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Name: Option<String>,
}

impl IntoJs<JsDjmdLabel> for DjmdLabel {
    fn into_js(self, env: &Env) -> JsDjmdLabel {
        JsDjmdLabel {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Name: self.Name,
        }
    }
}

impl From<JsDjmdLabel> for DjmdLabel {
    fn from(model: JsDjmdLabel) -> Self {
        DjmdLabel {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Name: model.Name,
        }
    }
}

#[napi(object, js_name = "DjmdMenuItems")]
pub struct JsDjmdMenuItems {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Class: Option<i32>,
    pub Name: Option<String>,
}

impl IntoJs<JsDjmdMenuItems> for DjmdMenuItems {
    fn into_js(self, env: &Env) -> JsDjmdMenuItems {
        JsDjmdMenuItems {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Class: self.Class,
            Name: self.Name,
        }
    }
}

impl From<JsDjmdMenuItems> for DjmdMenuItems {
    fn from(model: JsDjmdMenuItems) -> Self {
        DjmdMenuItems {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Class: model.Class,
            Name: model.Name,
        }
    }
}

#[napi(object, js_name = "DjmdMixerParam")]
pub struct JsDjmdMixerParam {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ContentID: Option<String>,
    pub GainHigh: Option<i32>,
    pub GainLow: Option<i32>,
    pub PeakHigh: Option<i32>,
    pub PeakLow: Option<i32>,
}

impl IntoJs<JsDjmdMixerParam> for DjmdMixerParam {
    fn into_js(self, env: &Env) -> JsDjmdMixerParam {
        JsDjmdMixerParam {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ContentID: self.ContentID,
            GainHigh: self.GainHigh,
            GainLow: self.GainLow,
            PeakHigh: self.PeakHigh,
            PeakLow: self.PeakLow,
        }
    }
}

impl From<JsDjmdMixerParam> for DjmdMixerParam {
    fn from(model: JsDjmdMixerParam) -> Self {
        DjmdMixerParam {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ContentID: model.ContentID,
            GainHigh: model.GainHigh,
            GainLow: model.GainLow,
            PeakHigh: model.PeakHigh,
            PeakLow: model.PeakLow,
        }
    }
}

#[napi(object, js_name = "DjmdMyTag")]
pub struct JsDjmdMyTag {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

impl IntoJs<JsDjmdMyTag> for DjmdMyTag {
    fn into_js(self, env: &Env) -> JsDjmdMyTag {
        JsDjmdMyTag {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Seq: self.Seq,
            Name: self.Name,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
        }
    }
}

impl From<JsDjmdMyTag> for DjmdMyTag {
    fn from(model: JsDjmdMyTag) -> Self {
        DjmdMyTag {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Seq: model.Seq,
            Name: model.Name,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
        }
    }
}

#[napi(object, js_name = "DjmdSongMyTag")]
pub struct JsDjmdSongMyTag {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub MyTagID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoJs<JsDjmdSongMyTag> for DjmdSongMyTag {
    fn into_js(self, env: &Env) -> JsDjmdSongMyTag {
        JsDjmdSongMyTag {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            MyTagID: self.MyTagID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        }
    }
}

impl From<JsDjmdSongMyTag> for DjmdSongMyTag {
    fn from(model: JsDjmdSongMyTag) -> Self {
        DjmdSongMyTag {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            MyTagID: model.MyTagID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

#[napi(object, js_name = "DjmdPlaylist")]
pub struct JsDjmdPlaylist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub SmartList: Option<String>,
}

impl IntoJs<JsDjmdPlaylist> for DjmdPlaylist {
    fn into_js(self, env: &Env) -> JsDjmdPlaylist {
        JsDjmdPlaylist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Seq: self.Seq,
            Name: self.Name,
            ImagePath: self.ImagePath,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
            SmartList: self.SmartList,
        }
    }
}

impl From<JsDjmdPlaylist> for DjmdPlaylist {
    fn from(model: JsDjmdPlaylist) -> Self {
        DjmdPlaylist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Seq: model.Seq,
            Name: model.Name,
            ImagePath: model.ImagePath,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
            SmartList: model.SmartList,
        }
    }
}

#[napi(object, js_name = "PlaylistTreeItem")]
pub struct JsPlaylistTreeItem {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub SmartList: Option<String>,

    pub Children: Vec<JsPlaylistTreeItem>,
}

impl IntoJs<JsPlaylistTreeItem> for PlaylistTreeItem {
    fn into_js(self, env: &Env) -> JsPlaylistTreeItem {
        let mut model = JsPlaylistTreeItem {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Seq: self.Seq,
            Name: self.Name,
            ImagePath: self.ImagePath,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
            SmartList: self.SmartList,
            Children: Vec::new(),
        };

        for child in self.Children {
            let js_child = child.borrow().clone().into_js(env);
            model.Children.push(js_child);
        }

        model
    }
}

#[napi(object, js_name = "DjmdSongPlaylist")]
pub struct JsDjmdSongPlaylist {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub PlaylistID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoJs<JsDjmdSongPlaylist> for DjmdSongPlaylist {
    fn into_js(self, env: &Env) -> JsDjmdSongPlaylist {
        JsDjmdSongPlaylist {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            PlaylistID: self.PlaylistID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        }
    }
}

impl From<JsDjmdSongPlaylist> for DjmdSongPlaylist {
    fn from(model: JsDjmdSongPlaylist) -> Self {
        DjmdSongPlaylist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            PlaylistID: model.PlaylistID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

#[napi(object, js_name = "DjmdProperty")]
pub struct JsDjmdProperty {
    pub DBID: String,
    pub DBVersion: Option<String>,
    pub BaseDBDrive: Option<String>,
    pub CurrentDBDrive: Option<String>,
    pub DeviceID: Option<String>,
    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub Reserved5: Option<String>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
}

impl IntoJs<JsDjmdProperty> for DjmdProperty {
    fn into_js(self, env: &Env) -> JsDjmdProperty {
        JsDjmdProperty {
            DBID: self.DBID,
            DBVersion: self.DBVersion,
            BaseDBDrive: self.BaseDBDrive,
            CurrentDBDrive: self.CurrentDBDrive,
            DeviceID: self.DeviceID,
            Reserved1: self.Reserved1,
            Reserved2: self.Reserved2,
            Reserved3: self.Reserved3,
            Reserved4: self.Reserved4,
            Reserved5: self.Reserved5,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
        }
    }
}

impl From<JsDjmdProperty> for DjmdProperty {
    fn from(model: JsDjmdProperty) -> Self {
        DjmdProperty {
            DBID: model.DBID,
            DBVersion: model.DBVersion,
            BaseDBDrive: model.BaseDBDrive,
            CurrentDBDrive: model.CurrentDBDrive,
            DeviceID: model.DeviceID,
            Reserved1: model.Reserved1,
            Reserved2: model.Reserved2,
            Reserved3: model.Reserved3,
            Reserved4: model.Reserved4,
            Reserved5: model.Reserved5,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
        }
    }
}

#[napi(object, js_name = "DjmdCloudProperty")]
pub struct JsDjmdCloudProperty {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub Reserved5: Option<String>,
}

impl IntoJs<JsDjmdCloudProperty> for DjmdCloudProperty {
    fn into_js(self, env: &Env) -> JsDjmdCloudProperty {
        JsDjmdCloudProperty {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Reserved1: self.Reserved1,
            Reserved2: self.Reserved2,
            Reserved3: self.Reserved3,
            Reserved4: self.Reserved4,
            Reserved5: self.Reserved5,
        }
    }
}

impl From<JsDjmdCloudProperty> for DjmdCloudProperty {
    fn from(model: JsDjmdCloudProperty) -> Self {
        DjmdCloudProperty {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Reserved1: model.Reserved1,
            Reserved2: model.Reserved2,
            Reserved3: model.Reserved3,
            Reserved4: model.Reserved4,
            Reserved5: model.Reserved5,
        }
    }
}

#[napi(object, js_name = "DjmdRecommendLike")]
pub struct JsDjmdRecommendLike {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ContentID1: Option<String>,
    pub ContentID2: Option<String>,
    pub LikeRate: Option<i32>,
    pub DataCreatedH: Option<i32>,
    pub DataCreatedL: Option<i32>,
}

impl IntoJs<JsDjmdRecommendLike> for DjmdRecommendLike {
    fn into_js(self, env: &Env) -> JsDjmdRecommendLike {
        JsDjmdRecommendLike {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ContentID1: self.ContentID1,
            ContentID2: self.ContentID2,
            LikeRate: self.LikeRate,
            DataCreatedH: self.DataCreatedH,
            DataCreatedL: self.DataCreatedL,
        }
    }
}

impl From<JsDjmdRecommendLike> for DjmdRecommendLike {
    fn from(model: JsDjmdRecommendLike) -> Self {
        DjmdRecommendLike {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ContentID1: model.ContentID1,
            ContentID2: model.ContentID2,
            LikeRate: model.LikeRate,
            DataCreatedH: model.DataCreatedH,
            DataCreatedL: model.DataCreatedL,
        }
    }
}

#[napi(object, js_name = "DjmdRelatedTracks")]
pub struct JsDjmdRelatedTracks {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub Criteria: Option<String>,
}

impl IntoJs<JsDjmdRelatedTracks> for DjmdRelatedTracks {
    fn into_js(self, env: &Env) -> JsDjmdRelatedTracks {
        JsDjmdRelatedTracks {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Seq: self.Seq,
            Name: self.Name,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
            Criteria: self.Criteria,
        }
    }
}

impl From<JsDjmdRelatedTracks> for DjmdRelatedTracks {
    fn from(model: JsDjmdRelatedTracks) -> Self {
        DjmdRelatedTracks {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Seq: model.Seq,
            Name: model.Name,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
            Criteria: model.Criteria,
        }
    }
}

#[napi(object, js_name = "DjmdSongRelatedTracks")]
pub struct JsDjmdSongRelatedTracks {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub RelatedTracksID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoJs<JsDjmdSongRelatedTracks> for DjmdSongRelatedTracks {
    fn into_js(self, env: &Env) -> JsDjmdSongRelatedTracks {
        JsDjmdSongRelatedTracks {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            RelatedTracksID: self.RelatedTracksID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        }
    }
}

impl From<JsDjmdSongRelatedTracks> for DjmdSongRelatedTracks {
    fn from(model: JsDjmdSongRelatedTracks) -> Self {
        DjmdSongRelatedTracks {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            RelatedTracksID: model.RelatedTracksID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

#[napi(object, js_name = "DjmdSampler")]
pub struct JsDjmdSampler {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

impl IntoJs<JsDjmdSampler> for DjmdSampler {
    fn into_js(self, env: &Env) -> JsDjmdSampler {
        JsDjmdSampler {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Seq: self.Seq,
            Name: self.Name,
            Attribute: self.Attribute,
            ParentID: self.ParentID,
        }
    }
}

impl From<JsDjmdSampler> for DjmdSampler {
    fn from(model: JsDjmdSampler) -> Self {
        DjmdSampler {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Seq: model.Seq,
            Name: model.Name,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
        }
    }
}

#[napi(object, js_name = "DjmdSongSampler")]
pub struct JsDjmdSongSampler {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub SamplerID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoJs<JsDjmdSongSampler> for DjmdSongSampler {
    fn into_js(self, env: &Env) -> JsDjmdSongSampler {
        JsDjmdSongSampler {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            SamplerID: self.SamplerID,
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        }
    }
}

impl From<JsDjmdSongSampler> for DjmdSongSampler {
    fn from(model: JsDjmdSongSampler) -> Self {
        DjmdSongSampler {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            SamplerID: model.SamplerID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

#[napi(object, js_name = "DjmdSongTagList")]
pub struct JsDjmdSongTagList {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl IntoJs<JsDjmdSongTagList> for DjmdSongTagList {
    fn into_js(self, env: &Env) -> JsDjmdSongTagList {
        JsDjmdSongTagList {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            ContentID: self.ContentID,
            TrackNo: self.TrackNo,
        }
    }
}

impl From<JsDjmdSongTagList> for DjmdSongTagList {
    fn from(model: JsDjmdSongTagList) -> Self {
        DjmdSongTagList {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

#[napi(object, js_name = "DjmdSort")]
pub struct JsDjmdSort {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub MenuItemID: Option<String>,
    pub Seq: Option<i32>,
    pub Disable: Option<i32>,
}

impl IntoJs<JsDjmdSort> for DjmdSort {
    fn into_js(self, env: &Env) -> JsDjmdSort {
        JsDjmdSort {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            MenuItemID: self.MenuItemID,
            Seq: self.Seq,
            Disable: self.Disable,
        }
    }
}

impl From<JsDjmdSort> for DjmdSort {
    fn from(model: JsDjmdSort) -> Self {
        DjmdSort {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            MenuItemID: model.MenuItemID,
            Seq: model.Seq,
            Disable: model.Disable,
        }
    }
}

#[napi(object, js_name = "ImageFile")]
pub struct JsImageFile {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub TableName: Option<String>,
    pub TargetUUID: Option<String>,
    pub TargetID: Option<String>,
    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_local_file_status: Option<i32>,
    pub rb_in_progress: Option<i32>,
    pub rb_process_type: Option<i32>,
    pub rb_temp_path: Option<String>,
    pub rb_priority: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

impl IntoJs<JsImageFile> for ImageFile {
    fn into_js(self, env: &Env) -> JsImageFile {
        JsImageFile {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            TableName: self.TableName,
            TargetUUID: self.TargetUUID,
            TargetID: self.TargetID,
            Path: self.Path,
            Hash: self.Hash,
            Size: self.Size,
            rb_local_path: self.rb_local_path,
            rb_insync_hash: self.rb_insync_hash,
            rb_insync_local_usn: self.rb_insync_local_usn,
            rb_file_hash_dirty: self.rb_file_hash_dirty,
            rb_local_file_status: self.rb_local_file_status,
            rb_in_progress: self.rb_in_progress,
            rb_process_type: self.rb_process_type,
            rb_temp_path: self.rb_temp_path,
            rb_priority: self.rb_priority,
            rb_file_size_dirty: self.rb_file_size_dirty,
        }
    }
}

impl From<JsImageFile> for ImageFile {
    fn from(model: JsImageFile) -> Self {
        ImageFile {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            TableName: model.TableName,
            TargetUUID: model.TargetUUID,
            TargetID: model.TargetID,
            Path: model.Path,
            Hash: model.Hash,
            Size: model.Size,
            rb_local_path: model.rb_local_path,
            rb_insync_hash: model.rb_insync_hash,
            rb_insync_local_usn: model.rb_insync_local_usn,
            rb_file_hash_dirty: model.rb_file_hash_dirty,
            rb_local_file_status: model.rb_local_file_status,
            rb_in_progress: model.rb_in_progress,
            rb_process_type: model.rb_process_type,
            rb_temp_path: model.rb_temp_path,
            rb_priority: model.rb_priority,
            rb_file_size_dirty: model.rb_file_size_dirty,
        }
    }
}

#[napi(object, js_name = "SettingFile")]
pub struct JsSettingFile {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

impl IntoJs<JsSettingFile> for SettingFile {
    fn into_js(self, env: &Env) -> JsSettingFile {
        JsSettingFile {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            Path: self.Path,
            Hash: self.Hash,
            Size: self.Size,
            rb_local_path: self.rb_local_path,
            rb_insync_hash: self.rb_insync_hash,
            rb_insync_local_usn: self.rb_insync_local_usn,
            rb_file_hash_dirty: self.rb_file_hash_dirty,
            rb_file_size_dirty: self.rb_file_size_dirty,
        }
    }
}

impl From<JsSettingFile> for SettingFile {
    fn from(model: JsSettingFile) -> Self {
        SettingFile {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            Path: model.Path,
            Hash: model.Hash,
            Size: model.Size,
            rb_local_path: model.rb_local_path,
            rb_insync_hash: model.rb_insync_hash,
            rb_insync_local_usn: model.rb_insync_local_usn,
            rb_file_hash_dirty: model.rb_file_hash_dirty,
            rb_file_size_dirty: model.rb_file_size_dirty,
        }
    }
}

#[napi(object, js_name = "UuidIDMap")]
pub struct JsUuidIDMap {
    pub ID: String,
    pub UUID: String,
    pub rb_data_status: i32,
    pub rb_local_data_status: i32,
    pub rb_local_deleted: i32,
    pub rb_local_synced: i32,
    pub usn: Option<i32>,
    pub rb_local_usn: Option<i32>,
    pub created_at: JsDate,
    pub updated_at: JsDate,
    pub TableName: Option<String>,
    pub TargetUUID: Option<String>,
    pub CurrentID: Option<String>,
}

impl IntoJs<JsUuidIDMap> for UuidIDMap {
    fn into_js(self, env: &Env) -> JsUuidIDMap {
        JsUuidIDMap {
            ID: self.ID,
            UUID: self.UUID,
            rb_data_status: self.rb_data_status,
            rb_local_data_status: self.rb_local_data_status,
            rb_local_deleted: self.rb_local_deleted,
            rb_local_synced: self.rb_local_synced,
            usn: self.usn,
            rb_local_usn: self.rb_local_usn,
            created_at: datetime_to_js_date(env, &self.created_at),
            updated_at: datetime_to_js_date(env, &self.updated_at),
            TableName: self.TableName,
            TargetUUID: self.TargetUUID,
            CurrentID: self.CurrentID,
        }
    }
}

impl From<JsUuidIDMap> for UuidIDMap {
    fn from(model: JsUuidIDMap) -> Self {
        UuidIDMap {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: js_date_to_datetime(model.created_at),
            updated_at: js_date_to_datetime(model.updated_at),
            TableName: model.TableName,
            TargetUUID: model.TargetUUID,
            CurrentID: model.CurrentID,
        }
    }
}
