// Author: Dylan Jones
// Date:   2025-05-01

//! # Rekordbox master.db database schemas
//!
//! This module defines the Diesel schema for the Rekordbox SQLite database.
//! It includes table definitions and relationships for various entities such as
//! content, playlists, artists, albums, and more. These schema definitions are
//! used to map database tables to Rust structs and enable ORM operations.
//!
//! *Note*: The fields in the database tables have been renamed to follow Rust's
//! `snake_case` naming convention and to unify naming inconsistencies.

#![allow(non_snake_case)]

diesel::table! {
    agentRegistry (registry_id) {
        registry_id -> Text,
        created_at -> Text,
        updated_at -> Text,
        id_1 -> Nullable<Text>,
        id_2 -> Nullable<Text>,
        int_1 -> Nullable<Integer>,
        int_2 -> Nullable<Integer>,
        str_1 -> Nullable<Text>,
        str_2 -> Nullable<Text>,
        date_1 -> Nullable<Text>,
        date_2 -> Nullable<Text>,
        text_1 -> Nullable<Text>,
        text_2 -> Nullable<Text>,
    }
}

diesel::table! {
    cloudAgentRegistry (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        int_1 -> Nullable<Integer>,
        int_2 -> Nullable<Integer>,
        str_1 -> Nullable<Text>,
        str_2 -> Nullable<Text>,
        date_1 -> Nullable<Text>,
        date_2 -> Nullable<Text>,
        text_1 -> Nullable<Text>,
        text_2 -> Nullable<Text>,
    }
}

diesel::table! {
    contentActiveCensor (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "ActiveCensors"]
        active_censors -> Text,
        rb_activecensor_count -> Integer,
    }
}

diesel::table! {
    contentCue (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "Cues"]
        cues -> Text,
        rb_cue_count -> Integer,
    }
}

diesel::table! {
    contentFile (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "Path"]
        path -> Text,
        #[sql_name = "Hash"]
        hash -> Text,
        #[sql_name = "Size"]
        size -> Integer,
        rb_local_path -> Text,
        rb_insync_hash -> Nullable<Text>,
        rb_insync_local_usn -> Nullable<Integer>,
        rb_file_hash_dirty -> Nullable<Integer>,
        rb_local_file_status -> Nullable<Integer>,
        rb_in_progress -> Nullable<Integer>,
        rb_process_type -> Nullable<Integer>,
        rb_temp_path -> Nullable<Text>,
        rb_priority -> Nullable<Integer>,
        rb_file_size_dirty -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdActiveCensor (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "InMsec"]
        in_msec -> Nullable<Integer>,
        #[sql_name = "OutMsec"]
        out_msec -> Nullable<Integer>,
        #[sql_name = "Info"]
        info -> Nullable<Integer>,
        #[sql_name = "ParameterList"]
        parameter_list -> Nullable<Text>,
        #[sql_name = "ContentUUID"]
        content_uuid -> Nullable<Text>,
    }
}

diesel::table! {
    djmdAlbum (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Name"]
        name -> Text,
        #[sql_name = "AlbumArtistID"]
        album_artist_id -> Nullable<Text>,
        #[sql_name = "ImagePath"]
        image_path -> Nullable<Text>,
        #[sql_name = "Compilation"]
        compilation -> Integer,
        #[sql_name = "SearchStr"]
        search_str -> Nullable<Text>,
    }
}

diesel::table! {
    djmdArtist (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Name"]
        name -> Text,
        #[sql_name = "SearchStr"]
        search_str -> Nullable<Text>,
    }
}

diesel::table! {
    djmdCategory (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "MenuItemID"]
        menu_item_id -> Text,
        #[sql_name = "Seq"]
        seq -> Integer,
        #[sql_name = "Disable"]
        disable -> Integer,
        #[sql_name = "InfoOrder"]
        info_order -> Integer,
    }
}

diesel::table! {
    djmdColor (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ColorCode"]
        color_code -> Nullable<Text>,
        #[sql_name = "SortKey"]
        sort_key -> Integer,
        #[sql_name = "Commnt"]
        commnt -> Text,
    }
}

diesel::table! {
    djmdContent (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "FolderPath"]
        folder_path -> Nullable<Text>,
        #[sql_name = "FileNameL"]
        file_name_l -> Nullable<Text>,
        #[sql_name = "FileNameS"]
        file_name_s -> Nullable<Text>,
        #[sql_name = "Title"]
        title -> Nullable<Text>,
        #[sql_name = "ArtistID"]
        artist_id -> Nullable<Text>,
        #[sql_name = "AlbumID"]
        album_id -> Nullable<Text>,
        #[sql_name = "GenreID"]
        genre_id -> Nullable<Text>,
        #[sql_name = "BPM"]
        bpm -> Nullable<Integer>,
        #[sql_name = "Length"]
        length -> Nullable<Integer>,
        #[sql_name = "TrackNo"]
        track_no -> Nullable<Integer>,
        #[sql_name = "BitRate"]
        bit_rate -> Nullable<Integer>,
        #[sql_name = "BitDepth"]
        bit_depth -> Nullable<Integer>,
        #[sql_name = "Commnt"]
        commnt -> Nullable<Text>,
        #[sql_name = "FileType"]
        file_type -> Nullable<Integer>,
        #[sql_name = "Rating"]
        rating -> Nullable<Integer>,
        #[sql_name = "ReleaseYear"]
        release_year -> Nullable<Integer>,
        #[sql_name = "RemixerID"]
        remixer_id -> Nullable<Text>,
        #[sql_name = "LabelID"]
        label_id -> Nullable<Text>,
        #[sql_name = "OrgArtistID"]
        org_artist_id -> Nullable<Text>,
        #[sql_name = "KeyID"]
        key_id -> Nullable<Text>,
        #[sql_name = "StockDate"]
        stock_date -> Nullable<Text>,
        #[sql_name = "ColorID"]
        color_id -> Nullable<Text>,
        #[sql_name = "DJPlayCount"]
        dj_play_count -> Nullable<Integer>,
        #[sql_name = "ImagePath"]
        image_path -> Nullable<Text>,
        #[sql_name = "MasterDBID"]
        master_db_id -> Nullable<Text>,
        #[sql_name = "MasterSongID"]
        master_song_id -> Nullable<Text>,
        #[sql_name = "AnalysisDataPath"]
        analysis_data_path -> Nullable<Text>,
        #[sql_name = "SearchStr"]
        search_str -> Nullable<Text>,
        #[sql_name = "FileSize"]
        file_size -> Nullable<Integer>,
        #[sql_name = "DiscNo"]
        disc_no -> Nullable<Integer>,
        #[sql_name = "ComposerID"]
        composer_id -> Nullable<Text>,
        #[sql_name = "Subtitle"]
        subtitle -> Nullable<Text>,
        #[sql_name = "SampleRate"]
        sample_rate -> Nullable<Integer>,
        #[sql_name = "DisableQuantize"]
        disable_quantize -> Nullable<Integer>,
        #[sql_name = "Analysed"]
        analysed -> Nullable<Integer>,
        #[sql_name = "ReleaseDate"]
        release_date -> Nullable<Text>,
        #[sql_name = "DateCreated"]
        date_created -> Nullable<Text>,
        #[sql_name = "ContentLink"]
        content_link -> Nullable<Integer>,
        #[sql_name = "Tag"]
        tag -> Nullable<Text>,
        #[sql_name = "ModifiedByRBM"]
        modified_by_rbm -> Nullable<Text>,
        #[sql_name = "HotCueAutoLoad"]
        hot_cue_auto_load -> Nullable<Text>,
        #[sql_name = "DeliveryControl"]
        delivery_control -> Nullable<Text>,
        #[sql_name = "DeliveryComment"]
        delivery_comment -> Nullable<Text>,
        #[sql_name = "CueUpdated"]
        cue_updated -> Nullable<Text>,
        #[sql_name = "AnalysisUpdated"]
        analysis_updated -> Nullable<Text>,
        #[sql_name = "TrackInfoUpdated"]
        track_info_updated -> Nullable<Text>,
        #[sql_name = "Lyricist"]
        lyricist -> Nullable<Text>,
        #[sql_name = "ISRC"]
        isrc -> Nullable<Text>,
        #[sql_name = "SamplerTrackInfo"]
        sampler_track_info -> Nullable<Integer>,
        #[sql_name = "SamplerPlayOffset"]
        sampler_play_offset -> Nullable<Integer>,
        #[sql_name = "SamplerGain"]
        sampler_gain -> Nullable<Double>,
        #[sql_name = "VideoAssociate"]
        video_associate -> Nullable<Text>,
        #[sql_name = "LyricStatus"]
        lyric_status -> Nullable<Integer>,
        #[sql_name = "ServiceID"]
        service_id -> Nullable<Integer>,
        #[sql_name = "OrgFolderPath"]
        org_folder_path -> Nullable<Text>,
        #[sql_name = "Reserved1"]
        reserved1 -> Nullable<Text>,
        #[sql_name = "Reserved2"]
        reserved2 -> Nullable<Text>,
        #[sql_name = "Reserved3"]
        reserved3 -> Nullable<Text>,
        #[sql_name = "Reserved4"]
        reserved4 -> Nullable<Text>,
        #[sql_name = "ExtInfo"]
        ext_info -> Nullable<Text>,
        rb_file_id -> Nullable<Text>,
        #[sql_name = "DeviceID"]
        device_id -> Nullable<Text>,
        #[sql_name = "rb_LocalFolderPath"]
        rb_local_folder_path -> Nullable<Text>,
        #[sql_name = "SrcID"]
        src_id -> Nullable<Text>,
        #[sql_name = "SrcTitle"]
        src_title -> Nullable<Text>,
        #[sql_name = "SrcArtistName"]
        src_artist_name -> Nullable<Text>,
        #[sql_name = "SrcAlbumName"]
        src_album_name -> Nullable<Text>,
        #[sql_name = "SrcLength"]
        src_length -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdCue (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "InMsec"]
        in_msec -> Integer,
        #[sql_name = "InFrame"]
        in_frame -> Integer,
        #[sql_name = "InMpegFrame"]
        in_mpeg_frame -> Integer,
        #[sql_name = "InMpegAbs"]
        in_mpeg_abs -> Integer,
        #[sql_name = "OutMsec"]
        out_msec -> Integer,
        #[sql_name = "OutFrame"]
        out_frame -> Integer,
        #[sql_name = "OutMpegFrame"]
        out_mpeg_frame -> Integer,
        #[sql_name = "OutMpegAbs"]
        out_mpeg_abs -> Integer,
        #[sql_name = "Kind"]
        kind -> Integer,
        #[sql_name = "Color"]
        color -> Integer,
        #[sql_name = "ColorTableIndex"]
        color_table_index -> Nullable<Integer>,
        #[sql_name = "ActiveLoop"]
        active_loop -> Nullable<Integer>,
        #[sql_name = "Comment"]
        comment -> Nullable<Text>,
        #[sql_name = "BeatLoopSize"]
        beat_loop_size -> Nullable<Integer>,
        #[sql_name = "CueMicrosec"]
        cue_microsec -> Nullable<Integer>,
        #[sql_name = "InPointSeekInfo"]
        in_point_seek_info -> Nullable<Text>,
        #[sql_name = "OutPointSeekInfo"]
        out_point_seek_info -> Nullable<Text>,
        #[sql_name = "ContentUUID"]
        content_uuid -> Nullable<Text>,
    }
}

diesel::table! {
    djmdDevice (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "MasterDBID"]
        master_db_id -> Text,
        #[sql_name = "Name"]
        name -> Text,
    }
}

diesel::table! {
    djmdGenre (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Name"]
        name -> Text,
    }
}

diesel::table! {
    djmdHistory (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Seq"]
        seq -> Integer,
        #[sql_name = "Name"]
        name -> Text,
        #[sql_name = "Attribute"]
        attribute -> Integer,
        #[sql_name = "ParentID"]
        parent_id -> Text,
        #[sql_name = "DateCreated"]
        date_created -> Text,
    }
}

diesel::table! {
    djmdSongHistory (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "HistoryID"]
        history_id -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "TrackNo"]
        track_no -> Integer,
    }
}

diesel::table! {
    djmdHotCueBanklist (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Seq"]
        seq -> Integer,
        #[sql_name = "Name"]
        name -> Text,
        #[sql_name = "ImagePath"]
        image_path -> Nullable<Text>,
        #[sql_name = "Attribute"]
        attribute -> Integer,
        #[sql_name = "ParentID"]
        parent_id -> Text,
    }
}

diesel::table! {
    djmdSongHotCueBanklist (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "HotCueBanklistID"]
        hot_cue_banklist_id -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "TrackNo"]
        track_no -> Integer,
        #[sql_name = "CueID"]
        cue_id -> Text,
        #[sql_name = "InMsec"]
        in_msec -> Integer,
        #[sql_name = "InFrame"]
        in_frame -> Integer,
        #[sql_name = "InMpegFrame"]
        in_mpeg_frame -> Integer,
        #[sql_name = "InMpegAbs"]
        in_mpeg_abs -> Integer,
        #[sql_name = "OutMsec"]
        out_msec -> Integer,
        #[sql_name = "OutFrame"]
        out_frame -> Integer,
        #[sql_name = "OutMpegFrame"]
        out_mpeg_frame -> Integer,
        #[sql_name = "OutMpegAbs"]
        out_mpeg_abs -> Integer,
        #[sql_name = "Color"]
        color -> Integer,
        #[sql_name = "ColorTableIndex"]
        color_table_index -> Nullable<Integer>,
        #[sql_name = "ActiveLoop"]
        active_loop -> Nullable<Integer>,
        #[sql_name = "Comment"]
        comment -> Nullable<Text>,
        #[sql_name = "BeatLoopSize"]
        beat_loop_size -> Nullable<Integer>,
        #[sql_name = "CueMicrosec"]
        cue_microsec -> Nullable<Integer>,
        #[sql_name = "InPointSeekInfo"]
        in_point_seek_info -> Nullable<Text>,
        #[sql_name = "OutPointSeekInfo"]
        out_point_seek_info -> Nullable<Text>,
        #[sql_name = "HotCueBanklistUUID"]
        hot_cue_banklist_uuid -> Nullable<Text>,
    }
}

diesel::table! {
    djmdKey (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ScaleName"]
        name -> Text,
        #[sql_name = "Seq"]
        seq -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdLabel (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Name"]
        name -> Text,
    }
}

diesel::table! {
    djmdMenuItems (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Class"]
        class -> Integer,
        #[sql_name = "Name"]
        name -> Text,
    }
}

diesel::table! {
    djmdMixerParam (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "GainHigh"]
        gain_high -> Integer,
        #[sql_name = "GainLow"]
        gain_low -> Integer,
        #[sql_name = "PeakHigh"]
        peak_high -> Integer,
        #[sql_name = "PeakLow"]
        peak_low -> Integer,
    }
}

diesel::table! {
    djmdMyTag (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Seq"]
        seq -> Integer,
        #[sql_name = "Name"]
        name -> Text,
        #[sql_name = "Attribute"]
        attribute -> Integer,
        #[sql_name = "ParentID"]
        parent_id -> Text,
    }
}

diesel::table! {
    djmdSongMyTag (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "MyTagID"]
        my_tag_id -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "TrackNo"]
        track_no -> Integer,
    }
}

diesel::table! {
    djmdPlaylist (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Seq"]
        seq -> Integer,
        #[sql_name = "Name"]
        name -> Text,
        #[sql_name = "ImagePath"]
        image_path -> Nullable<Text>,
        #[sql_name = "Attribute"]
        attribute -> Integer,
        #[sql_name = "ParentID"]
        parent_id -> Text,
        #[sql_name = "SmartList"]
        smart_list -> Nullable<Text>,
    }
}

diesel::table! {
    djmdSongPlaylist (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "PlaylistID"]
        playlist_id -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "TrackNo"]
        track_no -> Integer,
    }
}

diesel::table! {
    djmdProperty (db_id) {
        #[sql_name = "DBID"]
        db_id -> Text,
        #[sql_name = "DBVersion"]
        db_version -> Text,
        #[sql_name = "BaseDBDrive"]
        base_db_drive -> Text,
        #[sql_name = "CurrentDBDrive"]
        current_db_drive -> Text,
        #[sql_name = "DeviceID"]
        device_id -> Text,
        #[sql_name = "Reserved1"]
        reserved1 -> Nullable<Text>,
        #[sql_name = "Reserved2"]
        reserved2 -> Nullable<Text>,
        #[sql_name = "Reserved3"]
        reserved3 -> Nullable<Text>,
        #[sql_name = "Reserved4"]
        reserved4 -> Nullable<Text>,
        #[sql_name = "Reserved5"]
        reserved5 -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    djmdCloudProperty (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Reserved1"]
        reserved1 -> Nullable<Text>,
        #[sql_name = "Reserved2"]
        reserved2 -> Nullable<Text>,
        #[sql_name = "Reserved3"]
        reserved3 -> Nullable<Text>,
        #[sql_name = "Reserved4"]
        reserved4 -> Nullable<Text>,
        #[sql_name = "Reserved5"]
        reserved5 -> Nullable<Text>,
    }
}

diesel::table! {
    djmdRecommendLike (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ContentID1"]
        content_id1 -> Text,
        #[sql_name = "ContentID2"]
        content_id2 -> Text,
        #[sql_name = "LikeRate"]
        like_rate -> Nullable<Integer>,
        #[sql_name = "DataCreatedH"]
        data_created_h -> Nullable<Integer>,
        #[sql_name = "DataCreatedL"]
        data_created_l -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdRelatedTracks (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Seq"]
        seq -> Integer,
        #[sql_name = "Name"]
        name -> Text,
        #[sql_name = "Attribute"]
        attribute -> Integer,
        #[sql_name = "ParentID"]
        parent_id -> Text,
        #[sql_name = "Criteria"]
        criteria -> Text,
    }
}

diesel::table! {
    djmdSongRelatedTracks (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "RelatedTracksID"]
        related_tracks_id -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "TrackNo"]
        track_no -> Integer,
    }
}

diesel::table! {
    djmdSampler (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Seq"]
        seq -> Integer,
        #[sql_name = "Name"]
        name -> Text,
        #[sql_name = "Attribute"]
        attribute -> Integer,
        #[sql_name = "ParentID"]
        parent_id -> Text,
    }
}

diesel::table! {
    djmdSongSampler (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "SamplerID"]
        sampler_id -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "TrackNo"]
        track_no -> Integer,
    }
}

diesel::table! {
    djmdSongTagList (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "ContentID"]
        content_id -> Text,
        #[sql_name = "TrackNo"]
        track_no -> Integer,
    }
}

diesel::table! {
    djmdSort (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "MenuItemID"]
        menu_item_id -> Text,
        #[sql_name = "Seq"]
        seq -> Integer,
        #[sql_name = "Disable"]
        disable -> Integer,
    }
}

diesel::table! {
    hotCueBanklistCue (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "HotCueBanklistID"]
        hot_cue_banklist_id -> Text,
        #[sql_name = "Cues"]
        cues -> Nullable<Text>,
        rb_cue_count -> Nullable<Integer>,
    }
}

diesel::table! {
    imageFile (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "TableName"]
        table_name -> Nullable<Text>,
        #[sql_name = "TargetUUID"]
        target_uuid -> Nullable<Text>,
        #[sql_name = "TargetID"]
        target_id -> Nullable<Text>,
        #[sql_name = "Path"]
        path -> Nullable<Text>,
        #[sql_name = "Hash"]
        hash -> Nullable<Text>,
        #[sql_name = "Size"]
        size -> Nullable<Integer>,
        rb_local_path -> Nullable<Text>,
        rb_insync_hash -> Nullable<Text>,
        rb_insync_local_usn -> Nullable<Integer>,
        rb_file_hash_dirty -> Nullable<Integer>,
        rb_local_file_status -> Nullable<Integer>,
        rb_in_progress -> Nullable<Integer>,
        rb_process_type -> Nullable<Integer>,
        rb_temp_path -> Nullable<Text>,
        rb_priority -> Nullable<Integer>,
        rb_file_size_dirty -> Nullable<Integer>,
    }
}

diesel::table! {
    settingFile (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "Path"]
        path -> Text,
        #[sql_name = "Hash"]
        hash -> Text,
        #[sql_name = "Size"]
        size -> Integer,
        rb_local_path -> Nullable<Text>,
        rb_insync_hash -> Nullable<Text>,
        rb_insync_local_usn -> Nullable<Integer>,
        rb_file_hash_dirty -> Nullable<Integer>,
        rb_file_size_dirty -> Nullable<Integer>,
    }
}

diesel::table! {
    uuidIDMap (id) {
        #[sql_name = "ID"]
        id -> Text,
        #[sql_name = "UUID"]
        uuid -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        #[sql_name = "TableName"]
        table_name -> Text,
        #[sql_name = "TargetUUID"]
        target_uuid -> Text,
        #[sql_name = "CurrentID"]
        current_id -> Text,
    }
}

diesel::joinable!(contentActiveCensor -> djmdContent (content_id));
diesel::joinable!(contentCue -> djmdContent (content_id));
diesel::joinable!(contentFile -> djmdContent (content_id));
diesel::joinable!(djmdActiveCensor -> djmdContent (content_id));
diesel::joinable!(djmdAlbum -> djmdArtist (album_artist_id));
diesel::joinable!(djmdCategory -> djmdMenuItems (menu_item_id));
diesel::joinable!(djmdContent -> djmdArtist (artist_id));
diesel::joinable!(djmdContent -> djmdAlbum (album_id));
diesel::joinable!(djmdContent -> djmdGenre (genre_id));
diesel::joinable!(djmdContent -> djmdLabel (label_id));
diesel::joinable!(djmdContent -> djmdKey (key_id));
diesel::joinable!(djmdContent -> djmdColor (color_id));
diesel::joinable!(djmdContent -> djmdDevice (device_id));
diesel::joinable!(djmdCue -> djmdContent (content_id));
diesel::joinable!(djmdSongHistory -> djmdHistory (history_id));
diesel::joinable!(djmdSongHistory -> djmdContent (content_id));
diesel::joinable!(djmdSongHotCueBanklist -> djmdContent (content_id));
diesel::joinable!(djmdMixerParam -> djmdContent (content_id));
diesel::joinable!(djmdSongPlaylist -> djmdContent (content_id));
diesel::joinable!(djmdSongRelatedTracks -> djmdContent (content_id));
diesel::joinable!(djmdSongSampler -> djmdContent (content_id));
diesel::joinable!(djmdSongTagList -> djmdContent (content_id));
diesel::joinable!(djmdSongMyTag -> djmdMyTag (my_tag_id));
diesel::joinable!(djmdSongMyTag -> djmdContent (content_id));
diesel::joinable!(djmdSort -> djmdMenuItems (menu_item_id));

diesel::allow_tables_to_appear_in_same_query!(
    contentActiveCensor,
    contentCue,
    contentFile,
    djmdActiveCensor,
    djmdAlbum,
    djmdArtist,
    djmdCategory,
    djmdColor,
    djmdContent,
    djmdCue,
    djmdDevice,
    djmdGenre,
    djmdHistory,
    djmdSongHistory,
    djmdHotCueBanklist,
    djmdSongHotCueBanklist,
    djmdKey,
    djmdLabel,
    djmdMenuItems,
    djmdMixerParam,
    djmdMyTag,
    djmdSongMyTag,
    djmdPlaylist,
    djmdSongPlaylist,
    djmdRecommendLike,
    djmdRelatedTracks,
    djmdSongRelatedTracks,
    djmdSampler,
    djmdSongSampler,
    djmdSongTagList,
    djmdSort,
    hotCueBanklistCue,
);
