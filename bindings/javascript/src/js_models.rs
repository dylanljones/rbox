// Auto-generated code for JavaScript bindings

#![allow(non_snake_case)]

use napi_derive::napi;
use rbox::masterdb::models::*;

#[napi(object, js_name = "AgentRegistry")]
pub struct JsAgentRegistry {
    pub registry_id: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
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

impl From<AgentRegistry> for JsAgentRegistry {
    fn from(model: AgentRegistry) -> Self {
        JsAgentRegistry {
            registry_id: model.registry_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

impl From<JsAgentRegistry> for AgentRegistry {
    fn from(js_model: JsAgentRegistry) -> Self {
        AgentRegistry {
            registry_id: js_model.registry_id,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            id_1: js_model.id_1,
            id_2: js_model.id_2,
            int_1: js_model.int_1,
            int_2: js_model.int_2,
            str_1: js_model.str_1,
            str_2: js_model.str_2,
            date_1: js_model.date_1,
            date_2: js_model.date_2,
            text_1: js_model.text_1,
            text_2: js_model.text_2,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub int_1: Option<i32>,
    pub int_2: Option<i32>,
    pub str_1: Option<String>,
    pub str_2: Option<String>,
    pub date_1: Option<String>,
    pub date_2: Option<String>,
    pub text_1: Option<String>,
    pub text_2: Option<String>,
}

impl From<CloudAgentRegistry> for JsCloudAgentRegistry {
    fn from(model: CloudAgentRegistry) -> Self {
        JsCloudAgentRegistry {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

impl From<JsCloudAgentRegistry> for CloudAgentRegistry {
    fn from(js_model: JsCloudAgentRegistry) -> Self {
        CloudAgentRegistry {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            int_1: js_model.int_1,
            int_2: js_model.int_2,
            str_1: js_model.str_1,
            str_2: js_model.str_2,
            date_1: js_model.date_1,
            date_2: js_model.date_2,
            text_1: js_model.text_1,
            text_2: js_model.text_2,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub ContentID: Option<String>,
    pub ActiveCensors: Option<String>,
    pub rb_activecensor_count: Option<i32>,
}

impl From<ContentActiveCensor> for JsContentActiveCensor {
    fn from(model: ContentActiveCensor) -> Self {
        JsContentActiveCensor {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            ContentID: model.ContentID,
            ActiveCensors: model.ActiveCensors,
            rb_activecensor_count: model.rb_activecensor_count,
        }
    }
}

impl From<JsContentActiveCensor> for ContentActiveCensor {
    fn from(js_model: JsContentActiveCensor) -> Self {
        ContentActiveCensor {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ContentID: js_model.ContentID,
            ActiveCensors: js_model.ActiveCensors,
            rb_activecensor_count: js_model.rb_activecensor_count,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub ContentID: Option<String>,
    pub Cues: Option<String>,
    pub rb_cue_count: Option<i32>,
}

impl From<ContentCue> for JsContentCue {
    fn from(model: ContentCue) -> Self {
        JsContentCue {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            ContentID: model.ContentID,
            Cues: model.Cues,
            rb_cue_count: model.rb_cue_count,
        }
    }
}

impl From<JsContentCue> for ContentCue {
    fn from(js_model: JsContentCue) -> Self {
        ContentCue {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ContentID: js_model.ContentID,
            Cues: js_model.Cues,
            rb_cue_count: js_model.rb_cue_count,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
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

impl From<ContentFile> for JsContentFile {
    fn from(model: ContentFile) -> Self {
        JsContentFile {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

impl From<JsContentFile> for ContentFile {
    fn from(js_model: JsContentFile) -> Self {
        ContentFile {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ContentID: js_model.ContentID,
            Path: js_model.Path,
            Hash: js_model.Hash,
            Size: js_model.Size,
            rb_local_path: js_model.rb_local_path,
            rb_insync_hash: js_model.rb_insync_hash,
            rb_insync_local_usn: js_model.rb_insync_local_usn,
            rb_file_hash_dirty: js_model.rb_file_hash_dirty,
            rb_local_file_status: js_model.rb_local_file_status,
            rb_in_progress: js_model.rb_in_progress,
            rb_process_type: js_model.rb_process_type,
            rb_temp_path: js_model.rb_temp_path,
            rb_priority: js_model.rb_priority,
            rb_file_size_dirty: js_model.rb_file_size_dirty,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub ContentID: Option<String>,
    pub InMsec: Option<i32>,
    pub OutMsec: Option<i32>,
    pub Info: Option<i32>,
    pub ParameterList: Option<String>,
    pub ContentUUID: Option<String>,
}

impl From<DjmdActiveCensor> for JsDjmdActiveCensor {
    fn from(model: DjmdActiveCensor) -> Self {
        JsDjmdActiveCensor {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            ContentID: model.ContentID,
            InMsec: model.InMsec,
            OutMsec: model.OutMsec,
            Info: model.Info,
            ParameterList: model.ParameterList,
            ContentUUID: model.ContentUUID,
        }
    }
}

impl From<JsDjmdActiveCensor> for DjmdActiveCensor {
    fn from(js_model: JsDjmdActiveCensor) -> Self {
        DjmdActiveCensor {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ContentID: js_model.ContentID,
            InMsec: js_model.InMsec,
            OutMsec: js_model.OutMsec,
            Info: js_model.Info,
            ParameterList: js_model.ParameterList,
            ContentUUID: js_model.ContentUUID,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Name: Option<String>,
    pub AlbumArtistID: Option<String>,
    pub ImagePath: Option<String>,
    pub Compilation: Option<i32>,
    pub SearchStr: Option<String>,
}

impl From<DjmdAlbum> for JsDjmdAlbum {
    fn from(model: DjmdAlbum) -> Self {
        JsDjmdAlbum {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Name: model.Name,
            AlbumArtistID: model.AlbumArtistID,
            ImagePath: model.ImagePath,
            Compilation: model.Compilation,
            SearchStr: model.SearchStr,
        }
    }
}

impl From<JsDjmdAlbum> for DjmdAlbum {
    fn from(js_model: JsDjmdAlbum) -> Self {
        DjmdAlbum {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Name: js_model.Name,
            AlbumArtistID: js_model.AlbumArtistID,
            ImagePath: js_model.ImagePath,
            Compilation: js_model.Compilation,
            SearchStr: js_model.SearchStr,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Name: Option<String>,
    pub SearchStr: Option<String>,
}

impl From<DjmdArtist> for JsDjmdArtist {
    fn from(model: DjmdArtist) -> Self {
        JsDjmdArtist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Name: model.Name,
            SearchStr: model.SearchStr,
        }
    }
}

impl From<JsDjmdArtist> for DjmdArtist {
    fn from(js_model: JsDjmdArtist) -> Self {
        DjmdArtist {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Name: js_model.Name,
            SearchStr: js_model.SearchStr,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub MenuItemID: Option<String>,
    pub Seq: Option<i32>,
    pub Disable: Option<i32>,
    pub InfoOrder: Option<i32>,
}

impl From<DjmdCategory> for JsDjmdCategory {
    fn from(model: DjmdCategory) -> Self {
        JsDjmdCategory {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            MenuItemID: model.MenuItemID,
            Seq: model.Seq,
            Disable: model.Disable,
            InfoOrder: model.InfoOrder,
        }
    }
}

impl From<JsDjmdCategory> for DjmdCategory {
    fn from(js_model: JsDjmdCategory) -> Self {
        DjmdCategory {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            MenuItemID: js_model.MenuItemID,
            Seq: js_model.Seq,
            Disable: js_model.Disable,
            InfoOrder: js_model.InfoOrder,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub ColorCode: Option<String>,
    pub SortKey: Option<i32>,
    pub Commnt: Option<String>,
}

impl From<DjmdColor> for JsDjmdColor {
    fn from(model: DjmdColor) -> Self {
        JsDjmdColor {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            ColorCode: model.ColorCode,
            SortKey: model.SortKey,
            Commnt: model.Commnt,
        }
    }
}

impl From<JsDjmdColor> for DjmdColor {
    fn from(js_model: JsDjmdColor) -> Self {
        DjmdColor {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ColorCode: js_model.ColorCode,
            SortKey: js_model.SortKey,
            Commnt: js_model.Commnt,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
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

impl From<DjmdContent> for JsDjmdContent {
    fn from(model: DjmdContent) -> Self {
        JsDjmdContent {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

impl From<JsDjmdContent> for DjmdContent {
    fn from(js_model: JsDjmdContent) -> Self {
        DjmdContent {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            FolderPath: js_model.FolderPath,
            FileNameL: js_model.FileNameL,
            FileNameS: js_model.FileNameS,
            Title: js_model.Title,
            ArtistID: js_model.ArtistID,
            AlbumID: js_model.AlbumID,
            GenreID: js_model.GenreID,
            BPM: js_model.BPM,
            Length: js_model.Length,
            TrackNo: js_model.TrackNo,
            BitRate: js_model.BitRate,
            BitDepth: js_model.BitDepth,
            Commnt: js_model.Commnt,
            FileType: js_model.FileType,
            Rating: js_model.Rating,
            ReleaseYear: js_model.ReleaseYear,
            RemixerID: js_model.RemixerID,
            LabelID: js_model.LabelID,
            OrgArtistID: js_model.OrgArtistID,
            KeyID: js_model.KeyID,
            StockDate: js_model.StockDate,
            ColorID: js_model.ColorID,
            DJPlayCount: js_model.DJPlayCount,
            ImagePath: js_model.ImagePath,
            MasterDBID: js_model.MasterDBID,
            MasterSongID: js_model.MasterSongID,
            AnalysisDataPath: js_model.AnalysisDataPath,
            SearchStr: js_model.SearchStr,
            FileSize: js_model.FileSize,
            DiscNo: js_model.DiscNo,
            ComposerID: js_model.ComposerID,
            Subtitle: js_model.Subtitle,
            SampleRate: js_model.SampleRate,
            DisableQuantize: js_model.DisableQuantize,
            Analysed: js_model.Analysed,
            ReleaseDate: js_model.ReleaseDate,
            DateCreated: js_model.DateCreated,
            ContentLink: js_model.ContentLink,
            Tag: js_model.Tag,
            ModifiedByRBM: js_model.ModifiedByRBM,
            HotCueAutoLoad: js_model.HotCueAutoLoad,
            DeliveryControl: js_model.DeliveryControl,
            DeliveryComment: js_model.DeliveryComment,
            CueUpdated: js_model.CueUpdated,
            AnalysisUpdated: js_model.AnalysisUpdated,
            TrackInfoUpdated: js_model.TrackInfoUpdated,
            Lyricist: js_model.Lyricist,
            ISRC: js_model.ISRC,
            SamplerTrackInfo: js_model.SamplerTrackInfo,
            SamplerPlayOffset: js_model.SamplerPlayOffset,
            SamplerGain: js_model.SamplerGain,
            VideoAssociate: js_model.VideoAssociate,
            LyricStatus: js_model.LyricStatus,
            ServiceID: js_model.ServiceID,
            OrgFolderPath: js_model.OrgFolderPath,
            Reserved1: js_model.Reserved1,
            Reserved2: js_model.Reserved2,
            Reserved3: js_model.Reserved3,
            Reserved4: js_model.Reserved4,
            ExtInfo: js_model.ExtInfo,
            rb_file_id: js_model.rb_file_id,
            DeviceID: js_model.DeviceID,
            rb_LocalFolderPath: js_model.rb_LocalFolderPath,
            SrcID: js_model.SrcID,
            SrcTitle: js_model.SrcTitle,
            SrcArtistName: js_model.SrcArtistName,
            SrcAlbumName: js_model.SrcAlbumName,
            SrcLength: js_model.SrcLength,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
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

impl From<DjmdCue> for JsDjmdCue {
    fn from(model: DjmdCue) -> Self {
        JsDjmdCue {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

impl From<JsDjmdCue> for DjmdCue {
    fn from(js_model: JsDjmdCue) -> Self {
        DjmdCue {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ContentID: js_model.ContentID,
            InMsec: js_model.InMsec,
            InFrame: js_model.InFrame,
            InMpegFrame: js_model.InMpegFrame,
            InMpegAbs: js_model.InMpegAbs,
            OutMsec: js_model.OutMsec,
            OutFrame: js_model.OutFrame,
            OutMpegFrame: js_model.OutMpegFrame,
            OutMpegAbs: js_model.OutMpegAbs,
            Kind: js_model.Kind,
            Color: js_model.Color,
            ColorTableIndex: js_model.ColorTableIndex,
            ActiveLoop: js_model.ActiveLoop,
            Comment: js_model.Comment,
            BeatLoopSize: js_model.BeatLoopSize,
            CueMicrosec: js_model.CueMicrosec,
            InPointSeekInfo: js_model.InPointSeekInfo,
            OutPointSeekInfo: js_model.OutPointSeekInfo,
            ContentUUID: js_model.ContentUUID,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub MasterDBID: Option<String>,
    pub Name: Option<String>,
}

impl From<DjmdDevice> for JsDjmdDevice {
    fn from(model: DjmdDevice) -> Self {
        JsDjmdDevice {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            MasterDBID: model.MasterDBID,
            Name: model.Name,
        }
    }
}

impl From<JsDjmdDevice> for DjmdDevice {
    fn from(js_model: JsDjmdDevice) -> Self {
        DjmdDevice {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            MasterDBID: js_model.MasterDBID,
            Name: js_model.Name,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Name: Option<String>,
}

impl From<DjmdGenre> for JsDjmdGenre {
    fn from(model: DjmdGenre) -> Self {
        JsDjmdGenre {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Name: model.Name,
        }
    }
}

impl From<JsDjmdGenre> for DjmdGenre {
    fn from(js_model: JsDjmdGenre) -> Self {
        DjmdGenre {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Name: js_model.Name,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub DateCreated: Option<String>,
}

impl From<DjmdHistory> for JsDjmdHistory {
    fn from(model: DjmdHistory) -> Self {
        JsDjmdHistory {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Seq: model.Seq,
            Name: model.Name,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
            DateCreated: model.DateCreated,
        }
    }
}

impl From<JsDjmdHistory> for DjmdHistory {
    fn from(js_model: JsDjmdHistory) -> Self {
        DjmdHistory {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Seq: js_model.Seq,
            Name: js_model.Name,
            Attribute: js_model.Attribute,
            ParentID: js_model.ParentID,
            DateCreated: js_model.DateCreated,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub HistoryID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl From<DjmdSongHistory> for JsDjmdSongHistory {
    fn from(model: DjmdSongHistory) -> Self {
        JsDjmdSongHistory {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            HistoryID: model.HistoryID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

impl From<JsDjmdSongHistory> for DjmdSongHistory {
    fn from(js_model: JsDjmdSongHistory) -> Self {
        DjmdSongHistory {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            HistoryID: js_model.HistoryID,
            ContentID: js_model.ContentID,
            TrackNo: js_model.TrackNo,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

impl From<DjmdHotCueBanklist> for JsDjmdHotCueBanklist {
    fn from(model: DjmdHotCueBanklist) -> Self {
        JsDjmdHotCueBanklist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Seq: model.Seq,
            Name: model.Name,
            ImagePath: model.ImagePath,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
        }
    }
}

impl From<JsDjmdHotCueBanklist> for DjmdHotCueBanklist {
    fn from(js_model: JsDjmdHotCueBanklist) -> Self {
        DjmdHotCueBanklist {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Seq: js_model.Seq,
            Name: js_model.Name,
            ImagePath: js_model.ImagePath,
            Attribute: js_model.Attribute,
            ParentID: js_model.ParentID,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
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

impl From<DjmdSongHotCueBanklist> for JsDjmdSongHotCueBanklist {
    fn from(model: DjmdSongHotCueBanklist) -> Self {
        JsDjmdSongHotCueBanklist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

impl From<JsDjmdSongHotCueBanklist> for DjmdSongHotCueBanklist {
    fn from(js_model: JsDjmdSongHotCueBanklist) -> Self {
        DjmdSongHotCueBanklist {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ContentID: js_model.ContentID,
            TrackNo: js_model.TrackNo,
            CueID: js_model.CueID,
            InMsec: js_model.InMsec,
            InFrame: js_model.InFrame,
            InMpegFrame: js_model.InMpegFrame,
            InMpegAbs: js_model.InMpegAbs,
            OutMsec: js_model.OutMsec,
            OutFrame: js_model.OutFrame,
            OutMpegFrame: js_model.OutMpegFrame,
            OutMpegAbs: js_model.OutMpegAbs,
            Color: js_model.Color,
            ColorTableIndex: js_model.ColorTableIndex,
            ActiveLoop: js_model.ActiveLoop,
            Comment: js_model.Comment,
            BeatLoopSize: js_model.BeatLoopSize,
            CueMicrosec: js_model.CueMicrosec,
            InPointSeekInfo: js_model.InPointSeekInfo,
            OutPointSeekInfo: js_model.OutPointSeekInfo,
            HotCueBanklistUUID: js_model.HotCueBanklistUUID,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub HotCueBanklistID: Option<String>,
    pub Cues: Option<String>,
    pub rb_cue_count: Option<i32>,
}

impl From<HotCueBanklistCue> for JsHotCueBanklistCue {
    fn from(model: HotCueBanklistCue) -> Self {
        JsHotCueBanklistCue {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            HotCueBanklistID: model.HotCueBanklistID,
            Cues: model.Cues,
            rb_cue_count: model.rb_cue_count,
        }
    }
}

impl From<JsHotCueBanklistCue> for HotCueBanklistCue {
    fn from(js_model: JsHotCueBanklistCue) -> Self {
        HotCueBanklistCue {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            HotCueBanklistID: js_model.HotCueBanklistID,
            Cues: js_model.Cues,
            rb_cue_count: js_model.rb_cue_count,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub ScaleName: Option<String>,
    pub Seq: Option<i32>,
}

impl From<DjmdKey> for JsDjmdKey {
    fn from(model: DjmdKey) -> Self {
        JsDjmdKey {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            ScaleName: model.ScaleName,
            Seq: model.Seq,
        }
    }
}

impl From<JsDjmdKey> for DjmdKey {
    fn from(js_model: JsDjmdKey) -> Self {
        DjmdKey {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ScaleName: js_model.ScaleName,
            Seq: js_model.Seq,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Name: Option<String>,
}

impl From<DjmdLabel> for JsDjmdLabel {
    fn from(model: DjmdLabel) -> Self {
        JsDjmdLabel {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Name: model.Name,
        }
    }
}

impl From<JsDjmdLabel> for DjmdLabel {
    fn from(js_model: JsDjmdLabel) -> Self {
        DjmdLabel {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Name: js_model.Name,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Class: Option<i32>,
    pub Name: Option<String>,
}

impl From<DjmdMenuItems> for JsDjmdMenuItems {
    fn from(model: DjmdMenuItems) -> Self {
        JsDjmdMenuItems {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Class: model.Class,
            Name: model.Name,
        }
    }
}

impl From<JsDjmdMenuItems> for DjmdMenuItems {
    fn from(js_model: JsDjmdMenuItems) -> Self {
        DjmdMenuItems {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Class: js_model.Class,
            Name: js_model.Name,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub ContentID: Option<String>,
    pub GainHigh: Option<i32>,
    pub GainLow: Option<i32>,
    pub PeakHigh: Option<i32>,
    pub PeakLow: Option<i32>,
}

impl From<DjmdMixerParam> for JsDjmdMixerParam {
    fn from(model: DjmdMixerParam) -> Self {
        JsDjmdMixerParam {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            ContentID: model.ContentID,
            GainHigh: model.GainHigh,
            GainLow: model.GainLow,
            PeakHigh: model.PeakHigh,
            PeakLow: model.PeakLow,
        }
    }
}

impl From<JsDjmdMixerParam> for DjmdMixerParam {
    fn from(js_model: JsDjmdMixerParam) -> Self {
        DjmdMixerParam {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ContentID: js_model.ContentID,
            GainHigh: js_model.GainHigh,
            GainLow: js_model.GainLow,
            PeakHigh: js_model.PeakHigh,
            PeakLow: js_model.PeakLow,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

impl From<DjmdMyTag> for JsDjmdMyTag {
    fn from(model: DjmdMyTag) -> Self {
        JsDjmdMyTag {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Seq: model.Seq,
            Name: model.Name,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
        }
    }
}

impl From<JsDjmdMyTag> for DjmdMyTag {
    fn from(js_model: JsDjmdMyTag) -> Self {
        DjmdMyTag {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Seq: js_model.Seq,
            Name: js_model.Name,
            Attribute: js_model.Attribute,
            ParentID: js_model.ParentID,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub MyTagID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl From<DjmdSongMyTag> for JsDjmdSongMyTag {
    fn from(model: DjmdSongMyTag) -> Self {
        JsDjmdSongMyTag {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            MyTagID: model.MyTagID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

impl From<JsDjmdSongMyTag> for DjmdSongMyTag {
    fn from(js_model: JsDjmdSongMyTag) -> Self {
        DjmdSongMyTag {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            MyTagID: js_model.MyTagID,
            ContentID: js_model.ContentID,
            TrackNo: js_model.TrackNo,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub SmartList: Option<String>,
}

impl From<DjmdPlaylist> for JsDjmdPlaylist {
    fn from(model: DjmdPlaylist) -> Self {
        JsDjmdPlaylist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Seq: model.Seq,
            Name: model.Name,
            ImagePath: model.ImagePath,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
            SmartList: model.SmartList,
        }
    }
}

impl From<JsDjmdPlaylist> for DjmdPlaylist {
    fn from(js_model: JsDjmdPlaylist) -> Self {
        DjmdPlaylist {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Seq: js_model.Seq,
            Name: js_model.Name,
            ImagePath: js_model.ImagePath,
            Attribute: js_model.Attribute,
            ParentID: js_model.ParentID,
            SmartList: js_model.SmartList,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub ImagePath: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub SmartList: Option<String>,

    pub Children: Vec<JsPlaylistTreeItem>,
}

impl From<PlaylistTreeItem> for JsPlaylistTreeItem {
    fn from(model: PlaylistTreeItem) -> Self {
        let mut js_model = Self {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Seq: model.Seq,
            Name: model.Name,
            ImagePath: model.ImagePath,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
            SmartList: model.SmartList,
            Children: Vec::new(),
        };

        for child in model.Children {
            let js_child = JsPlaylistTreeItem::from(child.borrow().clone());
            js_model.Children.push(js_child);
        }

        js_model
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub PlaylistID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl From<DjmdSongPlaylist> for JsDjmdSongPlaylist {
    fn from(model: DjmdSongPlaylist) -> Self {
        JsDjmdSongPlaylist {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            PlaylistID: model.PlaylistID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

impl From<JsDjmdSongPlaylist> for DjmdSongPlaylist {
    fn from(js_model: JsDjmdSongPlaylist) -> Self {
        DjmdSongPlaylist {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            PlaylistID: js_model.PlaylistID,
            ContentID: js_model.ContentID,
            TrackNo: js_model.TrackNo,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<DjmdProperty> for JsDjmdProperty {
    fn from(model: DjmdProperty) -> Self {
        JsDjmdProperty {
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
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<JsDjmdProperty> for DjmdProperty {
    fn from(js_model: JsDjmdProperty) -> Self {
        DjmdProperty {
            DBID: js_model.DBID,
            DBVersion: js_model.DBVersion,
            BaseDBDrive: js_model.BaseDBDrive,
            CurrentDBDrive: js_model.CurrentDBDrive,
            DeviceID: js_model.DeviceID,
            Reserved1: js_model.Reserved1,
            Reserved2: js_model.Reserved2,
            Reserved3: js_model.Reserved3,
            Reserved4: js_model.Reserved4,
            Reserved5: js_model.Reserved5,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Reserved1: Option<String>,
    pub Reserved2: Option<String>,
    pub Reserved3: Option<String>,
    pub Reserved4: Option<String>,
    pub Reserved5: Option<String>,
}

impl From<DjmdCloudProperty> for JsDjmdCloudProperty {
    fn from(model: DjmdCloudProperty) -> Self {
        JsDjmdCloudProperty {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Reserved1: model.Reserved1,
            Reserved2: model.Reserved2,
            Reserved3: model.Reserved3,
            Reserved4: model.Reserved4,
            Reserved5: model.Reserved5,
        }
    }
}

impl From<JsDjmdCloudProperty> for DjmdCloudProperty {
    fn from(js_model: JsDjmdCloudProperty) -> Self {
        DjmdCloudProperty {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Reserved1: js_model.Reserved1,
            Reserved2: js_model.Reserved2,
            Reserved3: js_model.Reserved3,
            Reserved4: js_model.Reserved4,
            Reserved5: js_model.Reserved5,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub ContentID1: Option<String>,
    pub ContentID2: Option<String>,
    pub LikeRate: Option<i32>,
    pub DataCreatedH: Option<i32>,
    pub DataCreatedL: Option<i32>,
}

impl From<DjmdRecommendLike> for JsDjmdRecommendLike {
    fn from(model: DjmdRecommendLike) -> Self {
        JsDjmdRecommendLike {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            ContentID1: model.ContentID1,
            ContentID2: model.ContentID2,
            LikeRate: model.LikeRate,
            DataCreatedH: model.DataCreatedH,
            DataCreatedL: model.DataCreatedL,
        }
    }
}

impl From<JsDjmdRecommendLike> for DjmdRecommendLike {
    fn from(js_model: JsDjmdRecommendLike) -> Self {
        DjmdRecommendLike {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ContentID1: js_model.ContentID1,
            ContentID2: js_model.ContentID2,
            LikeRate: js_model.LikeRate,
            DataCreatedH: js_model.DataCreatedH,
            DataCreatedL: js_model.DataCreatedL,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
    pub Criteria: Option<String>,
}

impl From<DjmdRelatedTracks> for JsDjmdRelatedTracks {
    fn from(model: DjmdRelatedTracks) -> Self {
        JsDjmdRelatedTracks {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Seq: model.Seq,
            Name: model.Name,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
            Criteria: model.Criteria,
        }
    }
}

impl From<JsDjmdRelatedTracks> for DjmdRelatedTracks {
    fn from(js_model: JsDjmdRelatedTracks) -> Self {
        DjmdRelatedTracks {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Seq: js_model.Seq,
            Name: js_model.Name,
            Attribute: js_model.Attribute,
            ParentID: js_model.ParentID,
            Criteria: js_model.Criteria,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub RelatedTracksID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl From<DjmdSongRelatedTracks> for JsDjmdSongRelatedTracks {
    fn from(model: DjmdSongRelatedTracks) -> Self {
        JsDjmdSongRelatedTracks {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            RelatedTracksID: model.RelatedTracksID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

impl From<JsDjmdSongRelatedTracks> for DjmdSongRelatedTracks {
    fn from(js_model: JsDjmdSongRelatedTracks) -> Self {
        DjmdSongRelatedTracks {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            RelatedTracksID: js_model.RelatedTracksID,
            ContentID: js_model.ContentID,
            TrackNo: js_model.TrackNo,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Seq: Option<i32>,
    pub Name: Option<String>,
    pub Attribute: Option<i32>,
    pub ParentID: Option<String>,
}

impl From<DjmdSampler> for JsDjmdSampler {
    fn from(model: DjmdSampler) -> Self {
        JsDjmdSampler {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            Seq: model.Seq,
            Name: model.Name,
            Attribute: model.Attribute,
            ParentID: model.ParentID,
        }
    }
}

impl From<JsDjmdSampler> for DjmdSampler {
    fn from(js_model: JsDjmdSampler) -> Self {
        DjmdSampler {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Seq: js_model.Seq,
            Name: js_model.Name,
            Attribute: js_model.Attribute,
            ParentID: js_model.ParentID,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub SamplerID: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl From<DjmdSongSampler> for JsDjmdSongSampler {
    fn from(model: DjmdSongSampler) -> Self {
        JsDjmdSongSampler {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            SamplerID: model.SamplerID,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

impl From<JsDjmdSongSampler> for DjmdSongSampler {
    fn from(js_model: JsDjmdSongSampler) -> Self {
        DjmdSongSampler {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            SamplerID: js_model.SamplerID,
            ContentID: js_model.ContentID,
            TrackNo: js_model.TrackNo,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub ContentID: Option<String>,
    pub TrackNo: Option<i32>,
}

impl From<DjmdSongTagList> for JsDjmdSongTagList {
    fn from(model: DjmdSongTagList) -> Self {
        JsDjmdSongTagList {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            ContentID: model.ContentID,
            TrackNo: model.TrackNo,
        }
    }
}

impl From<JsDjmdSongTagList> for DjmdSongTagList {
    fn from(js_model: JsDjmdSongTagList) -> Self {
        DjmdSongTagList {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            ContentID: js_model.ContentID,
            TrackNo: js_model.TrackNo,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub MenuItemID: Option<String>,
    pub Seq: Option<i32>,
    pub Disable: Option<i32>,
}

impl From<DjmdSort> for JsDjmdSort {
    fn from(model: DjmdSort) -> Self {
        JsDjmdSort {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            MenuItemID: model.MenuItemID,
            Seq: model.Seq,
            Disable: model.Disable,
        }
    }
}

impl From<JsDjmdSort> for DjmdSort {
    fn from(js_model: JsDjmdSort) -> Self {
        DjmdSort {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            MenuItemID: js_model.MenuItemID,
            Seq: js_model.Seq,
            Disable: js_model.Disable,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
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

impl From<ImageFile> for JsImageFile {
    fn from(model: ImageFile) -> Self {
        JsImageFile {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

impl From<JsImageFile> for ImageFile {
    fn from(js_model: JsImageFile) -> Self {
        ImageFile {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            TableName: js_model.TableName,
            TargetUUID: js_model.TargetUUID,
            TargetID: js_model.TargetID,
            Path: js_model.Path,
            Hash: js_model.Hash,
            Size: js_model.Size,
            rb_local_path: js_model.rb_local_path,
            rb_insync_hash: js_model.rb_insync_hash,
            rb_insync_local_usn: js_model.rb_insync_local_usn,
            rb_file_hash_dirty: js_model.rb_file_hash_dirty,
            rb_local_file_status: js_model.rb_local_file_status,
            rb_in_progress: js_model.rb_in_progress,
            rb_process_type: js_model.rb_process_type,
            rb_temp_path: js_model.rb_temp_path,
            rb_priority: js_model.rb_priority,
            rb_file_size_dirty: js_model.rb_file_size_dirty,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub Path: Option<String>,
    pub Hash: Option<String>,
    pub Size: Option<i32>,
    pub rb_local_path: Option<String>,
    pub rb_insync_hash: Option<String>,
    pub rb_insync_local_usn: Option<i32>,
    pub rb_file_hash_dirty: Option<i32>,
    pub rb_file_size_dirty: Option<i32>,
}

impl From<SettingFile> for JsSettingFile {
    fn from(model: SettingFile) -> Self {
        JsSettingFile {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

impl From<JsSettingFile> for SettingFile {
    fn from(js_model: JsSettingFile) -> Self {
        SettingFile {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            Path: js_model.Path,
            Hash: js_model.Hash,
            Size: js_model.Size,
            rb_local_path: js_model.rb_local_path,
            rb_insync_hash: js_model.rb_insync_hash,
            rb_insync_local_usn: js_model.rb_insync_local_usn,
            rb_file_hash_dirty: js_model.rb_file_hash_dirty,
            rb_file_size_dirty: js_model.rb_file_size_dirty,
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub TableName: Option<String>,
    pub TargetUUID: Option<String>,
    pub CurrentID: Option<String>,
}

impl From<UuidIDMap> for JsUuidIDMap {
    fn from(model: UuidIDMap) -> Self {
        JsUuidIDMap {
            ID: model.ID,
            UUID: model.UUID,
            rb_data_status: model.rb_data_status,
            rb_local_data_status: model.rb_local_data_status,
            rb_local_deleted: model.rb_local_deleted,
            rb_local_synced: model.rb_local_synced,
            usn: model.usn,
            rb_local_usn: model.rb_local_usn,
            created_at: model.created_at,
            updated_at: model.updated_at,
            TableName: model.TableName,
            TargetUUID: model.TargetUUID,
            CurrentID: model.CurrentID,
        }
    }
}

impl From<JsUuidIDMap> for UuidIDMap {
    fn from(js_model: JsUuidIDMap) -> Self {
        UuidIDMap {
            ID: js_model.ID,
            UUID: js_model.UUID,
            rb_data_status: js_model.rb_data_status,
            rb_local_data_status: js_model.rb_local_data_status,
            rb_local_deleted: js_model.rb_local_deleted,
            rb_local_synced: js_model.rb_local_synced,
            usn: js_model.usn,
            rb_local_usn: js_model.rb_local_usn,
            created_at: js_model.created_at,
            updated_at: js_model.updated_at,
            TableName: js_model.TableName,
            TargetUUID: js_model.TargetUUID,
            CurrentID: js_model.CurrentID,
        }
    }
}
