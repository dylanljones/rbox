// Author: Dylan Jones
// Date:   2025-05-01

pub mod database;
pub mod enums;
pub mod models;
pub mod playlist_xml;
mod random_id;
pub mod schema;
mod util;

// Core
pub use database::MasterDb;
pub use enums::{AnalysisUpdated, Analyzed, FileType, PlaylistType};
pub use playlist_xml::MasterPlaylistXml;
pub use util::{format_datetime, parse_datetime, RekordboxDateString};
// Models
pub use models::AgentRegistry; // agentRegistry
pub use models::CloudAgentRegistry; // cloudAgentRegistry
pub use models::ContentActiveCensor; // contentActiveCensor
pub use models::ContentCue; // contentCue
pub use models::ContentFile; // contentFile
pub use models::DjmdActiveCensor; // djmdActiveCensor
pub use models::DjmdAlbum; // djmdAlbum
pub use models::DjmdArtist; // djmdArtist
pub use models::DjmdCategory; // djmdCategory
pub use models::DjmdCloudProperty; // djmdCloudProperty
pub use models::DjmdColor; // djmdColor
pub use models::DjmdContent; // djmdContent
pub use models::DjmdCue; // djmdCue
pub use models::DjmdDevice; // djmdDevice
pub use models::DjmdGenre; // djmdGenre
pub use models::DjmdHistory; // djmdHistory
pub use models::DjmdHotCueBanklist; // djmdHotCueBanklist
pub use models::DjmdKey; // djmdKey
pub use models::DjmdLabel; // djmdLabel
pub use models::DjmdMenuItems; // djmdMenuItems
pub use models::DjmdMixerParam; // djmdMixerParam
pub use models::DjmdMyTag; // djmdMyTag
pub use models::DjmdPlaylist; // djmdPlaylist
pub use models::DjmdPlaylistTreeItem; // djmdPlaylist
pub use models::DjmdProperty; // djmdProperty
pub use models::DjmdRecommendLike; // djmdRecommendLike
pub use models::DjmdRelatedTracks; // djmdRelatedTracks
pub use models::DjmdSampler; // djmdSampler
pub use models::DjmdSongHistory; // djmdSongHistory
pub use models::DjmdSongHotCueBanklist; // djmdSongHotCueBanklist
pub use models::DjmdSongMyTag; // djmdSongMyTag
pub use models::DjmdSongPlaylist; // djmdSongPlaylist
pub use models::DjmdSongRelatedTracks; // djmdSongRelatedTracks
pub use models::DjmdSongSampler; // djmdSongSampler
pub use models::DjmdSongTagList; // djmdSongTagList
pub use models::DjmdSort; // djmdSort
pub use models::HotCueBanklistCue; // hotCueBanklistCue
pub use models::ImageFile; // imageFile
pub use models::SettingFile; // settingFile
pub use models::UuidIDMap; // uuidIDMap

// Schema
pub use schema::agentRegistry::dsl::agentRegistry; // agentRegistry
pub use schema::cloudAgentRegistry::dsl::cloudAgentRegistry; // cloudAgentRegistry
pub use schema::contentActiveCensor::dsl::contentActiveCensor; // contentActiveCensor
pub use schema::contentCue::dsl::contentCue; // contentCue
pub use schema::contentFile::dsl::contentFile; // contentFile
pub use schema::djmdActiveCensor::dsl::djmdActiveCensor; // djmdActiveCensor
pub use schema::djmdAlbum::dsl::djmdAlbum; // djmdAlbum
pub use schema::djmdArtist::dsl::djmdArtist; // djmdArtist
pub use schema::djmdCategory::dsl::djmdCategory; // djmdCategory
pub use schema::djmdCloudProperty::dsl::djmdCloudProperty; // djmdCloudProperty
pub use schema::djmdColor::dsl::djmdColor; // djmdColor
pub use schema::djmdContent::dsl::djmdContent; // djmdContent
pub use schema::djmdCue::dsl::djmdCue; // djmdCue
pub use schema::djmdDevice::dsl::djmdDevice; // djmdDevice
pub use schema::djmdGenre::dsl::djmdGenre; // djmdGenre
pub use schema::djmdHistory::dsl::djmdHistory; // djmdHistory
pub use schema::djmdHotCueBanklist::dsl::djmdHotCueBanklist; // djmdHotCueBanklist
pub use schema::djmdKey::dsl::djmdKey; // djmdKey
pub use schema::djmdLabel::dsl::djmdLabel; // djmdLabel
pub use schema::djmdMenuItems::dsl::djmdMenuItems; // djmdMenuItems
pub use schema::djmdMixerParam::dsl::djmdMixerParam; // djmdMixerParam
pub use schema::djmdMyTag::dsl::djmdMyTag; // djmdMyTag
pub use schema::djmdPlaylist::dsl::djmdPlaylist; // djmdPlaylist
pub use schema::djmdProperty::dsl::djmdProperty; // djmdProperty
pub use schema::djmdRecommendLike::dsl::djmdRecommendLike; // djmdRecommendLike
pub use schema::djmdRelatedTracks::dsl::djmdRelatedTracks; // djmdRelatedTracks
pub use schema::djmdSampler::dsl::djmdSampler; // djmdSampler
pub use schema::djmdSongHistory::dsl::djmdSongHistory; // djmdSongHistory
pub use schema::djmdSongHotCueBanklist::dsl::djmdSongHotCueBanklist; // djmdSongHotCueBanklist
pub use schema::djmdSongMyTag::dsl::djmdSongMyTag; // djmdSongMyTag
pub use schema::djmdSongPlaylist::dsl::djmdSongPlaylist; // djmdSongPlaylist
pub use schema::djmdSongRelatedTracks::dsl::djmdSongRelatedTracks; // djmdSongRelatedTracks
pub use schema::djmdSongSampler::dsl::djmdSongSampler; // djmdSongSampler
pub use schema::djmdSongTagList::dsl::djmdSongTagList; // djmdSongTagList
pub use schema::djmdSort::dsl::djmdSort; // djmdSort
pub use schema::hotCueBanklistCue::dsl::hotCueBanklistCue; // hotCueBanklistCue
pub use schema::imageFile::dsl::imageFile; // imageFile
pub use schema::settingFile::dsl::settingFile; // settingFile
pub use schema::uuidIDMap::dsl::uuidIDMap; // uuidIDMap
