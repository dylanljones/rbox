# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-05-01

from dataclasses import asdict, dataclass, field
from typing import Any, Dict, List, Optional

__all__ = [
    "Base",
    "AgentRegistry",
    "CloudAgentRegistry",
    "ContentActiveCensor",
    "ContentCue",
    "ContentFile",
    "DjmdActiveCensor",
    "DjmdAlbum",
    "DjmdArtist",
    "DjmdCategory",
    "DjmdColor",
    "DjmdContent",
    "DjmdCue",
    "DjmdDevice",
    "DjmdGenre",
    "DjmdHistory",
    "DjmdSongHistory",
    "DjmdHotCueBanklist",
    "DjmdSongHotCueBanklist",
    "DjmdKey",
    "DjmdLabel",
    "DjmdMenuItems",
    "DjmdMixerParam",
    "DjmdMyTag",
    "DjmdSongMyTag",
    "DjmdPlaylist",
    "DjmdSongPlaylist",
    "PlaylistTree",
    "DjmdProperty",
    "DjmdCloudProperty",
    "DjmdRecommendLike",
    "DjmdRelatedTracks",
    "DjmdSongRelatedTracks",
    "DjmdSampler",
    "DjmdSongSampler",
    "DjmdSongTagList",
    "DjmdSort",
    "HotCueBanklistCue",
    "ImageFile",
    "SettingFile",
    "UuidIDMap",
]


@dataclass
class Base:
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class StatsFull(Base):
    ID: str
    UUID: str
    rb_data_status: int = field(repr=False)
    rb_local_data_status: int = field(repr=False)
    rb_local_deleted: int = field(repr=False)
    rb_local_synced: int = field(repr=False)
    usn: Optional[int] = field(repr=False, default=None)
    rb_local_usn: Optional[int] = field(repr=False, default=None)
    created_at: Optional[str] = field(repr=False, default=None)
    updated_at: Optional[str] = field(repr=False, default=None)


@dataclass
class AgentRegistry(Base):
    registry_id: str
    created_at: Optional[str] = field(repr=False, default=None)
    updated_at: Optional[str] = field(repr=False, default=None)
    id_1: Optional[str] = None
    id_2: Optional[str] = None
    int_1: Optional[int] = None
    int_2: Optional[int] = None
    str_1: Optional[str] = None
    str_2: Optional[str] = None
    date_1: Optional[str] = None
    date_2: Optional[str] = None
    text_1: Optional[str] = None
    text_2: Optional[str] = None


@dataclass
class CloudAgentRegistry(StatsFull):
    int_1: Optional[int] = None
    int_2: Optional[int] = None
    str_1: Optional[str] = None
    str_2: Optional[str] = None
    date_1: Optional[str] = None
    date_2: Optional[str] = None
    text_1: Optional[str] = None
    text_2: Optional[str] = None


@dataclass
class ContentActiveCensor(StatsFull):
    ContentID: Optional[str] = None
    ActiveCensors: Optional[str] = None
    rb_activecensor_count: Optional[int] = None


@dataclass
class ContentCue(StatsFull):
    ContentID: Optional[str] = None
    Cues: Optional[str] = None
    rb_cue_count: Optional[int] = None


@dataclass
class ContentFile(StatsFull):
    ContentID: Optional[str] = None
    Path: Optional[str] = None
    Hash: Optional[str] = None
    Size: Optional[int] = None
    rb_local_path: Optional[str] = None
    rb_insync_hash: Optional[str] = None
    rb_insync_local_usn: Optional[int] = None
    rb_file_hash_dirty: Optional[int] = None
    rb_local_file_status: Optional[int] = None
    rb_in_progress: Optional[int] = None
    rb_process_type: Optional[int] = None
    rb_temp_path: Optional[str] = None
    rb_priority: Optional[int] = None
    rb_file_size_dirty: Optional[int] = None


@dataclass
class DjmdActiveCensor(StatsFull):
    ContentID: Optional[str] = None
    InMsec: Optional[int] = None
    OutMsec: Optional[int] = None
    Info: Optional[int] = None
    ParameterList: Optional[str] = None
    ContentUUID: Optional[str] = None


@dataclass
class DjmdAlbum(StatsFull):
    Name: Optional[str] = None
    AlbumArtistID: Optional[str] = None
    ImagePath: Optional[str] = None
    Compilation: Optional[int] = None
    SearchStr: Optional[str] = None


@dataclass
class DjmdArtist(StatsFull):
    Name: Optional[str] = None
    SearchStr: Optional[str] = None


@dataclass
class DjmdCategory(StatsFull):
    MenuItemID: Optional[str] = None
    Seq: Optional[int] = None
    Disable: Optional[int] = None
    InfoOrder: Optional[int] = None


@dataclass
class DjmdColor(StatsFull):
    ColorCode: Optional[str] = None
    SortKey: Optional[int] = None
    Commnt: Optional[str] = None


@dataclass
class DjmdContent(StatsFull):
    FolderPath: Optional[str] = None
    FileNameL: Optional[str] = None
    FileNameS: Optional[str] = None
    Title: Optional[str] = None
    ArtistID: Optional[str] = None
    AlbumID: Optional[str] = None
    GenreID: Optional[str] = None
    BPM: Optional[int] = None
    Length: Optional[int] = None
    TrackNo: Optional[int] = None
    BitRate: Optional[int] = None
    BitDepth: Optional[int] = None
    Commnt: Optional[str] = None
    FileType: Optional[int] = None
    Rating: Optional[int] = None
    ReleaseYear: Optional[int] = None
    RemixerID: Optional[str] = None
    LabelID: Optional[str] = None
    OrgArtistID: Optional[str] = None
    KeyID: Optional[str] = None
    StockDate: Optional[str] = None
    ColorID: Optional[str] = None
    DJPlayCount: Optional[int] = None
    ImagePath: Optional[str] = None
    MasterDBID: Optional[str] = None
    MasterSongID: Optional[str] = None
    AnalysisDataPath: Optional[str] = None
    SearchStr: Optional[str] = None
    FileSize: Optional[int] = None
    DiscNo: Optional[int] = None
    ComposerID: Optional[str] = None
    Subtitle: Optional[str] = None
    SampleRate: Optional[int] = None
    DisableQuantize: Optional[int] = None
    Analysed: Optional[int] = None
    ReleaseDate: Optional[str] = None
    DateCreated: Optional[str] = None
    ContentLink: Optional[int] = None
    Tag: Optional[str] = None
    ModifiedByRBM: Optional[str] = None
    HotCueAutoLoad: Optional[str] = None
    DeliveryControl: Optional[str] = None
    DeliveryComment: Optional[str] = None
    CueUpdated: Optional[str] = None
    AnalysisUpdated: Optional[str] = None
    TrackInfoUpdated: Optional[str] = None
    Lyricist: Optional[str] = None
    ISRC: Optional[str] = None
    SamplerTrackInfo: Optional[int] = None
    SamplerPlayOffset: Optional[int] = None
    SamplerGain: Optional[float] = None
    VideoAssociate: Optional[str] = None
    LyricStatus: Optional[int] = None
    ServiceID: Optional[int] = None
    OrgFolderPath: Optional[str] = None
    Reserved1: Optional[str] = None
    Reserved2: Optional[str] = None
    Reserved3: Optional[str] = None
    Reserved4: Optional[str] = None
    ExtInfo: Optional[str] = None
    rb_file_id: Optional[str] = None
    DeviceID: Optional[str] = None
    rb_LocalFolderPath: Optional[str] = None
    SrcID: Optional[str] = None
    SrcTitle: Optional[str] = None
    SrcArtistName: Optional[str] = None
    SrcAlbumName: Optional[str] = None
    SrcLength: Optional[int] = None


@dataclass
class DjmdCue(StatsFull):
    ContentID: Optional[str] = None
    InMsec: Optional[int] = None
    InFrame: Optional[int] = None
    InMpegFrame: Optional[int] = None
    InMpegAbs: Optional[int] = None
    OutMsec: Optional[int] = None
    OutFrame: Optional[int] = None
    OutMpegFrame: Optional[int] = None
    OutMpegAbs: Optional[int] = None
    Kind: Optional[int] = None
    Color: Optional[int] = None
    ColorTableIndex: Optional[int] = None
    ActiveLoop: Optional[int] = None
    Comment: Optional[str] = None
    BeatLoopSize: Optional[int] = None
    CueMicrosec: Optional[int] = None
    InPointSeekInfo: Optional[str] = None
    OutPointSeekInfo: Optional[str] = None
    ContentUUID: Optional[str] = None


@dataclass
class DjmdDevice(StatsFull):
    MasterDBID: Optional[str] = None
    Name: Optional[str] = None


@dataclass
class DjmdGenre(StatsFull):
    Name: Optional[str] = None


@dataclass
class DjmdHistory(StatsFull):
    Seq: Optional[int] = None
    Name: Optional[str] = None
    Attribute: Optional[int] = None
    ParentID: Optional[str] = None
    DateCreated: Optional[str] = None


@dataclass
class DjmdSongHistory(StatsFull):
    HistoryID: Optional[str] = None
    ContentID: Optional[str] = None
    TrackNo: Optional[int] = None


@dataclass
class DjmdHotCueBanklist(StatsFull):
    Seq: Optional[int] = None
    Name: Optional[str] = None
    ImagePath: Optional[str] = None
    Attribute: Optional[int] = None
    ParentID: Optional[str] = None


@dataclass
class DjmdSongHotCueBanklist(StatsFull):
    ContentID: Optional[str] = None
    TrackNo: Optional[int] = None
    CueID: Optional[str] = None
    InMsec: Optional[int] = None
    InFrame: Optional[int] = None
    InMpegFrame: Optional[int] = None
    InMpegAbs: Optional[int] = None
    OutMsec: Optional[int] = None
    OutFrame: Optional[int] = None
    OutMpegFrame: Optional[int] = None
    OutMpegAbs: Optional[int] = None
    Color: Optional[int] = None
    ColorTableIndex: Optional[int] = None
    ActiveLoop: Optional[int] = None
    Comment: Optional[str] = None
    BeatLoopSize: Optional[int] = None
    CueMicrosec: Optional[int] = None
    InPointSeekInfo: Optional[str] = None
    OutPointSeekInfo: Optional[str] = None
    HotCueBanklistUUID: Optional[str] = None


@dataclass
class DjmdKey(StatsFull):
    ScaleName: Optional[str] = None
    Seq: Optional[int] = None


@dataclass
class DjmdLabel(StatsFull):
    Name: Optional[str] = None


@dataclass
class DjmdMenuItems(StatsFull):
    Class: Optional[int] = None
    Name: Optional[str] = None


@dataclass
class DjmdMixerParam(StatsFull):
    ContentID: Optional[str] = None
    GainHigh: Optional[int] = None
    GainLow: Optional[int] = None
    PeakHigh: Optional[int] = None
    PeakLow: Optional[int] = None


@dataclass
class DjmdMyTag(StatsFull):
    Seq: Optional[int] = None
    Name: Optional[str] = None
    Attribute: Optional[int] = None
    ParentID: Optional[str] = None


@dataclass
class DjmdSongMyTag(StatsFull):
    MyTagID: Optional[str] = None
    ContentID: Optional[str] = None
    TrackNo: Optional[int] = None


@dataclass
class DjmdPlaylist(StatsFull):
    Seq: Optional[int] = None
    Name: Optional[str] = None
    ImagePath: Optional[str] = None
    Attribute: Optional[int] = None
    ParentID: Optional[str] = None
    SmartList: Optional[str] = None


@dataclass
class PlaylistTree(StatsFull):
    Seq: Optional[int] = None
    Name: Optional[str] = None
    ImagePath: Optional[str] = None
    Attribute: Optional[int] = None
    ParentID: Optional[str] = None
    SmartList: Optional[str] = None

    Children: List["PlaylistTree"] = field(default_factory=list)


@dataclass
class DjmdSongPlaylist(StatsFull):
    PlaylistID: Optional[str] = None
    ContentID: Optional[str] = None
    TrackNo: Optional[int] = None


@dataclass
class DjmdProperty(Base):
    DBID: str
    DBVersion: Optional[str] = None
    BaseDBDrive: Optional[str] = None
    CurrentDBDrive: Optional[str] = None
    DeviceID: Optional[str] = None
    Reserved1: Optional[str] = None
    Reserved2: Optional[str] = None
    Reserved3: Optional[str] = None
    Reserved4: Optional[str] = None
    Reserved5: Optional[str] = None
    created_at: Optional[str] = field(repr=False, default=None)
    updated_at: Optional[str] = field(repr=False, default=None)


@dataclass
class DjmdCloudProperty(StatsFull):
    Reserved1: Optional[str] = None
    Reserved2: Optional[str] = None
    Reserved3: Optional[str] = None
    Reserved4: Optional[str] = None
    Reserved5: Optional[str] = None


@dataclass
class DjmdRecommendLike(StatsFull):
    ContentID1: Optional[str] = None
    ContentID2: Optional[str] = None
    LikeRate: Optional[int] = None
    DataCreatedH: Optional[int] = None
    DataCreatedL: Optional[int] = None


@dataclass
class DjmdRelatedTracks(StatsFull):
    Seq: Optional[int] = None
    Name: Optional[str] = None
    Attribute: Optional[int] = None
    ParentID: Optional[str] = None
    Criteria: Optional[str] = None


@dataclass
class DjmdSongRelatedTracks(StatsFull):
    RelatedTracksID: Optional[str] = None
    ContentID: Optional[str] = None
    TrackNo: Optional[int] = None


@dataclass
class DjmdSampler(StatsFull):
    Seq: Optional[int] = None
    Name: Optional[str] = None
    Attribute: Optional[int] = None
    ParentID: Optional[str] = None


@dataclass
class DjmdSongSampler(StatsFull):
    SamplerID: Optional[str] = None
    ContentID: Optional[str] = None
    TrackNo: Optional[int] = None


@dataclass
class DjmdSongTagList(StatsFull):
    ContentID: Optional[str] = None
    TrackNo: Optional[int] = None


@dataclass
class DjmdSort(StatsFull):
    MenuItemID: Optional[str] = None
    Seq: Optional[int] = None
    Disable: Optional[int] = None


@dataclass
class HotCueBanklistCue(StatsFull):
    HotCueBanklistID: Optional[str] = None
    Cues: Optional[str] = None
    rb_cue_count: Optional[int] = None


@dataclass
class ImageFile(StatsFull):
    TableName: Optional[str] = None
    TargetUUID: Optional[str] = None
    TargetID: Optional[str] = None
    Path: Optional[str] = None
    Hash: Optional[str] = None
    Size: Optional[int] = None
    rb_local_path: Optional[str] = None
    rb_insync_hash: Optional[str] = None
    rb_insync_local_usn: Optional[int] = None
    rb_file_hash_dirty: Optional[int] = None
    rb_local_file_status: Optional[int] = None
    rb_in_progress: Optional[int] = None
    rb_process_type: Optional[int] = None
    rb_temp_path: Optional[str] = None
    rb_priority: Optional[int] = None
    rb_file_size_dirty: Optional[int] = None


@dataclass
class SettingFile(StatsFull):
    Path: Optional[str] = None
    Hash: Optional[str] = None
    Size: Optional[int] = None
    rb_local_path: Optional[str] = None
    rb_insync_hash: Optional[str] = None
    rb_insync_local_usn: Optional[int] = None
    rb_file_hash_dirty: Optional[int] = None
    rb_file_size_dirty: Optional[int] = None


@dataclass
class UuidIDMap(StatsFull):
    TableName: Optional[str] = None
    TargetUUID: Optional[str] = None
    CurrentID: Optional[str] = None
