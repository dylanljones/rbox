# -*- coding: utf-8 -*-
# Author: Dylan Jones
# Date:   2025-06-01

from pathlib import Path
from typing import Dict, List, Optional, Union

from . import models
from ._rbox import PyMasterDb as _PyMasterDb
from .anlz import Anlz


class MasterDb:
    def __init__(self, path: Union[str, Path] = None) -> None:
        if path is None:
            db = _PyMasterDb.open()
        else:
            path = Path(path)
            if not path.exists():
                raise FileNotFoundError(f"File not found: {path}")
            db = _PyMasterDb(str(path))
        self._db = db

    @classmethod
    def open(cls) -> "MasterDb":
        return cls()

    def set_unsafe_writes(self, unsafe: bool) -> None:
        """Set unsafe writes mode."""
        self._db.set_unsafe_writes(unsafe)

    # -- AgentRegistry ----------------------------------------------------------------------------

    def get_agent_registry(self) -> List[models.AgentRegistry]:
        """Get all agent registry entries."""
        return self._db.get_agent_registry()

    def get_agent_registry_by_id(self, registry_id: str) -> Optional[models.AgentRegistry]:
        """Get agent registry by ID."""
        return self._db.get_agent_registry_by_id(registry_id)

    def get_local_usn(self) -> int:
        """Get local USN."""
        return self._db.get_local_usn()

    # -- CloudAgentRegistry -----------------------------------------------------------------------

    def get_cloud_agent_registry(self) -> List[models.CloudAgentRegistry]:
        """Get all cloud agent registry entries."""
        return self._db.get_cloud_agent_registry()

    def get_cloud_agent_registry_by_id(
        self, registry_id: str
    ) -> Optional[models.CloudAgentRegistry]:
        """Get cloud agent registry by ID."""
        return self._db.get_cloud_agent_registry_by_id(registry_id)

    # -- ContentActiveCensor ----------------------------------------------------------------------

    def get_content_active_censor(self) -> List[models.ContentActiveCensor]:
        """Get all content active censor entries."""
        return self._db.get_content_active_censor()

    def get_content_active_censor_by_id(
        self, censor_id: str
    ) -> Optional[models.ContentActiveCensor]:
        """Get content active censor by ID."""
        return self._db.get_content_active_censor_by_id(censor_id)

    # -- ContentCue -------------------------------------------------------------------------------

    def get_content_cue(self) -> List[models.ContentCue]:
        """Get all content cue entries."""
        return self._db.get_content_cue()

    def get_content_cue_by_id(self, cue_id: str) -> Optional[models.ContentCue]:
        """Get content cue by ID."""
        return self._db.get_content_cue_by_id(cue_id)

    # -- ContentFile ------------------------------------------------------------------------------

    def get_content_file(self) -> List[models.ContentFile]:
        """Get all content file entries."""
        return self._db.get_content_file()

    def get_content_file_by_id(self, file_id: str) -> Optional[models.ContentFile]:
        """Get content file by ID."""
        return self._db.get_content_file_by_id(file_id)

    # -- ActiveCensor -----------------------------------------------------------------------------

    def get_active_censor(self) -> List[models.DjmdActiveCensor]:
        """Get all active censor entries."""
        return self._db.get_active_censor()

    def get_active_censor_by_id(self, censor_id: str) -> Optional[models.DjmdActiveCensor]:
        """Get active censor by ID."""
        return self._db.get_active_censor_by_id(censor_id)

    # -- Album ------------------------------------------------------------------------------------

    def get_album(self) -> List[models.DjmdAlbum]:
        """Get all album entries."""
        return self._db.get_album()

    def get_album_by_id(self, album_id: str) -> Optional[models.DjmdAlbum]:
        """Get album by ID."""
        return self._db.get_album_by_id(album_id)

    def get_album_by_name(self, album_name: str) -> Optional[models.DjmdAlbum]:
        """Get album by name."""
        return self._db.get_album_by_name(album_name)

    def insert_album(
        self, name: str, artist_id: str = None, image_path: str = None, compilation: int = None
    ) -> models.DjmdAlbum:
        """Insert a new album."""
        return self._db.insert_album(name, artist_id, image_path, compilation)

    def update_album(self, album: models.DjmdAlbum) -> models.DjmdAlbum:
        """Update album."""
        return self._db.update_album(album)

    def delete_album(self, album_id: str) -> None:
        """Delete album."""
        self._db.delete_album(album_id)

    # -- Artist -----------------------------------------------------------------------------------

    def get_artist(self) -> List[models.DjmdArtist]:
        """Get all artist entries."""
        return self._db.get_artist()

    def get_artist_by_id(self, artist_id: str) -> Optional[models.DjmdArtist]:
        """Get artist by ID."""
        return self._db.get_artist_by_id(artist_id)

    def get_artist_by_name(self, artist_name: str) -> Optional[models.DjmdArtist]:
        """Get artist by name."""
        return self._db.get_artist_by_name(artist_name)

    def insert_artist(self, name: str) -> models.DjmdArtist:
        """Insert a new artist."""
        return self._db.insert_artist(name)

    def update_artist(self, artist: models.DjmdArtist) -> models.DjmdArtist:
        """Update artist."""
        return self._db.update_artist(artist)

    def delete_artist(self, artist_id: str) -> None:
        """Delete artist."""
        self._db.delete_artist(artist_id)

    # -- Category ---------------------------------------------------------------------------------

    def get_category(self) -> List[models.DjmdCategory]:
        """Get all category entries."""
        return self._db.get_category()

    def get_category_by_id(self, category_id: str) -> Optional[models.DjmdCategory]:
        """Get category by ID."""
        return self._db.get_category_by_id(category_id)

    # -- Color ------------------------------------------------------------------------------------

    def get_color(self) -> List[models.DjmdColor]:
        """Get all color entries."""
        return self._db.get_color()

    def get_color_by_id(self, color_id: str) -> Optional[models.DjmdColor]:
        """Get color by ID."""
        return self._db.get_color_by_id(color_id)

    # -- Content ----------------------------------------------------------------------------------

    def get_content(self) -> List[models.DjmdContent]:
        """Get all content from the database."""
        return self._db.get_content()

    def get_content_by_id(self, content_id: str) -> Optional[models.DjmdContent]:
        """Get content by ID."""
        return self._db.get_content_by_id(content_id)

    def get_content_by_path(self, path: str) -> Optional[models.DjmdContent]:
        """Get content by path."""
        return self._db.get_content_by_path(path)

    def get_content_anlz_dir(self, content_id: str) -> Path:
        """Get the ANLZ directory for a content ID."""
        return Path(self._db.get_content_anlz_dir(content_id))

    def get_content_anlz_paths(self, content_id: str) -> Dict[str, Path]:
        """Get the ANLZ paths for a content ID."""
        paths = self._db.get_content_anlz_paths(content_id)
        return {ext: Path(p) for ext, p in paths.items()}

    def get_content_anlz_files(self, content_id: str) -> Dict[str, Anlz]:
        """Get the ANLZ file handlers for a content ID."""
        paths = self._db.get_content_anlz_paths(content_id)
        return {ext: Anlz(p) for ext, p in paths.items()}

    def insert_content(self, path: Union[Path, str]) -> models.DjmdContent:
        """Insert a new content into the database."""
        return self._db.insert_content(str(path))

    def update_content(self, content: models.DjmdContent) -> None:
        """Update content in the database."""
        self._db.update_content(content)

    def update_content_album(self, content_id: str, name: str) -> None:
        self._db.update_content_album(content_id, name)

    def update_content_artist(self, content_id: str, name: str) -> None:
        self._db.update_content_artist(content_id, name)

    def update_content_remixer(self, content_id: str, name: str) -> None:
        self._db.update_content_remixer(content_id, name)

    def update_content_original_artist(self, content_id: str, name: str) -> None:
        self._db.update_content_original_artist(content_id, name)

    def update_content_composer(self, content_id: str, name: str) -> None:
        self._db.update_content_composer(content_id, name)

    def update_content_genre(self, content_id: str, name: str) -> None:
        self._db.update_content_genre(content_id, name)

    def update_content_label(self, content_id: str, name: str) -> None:
        self._db.update_content_label(content_id, name)

    def update_content_key(self, content_id: str, name: str) -> None:
        self._db.update_content_key(content_id, name)

    # def delete_content(self, content_id: str) -> None:
    #     """Delete content from the database."""
    #     self._db.delete_content(content_id)

    # -- Cue --------------------------------------------------------------------------------------

    def get_cue(self) -> List[models.DjmdCue]:
        """Get all cue entries."""
        return self._db.get_cue()

    def get_cue_by_id(self, cue_id: str) -> Optional[models.DjmdCue]:
        """Get cue by ID."""
        return self._db.get_cue_by_id(cue_id)

    # -- Device -----------------------------------------------------------------------------------

    def get_device(self) -> List[models.DjmdDevice]:
        """Get all device entries."""
        return self._db.get_device()

    def get_device_by_id(self, device_id: str) -> Optional[models.DjmdDevice]:
        """Get device by ID."""
        return self._db.get_device_by_id(device_id)

    # -- Genre ------------------------------------------------------------------------------------

    def get_genre(self) -> List[models.DjmdGenre]:
        """Get all genre entries."""
        return self._db.get_genre()

    def get_genre_by_id(self, genre_id: str) -> Optional[models.DjmdGenre]:
        """Get genre by ID."""
        return self._db.get_genre_by_id(genre_id)

    def get_genre_by_name(self, genre_name: str) -> Optional[models.DjmdGenre]:
        """Get genre by ID."""
        return self._db.get_genre_by_name(genre_name)

    def insert_genre(self, name: str) -> models.DjmdGenre:
        """Insert a new genre."""
        return self._db.insert_genre(name)

    def update_genre(self, genre: models.DjmdGenre) -> models.DjmdGenre:
        """Update genre."""
        return self._db.update_genre(genre)

    def delete_genre(self, genre_id: str) -> None:
        """Delete genre."""
        self._db.delete_genre(genre_id)

    # -- History ----------------------------------------------------------------------------------

    def get_history(self) -> List[models.DjmdHistory]:
        """Get all history entries."""
        return self._db.get_history()

    def get_history_by_id(self, history_id: str) -> Optional[models.DjmdHistory]:
        """Get history by ID."""
        return self._db.get_history_by_id(history_id)

    def get_history_songs(self, history_id: str) -> List[models.DjmdSongHistory]:
        """Get songs in a history by ID."""
        return self._db.get_history_songs(history_id)

    def get_history_contents(self, history_id: str) -> List[models.DjmdContent]:
        """Get contents in a history by ID."""
        return self._db.get_history_contents(history_id)

    # -- HotCueBanklist ---------------------------------------------------------------------------

    def get_hot_cue_banklist(self) -> List[models.DjmdHotCueBanklist]:
        """Get all hot cue banklist entries."""
        return self._db.get_hot_cue_banklist()

    def get_hot_cue_banklist_by_id(self, banklist_id: str) -> Optional[models.DjmdHotCueBanklist]:
        """Get hot cue banklist by ID."""
        return self._db.get_hot_cue_banklist_by_id(banklist_id)

    def get_hot_cue_banklist_children(self, banklist_id: str) -> List[models.DjmdHotCueBanklist]:
        """Get child hot cue banklists by ID."""
        return self._db.get_hot_cue_banklist_children(banklist_id)

    def get_hot_cue_banklist_songs(self, banklist_id: str) -> List[models.DjmdSongHotCueBanklist]:
        """Get songs in a hot cue banklist by ID."""
        return self._db.get_hot_cue_banklist_songs(banklist_id)

    def get_hot_cue_banklist_contents(self, banklist_id: str) -> List[models.DjmdContent]:
        """Get contents in a hot cue banklist by ID."""
        return self._db.get_hot_cue_banklist_contents(banklist_id)

    def get_hot_cue_banklist_cues(self, banklist_id: str) -> List[models.HotCueBanklistCue]:
        """Get cues in a hot cue banklist by ID."""
        return self._db.get_hot_cue_banklist_cues(banklist_id)

    # -- Key --------------------------------------------------------------------------------------

    def get_key(self) -> List[models.DjmdKey]:
        """Get all key entries."""
        return self._db.get_key()

    def get_key_by_id(self, key_id: str) -> Optional[models.DjmdKey]:
        """Get key by ID."""
        return self._db.get_key_by_id(key_id)

    def get_key_by_name(self, key_name: str) -> Optional[models.DjmdKey]:
        """Get key by name."""
        return self._db.get_key_by_name(key_name)

    def insert_key(self, name: str) -> models.DjmdKey:
        """Insert a new key."""
        return self._db.insert_key(name)

    def update_key(self, key: models.DjmdKey) -> models.DjmdKey:
        """Update key."""
        return self._db.update_key(key)

    def delete_key(self, key_id: str) -> None:
        """Delete key."""
        self._db.delete_key(key_id)

    # -- Label ------------------------------------------------------------------------------------

    def get_label(self) -> List[models.DjmdLabel]:
        """Get all label entries."""
        return self._db.get_label()

    def get_label_by_id(self, label_id: str) -> Optional[models.DjmdLabel]:
        """Get label by ID."""
        return self._db.get_label_by_id(label_id)

    def get_label_by_name(self, label_name: str) -> Optional[models.DjmdLabel]:
        """Get label by ID."""
        return self._db.get_label_by_name(label_name)

    def insert_label(self, name: str) -> models.DjmdLabel:
        """Insert a new label."""
        return self._db.insert_label(name)

    def update_label(self, label: models.DjmdLabel) -> models.DjmdLabel:
        """Update label."""
        return self._db.update_label(label)

    def delete_label(self, label_id: str) -> None:
        """Delete label."""
        self._db.delete_label(label_id)

    # -- MenuItems --------------------------------------------------------------------------------

    def get_menu_item(self) -> List[models.DjmdMenuItems]:
        """Get all menu items."""
        return self._db.get_menu_item()

    def get_menu_item_by_id(self, item_id: str) -> Optional[models.DjmdMenuItems]:
        """Get menu item by ID."""
        return self._db.get_menu_item_by_id(item_id)

    # -- MixerParam -------------------------------------------------------------------------------

    def get_mixer_param(self) -> List[models.DjmdMixerParam]:
        """Get all mixer parameters."""
        return self._db.get_mixer_param()

    def get_mixer_param_by_id(self, param_id: str) -> Optional[models.DjmdMixerParam]:
        """Get mixer parameter by ID."""
        return self._db.get_mixer_param_by_id(param_id)

    # -- MyTag ------------------------------------------------------------------------------------

    def get_my_tag(self) -> List[models.DjmdMyTag]:
        """Get all my tags."""
        return self._db.get_my_tag()

    def get_my_tag_children(self, tag_id: str) -> List[models.DjmdMyTag]:
        """Get child tags by ID."""
        return self._db.get_my_tag_children(tag_id)

    def get_my_tag_by_id(self, tag_id: str) -> Optional[models.DjmdMyTag]:
        """Get my tag by ID."""
        return self._db.get_my_tag_by_id(tag_id)

    def get_my_tag_songs(self, tag_id: str) -> List[models.DjmdSongMyTag]:
        """Get songs in a my tag by ID."""
        return self._db.get_my_tag_songs(tag_id)

    def get_my_tag_contents(self, tag_id: str) -> List[models.DjmdContent]:
        """Get contents in a my tag by ID."""
        return self._db.get_my_tag_contents(tag_id)

    # -- Playlists --------------------------------------------------------------------------------

    def get_playlist(self) -> List[models.DjmdPlaylist]:
        """Get all playlists from the database."""
        return self._db.get_playlist()

    def get_playlist_tree(self) -> List[models.DjmdPlaylistTreeItem]:
        """Get all playlists from the database."""
        return self._db.get_playlist_tree()

    def get_playlist_children(self, playlist_id: str) -> List[models.DjmdPlaylist]:
        """Get child playlists by ID."""
        return self._db.get_playlist_children(playlist_id)

    def get_playlist_by_id(self, playlist_id: str) -> Optional[models.DjmdPlaylist]:
        """Get playlist by ID."""
        return self._db.get_playlist_by_id(playlist_id)

    def get_playlist_by_path(self, path: List[str]) -> Optional[models.DjmdPlaylist]:
        """Get a playlist by a path of names."""
        return self._db.get_playlist_by_path(path)

    def get_playlist_songs(self, playlist_id: str) -> List[models.DjmdSongPlaylist]:
        """Get songs in a playlist by ID."""
        return self._db.get_playlist_songs(playlist_id)

    def get_playlist_contents(self, playlist_id: str) -> List[models.DjmdContent]:
        """Get contents in a playlist by ID."""
        return self._db.get_playlist_contents(playlist_id)

    def get_playlist_song_by_id(self, song_id: str) -> models.DjmdSongPlaylist:
        """Get playlist song by ID."""
        return self._db.get_playlist_song_by_id(song_id)

    def insert_playlist(
        self,
        name: str,
        attribute: int,
        parent_id: str = None,
        seq: int = None,
        image_path: str = None,
        smart_list: str = None,
    ) -> models.DjmdPlaylist:
        """Insert a new playlist."""
        return self._db.insert_playlist(name, attribute, parent_id, seq, image_path, smart_list)

    def rename_playlist(self, playlist_id: str, name: str) -> models.DjmdPlaylist:
        """Rename a playlist."""
        return self._db.rename_playlist(playlist_id, name)

    def move_playlist(
        self, playlist_id: str, seq: int = None, parent_id: str = None
    ) -> models.DjmdPlaylist:
        """Move a playlist."""
        return self._db.move_playlist(playlist_id, seq, parent_id)

    def delete_playlist(self, playlist_id: str) -> None:
        """Delete playlist from the database."""
        self._db.delete_playlist(playlist_id)

    def insert_playlist_song(
        self, playlist_id: str, content_id: str, seq: int = None
    ) -> models.DjmdSongPlaylist:
        """Insert a new song into a playlist."""
        return self._db.insert_playlist_song(playlist_id, content_id, seq)

    def move_playlist_song(self, song_id: str, seq: int) -> models.DjmdSongPlaylist:
        """Move a song in a playlist."""
        return self._db.move_playlist_song(song_id, seq)

    def delete_playlist_song(self, song_id: str) -> None:
        """Delete song from playlist."""
        self._db.delete_playlist_song(song_id)

    # -- Property ---------------------------------------------------------------------------------

    def get_property(self) -> List[models.DjmdProperty]:
        """Get all properties."""
        return self._db.get_property()

    def get_property_by_id(self, property_id: str) -> Optional[models.DjmdProperty]:
        """Get property by ID."""
        return self._db.get_property_by_id(property_id)

    # -- CloudProperty ----------------------------------------------------------------------------

    def get_cloud_property(self) -> List[models.DjmdCloudProperty]:
        """Get all cloud properties."""
        return self._db.get_cloud_property()

    def get_cloud_property_by_id(self, property_id: str) -> Optional[models.DjmdCloudProperty]:
        """Get cloud property by ID."""
        return self._db.get_cloud_property_by_id(property_id)

    # -- RecommendLike ----------------------------------------------------------------------------

    def get_recommend_like(self) -> List[models.DjmdRecommendLike]:
        """Get all recommend likes."""
        return self._db.get_recommend_like()

    def get_recommend_like_by_id(self, recommend_id: str) -> Optional[models.DjmdRecommendLike]:
        """Get recommend like by ID."""
        return self._db.get_recommend_like_by_id(recommend_id)

    # -- RelatedTracks ----------------------------------------------------------------------------

    def get_related_tracks(self) -> List[models.DjmdRelatedTracks]:
        """Get all related tracks."""
        return self._db.get_related_tracks()

    def get_related_tracks_children(self, track_id: str) -> List[models.DjmdRelatedTracks]:
        """Get child related tracks by ID."""
        return self._db.get_related_tracks_children(track_id)

    def get_related_tracks_by_id(self, track_id: str) -> Optional[models.DjmdRelatedTracks]:
        """Get related tracks by ID."""
        return self._db.get_related_tracks_by_id(track_id)

    def get_related_tracks_songs(self, track_id: str) -> List[models.DjmdSongRelatedTracks]:
        """Get songs in related tracks by ID."""
        return self._db.get_related_tracks_songs(track_id)

    def get_related_tracks_contents(self, track_id: str) -> List[models.DjmdContent]:
        """Get contents in related tracks by ID."""
        return self._db.get_related_tracks_contents(track_id)

    # -- Sampler ----------------------------------------------------------------------------------

    def get_sampler(self) -> List[models.DjmdSampler]:
        """Get all samplers."""
        return self._db.get_sampler()

    def get_sampler_children(self, sampler_id: str) -> List[models.DjmdSampler]:
        """Get child samplers by ID."""
        return self._db.get_sampler_children(sampler_id)

    def get_sampler_by_id(self, sampler_id: str) -> Optional[models.DjmdSampler]:
        """Get sampler by ID."""
        return self._db.get_sampler_by_id(sampler_id)

    def get_sampler_songs(self, sampler_id: str) -> List[models.DjmdSongSampler]:
        """Get songs in a sampler by ID."""
        return self._db.get_sampler_songs(sampler_id)

    def get_sampler_contents(self, sampler_id: str) -> List[models.DjmdContent]:
        """Get contents in a sampler by ID."""
        return self._db.get_sampler_contents(sampler_id)

    # -- SongTagList ------------------------------------------------------------------------------

    def get_song_tag_list(self) -> List[models.DjmdSongTagList]:
        """Get all song tag lists."""
        return self._db.get_song_tag_list()

    def get_song_tag_list_by_id(self, tag_list_id: str) -> models.DjmdSongTagList:
        """Get song tag list by ID."""
        return self._db.get_song_tag_list_by_id(tag_list_id)

    # -- Sort -------------------------------------------------------------------------------------

    def get_sort(self) -> List[models.DjmdSort]:
        """Get all sort entries."""
        return self._db.get_sort()

    def get_sort_by_id(self, sort_id: str) -> Optional[models.DjmdSort]:
        """Get sort by ID."""
        return self._db.get_sort_by_id(sort_id)

    # -- ImageFile --------------------------------------------------------------------------------

    def get_image_file(self) -> List[models.ImageFile]:
        """Get all image files."""
        return self._db.get_image_file()

    def get_image_file_by_id(self, image_id: str) -> Optional[models.ImageFile]:
        """Get image file by ID."""
        return self._db.get_image_file_by_id(image_id)

    # -- SettingFile ------------------------------------------------------------------------------

    def get_setting_file(self) -> List[models.SettingFile]:
        """Get all setting files."""
        return self._db.get_setting_file()

    def get_setting_file_by_id(self, setting_id: str) -> Optional[models.SettingFile]:
        """Get setting file by ID."""
        return self._db.get_setting_file_by_id(setting_id)

    # -- UuidIDMap --------------------------------------------------------------------------------

    def get_uuid_id_map(self) -> List[models.UuidIDMap]:
        """Get all UUID ID maps."""
        return self._db.get_uuid_id_map()

    def get_uuid_id_map_by_id(self, map_id: str) -> Optional[models.UuidIDMap]:
        """Get UUID ID map by ID."""
        return self._db.get_uuid_id_map_by_id(map_id)
