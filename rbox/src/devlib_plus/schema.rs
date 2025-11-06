// Author: Dylan Jones
// Date:   2025-08-16

//! Rekordbox exportLibrary.db database schemas

#![allow(non_snake_case)]

diesel::table! {
    album (id) {
        #[sql_name = "album_id"]
        id -> Integer,
        name -> Text,
        artist_id -> Nullable<Integer>,
        image_id -> Nullable<Integer>,
        #[sql_name = "isComplation"]
        is_complation -> Integer,
        #[sql_name = "nameForSearch"]
        name_for_search -> Nullable<Text>,
    }
}

diesel::table! {
    artist (id) {
        #[sql_name = "artist_id"]
        id -> Integer,
        name -> Text,
        #[sql_name = "nameForSearch"]
        name_for_search -> Nullable<Text>,
    }
}

diesel::table! {
    category (id) {
        #[sql_name = "category_id"]
        id -> Integer,
        #[sql_name = "menuItem_id"]
        menu_item_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        #[sql_name = "isVisible"]
        is_visible -> Integer,
    }
}

diesel::table! {
    color (id) {
        #[sql_name = "color_id"]
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    content (id) {
        #[sql_name = "content_id"]
        id -> Integer,
        title -> Nullable<Text>,
        #[sql_name = "titleForSearch"]
        title_for_search -> Nullable<Text>,
        subtitle -> Nullable<Text>,
        bpmx100 -> Nullable<Integer>,
        length -> Nullable<Integer>,
        #[sql_name = "trackNo"]
        track_no -> Nullable<Integer>,
        #[sql_name = "discNo"]
        disc_no -> Nullable<Integer>,
        #[sql_name = "artist_id_artist"]
        artist_id -> Nullable<Integer>,
        #[sql_name = "artist_id_remixer"]
        remixer_id -> Nullable<Integer>,
        #[sql_name = "originalArtist_id"]
        original_artist_id -> Nullable<Integer>,
        #[sql_name = "artist_id_composer"]
        composer_id -> Nullable<Integer>,
        #[sql_name = "artist_id_lyricist"]
        lyricist_id -> Nullable<Integer>,
        album_id -> Nullable<Integer>,
        genre_id -> Nullable<Integer>,
        label_id -> Nullable<Integer>,
        key_id -> Nullable<Integer>,
        color_id -> Nullable<Integer>,
        image_id -> Nullable<Integer>,
        #[sql_name = "djComment"]
        dj_comment -> Nullable<Text>,
        rating -> Nullable<Integer>,
        #[sql_name = "releaseYear"]
        release_year -> Nullable<Integer>,
        #[sql_name = "releaseDate"]
        release_date -> Nullable<Text>,
        #[sql_name = "dateCreated"]
        date_created -> Text,
        #[sql_name = "dateAdded"]
        date_added -> Text,
        path -> Text,
        #[sql_name = "fileName"]
        file_name -> Nullable<Text>,
        #[sql_name = "fileSize"]
        file_size -> Nullable<Integer>,
        #[sql_name = "fileType"]
        file_type -> Nullable<Integer>,
        bitrate -> Nullable<Integer>,
        #[sql_name = "bitDepth"]
        bit_depth -> Nullable<Integer>,
        #[sql_name = "samplingRate"]
        sampling_rate -> Nullable<Integer>,
        isrc -> Nullable<Text>,
        #[sql_name = "isHotCueAutoLoadOn"]
        is_hot_cue_auto_load_on -> Nullable<Integer>,
        #[sql_name = "isKuvoDeliverStatusOn"]
        is_kuvo_deliver_status_on -> Nullable<Integer>,
        #[sql_name = "kuvoDeliveryComment"]
        kuvo_delivery_comment -> Nullable<Text>,
        #[sql_name = "masterDbId"]
        master_db_id -> Nullable<Integer>,
        #[sql_name = "masterContentId"]
        master_content_id -> Nullable<Integer>,
        #[sql_name = "analysisDataFilePath"]
        analysis_data_file_path -> Nullable<Text>,
        #[sql_name = "analysedBits"]
        analysed_bits -> Nullable<Integer>,
        #[sql_name = "contentLink"]
        content_link -> Nullable<Integer>,
        #[sql_name = "hasModified"]
        has_modified -> Nullable<Integer>,
        #[sql_name = "cueUpdateCount"]
        cue_update_count -> Nullable<Integer>,
        #[sql_name = "analysisDataUpdateCount"]
        analysis_data_update_count -> Nullable<Integer>,
        #[sql_name = "informationUpdateCount"]
        information_update_count -> Nullable<Integer>,
    }
}

diesel::table! {
    cue (id) {
        #[sql_name = "cue_id"]
        id -> Integer,
        content_id -> Integer,
        kind -> Nullable<Integer>,
        #[sql_name = "colorTableIndex"]
        color_table_index -> Nullable<Integer>,
        #[sql_name = "cueComment"]
        cue_comment -> Nullable<Text>,
        #[sql_name = "isActiveLoop"]
        is_active_loop -> Nullable<Integer>,
        #[sql_name = "beatLoopNumerator"]
        beat_loop_numerator -> Nullable<Integer>,
        #[sql_name = "beatLoopDenominator"]
        beat_loop_denominator -> Nullable<Integer>,
        #[sql_name = "inUsec"]
        in_usec -> Nullable<Integer>,
        #[sql_name = "outUsec"]
        out_usec -> Nullable<Integer>,
        #[sql_name = "in150FramePerSec"]
        in150_frame_per_sec -> Nullable<Integer>,
        #[sql_name = "out150FramePerSec"]
        out150_frame_per_sec -> Nullable<Integer>,
        #[sql_name = "inMpegFrameNumber"]
        in_mpeg_frame_number -> Nullable<Integer>,
        #[sql_name = "outMpegFrameNumber"]
        out_mpeg_frame_number -> Nullable<Integer>,
        #[sql_name = "inMpegAbs"]
        in_mpeg_abs -> Nullable<Integer>,
        #[sql_name = "outMpegAbs"]
        out_mpeg_abs -> Nullable<Integer>,
        #[sql_name = "inDecodingStartFramePosition"]
        in_decoding_start_frame_position -> Nullable<Integer>,
        #[sql_name = "outDecodingStartFramePosition"]
        out_decoding_start_frame_position -> Nullable<Integer>,
        #[sql_name = "inFileOffsetInBlock"]
        in_file_offset_in_block -> Nullable<Integer>,
        #[sql_name = "outFileOffsetInBlock"]
        out_file_offset_in_block -> Nullable<Integer>,
        #[sql_name = "inNumberOfSampleInBlock"]
        in_number_of_sample_in_block -> Nullable<Integer>,
        #[sql_name = "outNumberOfSampleInBlock"]
        out_number_of_sample_in_block -> Nullable<Integer>,
    }
}

diesel::table! {
    genre (id) {
        #[sql_name = "genre_id"]
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    history (id) {
        #[sql_name = "history_id"]
        id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        name -> Text,
        attribute -> Integer,
        history_id_parent -> Nullable<Integer>,
    }
}

diesel::table! {
    history_content (history_id, content_id) {
        history_id -> Integer,
        content_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
    }
}

diesel::table! {
    hotCueBankList (id) {
        #[sql_name = "hotCueBankList_id"]
        id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        name -> Nullable<Text>,
        image_id -> Nullable<Integer>,
        attribute -> Integer,
        #[sql_name = "hotCueBankList_id_parent"]
        hot_cue_bank_list_id_parent -> Nullable<Integer>,
    }
}

diesel::table! {
    hotCueBankList_cue (hot_cue_bank_list_id, cue_id) {
        #[sql_name = "hotCueBankList_id"]
        hot_cue_bank_list_id -> Integer,
        cue_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
    }
}

diesel::table! {
    image (id) {
        #[sql_name = "image_id"]
        id -> Integer,
        path -> Text,
    }
}

diesel::table! {
    key (id) {
        #[sql_name = "key_id"]
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    label (id) {
        #[sql_name = "label_id"]
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    menuItem (id) {
        #[sql_name = "menuItem_id"]
        id -> Integer,
        kind -> Integer,
        name -> Text,
    }
}

diesel::table! {
    myTag (id) {
        #[sql_name = "myTag_id"]
        id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        name -> Text,
        attribute -> Integer,
        #[sql_name = "myTag_id_parent"]
        my_tag_id_parent -> Nullable<Integer>,
    }
}

diesel::table! {
    myTag_content (my_tag_id, content_id) {
        #[sql_name = "myTag_id"]
        my_tag_id -> Integer,
        content_id -> Integer,
    }
}

diesel::table! {
    playlist (id) {
        #[sql_name = "playlist_id"]
        id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        name -> Text,
    }
}

diesel::table! {
    playlist_content (playlist_id, content_id) {
        playlist_id -> Integer,
        content_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
    }
}

diesel::table! {
    property (device_name) {
        #[sql_name = "deviceName"]
        device_name -> Text,
        #[sql_name = "dbVersion"]
        db_version -> Integer,
        #[sql_name = "numberOfContents"]
        number_of_contents -> Integer,
        #[sql_name = "createdDate"]
        created_date -> Text,
        #[sql_name = "backGroundColorType"]
        back_ground_color_type -> Nullable<Integer>,
        #[sql_name = "myTagMasterDBID"]
        my_tag_master_dbid -> Nullable<Integer>,
    }
}

diesel::table! {
    recommendedLike (content_id_1, content_id_2) {
        content_id_1 -> Integer,
        content_id_2 -> Integer,
        rating -> Integer,
        #[sql_name = "createdDate"]
        created_date -> Text,
    }
}

diesel::table! {
    sort (id) {
        #[sql_name = "sort_id"]
        id -> Integer,
        #[sql_name = "menuItem_id"]
        menu_item_id -> Integer,
        #[sql_name = "sequenceNo"]
        sequence_no -> Integer,
        #[sql_name = "isVisible"]
        is_visible -> Integer,
        #[sql_name = "isSelectedAsSubColumn"]
        is_selected_as_sub_column -> Integer,
    }
}

diesel::joinable!(album -> artist (artist_id));
diesel::joinable!(album -> image (image_id));
diesel::joinable!(category -> menuItem (menu_item_id));
diesel::joinable!(content -> artist (artist_id));
diesel::joinable!(content -> album (album_id));
diesel::joinable!(content -> genre (genre_id));
diesel::joinable!(content -> label (label_id));
diesel::joinable!(content -> key (key_id));
diesel::joinable!(content -> color (color_id));
diesel::joinable!(content -> image (image_id));
diesel::allow_tables_to_appear_in_same_query!(
    content, artist, album, genre, label, key, color, image, category, menuItem
);
