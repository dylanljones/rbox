# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-05-01

from datetime import datetime
from typing import Any, Dict, Iterable, List, Optional

Obj = Dict[str, Any]

def is_rekordbox_running() -> bool:
    """Check if Rekordbox is running."""
    ...

class _Base:
    def __len__(self) -> int:
        """Return the number of items."""
        ...

    def __iter__(self) -> Iterable[str]:
        """Return an iterator over the keys."""
        ...

    def __getitem__(self, key: str) -> Any:
        """Get an item by key."""
        ...

    def __setitem__(self, key: str, value: Any) -> None:
        """Set an item by key."""
        ...

    def to_dict(self) -> Dict[str, Any]:
        """Convert the object to a dictionary."""
        ...

class _StatsFull:
    ID: str
    UUID: str
    rb_data_status: int
    rb_local_data_status: int
    rb_local_deleted: int
    rb_local_synced: int
    created_at: datetime
    updated_at: datetime
    usn: Optional[int]
    rb_local_usn: Optional[int]

class AgentRegistry(_Base):
    registry_id: str
    created_at: str
    updated_at: str
    id_1: Optional[str]
    id_2: Optional[str]
    int_1: Optional[int]
    int_2: Optional[int]
    str_1: Optional[str]
    str_2: Optional[str]
    date_1: Optional[str]
    date_2: Optional[str]
    text_1: Optional[str]
    text_2: Optional[str]

class CloudAgentRegistry(_StatsFull, _Base):
    int_1: Optional[int]
    int_2: Optional[int]
    str_1: Optional[str]
    str_2: Optional[str]
    date_1: Optional[str]
    date_2: Optional[str]
    text_1: Optional[str]
    text_2: Optional[str]

class ContentActiveCensor(_StatsFull, _Base):
    ContentID: Optional[str]
    ActiveCensors: Optional[str]
    rb_activecensor_count: Optional[int]

class ContentCue(_StatsFull, _Base):
    ContentID: Optional[str]
    Cues: Optional[str]
    rb_cue_count: Optional[int]

class ContentFile(_StatsFull, _Base):
    ContentID: Optional[str]
    Path: Optional[str]
    Hash: Optional[str]
    Size: Optional[int]
    rb_local_path: Optional[str]
    rb_insync_hash: Optional[str]
    rb_insync_local_usn: Optional[int]
    rb_file_hash_dirty: Optional[int]
    rb_local_file_status: Optional[int]
    rb_in_progress: Optional[int]
    rb_process_type: Optional[int]
    rb_temp_path: Optional[str]
    rb_priority: Optional[int]
    rb_file_size_dirty: Optional[int]

class DjmdActiveCensor(_StatsFull, _Base):
    ContentID: Optional[str]
    InMsec: Optional[int]
    OutMsec: Optional[int]
    Info: Optional[int]
    ParameterList: Optional[str]
    ContentUUID: Optional[str]

class DjmdAlbum(_StatsFull, _Base):
    Name: Optional[str]
    AlbumArtistID: Optional[str]
    ImagePath: Optional[str]
    Compilation: Optional[int]
    SearchStr: Optional[str]

class DjmdArtist(_StatsFull, _Base):
    Name: Optional[str]
    SearchStr: Optional[str]

class DjmdCategory(_StatsFull, _Base):
    MenuItemID: Optional[str]
    Seq: Optional[int]
    Disable: Optional[int]
    InfoOrder: Optional[int]

class DjmdColor(_StatsFull, _Base):
    ColorCode: Optional[str]
    SortKey: Optional[int]
    Commnt: Optional[str]

class DjmdContent(_StatsFull, _Base):
    FolderPath: Optional[str]
    FileNameL: Optional[str]
    FileNameS: Optional[str]
    Title: Optional[str]
    ArtistID: Optional[str]
    AlbumID: Optional[str]
    GenreID: Optional[str]
    BPM: Optional[int]
    Length: Optional[int]
    TrackNo: Optional[int]
    BitRate: Optional[int]
    BitDepth: Optional[int]
    Commnt: Optional[str]
    FileType: Optional[int]
    Rating: Optional[int]
    ReleaseYear: Optional[int]
    RemixerID: Optional[str]
    LabelID: Optional[str]
    OrgArtistID: Optional[str]
    KeyID: Optional[str]
    StockDate: Optional[str]
    ColorID: Optional[str]
    DJPlayCount: Optional[int]
    ImagePath: Optional[str]
    MasterDBID: Optional[str]
    MasterSongID: Optional[str]
    AnalysisDataPath: Optional[str]
    SearchStr: Optional[str]
    FileSize: Optional[int]
    DiscNo: Optional[int]
    ComposerID: Optional[str]
    Subtitle: Optional[str]
    SampleRate: Optional[int]
    DisableQuantize: Optional[int]
    Analysed: Optional[int]
    ReleaseDate: Optional[str]
    DateCreated: Optional[str]
    ContentLink: Optional[int]
    Tag: Optional[str]
    ModifiedByRBM: Optional[str]
    HotCueAutoLoad: Optional[str]
    DeliveryControl: Optional[str]
    DeliveryComment: Optional[str]
    CueUpdated: Optional[str]
    AnalysisUpdated: Optional[str]
    TrackInfoUpdated: Optional[str]
    Lyricist: Optional[str]
    ISRC: Optional[str]
    SamplerTrackInfo: Optional[int]
    SamplerPlayOffset: Optional[int]
    SamplerGain: Optional[float]
    VideoAssociate: Optional[str]
    LyricStatus: Optional[int]
    ServiceID: Optional[int]
    OrgFolderPath: Optional[str]
    Reserved1: Optional[str]
    Reserved2: Optional[str]
    Reserved3: Optional[str]
    Reserved4: Optional[str]
    ExtInfo: Optional[str]
    rb_file_id: Optional[str]
    DeviceID: Optional[str]
    rb_LocalFolderPath: Optional[str]
    SrcID: Optional[str]
    SrcTitle: Optional[str]
    SrcArtistName: Optional[str]
    SrcAlbumName: Optional[str]
    SrcLength: Optional[int]

class DjmdCue(_StatsFull, _Base):
    ContentID: Optional[str]
    InMsec: Optional[int]
    InFrame: Optional[int]
    InMpegFrame: Optional[int]
    InMpegAbs: Optional[int]
    OutMsec: Optional[int]
    OutFrame: Optional[int]
    OutMpegFrame: Optional[int]
    OutMpegAbs: Optional[int]
    Kind: Optional[int]
    Color: Optional[int]
    ColorTableIndex: Optional[int]
    ActiveLoop: Optional[int]
    Comment: Optional[str]
    BeatLoopSize: Optional[int]
    CueMicrosec: Optional[int]
    InPointSeekInfo: Optional[str]
    OutPointSeekInfo: Optional[str]
    ContentUUID: Optional[str]

class DjmdDevice(_StatsFull, _Base):
    MasterDBID: Optional[str]
    Name: Optional[str]

class DjmdGenre(_StatsFull, _Base):
    Name: Optional[str]

class DjmdHistory(_StatsFull, _Base):
    Seq: Optional[int]
    Name: Optional[str]
    Attribute: Optional[int]
    ParentID: Optional[str]
    DateCreated: Optional[str]

class DjmdSongHistory(_StatsFull, _Base):
    HistoryID: Optional[str]
    ContentID: Optional[str]
    TrackNo: Optional[int]

class DjmdHotCueBanklist(_StatsFull, _Base):
    Seq: Optional[int]
    Name: Optional[str]
    ImagePath: Optional[str]
    Attribute: Optional[int]
    ParentID: Optional[str]

class DjmdSongHotCueBanklist(_StatsFull, _Base):
    ContentID: Optional[str]
    TrackNo: Optional[int]
    CueID: Optional[str]
    InMsec: Optional[int]
    InFrame: Optional[int]
    InMpegFrame: Optional[int]
    InMpegAbs: Optional[int]
    OutMsec: Optional[int]
    OutFrame: Optional[int]
    OutMpegFrame: Optional[int]
    OutMpegAbs: Optional[int]
    Color: Optional[int]
    ColorTableIndex: Optional[int]
    ActiveLoop: Optional[int]
    Comment: Optional[str]
    BeatLoopSize: Optional[int]
    CueMicrosec: Optional[int]
    InPointSeekInfo: Optional[str]
    OutPointSeekInfo: Optional[str]
    HotCueBanklistUUID: Optional[str]

class DjmdKey(_StatsFull, _Base):
    ScaleName: Optional[str]
    Seq: Optional[int]

class DjmdLabel(_StatsFull, _Base):
    Name: Optional[str]

class DjmdMenuItems(_StatsFull, _Base):
    Class: Optional[int]
    Name: Optional[str]

class DjmdMixerParam(_StatsFull, _Base):
    ContentID: Optional[str]
    GainHigh: Optional[int]
    GainLow: Optional[int]
    PeakHigh: Optional[int]
    PeakLow: Optional[int]

class DjmdMyTag(_StatsFull, _Base):
    Seq: Optional[int]
    Name: Optional[str]
    Attribute: Optional[int]
    ParentID: Optional[str]

class DjmdSongMyTag(_StatsFull, _Base):
    MyTagID: Optional[str]
    ContentID: Optional[str]
    TrackNo: Optional[int]

class DjmdPlaylist(_StatsFull, _Base):
    Seq: Optional[int]
    Name: Optional[str]
    ImagePath: Optional[str]
    Attribute: Optional[int]
    ParentID: Optional[str]
    SmartList: Optional[str]

class DjmdPlaylistTreeItem(_StatsFull, _Base):
    Seq: Optional[int]
    Name: Optional[str]
    ImagePath: Optional[str]
    Attribute: Optional[int]
    ParentID: Optional[str]
    SmartList: Optional[str]

    Children: List["DjmdPlaylistTreeItem"]

class DjmdSongPlaylist(_StatsFull, _Base):
    PlaylistID: Optional[str]
    ContentID: Optional[str]
    TrackNo: Optional[int]

class DjmdProperty(_Base):
    DBID: str
    created_at: str
    updated_at: str
    DBVersion: Optional[str]
    BaseDBDrive: Optional[str]
    CurrentDBDrive: Optional[str]
    DeviceID: Optional[str]
    Reserved1: Optional[str]
    Reserved2: Optional[str]
    Reserved3: Optional[str]
    Reserved4: Optional[str]
    Reserved5: Optional[str]

class DjmdCloudProperty(_StatsFull, _Base):
    Reserved1: Optional[str]
    Reserved2: Optional[str]
    Reserved3: Optional[str]
    Reserved4: Optional[str]
    Reserved5: Optional[str]

class DjmdRecommendLike(_StatsFull, _Base):
    ContentID1: Optional[str]
    ContentID2: Optional[str]
    LikeRate: Optional[int]
    DataCreatedH: Optional[int]
    DataCreatedL: Optional[int]

class DjmdRelatedTracks(_StatsFull, _Base):
    Seq: Optional[int]
    Name: Optional[str]
    Attribute: Optional[int]
    ParentID: Optional[str]
    Criteria: Optional[str]

class DjmdSongRelatedTracks(_StatsFull, _Base):
    RelatedTracksID: Optional[str]
    ContentID: Optional[str]
    TrackNo: Optional[int]

class DjmdSampler(_StatsFull, _Base):
    Seq: Optional[int]
    Name: Optional[str]
    Attribute: Optional[int]
    ParentID: Optional[str]

class DjmdSongSampler(_StatsFull, _Base):
    SamplerID: Optional[str]
    ContentID: Optional[str]
    TrackNo: Optional[int]

class DjmdSongTagList(_StatsFull, _Base):
    ContentID: Optional[str]
    TrackNo: Optional[int]

class DjmdSort(_StatsFull, _Base):
    MenuItemID: Optional[str]
    Seq: Optional[int]
    Disable: Optional[int]

class HotCueBanklistCue(_StatsFull, _Base):
    HotCueBanklistID: Optional[str]
    Cues: Optional[str]
    rb_cue_count: Optional[int]

class ImageFile(_StatsFull, _Base):
    TableName: Optional[str]
    TargetUUID: Optional[str]
    TargetID: Optional[str]
    Path: Optional[str]
    Hash: Optional[str]
    Size: Optional[int]
    rb_local_path: Optional[str]
    rb_insync_hash: Optional[str]
    rb_insync_local_usn: Optional[int]
    rb_file_hash_dirty: Optional[int]
    rb_local_file_status: Optional[int]
    rb_in_progress: Optional[int]
    rb_process_type: Optional[int]
    rb_temp_path: Optional[str]
    rb_priority: Optional[int]
    rb_file_size_dirty: Optional[int]

class SettingFile(_StatsFull, _Base):
    Path: Optional[str]
    Hash: Optional[str]
    Size: Optional[int]
    rb_local_path: Optional[str]
    rb_insync_hash: Optional[str]
    rb_insync_local_usn: Optional[int]
    rb_file_hash_dirty: Optional[int]
    rb_file_size_dirty: Optional[int]

class UuidIDMap(_StatsFull, _Base):
    TableName: Optional[str]
    TargetUUID: Optional[str]
    CurrentID: Optional[str]

class PyMasterDb:
    def __init__(self, path: str) -> None: ...
    @classmethod
    def open(cls) -> "PyMasterDb": ...
    def set_unsafe_writes(self, bool: bool) -> None: ...
    def get_agent_registry(self) -> List[AgentRegistry]: ...
    def get_agent_registry_by_id(self, registry_id: str) -> Optional[AgentRegistry]: ...
    def get_local_usn(self) -> int: ...
    def get_cloud_agent_registry(self) -> List[CloudAgentRegistry]: ...
    def get_cloud_agent_registry_by_id(self, registry_id: str) -> Optional[CloudAgentRegistry]: ...
    def get_content_active_censor(self) -> List[ContentActiveCensor]: ...
    def get_content_active_censor_by_id(self, censor_id: str) -> Optional[ContentActiveCensor]: ...
    def get_content_cue(self) -> List[ContentCue]: ...
    def get_content_cue_by_id(self, cue_id: str) -> Optional[ContentCue]: ...
    def get_content_file(self) -> List[ContentFile]: ...
    def get_content_file_by_id(self, file_id: str) -> Optional[ContentFile]: ...
    def get_active_censor(self) -> List[DjmdActiveCensor]: ...
    def get_active_censor_by_id(self, censor_id: str) -> Optional[DjmdActiveCensor]: ...
    def get_album(self) -> List[DjmdAlbum]: ...
    def get_album_by_id(self, album_id: str) -> Optional[DjmdAlbum]: ...
    def get_album_by_name(self, album_name: str) -> Optional[DjmdAlbum]: ...
    def insert_album(
        self, name: str, artist_id: str = None, image_path: str = None, compilation: int = None
    ) -> DjmdAlbum: ...
    def update_album(self, album: DjmdAlbum) -> DjmdAlbum: ...
    def delete_album(self, album_id: str) -> None: ...
    def get_artist(self) -> List[DjmdArtist]: ...
    def get_artist_by_id(self, artist_id: str) -> Optional[DjmdArtist]: ...
    def get_artist_by_name(self, artist_name: str) -> Optional[DjmdArtist]: ...
    def insert_artist(self, name: str) -> DjmdArtist: ...
    def update_artist(self, artist: DjmdArtist) -> DjmdArtist: ...
    def delete_artist(self, artist_id: str) -> None: ...
    def get_category(self) -> List[DjmdCategory]: ...
    def get_category_by_id(self, category_id: str) -> Optional[DjmdCategory]: ...
    def get_color(self) -> List[DjmdColor]: ...
    def get_color_by_id(self, color_id: str) -> Optional[DjmdColor]: ...
    def get_content(self) -> List[DjmdContent]: ...
    def get_content_by_id(self, content_id: str) -> Optional[DjmdContent]: ...
    def get_content_by_path(self, path: str) -> Optional[DjmdContent]: ...
    def get_content_anlz_dir(self, content_id: str) -> str: ...
    def get_content_anlz_paths(self, content_id: str) -> Dict[str, str]: ...
    def insert_content(self, path: str) -> DjmdContent: ...
    def update_content(self, content: DjmdContent) -> None: ...
    def update_content_album(self, content_id: str, name: str) -> None: ...
    def update_content_artist(self, content_id: str, name: str) -> None: ...
    def update_content_remixer(self, content_id: str, name: str) -> None: ...
    def update_content_original_artist(self, content_id: str, name: str) -> None: ...
    def update_content_composer(self, content_id: str, name: str) -> None: ...
    def update_content_genre(self, content_id: str, name: str) -> None: ...
    def update_content_label(self, content_id: str, name: str) -> None: ...
    def update_content_key(self, content_id: str, name: str) -> None: ...
    # def delete_content(self, content_id: str) -> None: ...
    def get_cue(self) -> List[DjmdCue]: ...
    def get_cue_by_id(self, cue_id: str) -> Optional[DjmdCue]: ...
    def get_device(self) -> List[DjmdDevice]: ...
    def get_device_by_id(self, device_id: str) -> Optional[DjmdDevice]: ...
    def get_genre(self) -> List[DjmdGenre]: ...
    def get_genre_by_id(self, genre_id: str) -> Optional[DjmdGenre]: ...
    def get_genre_by_name(self, genre_name: str) -> Optional[DjmdGenre]: ...
    def insert_genre(self, name: str) -> DjmdGenre: ...
    def update_genre(self, genre: DjmdGenre) -> DjmdGenre: ...
    def delete_genre(self, genre_id: str) -> None: ...
    def get_history(self) -> List[DjmdHistory]: ...
    def get_history_by_id(self, history_id: str) -> Optional[DjmdHistory]: ...
    def get_history_songs(self, history_id: str) -> List[DjmdSongHistory]: ...
    def get_history_contents(self, history_id: str) -> List[DjmdContent]: ...
    def get_hot_cue_banklist(self) -> List[DjmdHotCueBanklist]: ...
    def get_hot_cue_banklist_by_id(self, banklist_id: str) -> Optional[DjmdHotCueBanklist]: ...
    def get_hot_cue_banklist_children(self, banklist_id: str) -> List[DjmdHotCueBanklist]: ...
    def get_hot_cue_banklist_songs(self, banklist_id: str) -> List[DjmdSongHotCueBanklist]: ...
    def get_hot_cue_banklist_contents(self, banklist_id: str) -> List[DjmdContent]: ...
    def get_hot_cue_banklist_cues(self, banklist_id: str) -> List[HotCueBanklistCue]: ...
    def get_key(self) -> List[DjmdKey]: ...
    def get_key_by_id(self, key_id: str) -> Optional[DjmdKey]: ...
    def get_key_by_name(self, key_name: str) -> Optional[DjmdKey]: ...
    def insert_key(self, name: str) -> DjmdKey: ...
    def update_key(self, key: DjmdKey) -> DjmdKey: ...
    def delete_key(self, key_id: str) -> None: ...
    def get_label(self) -> List[DjmdLabel]: ...
    def get_label_by_id(self, label_id: str) -> Optional[DjmdLabel]: ...
    def get_label_by_name(self, label_name: str) -> Optional[DjmdLabel]: ...
    def insert_label(self, name: str) -> DjmdLabel: ...
    def update_label(self, label: DjmdLabel) -> DjmdLabel: ...
    def delete_label(self, label_id: str) -> None: ...
    def get_menu_item(self) -> List[DjmdMenuItems]: ...
    def get_menu_item_by_id(self, item_id: str) -> Optional[DjmdMenuItems]: ...
    def get_mixer_param(self) -> List[DjmdMixerParam]: ...
    def get_mixer_param_by_id(self, param_id: str) -> Optional[DjmdMixerParam]: ...
    def get_my_tag(self) -> List[DjmdMyTag]: ...
    def get_my_tag_children(self, tag_id: str) -> List[DjmdMyTag]: ...
    def get_my_tag_by_id(self, tag_id: str) -> Optional[DjmdMyTag]: ...
    def get_my_tag_songs(self, tag_id: str) -> List[DjmdSongMyTag]: ...
    def get_my_tag_contents(self, tag_id: str) -> List[DjmdContent]: ...
    def get_playlist(self) -> List[DjmdPlaylist]: ...
    def get_playlist_tree(self) -> List[DjmdPlaylistTreeItem]: ...
    def get_playlist_children(self, playlist_id: str) -> List[DjmdPlaylist]: ...
    def get_playlist_by_id(self, playlist_id: str) -> Optional[DjmdPlaylist]: ...
    def get_playlist_by_path(self, path: List[str]) -> Optional[DjmdPlaylist]: ...
    def get_playlist_songs(self, playlist_id: str) -> List[DjmdSongPlaylist]: ...
    def get_playlist_contents(self, playlist_id: str) -> List[DjmdContent]: ...
    def get_playlist_song_by_id(self, song_id: str) -> Optional[DjmdSongPlaylist]: ...
    def insert_playlist(
        self,
        name: str,
        attribute: int,
        parent_id: str = None,
        seq: int = None,
        image_path: str = None,
        smart_list: str = None,
    ) -> DjmdPlaylist: ...
    def rename_playlist(self, playlist_id: str, name: str) -> DjmdPlaylist: ...
    def move_playlist(
        self, playlist_id: str, seq: int = None, parent_id: str = None
    ) -> DjmdPlaylist: ...
    def delete_playlist(self, playlist_id: str) -> None: ...
    def insert_playlist_song(
        self, playlist_id: str, content_id: str, seq: int = None
    ) -> DjmdSongPlaylist: ...
    def move_playlist_song(self, song_id: str, seq: int) -> DjmdSongPlaylist: ...
    def delete_playlist_song(self, song_id: str) -> None: ...
    def get_property(self) -> List[DjmdProperty]: ...
    def get_property_by_id(self, property_id: str) -> Optional[DjmdProperty]: ...
    def get_cloud_property(self) -> List[DjmdCloudProperty]: ...
    def get_cloud_property_by_id(self, property_id: str) -> Optional[DjmdCloudProperty]: ...
    def get_recommend_like(self) -> List[DjmdRecommendLike]: ...
    def get_recommend_like_by_id(self, recommend_id: str) -> Optional[DjmdRecommendLike]: ...
    def get_related_tracks(self) -> List[DjmdRelatedTracks]: ...
    def get_related_tracks_children(self, track_id: str) -> List[DjmdRelatedTracks]: ...
    def get_related_tracks_by_id(self, track_id: str) -> Optional[DjmdRelatedTracks]: ...
    def get_related_tracks_songs(self, track_id: str) -> List[DjmdSongRelatedTracks]: ...
    def get_related_tracks_contents(self, track_id: str) -> List[DjmdContent]: ...
    def get_sampler(self) -> List[DjmdSampler]: ...
    def get_sampler_children(self, sampler_id: str) -> List[DjmdSampler]: ...
    def get_sampler_by_id(self, sampler_id: str) -> Optional[DjmdSampler]: ...
    def get_sampler_songs(self, sampler_id: str) -> List[DjmdSongSampler]: ...
    def get_sampler_contents(self, sampler_id: str) -> List[DjmdContent]: ...
    def get_song_tag_list(self) -> List[DjmdSongTagList]: ...
    def get_song_tag_list_by_id(self, tag_list_id: str) -> Optional[DjmdSongTagList]: ...
    def get_sort(self) -> List[DjmdSort]: ...
    def get_sort_by_id(self, sort_id: str) -> Optional[DjmdSort]: ...
    def get_image_file(self) -> List[ImageFile]: ...
    def get_image_file_by_id(self, image_id: str) -> Optional[ImageFile]: ...
    def get_setting_file(self) -> List[SettingFile]: ...
    def get_setting_file_by_id(self, setting_id: str) -> Optional[SettingFile]: ...
    def get_uuid_id_map(self) -> List[UuidIDMap]: ...
    def get_uuid_id_map_by_id(self, uuid_id: str) -> Optional[UuidIDMap]: ...

class PyAnlz:
    def __init__(self, path: str) -> None: ...
    def dump_copy(self, path: str) -> None: ...
    def dump(self) -> None: ...
    def contains(self, tag_type: str) -> bool: ...
    def get_beat_grid(self) -> List[Obj]: ...
    def get_extended_beat_grid(self) -> List[Obj]: ...
    def get_hot_cues(self) -> List[Obj]: ...
    def get_memory_cues(self) -> List[Obj]: ...
    def get_extended_hot_cues(self) -> List[Obj]: ...
    def get_extended_memory_cues(self) -> List[Obj]: ...
    def get_tags(self) -> List[str]: ...
    def get_path(self) -> Optional[str]: ...
    def set_path(self, path: str) -> None: ...

class PyRekordboxXml:
    def __init__(self, path: str) -> None: ...
    @classmethod
    def load(cls, path: str) -> "PyRekordboxXml": ...
    def dump_copy(self, path: str) -> None: ...
    def dump(self) -> None: ...
    def get_tracks(self) -> List[Obj]: ...
    def get_track_by_id(self, track_id: str) -> Optional[Obj]: ...
    def get_track_by_location(self, location: str) -> Optional[Obj]: ...
    def get_track_by_key(self, key: str, key_type: int) -> Optional[Obj]: ...
    def add_track(self, track: Obj) -> None: ...
    def update_track(self, track: Obj) -> None: ...
    def remove_track(self, track_id: str) -> None: ...

class Error(Exception):
    """The base class of the other exceptions in this module.

    Use this to catch all errors with one single except statement
    """

    ...

class DatabaseError(Exception):
    """Exception raised for errors that are related to the master.db database."""

    ...

class AnlzError(Exception):
    """Exception raised for errors that are related to the ANLZ files."""

    ...

class XmlError(Exception):
    """Exception raised for errors that are related to the Rekordbox XML handler."""

    ...
