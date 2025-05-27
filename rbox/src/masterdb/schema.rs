// Author: Dylan Jones
// Date:   2025-05-01
//
// Rekordbox master.db database schemas

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
    cloudAgentRegistry (ID) {
        ID -> Text,
        UUID -> Text,
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
    contentActiveCensor (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ContentID -> Nullable<Text>,
        ActiveCensors -> Nullable<Text>,
        rb_activecensor_count -> Nullable<Integer>,
    }
}

diesel::table! {
    contentCue (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ContentID -> Nullable<Text>,
        Cues -> Nullable<Text>,
        rb_cue_count -> Nullable<Integer>,
    }
}

diesel::table! {
    contentFile (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ContentID -> Nullable<Text>,
        Path -> Nullable<Text>,
        Hash -> Nullable<Text>,
        Size -> Nullable<Integer>,
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
    djmdActiveCensor (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ContentID -> Nullable<Text>,
        InMsec -> Nullable<Integer>,
        OutMsec -> Nullable<Integer>,
        Info -> Nullable<Integer>,
        ParameterList -> Nullable<Text>,
        ContentUUID -> Nullable<Text>,
    }
}

diesel::table! {
    djmdAlbum (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Name -> Nullable<Text>,
        AlbumArtistID -> Nullable<Text>,
        ImagePath -> Nullable<Text>,
        Compilation -> Nullable<Integer>,
        SearchStr -> Nullable<Text>,
    }
}

diesel::table! {
    djmdArtist (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Name -> Nullable<Text>,
        SearchStr -> Nullable<Text>,
    }
}

diesel::table! {
    djmdCategory (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        MenuItemID -> Nullable<Text>,
        Seq -> Nullable<Integer>,
        Disable -> Nullable<Integer>,
        InfoOrder -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdColor (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ColorCode -> Nullable<Text>,
        SortKey -> Nullable<Integer>,
        Commnt -> Nullable<Text>,
    }
}

diesel::table! {
    djmdContent (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,

        FolderPath -> Nullable<Text>,
        FileNameL -> Nullable<Text>,
        FileNameS -> Nullable<Text>,
        Title -> Nullable<Text>,
        ArtistID -> Nullable<Text>,
        AlbumID -> Nullable<Text>,
        GenreID -> Nullable<Text>,
        BPM -> Nullable<Integer>,
        Length -> Nullable<Integer>,
        TrackNo -> Nullable<Integer>,
        BitRate -> Nullable<Integer>,
        BitDepth -> Nullable<Integer>,
        Commnt -> Nullable<Text>,
        FileType -> Nullable<Integer>,
        Rating -> Nullable<Integer>,
        ReleaseYear -> Nullable<Integer>,
        RemixerID -> Nullable<Text>,
        LabelID -> Nullable<Text>,
        OrgArtistID -> Nullable<Text>,
        KeyID -> Nullable<Text>,
        StockDate -> Nullable<Text>,
        ColorID -> Nullable<Text>,
        DJPlayCount -> Nullable<Integer>,
        ImagePath -> Nullable<Text>,
        MasterDBID -> Nullable<Text>,
        MasterSongID -> Nullable<Text>,
        AnalysisDataPath -> Nullable<Text>,
        SearchStr -> Nullable<Text>,
        FileSize -> Nullable<Integer>,
        DiscNo -> Nullable<Integer>,
        ComposerID -> Nullable<Text>,
        Subtitle -> Nullable<Text>,
        SampleRate -> Nullable<Integer>,
        DisableQuantize -> Nullable<Integer>,
        Analysed -> Nullable<Integer>,
        ReleaseDate -> Nullable<Text>,
        DateCreated -> Nullable<Text>,
        ContentLink -> Nullable<Integer>,
        Tag -> Nullable<Text>,
        ModifiedByRBM -> Nullable<Text>,
        HotCueAutoLoad -> Nullable<Text>,
        DeliveryControl -> Nullable<Text>,
        DeliveryComment -> Nullable<Text>,
        CueUpdated -> Nullable<Text>,
        AnalysisUpdated -> Nullable<Text>,
        TrackInfoUpdated -> Nullable<Text>,
        Lyricist -> Nullable<Text>,
        ISRC -> Nullable<Text>,
        SamplerTrackInfo -> Nullable<Integer>,
        SamplerPlayOffset -> Nullable<Integer>,
        SamplerGain -> Nullable<Double>,
        VideoAssociate -> Nullable<Text>,
        LyricStatus -> Nullable<Integer>,
        ServiceID -> Nullable<Integer>,
        OrgFolderPath -> Nullable<Text>,
        Reserved1 -> Nullable<Text>,
        Reserved2 -> Nullable<Text>,
        Reserved3 -> Nullable<Text>,
        Reserved4 -> Nullable<Text>,
        ExtInfo -> Nullable<Text>,
        rb_file_id -> Nullable<Text>,
        DeviceID -> Nullable<Text>,
        rb_LocalFolderPath -> Nullable<Text>,
        SrcID -> Nullable<Text>,
        SrcTitle -> Nullable<Text>,
        SrcArtistName -> Nullable<Text>,
        SrcAlbumName -> Nullable<Text>,
        SrcLength -> Nullable<Integer>,
    }
}

diesel::joinable!(djmdContent -> djmdArtist (ArtistID));

diesel::table! {
    djmdCue (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ContentID -> Nullable<Text>,
        InMsec -> Nullable<Integer>,
        InFrame -> Nullable<Integer>,
        InMpegFrame -> Nullable<Integer>,
        InMpegAbs -> Nullable<Integer>,
        OutMsec -> Nullable<Integer>,
        OutFrame -> Nullable<Integer>,
        OutMpegFrame -> Nullable<Integer>,
        OutMpegAbs -> Nullable<Integer>,
        Kind -> Nullable<Integer>,
        Color -> Nullable<Integer>,
        ColorTableIndex -> Nullable<Integer>,
        ActiveLoop -> Nullable<Integer>,
        Comment -> Nullable<Text>,
        BeatLoopSize -> Nullable<Integer>,
        CueMicrosec -> Nullable<Integer>,
        InPointSeekInfo -> Nullable<Text>,
        OutPointSeekInfo -> Nullable<Text>,
        ContentUUID -> Nullable<Text>,
    }
}

diesel::table! {
    djmdDevice (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        MasterDBID -> Nullable<Text>,
        Name -> Nullable<Text>,
    }
}

diesel::table! {
    djmdGenre (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Name -> Nullable<Text>,
    }
}

diesel::table! {
    djmdHistory (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Seq -> Nullable<Integer>,
        Name -> Nullable<Text>,
        Attribute -> Nullable<Integer>,
        ParentID -> Nullable<Text>,
        DateCreated -> Nullable<Text>,
    }
}

diesel::table! {
    djmdSongHistory (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        HistoryID -> Nullable<Text>,
        ContentID -> Nullable<Text>,
        TrackNo -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdHotCueBanklist (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Seq -> Nullable<Integer>,
        Name -> Nullable<Text>,
        ImagePath -> Nullable<Text>,
        Attribute -> Nullable<Integer>,
        ParentID -> Nullable<Text>,
    }
}

diesel::table! {
    djmdSongHotCueBanklist (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ContentID -> Nullable<Text>,
        TrackNo -> Nullable<Integer>,
        CueID -> Nullable<Text>,
        InMsec -> Nullable<Integer>,
        InFrame -> Nullable<Integer>,
        InMpegFrame -> Nullable<Integer>,
        InMpegAbs -> Nullable<Integer>,
        OutMsec -> Nullable<Integer>,
        OutFrame -> Nullable<Integer>,
        OutMpegFrame -> Nullable<Integer>,
        OutMpegAbs -> Nullable<Integer>,
        Color -> Nullable<Integer>,
        ColorTableIndex -> Nullable<Integer>,
        ActiveLoop -> Nullable<Integer>,
        Comment -> Nullable<Text>,
        BeatLoopSize -> Nullable<Integer>,
        CueMicrosec -> Nullable<Integer>,
        InPointSeekInfo -> Nullable<Text>,
        OutPointSeekInfo -> Nullable<Text>,
        HotCueBanklistUUID -> Nullable<Text>,
    }
}

diesel::table! {
    djmdKey (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ScaleName -> Nullable<Text>,
        Seq -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdLabel (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Name -> Nullable<Text>,
    }
}

diesel::table! {
    djmdMenuItems (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Class -> Nullable<Integer>,
        Name -> Nullable<Text>,
    }
}

diesel::table! {
    djmdMixerParam (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ContentID -> Nullable<Text>,
        GainHigh -> Nullable<Integer>,
        GainLow -> Nullable<Integer>,
        PeakHigh -> Nullable<Integer>,
        PeakLow -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdMyTag (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Seq -> Nullable<Integer>,
        Name -> Nullable<Text>,
        Attribute -> Nullable<Integer>,
        ParentID -> Nullable<Text>,
    }
}

diesel::table! {
    djmdSongMyTag (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        MyTagID -> Nullable<Text>,
        ContentID -> Nullable<Text>,
        TrackNo -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdPlaylist (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Seq -> Nullable<Integer>,
        Name -> Nullable<Text>,
        ImagePath -> Nullable<Text>,
        Attribute -> Nullable<Integer>,
        ParentID -> Nullable<Text>,
        SmartList -> Nullable<Text>,
    }
}

diesel::table! {
    djmdSongPlaylist (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        PlaylistID -> Nullable<Text>,
        ContentID -> Nullable<Text>,
        TrackNo -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdProperty (DBID) {
        DBID -> Text,
        DBVersion -> Nullable<Text>,
        BaseDBDrive -> Nullable<Text>,
        CurrentDBDrive -> Nullable<Text>,
        DeviceID -> Nullable<Text>,
        Reserved1 -> Nullable<Text>,
        Reserved2 -> Nullable<Text>,
        Reserved3 -> Nullable<Text>,
        Reserved4 -> Nullable<Text>,
        Reserved5 -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    djmdCloudProperty (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Reserved1 -> Nullable<Text>,
        Reserved2 -> Nullable<Text>,
        Reserved3 -> Nullable<Text>,
        Reserved4 -> Nullable<Text>,
        Reserved5 -> Nullable<Text>,
    }
}

diesel::table! {
    djmdRecommendLike (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ContentID1 -> Nullable<Text>,
        ContentID2 -> Nullable<Text>,
        LikeRate -> Nullable<Integer>,
        DataCreatedH -> Nullable<Integer>,
        DataCreatedL -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdRelatedTracks (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Seq -> Nullable<Integer>,
        Name -> Nullable<Text>,
        Attribute -> Nullable<Integer>,
        ParentID -> Nullable<Text>,
        Criteria -> Nullable<Text>,
    }
}

diesel::table! {
    djmdSongRelatedTracks (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        RelatedTracksID -> Nullable<Text>,
        ContentID -> Nullable<Text>,
        TrackNo -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdSampler (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Seq -> Nullable<Integer>,
        Name -> Nullable<Text>,
        Attribute -> Nullable<Integer>,
        ParentID -> Nullable<Text>,
    }
}

diesel::table! {
    djmdSongSampler (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        SamplerID -> Nullable<Text>,
        ContentID -> Nullable<Text>,
        TrackNo -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdSongTagList (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        ContentID -> Nullable<Text>,
        TrackNo -> Nullable<Integer>,
    }
}

diesel::table! {
    djmdSort (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        MenuItemID -> Nullable<Text>,
        Seq -> Nullable<Integer>,
        Disable -> Nullable<Integer>,
    }
}

diesel::table! {
    hotCueBanklistCue (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        HotCueBanklistID -> Nullable<Text>,
        Cues -> Nullable<Text>,
        rb_cue_count -> Nullable<Integer>,
    }
}

diesel::table! {
    imageFile (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        TableName -> Nullable<Text>,
        TargetUUID -> Nullable<Text>,
        TargetID -> Nullable<Text>,
        Path -> Nullable<Text>,
        Hash -> Nullable<Text>,
        Size -> Nullable<Integer>,
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
    settingFile (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        Path -> Nullable<Text>,
        Hash -> Nullable<Text>,
        Size -> Nullable<Integer>,
        rb_local_path -> Nullable<Text>,
        rb_insync_hash -> Nullable<Text>,
        rb_insync_local_usn -> Nullable<Integer>,
        rb_file_hash_dirty -> Nullable<Integer>,
        rb_file_size_dirty -> Nullable<Integer>,
    }
}

diesel::table! {
    uuidIDMap (ID) {
        ID -> Text,
        UUID -> Text,
        rb_data_status -> Integer,
        rb_local_data_status -> Integer,
        rb_local_deleted -> Integer,
        rb_local_synced -> Integer,
        usn -> Nullable<Integer>,
        rb_local_usn -> Nullable<Integer>,
        created_at -> Text,
        updated_at -> Text,
        TableName -> Nullable<Text>,
        TargetUUID -> Nullable<Text>,
        CurrentID -> Nullable<Text>,
    }
}
