// Author: Dylan Jones
// Date:   2025-05-01

mod conn;
mod database;
pub mod enums;
pub mod models;
pub mod playlist_xml;
pub mod schema;
pub mod smart_list;

pub use conn::{create_connection_pool, establish_connection};
pub use database::MasterDb;
pub use playlist_xml::MasterPlaylistXml;
pub use smart_list::{Condition, LogicalOperator, Operator, Property, SmartList};

pub use enums::*;
pub use models::*;

pub mod dsl {
    use super::schema;

    pub use schema::agentRegistry::dsl::agentRegistry;
    pub use schema::cloudAgentRegistry::dsl::cloudAgentRegistry;
    pub use schema::contentActiveCensor::dsl::contentActiveCensor;
    pub use schema::contentCue::dsl::contentCue;
    pub use schema::contentFile::dsl::contentFile;
    pub use schema::djmdActiveCensor::dsl::djmdActiveCensor;
    pub use schema::djmdAlbum::dsl::djmdAlbum;
    pub use schema::djmdArtist::dsl::djmdArtist;
    pub use schema::djmdCategory::dsl::djmdCategory;
    pub use schema::djmdCloudProperty::dsl::djmdCloudProperty;
    pub use schema::djmdColor::dsl::djmdColor;
    pub use schema::djmdContent::dsl::djmdContent;
    pub use schema::djmdCue::dsl::djmdCue;
    pub use schema::djmdDevice::dsl::djmdDevice;
    pub use schema::djmdGenre::dsl::djmdGenre;
    pub use schema::djmdHistory::dsl::djmdHistory;
    pub use schema::djmdHotCueBanklist::dsl::djmdHotCueBanklist;
    pub use schema::djmdKey::dsl::djmdKey;
    pub use schema::djmdLabel::dsl::djmdLabel;
    pub use schema::djmdMenuItems::dsl::djmdMenuItems;
    pub use schema::djmdMixerParam::dsl::djmdMixerParam;
    pub use schema::djmdMyTag::dsl::djmdMyTag;
    pub use schema::djmdPlaylist::dsl::djmdPlaylist;
    pub use schema::djmdProperty::dsl::djmdProperty;
    pub use schema::djmdRecommendLike::dsl::djmdRecommendLike;
    pub use schema::djmdRelatedTracks::dsl::djmdRelatedTracks;
    pub use schema::djmdSampler::dsl::djmdSampler;
    pub use schema::djmdSongHistory::dsl::djmdSongHistory;
    pub use schema::djmdSongHotCueBanklist::dsl::djmdSongHotCueBanklist;
    pub use schema::djmdSongMyTag::dsl::djmdSongMyTag;
    pub use schema::djmdSongPlaylist::dsl::djmdSongPlaylist;
    pub use schema::djmdSongRelatedTracks::dsl::djmdSongRelatedTracks;
    pub use schema::djmdSongSampler::dsl::djmdSongSampler;
    pub use schema::djmdSongTagList::dsl::djmdSongTagList;
    pub use schema::djmdSort::dsl::djmdSort;
    pub use schema::hotCueBanklistCue::dsl::hotCueBanklistCue;
    pub use schema::imageFile::dsl::imageFile;
    pub use schema::settingFile::dsl::settingFile;
    pub use schema::uuidIDMap::dsl::uuidIDMap;
}
