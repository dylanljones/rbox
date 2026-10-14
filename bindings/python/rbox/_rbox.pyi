# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

# Auto-generated Rekordbox Python Stubs

import enum
from datetime import datetime
from os import PathLike
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple, Union

from .anlz import AnlzData, AnlzSection

class Error(Exception):
    """The base class of the other exceptions in this module.

    Use this to catch all errors with one single except statement
    """

    ...

class OptionsError(Exception):
    """Exception raised for errors that are related to the master.db database."""

    ...

class MasterDbError(Exception):
    """Exception raised for errors that are related to the master.db database."""

    ...

class OneLibraryError(Exception):
    """Exception raised for errors that are related to the master.db database."""

    ...

class AnlzError(Exception):
    """Exception raised for errors that are related to the ANLZ files."""

    ...

class XmlError(Exception):
    """Exception raised for errors that are related to the Rekordbox XML handler."""

    ...

class SettingError(Exception):
    """Exception raised for errors that are related to the Rekordbox MySetting handler."""

    ...

# ---- Utility -------------------------------------------------------------------------------------

def rekordbox_agent_app_dir() -> str: ...
def options_json_path() -> str: ...
def rekordbox_app_dir() -> str: ...
def is_rekordbox_running() -> bool:
    """Checks if the Rekordbox application is currently running."""
    ...

class _RustMutableMapping:
    """Mutable mapping methods for Rust models.py."""

    def __len__(self) -> int:
        """Return the number of items."""
        ...

    def __contains__(self, key: str) -> bool:
        """Return True if the key is in the mapping."""
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

    def keys(self) -> Iterable[str]:
        """Return an iterator over the keys."""
        ...

    def values(self) -> Iterable[Any]:
        """Return an iterator over the values."""
        ...

    def items(self) -> Iterable[Tuple[str, Any]]:
        """Return an iterator over the keys and values."""
        ...

    def get(self, key: str, default: Any = None) -> Any:
        """Get an item by key or return the default value, if the key doesn't exist."""
        ...

    def update(self, data: Dict[str, Any]) -> None:
        """Update the fields from a dictionary."""
        ...

    def to_dict(self) -> Dict[str, Any]:
        """Convert the object to a dictionary."""
        ...

# ---- Options  -------------------------------------------------------------------------------------

class RekordboxOptions:
    def __init__(self, db_path: str, analysis_root: str, settings_root: str) -> "RekordboxOptions":
        """Creates a new `RekordboxOptions` instance."""
        ...

    @classmethod
    def load(cls, path: str) -> "RekordboxOptions":
        """Loads Rekordbox options from a JSON file."""
        ...

    @classmethod
    def open(cls) -> "RekordboxOptions":
        """Opens the Rekordbox options file from the default location."""
        ...

    @property
    def db_path(self) -> str:
        """Returns the path to the Rekordbox database."""
        ...

    @property
    def analysis_root(self) -> str:
        """Returns the path to the analysis data directory."""
        ...

    @property
    def settings_root(self) -> str:
        """Returns the path to the settings directory."""
        ...

# ---- Enums ----------------------------------------------------------------------------------------

class FileType(enum.IntEnum):
    INVALID: FileType
    MP3: FileType
    MP4: FileType
    ALAC: FileType
    FLAC: FileType
    M4A: FileType
    WAV: FileType
    AIFF: FileType
    StreamApple: FileType
    VIDEO: FileType
    AAC: FileType
    AIF: FileType

    def __int__(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: int | FileType) -> bool: ...
    def __ne__(self, other: int | FileType) -> bool: ...

class Analyzed(enum.IntEnum):
    NotAnalyzed: Analyzed
    Standard: Analyzed
    Advanced: Analyzed
    Locked: Analyzed

    def __int__(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: int | Analyzed) -> bool: ...
    def __ne__(self, other: int | Analyzed) -> bool: ...

class HistoryType(enum.IntEnum):
    List: HistoryType
    Folder: HistoryType

    def __int__(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: int | HistoryType) -> bool: ...
    def __ne__(self, other: int | HistoryType) -> bool: ...

class HotCueBanklistType(enum.IntEnum):
    List: HotCueBanklistType
    Folder: HotCueBanklistType

    def __int__(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: int | HotCueBanklistType) -> bool: ...
    def __ne__(self, other: int | HotCueBanklistType) -> bool: ...

class MyTagType(enum.IntEnum):
    List: MyTagType
    Folder: MyTagType

    def __int__(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: int | MyTagType) -> bool: ...
    def __ne__(self, other: int | MyTagType) -> bool: ...

class PlaylistType(enum.IntEnum):
    List: PlaylistType
    Folder: PlaylistType
    SmartList: PlaylistType
    CloudLibrarySync: PlaylistType

    def __int__(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: int | PlaylistType) -> bool: ...
    def __ne__(self, other: int | PlaylistType) -> bool: ...

class OneLibraryPlaylistType(enum.IntEnum):
    List: OneLibraryPlaylistType
    Folder: OneLibraryPlaylistType
    SmartList: OneLibraryPlaylistType
    CloudLibrarySync: OneLibraryPlaylistType

    def __int__(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: int | OneLibraryPlaylistType) -> bool: ...
    def __ne__(self, other: int | OneLibraryPlaylistType) -> bool: ...

class RelatedTracksType(enum.IntEnum):
    List: RelatedTracksType
    Folder: RelatedTracksType

    def __int__(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: int | RelatedTracksType) -> bool: ...
    def __ne__(self, other: int | RelatedTracksType) -> bool: ...

class SamplerType(SamplerType.IntEnum):
    List: RelatedTracksType
    Folder: SamplerType

    def __int__(self) -> int: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: int | SamplerType) -> bool: ...
    def __ne__(self, other: int | SamplerType) -> bool: ...

# ---- MasterDb  ------------------------------------------------------------------------------------

class AgentRegistry(_RustMutableMapping):
    """Represents the `agentRegistry` table in the Rekordbox database."""

    registry_id: str
    """A unique identifier for the registry entry."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    id_1: Optional[str]
    """An optional string field for additional identifier data."""
    id_2: Optional[str]
    """An optional string field for additional identifier data."""
    int_1: Optional[int]
    """An optional integer field for numerical data."""
    int_2: Optional[int]
    """An optional integer field for numerical data."""
    str_1: Optional[str]
    """An optional string field for textual data."""
    str_2: Optional[str]
    """An optional string field for textual data."""
    date_1: Optional[str]
    """An optional string field for date-related data."""
    date_2: Optional[str]
    """An optional string field for date-related data."""
    text_1: Optional[str]
    """An optional string field for extended text data."""
    text_2: Optional[str]
    """An optional string field for extended text data."""

class CloudAgentRegistry(_RustMutableMapping):
    """Represents the `cloudAgentRegistry` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    int_1: Optional[int]
    """An optional integer field for numerical data."""
    int_2: Optional[int]
    """An optional integer field for numerical data."""
    str_1: Optional[str]
    """An optional string field for textual data."""
    str_2: Optional[str]
    """An optional string field for textual data."""
    date_1: Optional[str]
    """An optional string field for date-related data."""
    date_2: Optional[str]
    """An optional string field for date-related data."""
    text_1: Optional[str]
    """An optional string field for extended text data."""
    text_2: Optional[str]
    """An optional string field for extended text data."""

class ContentActiveCensor(_RustMutableMapping):
    """Represents the `contentActiveCensor` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    content_id: str
    """The id of the associated [`DjmdContent`]."""
    active_censors: str
    """A string representing the active censors."""
    rb_activecensor_count: int
    """The count of active censors."""

class ContentCue(_RustMutableMapping):
    """Represents the `contentCue` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    content_id: str
    """The id of the associated [`DjmdContent`]."""
    cues: str
    """A JSON string representing the cue data."""
    rb_cue_count: int
    """The count of cues."""

class ContentFile(_RustMutableMapping):
    """Represents the `contentFile` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    content_id: str
    """The id of the associated [`DjmdContent`]."""
    path: str
    """The relative file path."""
    hash: str
    """The file hash."""
    size: int
    """The file size."""
    rb_local_path: str
    """The absolute file path."""
    rb_insync_hash: Optional[str]
    """An optional string representing the in-sync file hash."""
    rb_insync_local_usn: Optional[int]
    """An optional integer representing the in-sync local update sequence number."""
    rb_file_hash_dirty: Optional[int]
    """An optional integer indicating whether the file hash is dirty."""
    rb_local_file_status: Optional[int]
    """An optional integer representing the local file status."""
    rb_in_progress: Optional[int]
    """An optional integer indicating whether the file is in progress."""
    rb_process_type: Optional[int]
    """An optional integer representing the process type."""
    rb_temp_path: Optional[str]
    """An optional string representing the temporary file path."""
    rb_priority: Optional[int]
    """An optional integer representing the file priority."""
    rb_file_size_dirty: Optional[int]
    """An optional integer indicating whether the file size is dirty."""

class DjmdActiveCensor(_RustMutableMapping):
    """Represents the `djmdActiveCensor` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    content_id: str
    """The id of the associated [`DjmdContent`]."""
    in_msec: Optional[int]
    """An optional integer representing the start time of the censor in milliseconds."""
    out_msec: Optional[int]
    """An optional integer representing the end time of the censor in milliseconds."""
    info: Optional[int]
    """An optional integer representing additional information about the censor."""
    parameter_list: Optional[str]
    """An optional string representing the list of parameters for the censor."""
    content_uuid: Optional[str]
    """An optional string representing the uuid of the associated [`DjmdContent`]."""

class DjmdAlbumView:
    """Detached, shallow relationship snapshot. Attributes return row copies.

    To persist changes, retain a row, edit it, and call its database update method.
    """

    @property
    def album(self) -> DjmdAlbum: ...
    @property
    def artist(self) -> Optional[DjmdArtist]: ...

class DjmdAlbum(_RustMutableMapping):
    """Represents the `djmdAlbum` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    name: str
    """The name of the album."""
    album_artist_id: Optional[str]
    """An optional string representing the id of the album artist [`DjmdArtist`]."""
    image_path: Optional[str]
    """An optional string representing the path to the album's image."""
    compilation: int
    """A flag if the album is a compilation."""
    search_str: Optional[str]
    """An optional string used for searching the album."""

class NewDjmdAlbum(_RustMutableMapping):
    """Represents a new record insertale to the `djmdAlbum` table."""

    name: str
    """The name of the album."""
    album_artist_id: Optional[str]
    """An optional string representing the id of the album artist [`DjmdArtist`]."""
    image_path: Optional[str]
    """An optional string representing the path to the album's image."""
    compilation: Optional[int]
    """An optional integer indicating whether the album is a compilation."""
    search_str: Optional[str]
    """An optional string used for searching the album."""

class DjmdArtist(_RustMutableMapping):
    """Represents the `djmdArtist` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    name: str
    """An optional string representing the name of the artist."""
    search_str: Optional[str]
    """An optional string used for searching the artist."""

class NewDjmdArtist(_RustMutableMapping):
    """Represents a new record insertale to the `djmdArtist` table."""

    name: str
    """The name of the artist."""
    search_str: Optional[str]
    """An optional string used for searching the artist."""

class DjmdCategory(_RustMutableMapping):
    """Represents the `djmdCategory` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    menu_item_id: str
    """The id of the associated [`DjmdMenuItems`]."""
    seq: int
    """The sequence/order of the category."""
    disable: int
    """A flag indicating whether the category is disabled."""
    info_order: int
    """The info order for the category."""

class DjmdColor(_RustMutableMapping):
    """Represents the `djmdColor` table in the Rekordbox database."""

    id: str
    """A unique identifier for the color entry."""
    uuid: str
    """A unique universal identifier for the color entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    color_code: Optional[str]
    """An optional string representing the color code (e.g., hex value)."""
    sort_key: int
    """The sort key for ordering colors."""
    commnt: str
    """A comment (usually the name) associated with the color."""

class DjmdContentView:
    """Detached, shallow relationship snapshot. Attributes return row copies.

    To persist changes, retain a row, edit it, and call its database update method.
    """

    @property
    def content(self) -> DjmdContent: ...
    @property
    def artist(self) -> Optional[DjmdArtist]: ...
    @property
    def remixer(self) -> Optional[DjmdArtist]: ...
    @property
    def original_artist(self) -> Optional[DjmdArtist]: ...
    @property
    def composer(self) -> Optional[DjmdArtist]: ...
    @property
    def album(self) -> Optional[DjmdAlbum]: ...
    @property
    def album_artist(self) -> Optional[DjmdArtist]: ...
    @property
    def genre(self) -> Optional[DjmdGenre]: ...
    @property
    def label(self) -> Optional[DjmdLabel]: ...
    @property
    def key(self) -> Optional[DjmdKey]: ...
    @property
    def color(self) -> Optional[DjmdColor]: ...

class DjmdContent(_RustMutableMapping):
    """Represents the `djmdContent` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    folder_path: Optional[str]
    """An optional string representing the folder path of the track."""
    file_name_l: Optional[str]
    """An optional string representing the long file name of the track."""
    file_name_s: Optional[str]
    """An optional string representing the short file name of the track."""
    title: Optional[str]
    """An optional string representing the title of the track."""
    artist_id: Optional[str]
    """An optional string representing the ID of the associated artist in [`DjmdArtist`]."""
    album_id: Optional[str]
    """An optional string representing the ID of the associated album in [`DjmdAlbum`]."""
    genre_id: Optional[str]
    """An optional string representing the ID of the associated genre in [`DjmdGenre`]."""
    bpm: Optional[int]
    """An optional integer representing the BPM of the track."""
    length: Optional[int]
    """An optional integer representing the length of the track in milliseconds."""
    track_no: Optional[int]
    """An optional integer representing the track number in the album."""
    bit_rate: Optional[int]
    """An optional integer representing the bit rate of the track."""
    bit_depth: Optional[int]
    """An optional integer representing the bit depth of the track."""
    commnt: Optional[str]
    """An optional string representing comments about the track."""
    file_type: int
    """An optional integer representing the file type of the track."""
    rating: Optional[int]
    """An optional integer representing the rating of the track."""
    release_year: Optional[int]
    """An optional integer representing the release year of the track."""
    remixer_id: Optional[str]
    """An optional string representing the ID of the remixer in [`DjmdArtist`]."""
    label_id: Optional[str]
    """An optional string representing the ID of the label in [`DjmdLabel`]."""
    org_artist_id: Optional[str]
    """An optional string representing the ID of the original artist in [`DjmdArtist`]."""
    key_id: Optional[str]
    """An optional string representing the ID of the key in [`DjmdKey`]."""
    stock_date: Optional[str]
    """An optional string representing the stock date of the track."""
    color_id: Optional[str]
    """An optional string representing the ID of the color in [`DjmdColor`]."""
    dj_play_count: Optional[int]
    """An optional integer representing the DJ play count of the track."""
    image_path: Optional[str]
    """An optional string representing the path to the track's image."""
    master_db_id: Optional[str]
    """An optional string representing the ID of the master database."""
    master_song_id: Optional[str]
    """An optional string representing the ID of the master song."""
    analysis_data_path: Optional[str]
    """An optional string representing the path to the analysis data."""
    search_str: Optional[str]
    """An optional string used for searching the track."""
    file_size: Optional[int]
    """An optional integer representing the file size of the track."""
    disc_no: Optional[int]
    """An optional integer representing the disc number of the track."""
    composer_id: Optional[str]
    """An optional string representing the ID of the composer in [`DjmdArtist`]."""
    subtitle: Optional[str]
    """An optional string representing the subtitle of the track."""
    sample_rate: Optional[int]
    """An optional integer representing the sample rate of the track."""
    disable_quantize: Optional[int]
    """An optional integer indicating whether quantization is disabled for the track."""
    analysed: Optional[int]
    """An optional integer indicating whether the track has been analyzed."""
    release_date: Optional[str]
    """An optional string representing the release date of the track."""
    date_created: Optional[str]
    """An optional string representing the creation date of the track."""
    content_link: Optional[int]
    """An optional integer representing the content link of the track."""
    tag: Optional[str]
    """An optional string representing tags associated with the track."""
    modified_by_rbm: Optional[str]
    """An optional string indicating whether the track was modified by Rekordbox."""
    hot_cue_auto_load: Optional[str]
    """An optional string representing the auto-load hot cue setting."""
    delivery_control: Optional[str]
    """An optional string representing the delivery control setting."""
    delivery_comment: Optional[str]
    """An optional string representing comments about the delivery."""
    cue_updated: Optional[str]
    """An optional string indicating whether the cue points were updated."""
    analysis_updated: Optional[str]
    """An optional string indicating whether the analysis data was updated."""
    track_info_updated: Optional[str]
    """An optional string indicating whether the track info was updated."""
    lyricist: Optional[str]
    """An optional string representing the lyricist of the track."""
    isrc: Optional[str]
    """An optional string representing the ISRC (International Standard Recording Code) of the track."""
    sampler_track_info: Optional[int]
    """An optional integer representing sampler track information."""
    sampler_play_offset: Optional[int]
    """An optional integer representing the sampler play offset."""
    sampler_gain: Optional[float]
    """An optional float representing the sampler gain."""
    video_associate: Optional[str]
    """An optional string representing the associated video."""
    lyric_status: Optional[int]
    """An optional integer representing the lyric status of the track."""
    service_id: Optional[int]
    """An optional integer representing the service ID."""
    org_folder_path: Optional[str]
    """An optional string representing the original folder path of the track."""
    reserved1: Optional[str]
    """An optional string for reserved data."""
    reserved2: Optional[str]
    """An optional string for reserved data."""
    reserved3: Optional[str]
    """An optional string for reserved data."""
    reserved4: Optional[str]
    """An optional string for reserved data."""
    ext_info: Optional[str]
    """An optional string for extended information."""
    rb_file_id: Optional[str]
    """An optional string representing the Rekordbox file ID."""
    device_id: Optional[str]
    """An optional string representing the ID of the associated device in [`DjmdDevice`]."""
    rb_local_folder_path: Optional[str]
    """An optional string representing the local folder path in Rekordbox."""
    src_id: Optional[str]
    """An optional string representing the source ID."""
    src_title: Optional[str]
    """An optional string representing the source title."""
    src_artist_name: Optional[str]
    """An optional string representing the source artist name."""
    src_album_name: Optional[str]
    """An optional string representing the source album name."""
    src_length: Optional[int]
    """An optional integer representing the source length in milliseconds."""

class NewDjmdContent(_RustMutableMapping):
    """Represents a new record insertale to the `djmdContent` table."""

    folder_path: str
    """An optional string representing the folder path of the track."""
    title: Optional[str]
    """An optional string representing the title of the track."""
    artist_id: Optional[str]
    """An optional string representing the ID of the associated artist in [`DjmdArtist`]."""
    album_id: Optional[str]
    """An optional string representing the ID of the associated album in [`DjmdAlbum`]."""
    genre_id: Optional[str]
    """An optional string representing the ID of the associated genre in [`DjmdGenre`]."""
    bpm: Optional[int]
    """An optional integer representing the BPM of the track."""
    length: Optional[int]
    """An optional integer representing the length of the track in milliseconds."""
    track_no: Optional[int]
    """An optional integer representing the track number in the album."""
    bit_rate: Optional[int]
    """An optional integer representing the bit rate of the track."""
    bit_depth: Optional[int]
    """An optional integer representing the bit depth of the track."""
    commnt: Optional[str]
    """An optional string representing comments about the track."""
    rating: Optional[int]
    """An optional integer representing the rating of the track."""
    release_year: Optional[int]
    """An optional integer representing the release year of the track."""
    remixer_id: Optional[str]
    """An optional string representing the ID of the remixer in [`DjmdArtist`]."""
    label_id: Optional[str]
    """An optional string representing the ID of the label in [`DjmdLabel`]."""
    org_artist_id: Optional[str]
    """An optional string representing the ID of the original artist in [`DjmdArtist`]."""
    key_id: Optional[str]
    """An optional string representing the ID of the key in [`DjmdKey`]."""
    stock_date: Optional[str]
    """An optional string representing the stock date of the track."""
    search_str: Optional[str]
    """An optional string used for searching the track."""
    disc_no: Optional[int]
    """An optional integer representing the disc number of the track."""
    composer_id: Optional[str]
    """An optional string representing the ID of the composer in [`DjmdArtist`]."""
    subtitle: Optional[str]
    """An optional string representing the subtitle of the track."""
    sample_rate: Optional[int]
    """An optional integer representing the sample rate of the track."""
    release_date: Optional[str]
    """An optional string representing the release date of the track."""
    date_created: Optional[str]
    """An optional string representing the creation date of the track."""
    tag: Optional[str]
    """An optional string representing tags associated with the track."""
    lyricist: Optional[str]
    """An optional string representing the lyricist of the track."""
    isrc: Optional[str]
    """An optional string representing the ISRC (International Standard Recording Code) of the track."""

class DjmdCue(_RustMutableMapping):
    """Represents the `djmdCue` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    content_id: str
    """The ID of the associated track in [`DjmdContent`]."""
    in_msec: int
    """The cue's start time in milliseconds."""
    in_frame: int
    """The cue's start frame."""
    in_mpeg_frame: int
    """The cue's start MPEG frame or 0 if not a VBR/ABR MPEG file (see note)."""
    in_mpeg_abs: int
    """The cue's start MPEG absolute frame or 0 if not a VBR/ABR MPEG file."""
    out_msec: int
    """The cue's end time in milliseconds or -1 if not a loop."""
    out_frame: int
    """The cue's end frame or -1 if not a loop."""
    out_mpeg_frame: int
    """The cue's end MPEG frame or 0 if not a loop or not a VBR/ABR MPEG file (see note)"""
    out_mpeg_abs: int
    """The cue's end MPEG absolute frame or 0 if not a loop or not a VBR/ABR MPEG file"""
    kind: int
    """The type of the cue."""
    color: int
    """The color ID of the cue or -1 if no color."""
    color_table_index: Optional[int]
    """An optional integer representing the index of the color in the color table."""
    active_loop: Optional[int]
    """An optional integer indicating whether the cue is part of an active loop."""
    comment: Optional[str]
    """An optional string containing comments about the cue."""
    beat_loop_size: Optional[int]
    """An optional integer representing the size of the beat loop."""
    cue_microsec: Optional[int]
    """An optional integer representing the cue's position in microseconds."""
    in_point_seek_info: Optional[str]
    """An optional string containing seek information for the cue's start point."""
    out_point_seek_info: Optional[str]
    """An optional string containing seek information for the cue's end point."""
    content_uuid: Optional[str]
    """An optional string representing the UUID of the associated track in [`DjmdContent`]."""

class DjmdDevice(_RustMutableMapping):
    """Represents the `djmdDevice` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    master_db_id: str
    """The ID of the master database associated with the device."""
    name: str
    """The name of the device."""

class DjmdGenre(_RustMutableMapping):
    """Represents the `djmdGenre` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    name: str
    """An optional string representing the name of the genre."""

class NewDjmdGenre(_RustMutableMapping):
    """Represents a new record insertale to the `djmdGenre` table."""

    name: str
    """The name of the genre."""

class DjmdHistory(_RustMutableMapping):
    """Represents the `djmdHistory` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    seq: int
    """The sequence/order number of the history (1-based index)"""
    name: str
    """The name of the history."""
    attribute: HistoryType
    """The attribute of the tag, either history (`0`) or a folder (`1`)"""
    parent_id: str
    """The ID of the parent [`DjmdHistory`], `'root'` for top-level my-tag records."""
    date_created: str
    """The creation date of the history."""

class NewDjmdHistory(_RustMutableMapping):
    """Represents a new record insertale to the `djmdHistory` table."""

    name: str
    """The name of the history."""
    attribute: HistoryType
    """The attribute of the tag, either history (`0`) or a folder (`1`)"""
    seq: Optional[int]
    """The sequence/order number of the history (1-based index)"""
    parent_id: Optional[str]
    """The ID of the parent [`DjmdHistory`], `'root'` for top-level my-tag records."""
    date_created: Optional[str]
    """The creation date of the history."""

class DjmdSongHistory(_RustMutableMapping):
    """Represents the `djmdSongHistory` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    history_id: str
    """The ID of the associated history entry in [`DjmdHistory`]."""
    content_id: str
    """The ID of the associated content entry in [`DjmdContent`]."""
    track_no: int
    """The track number in the history entry."""

class NewDjmdSongHistory(_RustMutableMapping):
    """Represents a new record insertale to the `djmdSongHistory` table."""

    history_id: str
    """The ID of the associated history entry in [`DjmdHistory`]."""
    content_id: str
    """The ID of the associated content entry in [`DjmdContent`]."""
    track_no: Optional[int]
    """The track number in the history entry."""

class DjmdHotCueBanklist(_RustMutableMapping):
    """Represents the `djmdHotCueBanklist` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    seq: int
    """The sequence/order of the hot cue bank list (1-based index)"""
    name: str
    """The name of the hot cue bank list."""
    image_path: Optional[str]
    """An optional path to the image associated with the hot cue bank list."""
    attribute: HotCueBanklistType
    """The attribute of the hot cue bank list, either list (`0`) or a folder (`1`)"""
    parent_id: str
    """The ID of the parent [`DjmdHotCueBanklist`], `'root'` for top-level records."""

class DjmdSongHotCueBanklist(_RustMutableMapping):
    """Represents the `djmdSongHotCueBanklist` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    hot_cue_banklist_id: str
    """The ID of the associated [`DjmdHotCueBanklist`]."""
    content_id: str
    """The ID of the associated content in [`DjmdContent`]."""
    track_no: int
    """The track number in the hot cue bank list (1-based index)"""
    cue_id: str
    """The ID of the associated [`DjmdCue`]"""
    in_msec: int
    """The cue's start time in milliseconds."""
    in_frame: int
    """The cue's start frame."""
    in_mpeg_frame: int
    """The cue's start MPEG frame or 0 if not a VBR/ABR MPEG file (see note)."""
    in_mpeg_abs: int
    """The cue's start MPEG absolute frame or 0 if not a VBR/ABR MPEG file."""
    out_msec: int
    """The cue's end time in milliseconds or -1 if not a loop."""
    out_frame: int
    """The cue's end frame or -1 if not a loop."""
    out_mpeg_frame: int
    """The cue's end MPEG frame or 0 if not a loop or not a VBR/ABR MPEG file (see note)"""
    out_mpeg_abs: int
    """The cue's end MPEG absolute frame or 0 if not a loop or not a VBR/ABR MPEG file"""
    color: int
    """The color ID of the cue or -1 if no color."""
    color_table_index: Optional[int]
    """An optional integer representing the index of the color in the color table."""
    active_loop: Optional[int]
    """An optional integer indicating whether the cue is part of an active loop."""
    comment: Optional[str]
    """An optional string containing comments about the cue."""
    beat_loop_size: Optional[int]
    """An optional integer representing the size of the beat loop."""
    cue_microsec: Optional[int]
    """An optional integer representing the cue's position in microseconds."""
    in_point_seek_info: Optional[str]
    """An optional string containing seek information for the cue's start point."""
    out_point_seek_info: Optional[str]
    """An optional string containing seek information for the cue's end point."""
    hot_cue_banklist_uuid: Optional[str]
    """An optional string representing the UUID of the associate [`DjmdHotCueBanklist`]"""

class HotCueBanklistCue(_RustMutableMapping):
    """Represents the `hotCueBanklistCue` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    hot_cue_banklist_id: str
    """The ID of the associated [`DjmdSongHotCueBanklist`]."""
    cues: Optional[str]
    """An optional string containing the serialized cue data."""
    rb_cue_count: Optional[int]
    """An optional integer representing the count of cues."""

class DjmdKey(_RustMutableMapping):
    """Represents the `djmdKey` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    name: str
    """The name of the musical key."""
    seq: Optional[int]
    """An optional integer representing the sequence/order of the key."""

class NewDjmdKey(_RustMutableMapping):
    """Represents a new record insertale to the `djmdKey` table."""

    name: str
    """The name of the musical key."""
    seq: Optional[int]
    """An optional integer representing the sequence/order of the key."""

class DjmdLabel(_RustMutableMapping):
    """Represents the `djmdLabel` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    name: str
    """The name of the label."""

class NewDjmdLabel(_RustMutableMapping):
    """Represents a new record insertale to the `djmdLabel` table."""

    name: str
    """The name of the label."""

class DjmdMenuItems(_RustMutableMapping):
    """Represents the `djmdMenuItems` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    class_: int
    """The class of the menu item."""
    name: str
    """The name of the menu item."""

class DjmdMixerParam(_RustMutableMapping):
    """Represents the `djmdMixerParam` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    content_id: str
    """The ID of the associated [`DjmdContent`]."""
    gain_high: int
    """The upper 16 bits of a IEEE754 single-precision floating point number representing the gain."""
    gain_low: int
    """The lower 16 bits of a IEEE754 single-precision floating point number representing the gain."""
    peak_high: int
    """The upper 16 bits of a IEEE754 single-precision floating point number representing the peak."""
    peak_low: int
    """The lower 16 bits of a IEEE754 single-precision floating point number representing the peak."""

class DjmdMyTag(_RustMutableMapping):
    """Represents the `djmdMyTag` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    seq: int
    """The sequence/order number of the tag (1-based index)"""
    name: str
    """The name of the tag"""
    attribute: MyTagType
    """The attribute of the tag, either my-tag (`0`) or a folder (`1`)"""
    parent_id: str
    """The ID of the parent [`DjmdMyTag`], `'root'` for top-level my-tag records."""

class NewDjmdMyTag(_RustMutableMapping):
    """Represents a new record insertale to the `djmdMyTag` table."""

    name: str
    """The name of the tag"""
    attribute: MyTagType
    """The attribute of the tag, either my-tag (`0`) or a section (`1`)"""
    seq: Optional[int]
    """The sequence/order number of the tag (1-based index)"""
    parent_id: Optional[str]
    """The ID of the parent [`DjmdMyTag`]."""

class DjmdSongMyTag(_RustMutableMapping):
    """Represents the `djmdSongMyTag` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    my_tag_id: str
    """The ID of the associated [`DjmdMyTag`]."""
    content_id: str
    """The ID of the associated [`DjmdContent`]."""
    track_no: int
    """The track number in the tag."""

class NewDjmdSongMyTag(_RustMutableMapping):
    """Represents a new record insertale to the `djmdSongMyTag` table."""

    my_tag_id: str
    """The ID of the associated [`DjmdMyTag`]."""
    content_id: str
    """The ID of the associated [`DjmdContent`]."""
    track_no: Optional[int]
    """The track number in the tag."""

class DjmdPlaylist(_RustMutableMapping):
    """Represents the `djmdPlaylist` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    seq: int
    """The sequence/order number of the record (1-based index)"""
    name: str
    """The name of the playlist."""
    image_path: Optional[str]
    """An optional path to the image associated with the playlist."""
    attribute: PlaylistType
    """The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`)."""
    parent_id: str
    """The ID of the parent [`DjmdPlaylist`], `'root'` for top-level records."""
    smart_list: Optional[str]
    """An optional smart list configuration for the playlist (None if not a smart playlist)"""

class NewDjmdPlaylist(_RustMutableMapping):
    """Represents a new record insertale to the `djmdPlaylist` table."""

    name: str
    """The name of the playlist."""
    attribute: PlaylistType
    """The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`)."""
    seq: Optional[int]
    """The sequence/order number of the record (1-based index)"""
    parent_id: Optional[str]
    """The ID of the parent [`DjmdPlaylist`], `'root'` for top-level records (default)."""
    image_path: Optional[str]
    """An optional path to the image associated with the playlist."""
    smart_list: Optional[str]
    """An optional smart list configuration for the playlist (None if not a smart playlist)"""

class DjmdSongPlaylist(_RustMutableMapping):
    """Represents the `djmdSongPlaylist` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    playlist_id: str
    """The ID of the associated playlist in [`DjmdPlaylist`]."""
    content_id: str
    """The ID of the associated content in [`DjmdContent`]."""
    track_no: int
    """The track number in the playlist."""

class NewDjmdSongPlaylist(_RustMutableMapping):
    """Represents a new record insertale to the `djmdSongPlaylist` table."""

    playlist_id: str
    """The ID of the associated playlist in [`DjmdPlaylist`]."""
    content_id: str
    """The ID of the associated content in [`DjmdContent`]."""
    track_no: Optional[int]
    """The track number in the playlist."""

class DjmdProperty(_RustMutableMapping):
    """Represents the `djmdProperty` table in the Rekordbox database."""

    db_id: str
    """A unique identifier for the database entry."""
    db_version: str
    """The version of the database."""
    base_db_drive: str
    """The base drive of the database."""
    current_db_drive: str
    """The current drive of the database."""
    device_id: str
    """The ID of the associated device in [`DjmdDevice`]."""
    reserved1: Optional[str]
    """An optional string for reserved data."""
    reserved2: Optional[str]
    """An optional string for reserved data."""
    reserved3: Optional[str]
    """An optional string for reserved data."""
    reserved4: Optional[str]
    """An optional string for reserved data."""
    reserved5: Optional[str]
    """An optional string for reserved data."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""

class DjmdCloudProperty(_RustMutableMapping):
    """Represents the `djmdCloudProperty` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    reserved1: Optional[str]
    """An optional string for reserved data."""
    reserved2: Optional[str]
    """An optional string for reserved data."""
    reserved3: Optional[str]
    """An optional string for reserved data."""
    reserved4: Optional[str]
    """An optional string for reserved data."""
    reserved5: Optional[str]
    """An optional string for reserved data."""

class DjmdRecommendLike(_RustMutableMapping):
    """Represents the `djmdRecommendLike` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    content_id1: str
    """The ID of the first associated content in [`DjmdContent`]."""
    content_id2: str
    """The ID of the second associated content in [`DjmdContent`]."""
    like_rate: Optional[int]
    """An optional integer representing the like rate between the two associated contents."""
    data_created_h: Optional[int]
    """An optional integer representing the high part of the creation timestamp."""
    data_created_l: Optional[int]
    """An optional integer representing the low part of the creation timestamp."""

class DjmdRelatedTracks(_RustMutableMapping):
    """Represents the `djmdRelatedTracks` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    seq: int
    """The sequence/order number of the record (1-based index)"""
    name: str
    """The name of the related track record."""
    attribute: RelatedTracksType
    """The attribute of the related track, either list (`0`) or a folder (`1`)"""
    parent_id: str
    """The ID of the parent [`DjmdRelatedTracks`], `'root'` for top-level records."""
    criteria: str
    """A JSON string containing the criteria for the related tracks."""

class NewDjmdRelatedTracks(_RustMutableMapping):
    """Represents a new record insertale to the `djmdRelatedTracks` table."""

    name: str
    """The name of the related track."""
    attribute: RelatedTracksType
    """The attribute of the related track, either list (`0`) or a folder (`1`)"""
    seq: Optional[int]
    """The sequence/order number of the record (1-based index)"""
    parent_id: Optional[str]
    """The ID of the parent [`DjmdRelatedTracks`], `'root'` for top-level records."""
    criteria: Optional[str]
    """A JSON string containing the criteria for the related tracks."""

class DjmdSongRelatedTracks(_RustMutableMapping):
    """Represents the `djmdSongRelatedTracks` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    related_tracks_id: str
    """The ID of the associated related tracks entry in [`DjmdRelatedTracks`]."""
    content_id: str
    """The ID of the associated content entry in [`DjmdContent`]."""
    track_no: int
    """The track number in the related tracks entry."""

class NewDjmdSongRelatedTracks(_RustMutableMapping):
    """Represents a new record insertale to the `djmdSongRelatedTracks` table."""

    related_tracks_id: str
    """The ID of the associated playlist in [`DjmdPlaylist`]."""
    content_id: str
    """The ID of the associated content in [`DjmdContent`]."""
    track_no: Optional[int]
    """The track number in the playlist."""

class DjmdSampler(_RustMutableMapping):
    """Represents the `djmdSampler` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    seq: int
    """The sequence/order number of the record (1-based index)"""
    name: str
    """The name of the sampler."""
    attribute: SamplerType
    """The attribute of the sampler, either list (`0`) or a folder (`1`)"""
    parent_id: str
    """The ID of the parent [`DjmdSampler`], `'root'` for top-level records."""

class NewDjmdSampler(_RustMutableMapping):
    """Represents a new record insertale to the `djmdSampler` table."""

    name: str
    """The name of the sampler."""
    attribute: SamplerType
    """The attribute of the sampler, either list (`0`) or a folder (`1`)"""
    seq: Optional[int]
    """The sequence/order number of the record (1-based index)"""
    parent_id: Optional[str]
    """The ID of the parent [`DjmdSampler`], `'root'` for top-level records."""

class DjmdSongSampler(_RustMutableMapping):
    """Represents the `djmdSongSampler` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    sampler_id: str
    """The ID of the associated sampler in [`DjmdSampler`]."""
    content_id: str
    """The ID of the associated content in [`DjmdContent`]."""
    track_no: int
    """The sequence/order number of the record (1-based index)"""

class NewDjmdSongSampler(_RustMutableMapping):
    """Represents a new record insertale to the `djmdSongSampler` table."""

    sampler_id: str
    """The ID of the associated sampler in [`DjmdSampler`]."""
    content_id: str
    """The ID of the associated content in [`DjmdContent`]."""
    track_no: Optional[int]
    """The sequence/order number of the record (1-based index)"""

class DjmdSongTagList(_RustMutableMapping):
    """Represents the `djmdSongTagList` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    content_id: str
    """The ID of the associated content in [`DjmdContent`]."""
    track_no: int
    """The sequence/order number of the record (1-based index)"""

class DjmdSort(_RustMutableMapping):
    """Represents the `djmdSort` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    menu_item_id: str
    """The ID of the associated menu item in [`DjmdMenuItems`]."""
    seq: int
    """The sequence/order of the sort entry."""
    disable: int
    """A flag whether the sort entry is disabled."""

class ImageFile(_RustMutableMapping):
    """Represents the `imageFile` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    table_name: Optional[str]
    """An optional string representing the table name associated with the image file."""
    target_uuid: Optional[str]
    """An optional string representing the target UUID associated with the image file."""
    target_id: Optional[str]
    """An optional string representing the target ID associated with the image file."""
    path: Optional[str]
    """An optional string representing the file path of the image."""
    hash: Optional[str]
    """An optional string representing the hash of the image file."""
    size: Optional[int]
    """An optional integer representing the size of the image file."""
    rb_local_path: Optional[str]
    """An optional string representing the local path of the image file in Rekordbox."""
    rb_insync_hash: Optional[str]
    """An optional string representing the in-sync hash of the image file."""
    rb_insync_local_usn: Optional[int]
    """An optional integer representing the in-sync local update sequence number."""
    rb_file_hash_dirty: Optional[int]
    """An optional integer indicating whether the file hash is dirty."""
    rb_local_file_status: Optional[int]
    """An optional integer representing the local file status in Rekordbox."""
    rb_in_progress: Optional[int]
    """An optional integer indicating whether the file is in progress."""
    rb_process_type: Optional[int]
    """An optional integer representing the process type of the file."""
    rb_temp_path: Optional[str]
    """An optional string representing the temporary path of the file."""
    rb_priority: Optional[int]
    """An optional integer representing the priority of the file."""
    rb_file_size_dirty: Optional[int]
    """An optional integer indicating whether the file size is dirty."""

class SettingFile(_RustMutableMapping):
    """Represents the `settingFile` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    path: str
    """The relative file path of the setting file."""
    hash: str
    """The hash of the setting file."""
    size: int
    """The size of the setting file."""
    rb_local_path: Optional[str]
    """An optional string representing the local path of the setting file in Rekordbox."""
    rb_insync_hash: Optional[str]
    """An optional string representing the in-sync hash of the setting file."""
    rb_insync_local_usn: Optional[int]
    """An optional integer representing the in-sync local update sequence number."""
    rb_file_hash_dirty: Optional[int]
    """An optional integer indicating whether the file hash is dirty."""
    rb_file_size_dirty: Optional[int]
    """An optional integer indicating whether the file size is dirty."""

class UuidIDMap(_RustMutableMapping):
    """Represents the `uuidIDMap` table in the Rekordbox database."""

    id: str
    """A unique identifier for the entry."""
    uuid: str
    """A unique universal identifier for the entry."""
    rb_data_status: int
    """An integer representing the data status in Rekordbox."""
    rb_local_data_status: int
    """An integer representing the local data status in Rekordbox."""
    rb_local_deleted: int
    """An integer indicating whether the entry is locally deleted."""
    rb_local_synced: int
    """An integer indicating whether the entry is locally synced."""
    usn: Optional[int]
    """An optional integer representing the update sequence number."""
    rb_local_usn: Optional[int]
    """An optional integer representing the local update sequence number."""
    created_at: datetime
    """The timestamp when the entry was created, serialized/deserialized as `DateString`."""
    updated_at: datetime
    """The timestamp when the entry was last updated, serialized/deserialized as `DateString`."""
    table_name: str
    """Table name associated with the mapping."""
    target_uuid: str
    """The target UUID in the mapping."""
    current_id: str
    """The current ID in the mapping."""

class PyDjmdPlaylistTreeNode:
    id: str
    uuid: str
    rb_data_status: int
    rb_local_data_status: int
    rb_local_deleted: int
    rb_local_synced: int
    usn: Optional[int]
    rb_local_usn: Optional[int]
    created_at: datetime
    updated_at: datetime
    seq: int
    name: str
    image_path: Optional[str]
    attribute: PlaylistType
    parent_id: str
    smart_list: Optional[str]
    children: List[PyDjmdPlaylistTreeNode]

class PyMasterDb:
    def __init__(self, path: str) -> "PyMasterDb":
        """Open a Rekordbox database specified by path."""
        ...

    @classmethod
    def from_options(cls, options: RekordboxOptions) -> "PyMasterDb":
        """Open the Rekordbox database specified by the `RekordboxOptions`."""
        ...

    @classmethod
    def open(cls) -> "PyMasterDb":
        """Open the default Rekordbox `master.db` database."""
        ...

    def set_unsafe_writes(self, unsafe_writes: bool) -> None:
        """Sets the unsafe writes flag for the database."""
        ...

    def share_directory(self) -> Optional[str]:
        """Returns the path to the configured share directory."""
        ...

    def playlist_xml_path(self) -> Optional[str]:
        """Returns the path to the configured playlist XML file."""
        ...

    def get_agent_registries(self) -> List[AgentRegistry]:
        """Retrieves all entries from the [`AgentRegistry`] table in the database."""
        ...

    def get_agent_registry_by_id(self, id: str) -> Optional[AgentRegistry]:
        """Retrieves an `agentRegistry` entry by its unique identifier."""
        ...

    def get_local_usn(self) -> int:
        """Retrieves the local update sequence number (USN) from the `agentRegistry` table."""
        ...

    def get_cloud_agent_registries(self) -> List[CloudAgentRegistry]:
        """Retrieves all entries from the `cloudAgentRegistry` table in the database."""
        ...

    def get_cloud_agent_registry_by_id(self, id: str) -> Optional[CloudAgentRegistry]:
        """Retrieves a `cloudAgentRegistry` entry by its unique identifier."""
        ...

    def get_content_active_censors(self) -> List[ContentActiveCensor]:
        """Retrieves all entries from the `contentActiveCensor` table in the database."""
        ...

    def get_content_active_censor_by_id(self, id: str) -> Optional[ContentActiveCensor]:
        """Retrieves a `contentActiveCensor` entry by its unique identifier."""
        ...

    def get_content_cues(self) -> List[ContentCue]:
        """Retrieves all entries from the `contentCue` table in the database."""
        ...

    def get_content_cue_by_id(self, id: str) -> Optional[ContentCue]:
        """Retrieves a `contentCue` entry by its unique identifier."""
        ...

    def get_content_files(self) -> List[ContentFile]:
        """Retrieves all entries from the `contentFile` table in the database."""
        ...

    def get_content_file_by_id(self, id: str) -> Optional[ContentFile]:
        """Retrieves a `contentFile` entry by its unique identifier."""
        ...

    def get_active_censors(self) -> List[DjmdActiveCensor]:
        """Retrieves all entries from the `djmdActiveCensor` table in the database."""
        ...

    def get_active_censor_by_id(self, id: str) -> Optional[DjmdActiveCensor]:
        """Retrieves a `djmdActiveCensor` entry by its unique identifier."""
        ...

    def get_album_with_relations(self, id: str) -> Optional[DjmdAlbumView]:
        """Load a detached snapshot with all direct relationships."""
        ...

    def get_albums_with_relations(self) -> List[DjmdAlbumView]:
        """Load all rows with their direct relationships using batched queries."""
        ...

    def get_albums(self) -> List[DjmdAlbum]:
        """Retrieves all entries from the `djmdAlbum` table in the database."""
        ...

    def get_album_by_id(self, id: str) -> Optional[DjmdAlbum]:
        """Retrieves a `djmdAlbum` entry by its unique identifier."""
        ...

    def get_album_by_name(self, name: str) -> Optional[DjmdAlbum]:
        """Retrieves a `djmdAlbum` entry by its name."""
        ...

    def insert_album(self, album: NewDjmdAlbum) -> DjmdAlbum:
        """Inserts a new album into the `djmdAlbum` table in the database."""
        ...

    def create_album(
        self,
        name: str,
        artist_id: Optional[str] = None,
        image_path: Optional[str] = None,
        compilation: Optional[int] = None,
    ) -> DjmdAlbum:
        """Creates a new album and inserts it into the `djmdAlbum` table in the database."""
        ...

    def update_album(self, item: DjmdAlbum) -> DjmdAlbum:
        """Updates an existing `djmdAlbum` entry in the database."""
        ...

    def delete_album(self, id: str) -> None:
        """Deletes an album entry from the `djmdAlbum` table in the database."""
        ...

    def get_artists(self) -> List[DjmdArtist]:
        """Retrieves all entries from the `djmdArtist` table in the database."""
        ...

    def get_artist_by_id(self, id: str) -> Optional[DjmdArtist]:
        """Retrieves a `djmdArtist` entry by its unique identifier."""
        ...

    def get_artist_by_name(self, name: str) -> Optional[DjmdArtist]:
        """Retrieves a `djmdArtist` entry by its name."""
        ...

    def insert_artist(self, artist: NewDjmdArtist) -> DjmdArtist:
        """Inserts a new artist into the `djmdArtist` table in the database."""
        ...

    def create_artist(self, name: str) -> DjmdArtist:
        """Creates a new artist and inserts it into the `djmdArtist` table in the database."""
        ...

    def update_artist(self, item: DjmdArtist) -> DjmdArtist:
        """Updates an existing `djmdArtist` entry in the database."""
        ...

    def delete_artist(self, id: str) -> None:
        """Deletes an artist entry from the `djmdArtist` table in the database."""
        ...

    def get_categories(self) -> List[DjmdCategory]:
        """Retrieves all entries from the `djmdCategory` table in the database."""
        ...

    def get_category_by_id(self, id: str) -> Optional[DjmdCategory]:
        """Retrieves a `djmdCategory` entry by its unique identifier."""
        ...

    def get_colors(self) -> List[DjmdColor]:
        """Retrieves all entries from the `djmdColor` table in the database."""
        ...

    def get_color_by_id(self, id: str) -> Optional[DjmdColor]:
        """Retrieves a `djmdColor` entry by its unique identifier."""
        ...

    def get_content_with_relations(self, id: str) -> Optional[DjmdContentView]:
        """Load a detached snapshot with all direct relationships."""
        ...

    def get_contents_with_relations(self) -> List[DjmdContentView]:
        """Load all rows with their direct relationships using batched queries."""
        ...

    def get_contents(self) -> List[DjmdContent]:
        """Retrieves all entries from the `djmdContent` table in the database."""
        ...

    def get_content_by_id(self, id: str) -> Optional[DjmdContent]:
        """Retrieves a `djmdContent` entry by its unique identifier."""
        ...

    def get_contents_by_ids(self, ids: List[str]) -> List[DjmdContent]:
        """Retrieves a `djmdContent` entry by its unique identifier."""
        ...

    def get_content_by_path(self, path: str) -> Optional[DjmdContent]:
        """Retrieves a `djmdContent` entry by its folder path."""
        ...

    def get_content_anlz_dir(self, id: str) -> Optional[str]:
        """Returns the path to the directory containing the ANLZxxxx.xxx files for a given content ID."""
        ...

    def get_content_anlz_paths(self, id: str) -> Dict[str, str]:
        """Returns a struct containing the paths to ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 files."""
        ...

    def create_content(self, path: str) -> DjmdContent:
        """Creates a new content entry and inserts it into the `djmdContent` table."""
        ...

    def update_content(self, item: DjmdContent) -> None:
        """Updates an existing `djmdContent` entry in the database."""
        ...

    def update_content_album(self, id: str, name: str) -> None:
        """Update the content album field."""
        ...

    def update_content_artist(self, id: str, name: str) -> None:
        """Update the content artist name."""
        ...

    def update_content_remixer(self, id: str, name: str) -> None:
        """Update the content remixer name."""
        ...

    def update_content_original_artist(self, id: str, name: str) -> None:
        """Update the content original artist name."""
        ...

    def update_content_composer(self, id: str, name: str) -> None:
        """Update the content composer name."""
        ...

    def update_content_genre(self, id: str, name: str) -> None:
        """Update the content genre name."""
        ...

    def update_content_label(self, id: str, name: str) -> None:
        """Update the content label name."""
        ...

    def update_content_key(self, id: str, name: str) -> None:
        """Update the content key name."""
        ...

    def update_content_path(self, content_id: str, path: str) -> int:
        """Updates the file path of a content entry in the database."""
        ...

    def get_cues(self) -> List[DjmdCue]:
        """Retrieves all entries from the `djmdCue` table."""
        ...

    def get_cue_by_id(self, id: str) -> Optional[DjmdCue]:
        """Retrieves a `djmdCue` entry by its unique identifier."""
        ...

    def get_devices(self) -> List[DjmdDevice]:
        """Retrieves all entries from the `djmdDevice` table."""
        ...

    def get_device_by_id(self, id: str) -> Optional[DjmdDevice]:
        """Retrieves a `djmdDevice` entry by its unique identifier."""
        ...

    def get_genres(self) -> List[DjmdGenre]:
        """Retrieves all entries from the `djmdGenre` table in the database."""
        ...

    def get_genre_by_id(self, id: str) -> Optional[DjmdGenre]:
        """Retrieves a `djmdGenre` entry by its unique identifier."""
        ...

    def get_genre_by_name(self, name: str) -> Optional[DjmdGenre]:
        """Retrieves a `djmdGenre` entry by its name."""
        ...

    def insert_genre(self, genre: NewDjmdGenre) -> DjmdGenre:
        """Inserts a new genre into the `djmdGenre` table in the database."""
        ...

    def create_genre(self, name: str) -> DjmdGenre:
        """Creates a new genre and inserts it into the `djmdGenre` table in the database."""
        ...

    def update_genre(self, item: DjmdGenre) -> DjmdGenre:
        """Updates an existing `djmdGenre` entry in the database."""
        ...

    def delete_genre(self, id: str) -> None:
        """Deletes a genre entry from the `djmdGenre` table in the database."""
        ...

    def get_histories(self) -> List[DjmdHistory]:
        """Retrieves all entries from the `djmdHistory` table in the database."""
        ...

    def get_history_by_id(self, id: str) -> Optional[DjmdHistory]:
        """Retrieves a `djmdHistory` entry by its unique identifier."""
        ...

    def get_history_songs(self, id: str) -> List[DjmdSongHistory]:
        """Retrieves all song history entries for a given history ID, ordered by track number."""
        ...

    def get_history_contents(self, id: str) -> List[DjmdContent]:
        """Retrieves all content entries referenced by the song history for a given history ID."""
        ...

    def get_hot_cue_banklists(self) -> List[DjmdHotCueBanklist]:
        """Retrieves all entries from the `djmdHotCueBanklist` table in the database."""
        ...

    def get_hot_cue_banklist_by_id(self, id: str) -> Optional[DjmdHotCueBanklist]:
        """Retrieves a `djmdHotCueBanklist` entry by its unique identifier."""
        ...

    def get_hot_cue_banklist_children(self, parent_id: str) -> List[DjmdHotCueBanklist]:
        """Retrieves all child `djmdHotCueBanklist` entries for a given parent ID, ordered by sequence."""
        ...

    def get_hot_cue_banklist_songs(self, id: str) -> List[DjmdSongHotCueBanklist]:
        """Retrieves all `djmdSongHotCueBanklist` entries for a given hot cue banklist ID, ordered by track number."""
        ...

    def get_hot_cue_banklist_contents(self, id: str) -> List[DjmdContent]:
        """Retrieves all `djmdContent` entries referenced by the song hot cue banklist for a given hot cue banklist ID."""
        ...

    def get_hot_cue_banklist_cues(self, id: str) -> List[HotCueBanklistCue]:
        """Retrieves all `hotCueBanklistCue` entries for a given hot cue banklist ID."""
        ...

    def get_keys(self) -> List[DjmdKey]:
        """Retrieves all entries from the `djmdKey` table in the database."""
        ...

    def get_key_by_id(self, id: str) -> Optional[DjmdKey]:
        """Retrieves a `djmdKey` entry by its unique identifier."""
        ...

    def get_key_by_name(self, name: str) -> Optional[DjmdKey]:
        """Retrieves a `djmdKey` entry by its scale name."""
        ...

    def insert_key(self, key: NewDjmdKey) -> DjmdKey:
        """Inserts a new key into the `djmdKey` table in the database."""
        ...

    def create_key(self, name: str) -> DjmdKey:
        """Creates a new key and inserts it into the `djmdKey` table in the database."""
        ...

    def update_key(self, item: DjmdKey) -> DjmdKey:
        """Updates an existing `djmdKey` entry in the database."""
        ...

    def delete_key(self, id: str) -> None:
        """Deletes a key entry from the `djmdKey` table in the database."""
        ...

    def get_labels(self) -> List[DjmdLabel]:
        """Retrieves all entries from the `djmdLabel` table in the database."""
        ...

    def get_label_by_id(self, id: str) -> Optional[DjmdLabel]:
        """Retrieves a `djmdLabel` entry by its unique identifier."""
        ...

    def get_label_by_name(self, name: str) -> Optional[DjmdLabel]:
        """Retrieves a `djmdLabel` entry by its name."""
        ...

    def insert_label(self, label: NewDjmdLabel) -> DjmdLabel:
        """Inserts a new label into the `djmdLabel` table in the database."""
        ...

    def create_label(self, name: str) -> DjmdLabel:
        """Creates a new label and inserts it into the `djmdLabel` table in the database."""
        ...

    def update_label(self, item: DjmdLabel) -> DjmdLabel:
        """Updates an existing `djmdLabel` entry in the database."""
        ...

    def delete_label(self, id: str) -> None:
        """Deletes a label entry from the `djmdLabel` table in the database."""
        ...

    def get_menu_items(self) -> List[DjmdMenuItems]:
        """Retrieves all entries from the `djmdMenuItems` table in the database."""
        ...

    def get_menu_item_by_id(self, id: str) -> Optional[DjmdMenuItems]:
        """Retrieves a `djmdMenuItems` entry by its unique identifier."""
        ...

    def get_mixer_params(self) -> List[DjmdMixerParam]:
        """Retrieves all entries from the `djmdMixerParam` table in the database."""
        ...

    def get_mixer_param_by_id(self, id: str) -> Optional[DjmdMixerParam]:
        """Retrieves a `djmdMixerParam` entry by its unique identifier."""
        ...

    def get_my_tags(self) -> List[DjmdMyTag]:
        """Retrieves all entries from the `djmdMyTag` table in the database."""
        ...

    def get_my_tag_children(self, parent_id: str) -> List[DjmdMyTag]:
        """Retrieves all child `djmdMyTag` entries for a given parent ID, ordered by sequence."""
        ...

    def get_my_tag_by_id(self, id: str) -> Optional[DjmdMyTag]:
        """Retrieves a `djmdMyTag` entry by its unique identifier."""
        ...

    def get_my_tag_songs(self, id: str) -> List[DjmdSongMyTag]:
        """Retrieves all `djmdSongMyTag` entries for a given tag ID, ordered by track number."""
        ...

    def get_my_tag_contents(self, id: str) -> List[DjmdContent]:
        """Retrieves all `djmdContent` entries referenced by the songs for a given tag ID."""
        ...

    def get_playlists(self) -> List[DjmdPlaylist]:
        """Retrieves all entries from the `djmdPlaylist` table in the database."""
        ...

    def get_playlist_tree(self) -> List[PyDjmdPlaylistTreeNode]:
        """Returns a sorted tree of playlists as tree nodes."""
        ...

    def get_playlist_children(self, parent_id: str) -> List[DjmdPlaylist]:
        """Retrieves all child `djmdPlaylist` entries for a given parent ID, ordered by sequence."""
        ...

    def get_playlist_by_id(self, id: str) -> Optional[DjmdPlaylist]:
        """Retrieves a `djmdPlaylist` entry by its unique identifier."""
        ...

    def get_playlist_by_path(self, path: List[str]) -> Optional[DjmdPlaylist]:
        """Retrieves a `djmdPlaylist` entry by its hierarchical path."""
        ...

    def get_playlist_songs(self, id: str) -> List[DjmdSongPlaylist]:
        """Retrieves all `djmdSongPlaylist` entries for a given playlist ID, ordered by track number."""
        ...

    def get_playlist_contents(self, id: str) -> List[DjmdContent]:
        """Retrieves all `djmdContent` entries referenced by the songs for a given playlist ID."""
        ...

    def get_playlist_song_by_id(self, id: str) -> Optional[DjmdSongPlaylist]:
        """Retrieves a `djmdSongPlaylist` entry by its unique identifier."""
        ...

    def insert_playlist(self, playlist: NewDjmdPlaylist) -> DjmdPlaylist:
        """Inserts a new playlist into the `djmdPlaylist` table in the database."""
        ...

    def create_playlist_node(
        self,
        name: str,
        attribute: int,
        parent_id: Optional[str] = None,
        seq: Optional[int] = None,
        image_path: Optional[str] = None,
        smart_list: Optional[str] = None,
    ) -> DjmdPlaylist:
        """Creates a new playlist and inserts it into the `djmdPlaylist` table in the database."""
        ...

    def create_playlist(
        self,
        name: str,
        parent_id: Optional[str] = None,
        seq: Optional[int] = None,
        image_path: Optional[str] = None,
        smart_list: Optional[str] = None,
    ) -> DjmdPlaylist:
        """Creates a new regular playlist in the `djmdPlaylist` table."""
        ...

    def create_playlist_folder(
        self, name: str, parent_id: Optional[str] = None, seq: Optional[int] = None
    ) -> DjmdPlaylist:
        """Creates a new playlist folder in the `djmdPlaylist` table."""
        ...

    def rename_playlist(self, id: str, name: str) -> DjmdPlaylist:
        """Renames an existing playlist."""
        ...

    def move_playlist(self, id: str, seq: Optional[int] = None, parent_id: Optional[str] = None) -> DjmdPlaylist:
        """Moves a playlist to a new parent folder or changes its sequence within the same parent."""
        ...

    def move_playlists(self, ids: List[str], parent_id: str, start_seq: Optional[int] = None) -> List[DjmdPlaylist]:
        """Moves multiple playlists to a new parent folder or changes its sequence within the same parent."""
        ...

    def delete_playlist(self, id: str) -> None:
        """Deletes a playlist entry from the `djmdPlaylist` table in the database."""
        ...

    def insert_playlist_song(self, item: NewDjmdSongPlaylist) -> DjmdSongPlaylist:
        """Inserts a new `djmdSongPlaylist` song into a playlist."""
        ...

    def create_playlist_song(self, playlist_id: str, content_id: str, seq: Optional[int] = None) -> DjmdSongPlaylist:
        """Creates a new playlist song and inserts it into the `djmdSongPlaylist` table."""
        ...

    def move_playlist_song(self, id: str, seq: int) -> DjmdSongPlaylist:
        """Moves a song within a playlist to a new sequence position."""
        ...

    def move_playlist_songs(self, ids: List[str], start_seq: int) -> List[DjmdSongPlaylist]:
        """Moves multiple songs within a playlist to a new sequence position."""
        ...

    def delete_playlist_song(self, id: str) -> None:
        """Deletes a song entry from a playlist."""
        ...

    def delete_playlist_songs(self, ids: List[str]) -> int:
        """Deletes multiple songs from a playlist."""
        ...

    def get_properties(self) -> List[DjmdProperty]:
        """Retrieves all entries from the `djmdProperty` table in the database."""
        ...

    def get_property_by_id(self, id: str) -> Optional[DjmdProperty]:
        """Retrieves a `djmdProperty` entry by its unique identifier."""
        ...

    def get_cloud_properties(self) -> List[DjmdCloudProperty]:
        """Retrieves all entries from the `djmdCloudProperty` table in the database."""
        ...

    def get_cloud_property_by_id(self, id: str) -> Optional[DjmdCloudProperty]:
        """Retrieves a `djmdCloudProperty` entry by its unique identifier."""
        ...

    def get_recommend_likes(self) -> List[DjmdRecommendLike]:
        """Retrieves all entries from the `djmdRecommendLike` table in the database."""
        ...

    def get_recommend_like_by_id(self, id: str) -> Optional[DjmdRecommendLike]:
        """Retrieves a `djmdRecommendLike` entry by its unique identifier."""
        ...

    def get_related_tracks(self) -> List[DjmdRelatedTracks]:
        """Retrieves all entries from the `djmdRelatedTracks` table in the database."""
        ...

    def get_related_tracks_children(self, parent_id: str) -> List[DjmdRelatedTracks]:
        """Retrieves all child `djmdRelatedTracks` entries for a given parent ID, ordered by sequence."""
        ...

    def get_related_tracks_by_id(self, id: str) -> Optional[DjmdRelatedTracks]:
        """Retrieves a `djmdRelatedTracks` entry by its unique identifier."""
        ...

    def get_related_tracks_songs(self, id: str) -> List[DjmdSongRelatedTracks]:
        """Retrieves all `djmdSongRelatedTracks` entries for a given related tracks ID, ordered by track number."""
        ...

    def get_related_tracks_contents(self, id: str) -> List[DjmdContent]:
        """Retrieves all `djmdContent` entries referenced by the songs for a given related tracks ID."""
        ...

    def get_samplers(self) -> List[DjmdSampler]:
        """Retrieves all entries from the `djmdSampler` table in the database."""
        ...

    def get_sampler_children(self, parent_id: str) -> List[DjmdSampler]:
        """Retrieves all child `djmdSampler` entries for a given parent ID, ordered by sequence."""
        ...

    def get_sampler_by_id(self, id: str) -> Optional[DjmdSampler]:
        """Retrieves a `djmdSampler` entry by its unique identifier."""
        ...

    def get_sampler_songs(self, id: str) -> List[DjmdSongSampler]:
        """Retrieves all `djmdSongSampler` entries for a given sampler ID, ordered by track number."""
        ...

    def get_sampler_contents(self, id: str) -> List[DjmdContent]:
        """Retrieves all `djmdContent` entries referenced by the songs for a given sampler ID."""
        ...

    def get_song_tag_lists(self) -> List[DjmdSongTagList]:
        """Retrieves all entries from the `djmdSongTagList` table in the database."""
        ...

    def get_song_tag_list_by_id(self, id: str) -> Optional[DjmdSongTagList]:
        """Retrieves a `djmdSongTagList` entry by its unique identifier."""
        ...

    def get_sorts(self) -> List[DjmdSort]:
        """Retrieves all entries from the `djmdSort` table in the database, ordered by sequence."""
        ...

    def get_sort_by_id(self, id: str) -> Optional[DjmdSort]:
        """Retrieves a `djmdSort` entry by its unique identifier."""
        ...

    def get_image_files(self) -> List[ImageFile]:
        """Retrieves all entries from the `imageFile` table in the database."""
        ...

    def get_image_file_by_id(self, id: str) -> Optional[ImageFile]:
        """Retrieves an `imageFile` entry by its unique identifier."""
        ...

    def get_setting_files(self) -> List[SettingFile]:
        """Retrieves all entries from the `settingFile` table in the database."""
        ...

    def get_setting_file_by_id(self, id: str) -> Optional[SettingFile]:
        """Retrieves a `settingFile` entry by its unique identifier."""
        ...

    def get_uuid_id_maps(self) -> List[UuidIDMap]:
        """Retrieves all entries from the `uuidIDMap` table in the database."""
        ...

    def get_uuid_id_map_by_id(self, id: str) -> Optional[UuidIDMap]:
        """Retrieves a `uuidIDMap` entry by its unique identifier."""
        ...

# ---- OneLibrary  ----------------------------------------------------------------------------------

class AlbumView:
    """Detached, shallow relationship snapshot. Attributes return row copies.

    To persist changes, retain a row, edit it, and call its database update method.
    """

    @property
    def album(self) -> Album: ...
    @property
    def artist(self) -> Optional[Artist]: ...
    @property
    def image(self) -> Optional[Image]: ...

class Album(_RustMutableMapping):
    """Represents the `album` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the album."""
    name: str
    """The name of the album."""
    artist_id: Optional[int]
    """An optional foreign key referencing the album [`Artist`]."""
    image_id: Optional[int]
    """An optional foreign key referencing the album artwork [`Image`]."""
    is_compilation: int
    """A flag if the album is a compilation."""
    name_for_search: Optional[str]
    """An optional string used for searching the album."""

class NewAlbum(_RustMutableMapping):
    """Represents a new record insertale to the `album` table."""

    name: str
    """The name of the album."""
    artist_id: Optional[int]
    """An optional foreign key referencing the album artist [`Artist`]."""
    image_id: Optional[int]
    """An optional foreign key referencing the album artwork [`Image`]."""
    is_compilation: int
    """A flag if the album is a compilation."""
    name_for_search: Optional[str]
    """An optional string used for searching the album."""

class Artist(_RustMutableMapping):
    """Represents the `artist` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the artist."""
    name: str
    """The name of the artist."""
    name_for_search: Optional[str]
    """An optional field for storing a search-optimized version of the artist name."""

class NewArtist(_RustMutableMapping):
    """Represents a new record insertale to the `artist` table."""

    name: str
    """The name of the artist."""
    name_for_search: Optional[str]
    """An optional field for storing a search-optimized version of the artist name."""

class Remixer(_RustMutableMapping):
    id: int
    name: str
    name_for_search: Optional[str]

class OriginalArtist(_RustMutableMapping):
    id: int
    name: str
    name_for_search: Optional[str]

class Composer(_RustMutableMapping):
    id: int
    name: str
    name_for_search: Optional[str]

class Lyricist(_RustMutableMapping):
    id: int
    name: str
    name_for_search: Optional[str]

class CategoryView:
    """Detached, shallow relationship snapshot. Attributes return row copies.

    To persist changes, retain a row, edit it, and call its database update method.
    """

    @property
    def category(self) -> Category: ...
    @property
    def menu_item(self) -> Optional[MenuItem]: ...

class Category(_RustMutableMapping):
    """Represents the `category` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the category."""
    menu_item_id: int
    """A foreign key referencing the [`MenuItem`]."""
    seq: int
    """The sequence/order of the category."""
    is_visible: int
    """A flag if the category is visible."""

class NewCategory(_RustMutableMapping):
    """Represents a new record insertale to the `category` table."""

    menu_item_id: int
    """A foreign key referencing the [`MenuItem`]."""
    seq: int
    """The sequence/order of the category."""
    is_visible: int
    """A flag if the category is visible."""

class Color(_RustMutableMapping):
    """Represents the `color` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the color."""
    name: str
    """The name of the color"""

class NewColor(_RustMutableMapping):
    """Represents a new record insertale to the `color` table."""

    name: str
    """The name of the color"""

class ContentView:
    """Detached, shallow relationship snapshot. Attributes return row copies.

    To persist changes, retain a row, edit it, and call its database update method.
    """

    @property
    def content(self) -> Content: ...
    @property
    def artist(self) -> Optional[Artist]: ...
    @property
    def remixer(self) -> Optional[Artist]: ...
    @property
    def original_artist(self) -> Optional[Artist]: ...
    @property
    def composer(self) -> Optional[Artist]: ...
    @property
    def lyricist(self) -> Optional[Artist]: ...
    @property
    def album(self) -> Optional[Album]: ...
    @property
    def genre(self) -> Optional[Genre]: ...
    @property
    def label(self) -> Optional[Label]: ...
    @property
    def key(self) -> Optional[Key]: ...
    @property
    def color(self) -> Optional[Color]: ...
    @property
    def image(self) -> Optional[Image]: ...

class Content(_RustMutableMapping):
    """Represents the `content` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the content."""
    title: Optional[str]
    """The title of the track."""
    title_for_search: Optional[str]
    """An optional title used for searching."""
    subtitle: Optional[str]
    """The subtitle of the track"""
    bpmx100: Optional[int]
    """The beats per minute multiplied by 100."""
    length: Optional[int]
    """The length of the track in seconds."""
    track_no: Optional[int]
    """The track number of the track"""
    disc_no: Optional[int]
    """The disc number of the track"""
    artist_id: Optional[int]
    """Optional foreign key referencing the artist [`Artist`]."""
    remixer_id: Optional[int]
    """Optional foreign key referencing the remixer [`Artist`]."""
    original_artist_id: Optional[int]
    """Optional foreign key referencing the original artist [`Artist`]."""
    composer_id: Optional[int]
    """Optional foreign key referencing the composer [`Artist`]."""
    lyricist_id: Optional[int]
    """Optional foreign key referencing the lyricist [`Artist`]."""
    album_id: Optional[int]
    """Optional foreign key referencing the [`Album`]."""
    genre_id: Optional[int]
    """Optional foreign key referencing the [`Genre`]."""
    label_id: Optional[int]
    """Optional foreign key referencing the [`Label`]."""
    key_id: Optional[int]
    """Optional foreign key referencing the [`Key`]."""
    color_id: Optional[int]
    """Optional foreign key referencing the [`Color`]."""
    image_id: Optional[int]
    """Optional foreign key referencing the artwork [`Image`]."""
    dj_comment: Optional[str]
    """The ccomment of the track."""
    rating: Optional[int]
    """The rating of the track."""
    release_year: Optional[int]
    """The release year of the track."""
    release_date: Optional[str]
    """The release date of the track"""
    date_created: datetime
    """The date the record was created"""
    date_added: datetime
    """The date the record was added"""
    path: str
    """The relative file path of the track (usually `'/Contents/<Artist>/<Title>'`)"""
    file_name: Optional[str]
    """The file name of the track."""
    file_size: Optional[int]
    """The file size of the track"""
    file_type: int
    """The file type of the track"""
    bitrate: Optional[int]
    """The bitrate of the trackk"""
    bit_depth: Optional[int]
    """The bit depth of the track"""
    sampling_rate: Optional[int]
    """The sample rate of the track."""
    isrc: Optional[str]
    """The ISRC of the track"""
    is_hot_cue_auto_load_on: Optional[int]
    """A flag if hot-cue auto load is enabled"""
    is_kuvo_deliver_status_on: Optional[int]
    """A flag if kuvo delivery is enabled"""
    kuvo_delivery_comment: Optional[str]
    """A kuvo delivery comment"""
    master_db_id: Optional[int]
    """The masterdb ID"""
    master_content_id: Optional[int]
    """The ID of the contnent in the masterdb"""
    analysis_data_file_path: Optional[str]
    """The relative path to the analysis data"""
    analysed_bits: Optional[int]
    """The number of bits analysed (?)"""
    content_link: Optional[int]
    """The ID of the content in the content link database (seems to always be 788224)"""
    has_modified: Optional[int]
    """A flag if the content has been modified"""
    cue_update_count: Optional[int]
    """The number of updates to the associated cues"""
    analysis_data_update_count: Optional[int]
    """The number of updates to the analysis data"""
    information_update_count: Optional[int]
    """The number of updates to the content record"""

class NewContent(_RustMutableMapping):
    """Represents a new record insertale to the `content` table."""

    date_created: datetime
    """The date the record was created"""
    date_added: datetime
    """The date the record was added"""
    path: str
    """The relative file path of the track"""
    title: Optional[str]
    """The title of the track."""
    title_for_search: Optional[str]
    """An optional title used for searching."""
    subtitle: Optional[str]
    """The subtitle of the track"""
    bpmx100: Optional[int]
    """The beats per minute multiplied by 100."""
    length: Optional[int]
    """The length of the track in seconds."""
    track_no: Optional[int]
    """The track number of the track"""
    disc_no: Optional[int]
    """The disc number of the track"""
    artist_id: Optional[int]
    """Optional foreign key referencing the artist [`Artist`]."""
    remixer_id: Optional[int]
    """Optional foreign key referencing the remixer [`Artist`]."""
    original_artist_id: Optional[int]
    """Optional foreign key referencing the original artist [`Artist`]."""
    composer_id: Optional[int]
    """Optional foreign key referencing the composer [`Artist`]."""
    lyricist_id: Optional[int]
    """Optional foreign key referencing the lyricist [`Artist`]."""
    album_id: Optional[int]
    """Optional foreign key referencing the [`Album`]."""
    genre_id: Optional[int]
    """Optional foreign key referencing the [`Genre`]."""
    label_id: Optional[int]
    """Optional foreign key referencing the [`Label`]."""
    key_id: Optional[int]
    """Optional foreign key referencing the [`Key`]."""
    color_id: Optional[int]
    """Optional foreign key referencing the [`Color`]."""
    image_id: Optional[int]
    """Optional foreign key referencing the artwork [`Image`]."""
    dj_comment: Optional[str]
    """The ccomment of the track."""
    rating: Optional[int]
    """The rating of the track."""
    release_year: Optional[int]
    """The release year of the track."""
    release_date: Optional[str]
    """The release date of the track"""
    file_name: Optional[str]
    """The file name of the track."""
    file_size: Optional[int]
    """The file size of the track"""
    file_type: Optional[int]
    """The file typo of the track"""
    bitrate: Optional[int]
    """The bitrate of the trackk"""
    bit_depth: Optional[int]
    """The bit depth of the track"""
    sampling_rate: Optional[int]
    """The sample rate of the track."""
    isrc: Optional[str]
    """The ISRC of the track"""
    is_hot_cue_auto_load_on: Optional[int]
    """A flag if hot-cue auto load is enabled"""
    is_kuvo_deliver_status_on: Optional[int]
    """A flag if kuvo delivery is enabled"""
    kuvo_delivery_comment: Optional[str]
    """A kuvo delivery comment"""
    master_db_id: Optional[int]
    """The masterdb ID"""
    master_content_id: Optional[int]
    """The ID of the contnent in the masterdb"""
    analysis_data_file_path: Optional[str]
    """The relative path to the analysis data"""
    analysed_bits: Optional[int]
    """The number of bits analysed (?)"""
    content_link: Optional[int]
    """The ID of the content in the content link database (seems to always be 788224)"""
    has_modified: Optional[int]
    """A flag if the content has been modified"""
    cue_update_count: Optional[int]
    """The number of updates to the associated cues"""
    analysis_data_update_count: Optional[int]
    """The number of updates to the analysis data"""
    information_update_count: Optional[int]
    """The number of updates to the content record"""

class Cue(_RustMutableMapping):
    """Represents the `cue` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the cue."""
    content_id: int
    """A foreign key referencing the [`Content`]."""
    kind: Optional[int]
    """The type of the cue."""
    color_table_index: Optional[int]
    """An optional integer representing the index of the color in the color table."""
    cue_comment: Optional[str]
    """A comment for the cue"""
    is_active_loop: Optional[int]
    """A flag whether the cue is part of an active loop."""
    beat_loop_numerator: Optional[int]
    """Unknown"""
    beat_loop_denominator: Optional[int]
    """Unknonw"""
    in_usec: Optional[int]
    """The cue's start time in microseconds."""
    out_usec: Optional[int]
    """The cue's end time in microseconds."""
    in150_frame_per_sec: Optional[int]
    """The cue's start frame."""
    out150_frame_per_sec: Optional[int]
    """The cue's end frame or -1 if not a loop."""
    in_mpeg_frame_number: Optional[int]
    """The cue's start MPEG frame or 0 if not a VBR/ABR MPEG file (see note)."""
    out_mpeg_frame_number: Optional[int]
    """The cue's end MPEG frame or 0 if not a loop or not a VBR/ABR MPEG file (see note)"""
    in_mpeg_abs: Optional[int]
    """The cue's start MPEG absolute frame or 0 if not a VBR/ABR MPEG file."""
    out_mpeg_abs: Optional[int]
    """The cue's end MPEG absolute frame or 0 if not a loop or not a VBR/ABR MPEG file"""
    in_decoding_start_frame_position: Optional[int]
    """The cue's start frame position in the decoding block."""
    out_decoding_start_frame_position: Optional[int]
    """The cue's end frame position in the decoding block."""
    in_file_offset_in_block: Optional[int]
    """The cue's start file offset in the decoding block."""
    out_file_offset_in_block: Optional[int]
    """The cue's end file offset in the decoding block."""
    in_number_of_sample_in_block: Optional[int]
    """The cue's start sample position in the decoding block."""
    out_number_of_sample_in_block: Optional[int]
    """The cue's end sample position in the decoding block."""

class Genre(_RustMutableMapping):
    """Represents the `genre` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the genre."""
    name: str
    """The name of the genre."""

class NewGenre(_RustMutableMapping):
    """Represents a new record insertale to the `genre` table."""

    name: str
    """The name of the genre."""

class History(_RustMutableMapping):
    """Represents the `history` table in the Rekordbox One Library database."""

    id: int
    """A unique identifier for the entry."""
    seq: int
    """The sequence/order number of the history (1-based index)"""
    name: str
    """The name of the history."""
    attribute: HistoryType
    """The attribute of the tag, either history (`0`) or a folder (`1`)"""
    parent_id: int
    """The ID of the parent [`History`] or `0` for top-level record."""

class NewHistory(_RustMutableMapping):
    """Represents a new record insertale to the `history` table."""

    seq: int
    """The sequence/order number of the history (1-based index)"""
    name: str
    """The name of the history."""
    attribute: HistoryType
    """The attribute of the tag, either history (`0`) or a folder (`1`)"""
    parent_id: int
    """The ID of the parent [`History`] or `0` for top-level record."""

class HistoryContent(_RustMutableMapping):
    """Represents the `history_content` table in the Rekordbox One Library database."""

    history_id: int
    """The ID of the associated [`History`]."""
    content_id: int
    """The ID of the associated [`Content`]."""
    seq: int
    """The sequence/order number of the history content (1-based index)"""

class HotCueBankList(_RustMutableMapping):
    """Represents the `hotCueBankList` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the hot cue bank list."""
    seq: int
    """The sequence/order number of the hot cue bank list (1-based index)."""
    name: Optional[str]
    """The name of the hot cue bank list."""
    image_id: Optional[int]
    """An optional foreign key referencing an [`Image`]."""
    attribute: HotCueBanklistType
    """The hot cue bank list type, either list (`0`) or a folder (`1`)."""
    parent_id: Optional[int]
    """The ID of the parent [`HotCueBankList`], `0` for top-level records."""

class HotCueBankListCue(_RustMutableMapping):
    """Represents the `hotCueBankList_cue` table in the Rekordbox One Library database."""

    hot_cue_bank_list_id: int
    """The unique identifier of the hot cue bank list."""
    cue_id: int
    """The unique identifier of the cue."""
    seq: int
    """The sequence/order number of the hot cue bank list cue (1-based index)."""

class Image(_RustMutableMapping):
    """Represents the `image` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the image."""
    path: str
    """The relative file path of the image."""

class NewImage(_RustMutableMapping):
    """Represents a new record insertale to the `image` table."""

    path: str
    """The relative file path of the image."""

class Key(_RustMutableMapping):
    """Represents the `key` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the key."""
    name: str
    """The name of the key."""

class NewKey(_RustMutableMapping):
    """Represents a new record insertale to the `key` table."""

    name: str
    """The name of the key."""

class Label(_RustMutableMapping):
    """Represents the `label` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the label."""
    name: str
    """The name of the label."""

class NewLabel(_RustMutableMapping):
    """Represents a new record insertale to the `label` table."""

    name: str
    """The name of the label."""

class MenuItem(_RustMutableMapping):
    """Represents the `menuItem` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the record."""
    kind: int
    """The kind of menu item (similar to `class` in the main DB)."""
    name: str
    """The name of the menu item."""

class NewMenuItem(_RustMutableMapping):
    """Represents a new record insertale to the `menuItem` table."""

    kind: int
    """The kind of menu item (similar to `class` in the main DB)."""
    name: str
    """The name of the menu item."""

class MyTag(_RustMutableMapping):
    """Represents the `myTag` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the tag."""
    seq: int
    """The sequence/order number of the tag (1-based index)."""
    name: str
    """The name of the tag."""
    attribute: MyTagType
    """The tag type, either list (`0`) or a folder (`1`)."""
    parent_id: int
    """The ID of the parent [`MyTag`], `0` for top-level records."""

class NewMyTag(_RustMutableMapping):
    """Represents a new record insertale to the `myTag` table."""

    seq: int
    """The sequence/order number of the tag (1-based index)."""
    name: str
    """The name of the tag."""
    attribute: MyTagType
    """The tag type, either list (`0`) or a folder (`1`)."""
    parent_id: int
    """The ID of the parent [`MyTag`], `0` for top-level records."""

class MyTagContent(_RustMutableMapping):
    """Represents the `myTag_content` table in the Rekordbox One Library database."""

    my_tag_id: int
    """The ID of the associated [`MyTag`]."""
    content_id: int
    """The ID of the associated [`Content`]."""

class Playlist(_RustMutableMapping):
    """Represents the `playlist` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the playlist."""
    seq: int
    """The sequence/order number of the playlist (1-based index)."""
    name: str
    """The name of the playlist."""
    image_id: Optional[int]
    """Optional foreign key referencing the artwork [`Image`] (`0` or `None` if no image)."""
    attribute: OneLibraryPlaylistType
    """The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`)."""
    parent_id: int
    """The ID of the parent [`Playlist`], `0` for top-level records."""

class NewPlaylist(_RustMutableMapping):
    """Represents a new record insertale to the `playlist` table."""

    seq: int
    """The sequence/order number of the playlist (1-based index)."""
    name: str
    """The name of the playlist."""
    image_id: Optional[int]
    """Optional foreign key referencing the artwork [`Image`] (`0` or `None` if no image)."""
    attribute: OneLibraryPlaylistType
    """The playlist type, either list (`0`), a folder (`1`) or a smart list (`4`)."""
    parent_id: int
    """The ID of the parent [`Playlist`], `0` for top-level records."""

class PlaylistContent(_RustMutableMapping):
    """Represents the `playlist_content` table in the Rekordbox One Library database."""

    playlist_id: int
    """The ID of the associated [`Playlist`]."""
    content_id: int
    """The ID of the associated [`Content`]."""
    seq: int
    """The sequence/order number of the history content (1-based index)"""

class Property(_RustMutableMapping):
    """Represents the `property` table in the Rekordbox One Library database."""

    device_name: str
    """The name of the device ('' by default)"""
    db_version: int
    """The DB version (1000 by default)"""
    number_of_contents: int
    """The number of ['Content'] in the device"""
    created_date: datetime
    """The date of creation for the device"""
    back_ground_color_type: int
    """The background color type (0 by default)"""
    my_tag_master_dbid: int
    """The master.db database id of the tag data"""

class RecommendedLike(_RustMutableMapping):
    """Represents the `recommendedLike` table in the Rekordbox One Library database."""

    content_id_1: int
    """The ID of the first associated [`Content`]."""
    content_id_2: int
    """The ID of the second associated [`Content`]."""
    rating: int
    """The rating of the relationship, between 1-5."""
    created_date: datetime
    """The timestamp when the entry was created, serialized/deserialized as `Datetime<Utc>`."""

class Sort(_RustMutableMapping):
    """Represents the `sort` table in the Rekordbox One Library database."""

    id: int
    """The unique identifier of the record."""
    menu_item_id: int
    """An optional foreign key referencing the [`MenuItem`]."""
    seq: int
    """The sequence/order number of the *visible* items (1-based index)"""
    is_visible: int
    """A flag if the item is visible in the menu"""
    is_selected_as_sub_column: int
    """A flag if the item is selected as a sub-column"""

class NewSort(_RustMutableMapping):
    """Represents a new record insertale to the `sort` table."""

    menu_item_id: int
    """An optional foreign key referencing the [`MenuItem`]."""
    seq: int
    """The sequence/order number of the item (1-based index)"""
    is_visible: int
    """A flag if the item is visible in the menu"""
    is_selected_as_sub_column: int
    """A flag if the item is selected as a sub-column"""

class PyOneLibrary:
    def __init__(self, path: str) -> "PyOneLibrary":
        """Open a OneLibrary (Device Library Plus) database specified by path."""
        ...

    @classmethod
    def create(cls, path: str, my_tag_master_dbid: int) -> "PyOneLibrary":
        """Create a new OneLibrary (Device Library Plus) database."""
        ...

    @classmethod
    def create_if_not_exists(cls, path: str, my_tag_master_dbid: int) -> "PyOneLibrary":
        """Create a new OneLibrary database if it does not exist, otherwise return the existing one."""
        ...

    def get_album_with_relations(self, id: int) -> Optional[AlbumView]:
        """Load a detached snapshot with all direct relationships."""
        ...

    def get_albums_with_relations(self) -> List[AlbumView]:
        """Load all rows with their direct relationships using batched queries."""
        ...

    def get_albums(self) -> List[Album]:
        """Queries all records from the `album` table."""
        ...

    def get_album_by_id(self, id: int) -> Optional[Album]:
        """Queries a record from the `album` table by its primary key `id`."""
        ...

    def get_album_by_name(self, name: str) -> Optional[Album]:
        """Queries a record from the `album` table by its `name`."""
        ...

    def insert_album(self, album: NewAlbum) -> Album:
        """Inserts a new record into the `album` table."""
        ...

    def create_album(self, name: str, artist_id: Optional[int], image_id: Optional[int]) -> Album:
        """Creates a new record and inserts it into the `album` table."""
        ...

    def update_album(self, album: Album) -> Album:
        """Updates an existing record in the `album` table."""
        ...

    def delete_album(self, id: int) -> int:
        """Deletes a record from the `album` table by its primary key `id`."""
        ...

    def get_artists(self) -> List[Artist]:
        """Queries all records from the `artist` table."""
        ...

    def get_artist_by_id(self, id: int) -> Optional[Artist]:
        """Queries a record from the `artist` table by its primary key `id`."""
        ...

    def get_artist_by_name(self, name: str) -> Optional[Artist]:
        """Queries a record from the `artist` table by its `name`."""
        ...

    def insert_artist(self, artist: NewArtist) -> Artist:
        """Inserts a new record into the `artist` table."""
        ...

    def create_artist(self, name: str) -> Artist:
        """Creates a new record and inserts it into the `artist` table."""
        ...

    def update_artist(self, artist: Artist) -> Artist:
        """Updates an existing record in the `artist` table."""
        ...

    def delete_artist(self, id: int) -> int:
        """Deletes a record from the `artist` table by its primary key `id`."""
        ...

    def get_category_with_relations(self, id: int) -> Optional[CategoryView]:
        """Load a detached snapshot with all direct relationships."""
        ...

    def get_categories_with_relations(self) -> List[CategoryView]:
        """Load all rows with their direct relationships using batched queries."""
        ...

    def get_categories(self) -> List[Category]:
        """Queries all records from the `category` table."""
        ...

    def get_category_by_id(self, id: int) -> Optional[Category]:
        """Queries a record from the `category` table by its primary key `id`."""
        ...

    def insert_category(self, category: NewCategory) -> Category:
        """Inserts a new record into the `category` table."""
        ...

    def create_category(self, menu_item_id: int, seq: Optional[int], is_visible: Optional[int]) -> Category:
        """Creates a new record and inserts it into the `category` table."""
        ...

    def update_category(self, category: Category) -> Category:
        """Updates an existing record in the `category` table."""
        ...

    def delete_category(self, id: int) -> int:
        """Deletes a record from the `category` table by its primary key `id`."""
        ...

    def get_colors(self) -> List[Color]:
        """Queries all records from the `color` table."""
        ...

    def get_color_by_id(self, id: int) -> Optional[Color]:
        """Queries a record from the `color` table by its primary key `id`."""
        ...

    def insert_color(self, color: NewColor) -> Color:
        """Inserts a new record into the `color` table."""
        ...

    def create_color(self, name: str) -> Color:
        """Creates a new record and inserts it into the `color` table."""
        ...

    def update_color(self, color: Color) -> Color:
        """Updates an existing record in the `color` table."""
        ...

    def delete_color(self, id: int) -> int:
        """Deletes a record from the `color` table by its primary key `id`."""
        ...

    def get_content_with_relations(self, id: int) -> Optional[ContentView]:
        """Load a detached snapshot with all direct relationships."""
        ...

    def get_contents_with_relations(self) -> List[ContentView]:
        """Load all rows with their direct relationships using batched queries."""
        ...

    def get_contents(self) -> List[Content]:
        """Queries all records from the `color` table."""
        ...

    def get_content_by_id(self, id: int) -> Optional[Content]:
        """Queries a record from the `content` table by its primary key `id`."""
        ...

    def insert_content(self, content: NewContent) -> Content:
        """Inserts a new record into the `content` table."""
        ...

    def create_content(self, path: str) -> Content:
        """Creates a new record and inserts it into the `content` table."""
        ...

    def update_content(self, content: Content) -> Content:
        """Updates an existing record in the `content` table."""
        ...

    def delete_content(self, id: int) -> int:
        """Deletes a record from the `content` table by its primary key `id`."""
        ...

    def get_cues(self) -> List[Cue]:
        """Queries all records from the `color` table."""
        ...

    def get_cue_by_id(self, id: int) -> Optional[Cue]:
        """Queries a record from the `cue` table by its primary key `id`."""
        ...

    def update_cue(self, cue: Cue) -> Cue:
        """Updates an existing record in the `cue` table."""
        ...

    def delete_cue(self, id: int) -> int:
        """Deletes a record from the `cue` table by its primary key `id`."""
        ...

    def get_genres(self) -> List[Genre]:
        """Queries all records from the `genre` table."""
        ...

    def get_genre_by_id(self, id: int) -> Optional[Genre]:
        """Queries a record from the `genre` table by its primary key `id`."""
        ...

    def get_genre_by_name(self, name: str) -> Optional[Genre]:
        """Queries a record from the `genre` table by its `name`."""
        ...

    def insert_genre(self, genre: NewGenre) -> Genre:
        """Inserts a new record into the `genre` table."""
        ...

    def create_genre(self, name: str) -> Genre:
        """Creates a new record and inserts it into the `genre` table."""
        ...

    def update_genre(self, genre: Genre) -> Genre:
        """Updates an existing record in the `genre` table."""
        ...

    def delete_genre(self, id: int) -> int:
        """Deletes a record from the `genre` table by its primary key `id`."""
        ...

    def get_histories(self) -> List[History]:
        """Queries all records from the `history` table."""
        ...

    def get_history_children(self, id: int) -> List[History]:
        """Queries all child records from the `history` table by parent `id`."""
        ...

    def get_history_contents(self, id: int) -> List[Content]:
        """Queries all `content` records associated with a `history` record."""
        ...

    def get_history_by_id(self, id: int) -> Optional[History]:
        """Queries a record from the `history` table by its primary key `id`."""
        ...

    def insert_history(self, history: NewHistory) -> History:
        """Inserts a new record into the `history` table."""
        ...

    def create_history(self, name: str, attribute: HistoryType) -> History:
        """Creates a new record and inserts it into the `history` table."""
        ...

    def delete_history(self, id: int) -> int:
        """Deletes a record from the `history` table by its primary key `id`."""
        ...

    def create_history_content(self, history_id: int, content_id: int) -> HistoryContent:
        """Creates a new record and inserts it into the `history_content` table."""
        ...

    def delete_history_content(self, history_id: int, content_id: int) -> int:
        """Deletes a record from the `history_content` table by its primary key."""
        ...

    def get_hot_cue_bank_lists(self) -> List[HotCueBankList]:
        """Queries all records from the `color` table."""
        ...

    def get_hot_cue_bank_list_by_id(self, id: int) -> Optional[HotCueBankList]:
        """Queries a record from the `hot_cue_bank_list` table by its primary key `id`."""
        ...

    def get_images(self) -> List[Image]:
        """Queries all records from the `image` table."""
        ...

    def get_image_by_id(self, id: int) -> Optional[Image]:
        """Queries a record from the `image` table by its primary key `id`."""
        ...

    def get_image_by_path(self, path: str) -> Optional[Image]:
        """Queries a record from the `image` table by its `path`."""
        ...

    def insert_image(self, image: NewImage) -> Image:
        """Inserts a new record into the `image` table."""
        ...

    def create_image(self, path: str) -> Image:
        """Creates a new record and inserts it into the `image` table."""
        ...

    def update_image(self, image: Image) -> Image:
        """Updates an existing record in the `image` table."""
        ...

    def delete_image(self, id: int) -> int:
        """Deletes a record from the `image` table by its primary key `id`."""
        ...

    def get_keys(self) -> List[Key]:
        """Queries all records from the `key` table."""
        ...

    def get_key_by_id(self, id: int) -> Optional[Key]:
        """Queries a record from the `key` table by its primary key `id`."""
        ...

    def get_key_by_name(self, name: str) -> Optional[Key]:
        """Queries a record from the `key` table by its `name`."""
        ...

    def insert_key(self, key: NewKey) -> Key:
        """Inserts a new record into the `key` table."""
        ...

    def create_key(self, name: str) -> Key:
        """Creates a new record and inserts it into the `key` table."""
        ...

    def update_key(self, key: Key) -> Key:
        """Updates an existing record in the `key` table."""
        ...

    def delete_key(self, id: int) -> int:
        """Deletes a record from the `key` table by its primary key `id`."""
        ...

    def get_labels(self) -> List[Label]:
        """Queries all records from the `label` table."""
        ...

    def get_label_by_id(self, id: int) -> Optional[Label]:
        """Queries a record from the `label` table by its primary key `id`."""
        ...

    def get_label_by_name(self, name: str) -> Optional[Label]:
        """Queries a record from the `label` table by its `name`."""
        ...

    def insert_label(self, label: NewLabel) -> Label:
        """Inserts a new record into the `label` table."""
        ...

    def create_label(self, name: str) -> Label:
        """Creates a new record and inserts it into the `label` table."""
        ...

    def update_label(self, label: Label) -> Label:
        """Updates an existing record in the `label` table."""
        ...

    def delete_label(self, id: int) -> int:
        """Deletes a record from the `label` table by its primary key `id`."""
        ...

    def get_menu_items(self) -> List[MenuItem]:
        """Queries all records from the `menuItem` table."""
        ...

    def get_menu_item_by_id(self, id: int) -> Optional[MenuItem]:
        """Queries a record from the `menuItem` table by its primary key `id`."""
        ...

    def get_menu_item_by_name(self, name: str) -> Optional[MenuItem]:
        """Queries a record from the `menuItem` table by its `name`."""
        ...

    def insert_menu_item(self, menu_item: NewMenuItem) -> MenuItem:
        """Inserts a new record into the `menuItem` table."""
        ...

    def create_menu_item(self, name: str, kind: int) -> MenuItem:
        """Creates a new record and inserts it into the `menuItem` table."""
        ...

    def update_menu_item(self, menu_item: MenuItem) -> MenuItem:
        """Updates an existing record in the `menuItem` table."""
        ...

    def delete_menu_item(self, id: int) -> int:
        """Deletes a record from the `menuItem` table by its primary key `id`."""
        ...

    def get_my_tags(self) -> List[MyTag]:
        """Queries all records from the `myTag` table."""
        ...

    def get_my_tag_children(self, id: int) -> List[MyTag]:
        """Queries all child records from the `myTag` table by parent `id`."""
        ...

    def get_my_tag_contents(self, id: int) -> List[Content]:
        """Queries all `content` records associated with a `myTag` record."""
        ...

    def get_my_tag_by_id(self, id: int) -> Optional[MyTag]:
        """Queries a record from the `myTag` table by its primary key `id`."""
        ...

    def get_my_tag_by_content_id(self, cid: int) -> List[MyTag]:
        """Queries all records from the `myTag` table their associated `content_id` via `myTag_content`"""
        ...

    def insert_my_tag(self, my_tag: NewMyTag) -> MyTag:
        """Inserts a new record into the `myTag` table."""
        ...

    def create_my_tag(self, name: str, parent_id: Optional[int], seq: Optional[int]) -> MyTag:
        """Creates a new tag record and inserts it into the `myTag` table."""
        ...

    def create_my_tag_folder(self, name: str, parent_id: Optional[int], seq: Optional[int]) -> MyTag:
        """Creates a new my_tag folder record and inserts it into the `myTag` table."""
        ...

    def rename_my_tag(self, my_tag_id: int, name: str) -> int:
        """Renames an existing my_tag record in the `myTag` table."""
        ...

    def move_my_tag(self, my_tag_id: int, parent_id: Optional[int], seq: Optional[int]) -> int:
        """Moves a tag to a new parent folder or changes its sequence within the same parent."""
        ...

    def move_my_tags(self, my_tag_ids: List[int], parent_id: int, start_seq: Optional[int]) -> int:
        """Moves multiple tags to a new parent folder or changes its sequence within the same parent."""
        ...

    def delete_my_tag(self, id: int) -> int:
        """Deletes a record from the `myTag` table by its primary key `id`."""
        ...

    def delete_my_tags(self, ids: List[int]) -> int:
        """Deletes multiple records from the `myTag` table by its primary keys `id`."""
        ...

    def delete_my_tag_content(self, my_tag_id: int, content_id: int) -> int:
        """Deletes a record from the `myTag_content` table by its primary key."""
        ...

    def delete_my_tag_contents(self, ids: List[Tuple[int, int]]) -> int:
        """Deletes a record from the `myTag_content` table by its primary key."""
        ...

    def get_playlists(self) -> List[Playlist]:
        """Queries all records from the `playlist` table."""
        ...

    def get_playlist_children(self, id: int) -> List[Playlist]:
        """Queries all child records from the `playlist` table by parent `id`."""
        ...

    def get_playlist_contents(self, id: int) -> List[Content]:
        """Queries all `content` records associated with a `playlist` record."""
        ...

    def get_playlist_by_id(self, id: int) -> Optional[Playlist]:
        """Queries a record from the `playlist` table by its primary key `id`."""
        ...

    def insert_playlist(self, playlist: NewPlaylist) -> Playlist:
        """Inserts a new record into the `playlist` table."""
        ...

    def create_playlist(self, name: str, parent_id: Optional[int], seq: Optional[int]) -> Playlist:
        """Creates a new playlist record and inserts it into the `playlist` table."""
        ...

    def create_playlist_folder(self, name: str, parent_id: Optional[int], seq: Optional[int]) -> Playlist:
        """Creates a new playlist folder record and inserts it into the `playlist` table."""
        ...

    def rename_playlist(self, playlist_id: int, name: str) -> int:
        """Renames an existing playlist record in the `playlist` table."""
        ...

    def move_playlist(self, playlist_id: int, parent_id: Optional[int], seq: Optional[int]) -> int:
        """Moves a playlist to a new parent folder or changes its sequence within the same parent."""
        ...

    def move_playlists(self, playlist_ids: List[int], parent_id: int, start_seq: Optional[int]) -> int:
        """Moves multiple playlists to a new parent folder or changes its sequence within the same parent."""
        ...

    def delete_playlist(self, id: int) -> int:
        """Deletes a record from the `playlist` table by its primary key `id`."""
        ...

    def delete_playlists(self, ids: List[int]) -> int:
        """Deletes multiple records from the `playlist` table by its primary keys `id`."""
        ...

    def create_playlist_content(self, playlist_id: int, content_id: int, seq: Optional[int]) -> PlaylistContent:
        """Creates a new record and inserts it into the `playlist_content` table."""
        ...

    def move_playlist_content(self, playlist_id: int, content_id: int, seq: Optional[int]) -> int:
        """Moves a playlist content to a new sequence number."""
        ...

    def move_playlist_contents(self, ids: List[Tuple[int, int]], start_seq: Optional[int]) -> int:
        """Moves multiple playlist contents to a new sequence number."""
        ...

    def delete_playlist_content(self, playlist_id: int, content_id: int) -> int:
        """Deletes a record from the `playlist_content` table by its primary key."""
        ...

    def delete_playlist_contents(self, ids: List[Tuple[int, int]]) -> int:
        """Deletes a record from the `playlist_content` table by its primary key."""
        ...

    def get_properties(self) -> List[Property]:
        """Queries all records from the `property` table."""
        ...

    def get_property_by_device(self, device_name: str) -> Optional[Property]:
        """Queries a record from the `property` table by its primary key `device_name`."""
        ...

    def get_first_property(self) -> Optional[Property]:
        """Queries a record from the `property` table by its primary key `device_name`."""
        ...

    def insert_property(self, property: Property) -> Property:
        """Inserts a new record into the `property` table."""
        ...

    def create_property(self, device_name: str, my_tag_master_dbid: int) -> Property:
        """Creates a new record and inserts it into the `property` table."""
        ...

    def update_property(self, property: Property) -> Property:
        """Updates an existing record in the `property` table."""
        ...

    def delete_property(self, device_name: str) -> int:
        """Deletes a record from the `property` table by its primary key `device_name`."""
        ...

    def get_recommend_likes(self) -> List[RecommendedLike]:
        """Queries all records from the `recommendLike` table."""
        ...

    def get_recommend_like_by_id(self, content_id_1: int, content_id_2: int) -> Optional[RecommendedLike]:
        """Queries a record from the `recommendLike` table by its primary key `id`."""
        ...

    def get_recommend_like_by_content_id(self, content_id: int) -> List[RecommendedLike]:
        """Queries all records from the `recommendedLike` table that are linked to the given `content`."""
        ...

    def get_recommend_like_contents(self, content_id: int) -> List[int]:
        """Get related `content` ids for a given `content`."""
        ...

    def insert_recommend_like(self, recommend_like: RecommendedLike) -> RecommendedLike:
        """Inserts a new record into the `recommendLike` table."""
        ...

    def create_recommend_like(self, content_id_1: int, content_id_2: int, rating: Optional[int]) -> RecommendedLike:
        """Creates a new record and inserts it into the `recommendLike` table."""
        ...

    def update_recommend_like(self, recommend_like: RecommendedLike) -> RecommendedLike:
        """Updates an existing record in the `recommendLike` table."""
        ...

    def delete_recommend_like(self, content_id_1: int, content_id_2: int) -> int:
        """Deletes a record from the `recommendLike` table by its primary key."""
        ...

    def delete_recommend_like_by_content(self, content_id: int) -> int:
        """Deletes a record from the `recommendLike` table by its primary key."""
        ...

    def get_sorts(self) -> List[Sort]:
        """Queries all records from the `sort` table."""
        ...

    def get_sort_by_id(self, id: int) -> Optional[Sort]:
        """Queries a record from the `sort` table by its primary key `id`."""
        ...

    def insert_sort(self, sort: NewSort) -> Sort:
        """Inserts a new record into the `sort` table."""
        ...

    def create_sort(self, menu_item_id: int, seq: Optional[int], is_visible: Optional[int]) -> Sort:
        """Creates a new record and inserts it into the `sort` table."""
        ...

    def update_sort(self, sort: Sort) -> Sort:
        """Updates an existing record in the `sort` table."""
        ...

    def delete_sort(self, id: int) -> int:
        """Deletes a record from the `sort` table by its primary key `id`."""
        ...

# ---- ANLZ snapshots and handlers ---------------------------------------------------------------

class Beat:
    beat_number: int
    tempo: int
    time: int
    def __init__(self, beat_number: int, tempo: int, time: int) -> None: ...

class ExtBeat:
    microseconds: int
    def __init__(self, microseconds: int) -> None: ...

class BeatGrid:
    beats: List[Beat]
    header_flags: int
    def __init__(self, beats: List[Beat], header_flags: int) -> None: ...
    @staticmethod
    def new_static_bpm(bpm: float, first_beat_ms: int, end_ms: int) -> BeatGrid: ...
    def set_static_bpm(self, bpm: float, first_beat_ms: int, end_ms: int) -> None: ...

class ExtendedBeatGrid:
    first_beat: Beat
    last_beat: Beat
    offset_ms: int
    offset_flags: int
    checksum: int
    beats: List[ExtBeat]
    version: int
    def __init__(
        self,
        first_beat: Beat,
        last_beat: Beat,
        offset_ms: int,
        offset_flags: int,
        checksum: int,
        beats: List[ExtBeat],
        version: int,
    ) -> None: ...

class AnlzCue:
    hot_cue: int
    status: int
    order_first: int
    order_last: int
    cue_type: int
    time: int
    loop_time: int
    header_size: int
    u4: int
    u5: int
    u6: int
    u7: int
    def __init__(
        self,
        hot_cue: int,
        status: int,
        order_first: int,
        order_last: int,
        cue_type: int,
        time: int,
        loop_time: int,
        header_size: int,
        u4: int,
        u5: int,
        u6: int,
        u7: int,
    ) -> None: ...

class ExtendedCue:
    hot_cue: int
    cue_type: int
    time: int
    loop_time: int
    color: int
    loop_numerator: int
    loop_denominator: int
    comment: str
    hot_cue_color_index: int
    hot_cue_color_red: int
    hot_cue_color_green: int
    hot_cue_color_blue: int
    header_size: int
    u3: int
    u4: int
    u5: int
    comment_trailing_nuls: int
    trailing_data: bytes
    def __init__(
        self,
        hot_cue: int,
        cue_type: int,
        time: int,
        loop_time: int,
        color: int,
        loop_numerator: int,
        loop_denominator: int,
        comment: str,
        hot_cue_color_index: int,
        hot_cue_color_red: int,
        hot_cue_color_green: int,
        hot_cue_color_blue: int,
        header_size: int,
        u3: int,
        u4: int,
        u5: int,
        comment_trailing_nuls: int,
        trailing_data: Union[bytes, List[int]],
    ) -> None: ...

class CueList:
    list_type: int
    cues: List[AnlzCue]
    memory_count: int
    def __init__(self, list_type: int, cues: List[AnlzCue], memory_count: int) -> None: ...

class ExtendedCueList:
    list_type: int
    cues: List[ExtendedCue]
    def __init__(self, list_type: int, cues: List[ExtendedCue]) -> None: ...

class TinyWaveformColumn:
    height: int
    unused: int
    def __init__(self, height: int, unused: int) -> None: ...

class TinyWaveformPreview:
    data: List[TinyWaveformColumn]
    def __init__(self, data: List[TinyWaveformColumn]) -> None: ...

class WaveformColumn:
    height: int
    whiteness: int
    def __init__(self, height: int, whiteness: int) -> None: ...

class WaveformPreview:
    data: List[WaveformColumn]
    def __init__(self, data: List[WaveformColumn]) -> None: ...

class WaveformDetail:
    data: List[WaveformColumn]
    def __init__(self, data: List[WaveformColumn]) -> None: ...

class WaveformColorPreviewColumn:
    energy_bottom_half_freq: int
    energy_bottom_third_freq: int
    energy_mid_third_freq: int
    energy_top_third_freq: int
    u1: int
    u2: int
    def __init__(
        self,
        energy_bottom_half_freq: int,
        energy_bottom_third_freq: int,
        energy_mid_third_freq: int,
        energy_top_third_freq: int,
        u1: int,
        u2: int,
    ) -> None: ...

class WaveformColorPreview:
    data: List[WaveformColorPreviewColumn]
    def __init__(self, data: List[WaveformColorPreviewColumn]) -> None: ...

class WaveformColorDetailColumn:
    red: int
    green: int
    blue: int
    height: int
    low_bits: int
    fine_height_substep: int
    full_height: int
    def __init__(self, red: int, green: int, blue: int, height: int, low_bits: int) -> None: ...

class WaveformColorDetail:
    data: List[WaveformColorDetailColumn]
    unknown: int
    def __init__(self, data: List[WaveformColorDetailColumn], unknown: int) -> None: ...

class Waveform3BandColumn:
    mid: int
    high: int
    low: int
    def __init__(self, mid: int, high: int, low: int) -> None: ...

class Waveform3BandPreview:
    data: List[Waveform3BandColumn]
    def __init__(self, data: List[Waveform3BandColumn]) -> None: ...

class Waveform3BandDetail:
    data: List[Waveform3BandColumn]
    unknown: int
    def __init__(self, data: List[Waveform3BandColumn], unknown: int) -> None: ...

class Phrase:
    index: int
    beat: int
    kind: int
    k1: int
    k2: int
    b: int
    beat2: int
    beat3: int
    beat4: int
    k3: int
    fill: int
    beat_fill: int
    u1: int
    u2: int
    u3: int
    u4: int
    u5: int
    def __init__(
        self,
        index: int,
        beat: int,
        kind: int,
        k1: int,
        k2: int,
        b: int,
        beat2: int,
        beat3: int,
        beat4: int,
        k3: int,
        fill: int,
        beat_fill: int,
        u1: int,
        u2: int,
        u3: int,
        u4: int,
        u5: int,
    ) -> None: ...

class SongStructureData:
    mood: int
    end_beat: int
    bank: int
    phrases: List[Phrase]
    u1: int
    u2: int
    u3: int
    u4: int
    def __init__(
        self, mood: int, end_beat: int, bank: int, phrases: List[Phrase], u1: int, u2: int, u3: int, u4: int
    ) -> None: ...

class VBR:
    u1: int
    index: List[int]
    u2: int
    def __init__(self, u1: int, index: List[int], u2: int) -> None: ...

class SeekPoint:
    sample: int
    offset: int
    frame_samples: int
    def __init__(self, sample: int, offset: int, frame_samples: int) -> None: ...

class SeekIndex:
    u1: int
    total_samples: int
    entries: List[SeekPoint]
    def __init__(self, u1: int, total_samples: int, entries: List[SeekPoint]) -> None: ...

class Waveform3BandCalibration:
    low_gain: int
    mid_gain: int
    high_gain: int
    def __init__(self, low_gain: int, mid_gain: int, high_gain: int) -> None: ...

class VocalDetection:
    u1: int
    u2: int
    confidence: bytes
    def __init__(self, u1: int, u2: int, confidence: Union[bytes, List[int]]) -> None: ...

class SongStructure:
    version: int
    is_encrypted: bool
    data: SongStructureData
    def __init__(self, version: int, is_encrypted: bool, data: SongStructureData) -> None: ...

class UnknownSection:
    code: bytes
    header_data: bytes
    content_data: bytes
    def __init__(
        self, code: Union[bytes, List[int]], header_data: Union[bytes, List[int]], content_data: Union[bytes, List[int]]
    ) -> None: ...

class PyAnlz:
    def get_vbr(self) -> Optional[VBR]: ...
    def get_seek_index(self) -> Optional[SeekIndex]: ...
    def get_waveform_3band_calibration(self) -> Optional[Waveform3BandCalibration]: ...
    def get_vocal_detection(self) -> Optional[VocalDetection]: ...
    def __init__(self, path: str) -> None: ...
    def dump_copy(self, path: str) -> None: ...
    def dump(self) -> None: ...
    def contains(self, tag: str) -> bool: ...
    def get_tags(self) -> List[str]: ...
    def get_beat_grid(self) -> Optional[BeatGrid]: ...
    def get_extended_beat_grid(self) -> Optional[ExtendedBeatGrid]: ...
    def get_hot_cues(self) -> Optional[CueList]: ...
    def get_memory_cues(self) -> Optional[CueList]: ...
    def get_extended_hot_cues(self) -> Optional[ExtendedCueList]: ...
    def get_extended_memory_cues(self) -> Optional[ExtendedCueList]: ...
    def get_path(self) -> Optional[str]: ...
    def set_path(self, path: str) -> None: ...
    def get_vbr_data(self) -> Optional[bytes]: ...
    def get_tiny_waveform_preview(self) -> Optional[TinyWaveformPreview]: ...
    def get_waveform_preview(self) -> Optional[WaveformPreview]: ...
    def get_waveform_detail(self) -> Optional[WaveformDetail]: ...
    def get_waveform_color_preview(self) -> Optional[WaveformColorPreview]: ...
    def get_waveform_color_detail(self) -> Optional[WaveformColorDetail]: ...
    def get_waveform_3band_preview(self) -> Optional[Waveform3BandPreview]: ...
    def get_waveform_3band_detail(self) -> Optional[Waveform3BandDetail]: ...
    def get_song_structure(self) -> Optional[SongStructureData]: ...
    @staticmethod
    def new_dat(path: str) -> "PyAnlz": ...
    @staticmethod
    def new_ext(path: str) -> "PyAnlz": ...
    @staticmethod
    def new_2ex(path: str) -> "PyAnlz": ...
    @staticmethod
    def load(path: str) -> "PyAnlz": ...
    def section_count(self) -> int: ...
    def get_section(self, index: int) -> Optional[AnlzSection]: ...
    def get_section_by_tag(self, tag: str) -> Optional[AnlzSection]: ...
    def get_sections_by_tag(self, tag: str) -> List[AnlzSection]: ...
    def get_data(self) -> AnlzData: ...
    def into_data(self) -> AnlzData: ...
    def set_section(self, section: AnlzSection) -> None: ...
    def set_section_at(self, index: int, section: AnlzSection) -> None: ...
    def insert_section(self, index: int, section: AnlzSection) -> None: ...
    def push_section(self, section: AnlzSection) -> None: ...
    def remove_section(self, index: int) -> None: ...
    def remove_section_by_tag(self, tag: str) -> bool: ...
    def remove_sections_by_tag(self, tag: str) -> int: ...
    def remove_path(self) -> bool: ...
    def get_song_structure_section(self) -> Optional[SongStructure]: ...
    def set_song_structure_data(self, value: SongStructureData) -> None: ...
    def set_beat_grid(self, value: BeatGrid) -> None: ...
    def remove_beat_grid(self) -> bool: ...
    def set_extended_beat_grid(self, value: ExtendedBeatGrid) -> None: ...
    def remove_extended_beat_grid(self) -> bool: ...
    def set_cue_list(self, value: CueList) -> None: ...
    def remove_cue_list(self) -> bool: ...
    def set_extended_cue_list(self, value: ExtendedCueList) -> None: ...
    def remove_extended_cue_list(self) -> bool: ...
    def set_vbr(self, value: VBR) -> None: ...
    def remove_vbr(self) -> bool: ...
    def set_seek_index(self, value: SeekIndex) -> None: ...
    def remove_seek_index(self) -> bool: ...
    def set_tiny_waveform_preview(self, value: TinyWaveformPreview) -> None: ...
    def remove_tiny_waveform_preview(self) -> bool: ...
    def set_waveform_preview(self, value: WaveformPreview) -> None: ...
    def remove_waveform_preview(self) -> bool: ...
    def set_waveform_detail(self, value: WaveformDetail) -> None: ...
    def remove_waveform_detail(self) -> bool: ...
    def set_waveform_color_preview(self, value: WaveformColorPreview) -> None: ...
    def remove_waveform_color_preview(self) -> bool: ...
    def set_waveform_color_detail(self, value: WaveformColorDetail) -> None: ...
    def remove_waveform_color_detail(self) -> bool: ...
    def set_waveform_3band_preview(self, value: Waveform3BandPreview) -> None: ...
    def remove_waveform_3band_preview(self) -> bool: ...
    def set_waveform_3band_detail(self, value: Waveform3BandDetail) -> None: ...
    def remove_waveform_3band_detail(self) -> bool: ...
    def set_waveform_3band_calibration(self, value: Waveform3BandCalibration) -> None: ...
    def remove_waveform_3band_calibration(self) -> bool: ...
    def set_song_structure(self, value: SongStructure) -> None: ...
    def remove_song_structure(self) -> bool: ...
    def set_vocal_detection(self, value: VocalDetection) -> None: ...
    def remove_vocal_detection(self) -> bool: ...
    def set_hot_cues(self, value: CueList) -> None: ...
    def remove_hot_cues(self) -> bool: ...
    def set_memory_cues(self, value: CueList) -> None: ...
    def remove_memory_cues(self) -> bool: ...
    def set_extended_hot_cues(self, value: ExtendedCueList) -> None: ...
    def remove_extended_hot_cues(self) -> bool: ...
    def set_extended_memory_cues(self, value: ExtendedCueList) -> None: ...
    def remove_extended_memory_cues(self) -> bool: ...

# ---- Settings  ------------------------------------------------------------------------------------

class PySetting:
    def __init__(self, path: str) -> "PySetting": ...
    @classmethod
    def new_mysetting(cls, path: str) -> "PySetting": ...
    @classmethod
    def new_mysetting2(cls, path: str) -> "PySetting": ...
    @classmethod
    def new_djmmysetting(cls, path: str) -> "PySetting": ...
    @classmethod
    def new_devsetting(cls, path: str) -> "PySetting": ...
    @classmethod
    def load(cls, path: str) -> "PySetting": ...
    def dump_copy(self, path: str) -> None: ...
    def dump(self) -> None: ...
    def get_on_air_display(self) -> str: ...
    def set_on_air_display(self, on_air_display: str) -> None: ...
    def get_lcd_brightness(self) -> str: ...
    def set_lcd_brightness(self, lcd_brightness: str) -> None: ...
    def get_quantize(self) -> str: ...
    def set_quantize(self, quantize: str) -> None: ...
    def get_auto_cue_level(self) -> str: ...
    def set_auto_cue_level(self, auto_cue_level: str) -> None: ...
    def get_language(self) -> str: ...
    def set_language(self, language: str) -> None: ...
    def get_jog_ring_brightness(self) -> str: ...
    def set_jog_ring_brightness(self, jog_ring_brightness: str) -> None: ...
    def get_jog_ring_indicator(self) -> str: ...
    def set_jog_ring_indicator(self, jog_ring_indicator: str) -> None: ...
    def get_slip_flashing(self) -> str: ...
    def set_slip_flashing(self, slip_flashing: str) -> None: ...
    def get_disc_slot_illumination(self) -> str: ...
    def set_disc_slot_illumination(self, disc_slot_illumination: str) -> None: ...
    def get_eject_lock(self) -> str: ...
    def set_eject_lock(self, eject_lock: str) -> None: ...
    def get_sync(self) -> str: ...
    def set_sync(self, sync: str) -> None: ...
    def get_play_mode(self) -> str: ...
    def set_play_mode(self, play_mode: str) -> None: ...
    def get_quantize_beat_value(self) -> str: ...
    def set_quantize_beat_value(self, quantize_beat_value: str) -> None: ...
    def get_hotcue_autoload(self) -> str: ...
    def set_hotcue_autoload(self, hotcue_autoload: str) -> None: ...
    def get_hotcue_color(self) -> str: ...
    def set_hotcue_color(self, hotcue_color: str) -> None: ...
    def get_needle_lock(self) -> str: ...
    def set_needle_lock(self, needle_lock: str) -> None: ...
    def get_time_mode(self) -> str: ...
    def set_time_mode(self, time_mode: str) -> None: ...
    def get_jog_mode(self) -> str: ...
    def set_jog_mode(self, jog_mode: str) -> None: ...
    def get_auto_cue(self) -> str: ...
    def set_auto_cue(self, auto_cue: str) -> None: ...
    def get_master_tempo(self) -> str: ...
    def set_master_tempo(self, master_tempo: str) -> None: ...
    def get_tempo_range(self) -> str: ...
    def set_tempo_range(self, tempo_range: str) -> None: ...
    def get_phase_meter(self) -> str: ...
    def set_phase_meter(self, phase_meter: str) -> None: ...
    def get_vinyl_speed_adjust(self) -> str: ...
    def set_vinyl_speed_adjust(self, vinyl_speed_adjust: str) -> None: ...
    def get_jog_display_mode(self) -> str: ...
    def set_jog_display_mode(self, jog_display_mode: str) -> None: ...
    def get_pad_button_brightness(self) -> str: ...
    def set_pad_button_brightness(self, pad_button_brightness: str) -> None: ...
    def get_jog_lcd_brightness(self) -> str: ...
    def set_jog_lcd_brightness(self, jog_lcd_brightness: str) -> None: ...
    def get_waveform_divisions(self) -> str: ...
    def set_waveform_divisions(self, waveform_divisions: str) -> None: ...
    def get_waveform(self) -> str: ...
    def set_waveform(self, waveform: str) -> None: ...
    def get_beat_jump_beat_value(self) -> str: ...
    def set_beat_jump_beat_value(self, beat_jump_beat_value: str) -> None: ...
    def get_channel_fader_curve(self) -> str: ...
    def set_channel_fader_curve(self, channel_fader_curve: str) -> None: ...
    def get_crossfader_curve(self) -> str: ...
    def set_crossfader_curve(self, crossfader_curve: str) -> None: ...
    def get_headphones_pre_eq(self) -> str: ...
    def set_headphones_pre_eq(self, headphones_pre_eq: str) -> None: ...
    def get_headphones_mono_split(self) -> str: ...
    def set_headphones_mono_split(self, headphones_mono_split: str) -> None: ...
    def get_beat_fx_quantize(self) -> str: ...
    def set_beat_fx_quantize(self, beat_fx_quantize: str) -> None: ...
    def get_mic_low_cut(self) -> str: ...
    def set_mic_low_cut(self, mic_low_cut: str) -> None: ...
    def get_talk_over_mode(self) -> str: ...
    def set_talk_over_mode(self, talk_over_mode: str) -> None: ...
    def get_talk_over_level(self) -> str: ...
    def set_talk_over_level(self, talk_over_level: str) -> None: ...
    def get_midi_channel(self) -> str: ...
    def set_midi_channel(self, midi_channel: str) -> None: ...
    def get_midi_button_type(self) -> str: ...
    def set_midi_button_type(self, midi_button_type: str) -> None: ...
    def get_mixer_display_brightness(self) -> str: ...
    def set_mixer_display_brightness(self, mixer_display_brightness: str) -> None: ...
    def get_mixer_indicator_brightness(self) -> str: ...
    def set_mixer_indicator_brightness(self, mixer_indicator_brightness: str) -> None: ...
    def get_channel_fader_curve_long_fader(self) -> str: ...
    def set_channel_fader_curve_long_fader(self, channel_fader_curve_long_fader: str) -> None: ...
    def get_overview_waveform_type(self) -> str: ...
    def set_overview_waveform_type(self, overview_waveform_type: str) -> None: ...
    def get_waveform_color(self) -> str: ...
    def set_waveform_color(self, waveform_color: str) -> None: ...
    def get_key_display_format(self) -> str: ...
    def set_key_display_format(self, key_display_format: str) -> None: ...
    def get_waveform_current_position(self) -> str: ...
    def set_waveform_current_position(self, waveform_current_position: str) -> None: ...

# ---- XML  -----------------------------------------------------------------------------------------

class XmlTempo(_RustMutableMapping):
    inizio: float
    bpm: float
    metro: str
    battito: int

    def __init__(self, inizio: float, bpm: float, metro: str, battito: int) -> None: ...

class XmlPositionMark(_RustMutableMapping):
    name: str
    mark_type: int
    num: int
    start: float
    end: Optional[float]

    def __init__(self, name: str, mark_type: int, num: int, start: float, end: Optional[float]) -> None: ...

class XmlPlaylistTrack:
    key: str

class XmlTrack(_RustMutableMapping):
    trackid: str
    location: str
    tempos: List[XmlTempo]
    position_marks: List[XmlPositionMark]
    name: Optional[str] = None
    artist: Optional[str] = None
    composer: Optional[str] = None
    album: Optional[str] = None
    grouping: Optional[str] = None
    genre: Optional[str] = None
    kind: Optional[str] = None
    size: Optional[int] = None
    totaltime: Optional[float] = None
    discnumber: Optional[int] = None
    tracknumber: Optional[int] = None
    year: Optional[int] = None
    averagebpm: Optional[float] = None
    datemodified: Optional[datetime] = None
    dateadded: Optional[datetime] = None
    bitrate: Optional[int] = None
    samplerate: Optional[float] = None
    comments: Optional[str] = None
    playcount: Optional[int] = None
    lastplayed: Optional[datetime] = None
    rating: Optional[int] = None
    remixer: Optional[str] = None
    tonality: Optional[str] = None
    label: Optional[str] = None
    mix: Optional[str] = None
    color: Optional[str] = None

    def __init__(self, track_id: str, location: str) -> None: ...

class XmlPlaylistNode:
    name: str
    node_type: int
    key_type: Optional[int]
    tracks: Optional[List[XmlPlaylistTrack]]
    nodes: Optional[List[XmlPlaylistNode]]

    def __init__(self, node_type: int) -> None: ...
    @classmethod
    def playlist(cls, name: str, key_type: int) -> "XmlPlaylistNode": ...
    @classmethod
    def folder(cls, name: str) -> "XmlPlaylistNode": ...
    def add_node(self, node: "XmlPlaylistNode") -> None: ...
    def new_playlist(self, name: str, key_type: int) -> "XmlPlaylistNode": ...
    def new_folder(self, name: str) -> "XmlPlaylistNode": ...
    def remove_node(self, index: int) -> None: ...
    def clear_nodes(self) -> None: ...
    def get_child(self, index: int) -> "XmlPlaylistNode": ...
    def find_child(self, name: str) -> Optional["XmlPlaylistNode"]: ...
    def get_track(self, index: int) -> XmlPlaylistTrack: ...
    def new_track(self, track_key: str) -> None: ...
    def add_track(self, track: XmlPlaylistTrack) -> None: ...
    def remove_track(self, track_key: str) -> None: ...
    def clear_tracks(self) -> None: ...

class PyRekordboxXml:
    def get_version(self) -> str: ...
    def set_version(self, value: str) -> None: ...
    def get_product_name(self) -> str: ...
    def set_product_name(self, value: str) -> None: ...
    def get_product_version(self) -> str: ...
    def set_product_version(self, value: str) -> None: ...
    def get_product_company(self) -> str: ...
    def set_product_company(self, value: str) -> None: ...
    root_playlist: XmlPlaylistNode
    tracks: List[XmlTrack]

    def __init__(self, path: str) -> None: ...
    @classmethod
    def load(cls, path: str) -> "PyRekordboxXml": ...
    def to_string(self) -> str: ...
    def dump_copy(self, path: str) -> None: ...
    def dump(self) -> None: ...
    # def get_tracks(self) -> List[XmlTrack]: ...
    def get_track(self, index: int) -> Optional[XmlTrack]: ...
    def get_track_by_id(self, track_id: str) -> Optional[XmlTrack]: ...
    def get_track_by_location(self, location: str) -> Optional[XmlTrack]: ...
    def get_track_by_key(self, key: str, key_type: int) -> Optional[XmlTrack]: ...
    def add_track(self, track: XmlTrack) -> None: ...
    def new_track(self, track_id: str, location: str) -> XmlTrack: ...
    def update_track(self, track: XmlTrack) -> None: ...
    def remove_track(self, track_id: str) -> None: ...
    def clear_tracks(self) -> None: ...
    # def get_playlist_root(self) -> XmlPlaylistNode: ...

# Shared core track models. Paths are exposed as pathlib.Path objects.

class TrackMetadata:
    path: Path
    file_type: Optional[int]
    rating: Optional[int]
    search_str: Optional[str]
    tag: Optional[str]
    title: Optional[str]
    artist: Optional[str]
    album: Optional[str]
    album_artist: Optional[str]
    genre: Optional[str]
    bpm: Optional[float]
    length: Optional[float]
    track_no: Optional[int]
    bit_rate: Optional[int]
    bit_depth: Optional[int]
    commnt: Optional[str]
    release_year: Optional[int]
    remixer: Optional[str]
    label: Optional[str]
    org_artist: Optional[str]
    key: Optional[str]
    stock_date: Optional[str]
    file_size: Optional[int]
    disc_no: Optional[int]
    composer: Optional[str]
    subtitle: Optional[str]
    sample_rate: Optional[int]
    release_date: Optional[str]
    date_created: Optional[str]
    lyricist: Optional[str]
    isrc: Optional[str]
    def __init__(self, path: Union[str, PathLike[str]]) -> None: ...

class Track:
    id: str
    uuid: str
    path: Path
    file_type: Optional[int]
    rating: Optional[int]
    search_str: Optional[str]
    tag: Optional[str]
    title: Optional[str]
    artist: Optional[str]
    album: Optional[str]
    album_artist: Optional[str]
    genre: Optional[str]
    bpm: Optional[float]
    length: Optional[float]
    track_no: Optional[int]
    bit_rate: Optional[int]
    bit_depth: Optional[int]
    commnt: Optional[str]
    release_year: Optional[int]
    remixer: Optional[str]
    label: Optional[str]
    org_artist: Optional[str]
    key: Optional[str]
    stock_date: Optional[str]
    file_size: Optional[int]
    disc_no: Optional[int]
    composer: Optional[str]
    subtitle: Optional[str]
    sample_rate: Optional[int]
    release_date: Optional[str]
    date_created: Optional[str]
    lyricist: Optional[str]
    isrc: Optional[str]
    def __init__(self, path: Union[str, PathLike[str]]) -> None: ...
    @staticmethod
    def from_metadata(metadata: TrackMetadata) -> Track: ...
    def to_metadata(self) -> TrackMetadata: ...

# Deprecated XML compatibility alias.
Tempo = XmlTempo

# Deprecated XML compatibility alias.
PositionMark = XmlPositionMark

# Deprecated XML compatibility alias.
PlaylistTrack = XmlPlaylistTrack

# Deprecated XML compatibility alias.
PlaylistNode = XmlPlaylistNode
