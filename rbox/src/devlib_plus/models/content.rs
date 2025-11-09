// Author: Dylan Jones
// Date:   2025-09-02

use diesel::prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "pyo3")]
use py_derives::PyMutableMapping;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::album::Album;
use super::artist::{Artist, Composer, Lyricist, OriginalArtist, Remixer};
use super::color::Color;
use super::genre::Genre;
use super::image::Image;
use super::key::Key;
use super::label::Label;
use super::schema;
use super::{Date, DateString};
#[cfg(feature = "pyo3")]
use crate::util::{PyItemsIter, PyObjectIter, PyStrIter};

type JoinTuple = (
    Content,
    Option<Artist>,
    Option<Remixer>,
    Option<OriginalArtist>,
    Option<Composer>,
    Option<Lyricist>,
    Option<Album>,
    Option<Genre>,
    Option<Label>,
    Option<Key>,
    Option<Color>,
    Option<Image>,
);

#[derive(
    Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, AsChangeset, Associations,
)]
#[diesel(table_name = schema::content)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(Artist, foreign_key = artist_id))]
#[diesel(belongs_to(Remixer, foreign_key = remixer_id))]
#[diesel(belongs_to(OriginalArtist, foreign_key = original_artist_id))]
#[diesel(belongs_to(Composer, foreign_key = composer_id))]
#[diesel(belongs_to(Lyricist, foreign_key = lyricist_id))]
#[diesel(belongs_to(AlbumRaw, foreign_key = album_id))]
#[diesel(belongs_to(Genre, foreign_key = genre_id))]
#[diesel(belongs_to(Label, foreign_key = label_id))]
#[diesel(belongs_to(Key, foreign_key = key_id))]
#[diesel(belongs_to(Color, foreign_key = color_id))]
#[diesel(belongs_to(Image, foreign_key = image_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct Content {
    pub id: i32,
    pub title: Option<String>,
    pub title_for_search: Option<String>,
    pub subtitle: Option<String>,
    pub bpmx100: Option<i32>,
    pub length: Option<i32>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub artist_id: Option<i32>,
    pub remixer_id: Option<i32>,
    pub original_artist_id: Option<i32>,
    pub composer_id: Option<i32>,
    pub lyricist_id: Option<i32>,
    pub album_id: Option<i32>,
    pub genre_id: Option<i32>,
    pub label_id: Option<i32>,
    pub key_id: Option<i32>,
    pub color_id: Option<i32>,
    pub image_id: Option<i32>,
    pub dj_comment: Option<String>,
    pub rating: Option<i32>,
    pub release_year: Option<i32>,
    pub release_date: Option<String>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub date_created: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub date_added: Date,
    pub path: String,
    pub file_name: Option<String>,
    pub file_size: Option<i32>,
    pub file_type: Option<i32>,
    pub bitrate: Option<i32>,
    pub bit_depth: Option<i32>,
    pub sampling_rate: Option<i32>,
    pub isrc: Option<String>,
    pub is_hot_cue_auto_load_on: Option<i32>,
    pub is_kuvo_deliver_status_on: Option<i32>,
    pub kuvo_delivery_comment: Option<String>,
    pub master_db_id: Option<i32>,
    pub master_content_id: Option<i32>,
    pub analysis_data_file_path: Option<String>,
    pub analysed_bits: Option<i32>,
    pub content_link: Option<i32>,
    pub has_modified: Option<i32>,
    pub cue_update_count: Option<i32>,
    pub analysis_data_update_count: Option<i32>,
    pub information_update_count: Option<i32>,
}

impl Content {
    pub fn with_relations(
        self,
        art: Option<Artist>,
        rem: Option<Remixer>,
        orig_art: Option<OriginalArtist>,
        comp: Option<Composer>,
        lyr: Option<Lyricist>,
        alb: Option<Album>,
        gen: Option<Genre>,
        lab: Option<Label>,
        key_: Option<Key>,
        col: Option<Color>,
        img: Option<Image>,
    ) -> ContentJoin {
        ContentJoin {
            id: self.id,
            title: self.title,
            title_for_search: self.title_for_search,
            subtitle: self.subtitle,
            bpmx100: self.bpmx100,
            length: self.length,
            track_no: self.track_no,
            disc_no: self.disc_no,
            artist_id: self.artist_id,
            remixer_id: self.remixer_id,
            original_artist_id: self.original_artist_id,
            composer_id: self.composer_id,
            lyricist_id: self.lyricist_id,
            album_id: self.album_id,
            genre_id: self.genre_id,
            label_id: self.label_id,
            key_id: self.key_id,
            color_id: self.color_id,
            image_id: self.image_id,
            dj_comment: self.dj_comment,
            rating: self.rating,
            release_year: self.release_year,
            release_date: self.release_date,
            date_created: self.date_created,
            date_added: self.date_added,
            path: self.path,
            file_name: self.file_name,
            file_size: self.file_size,
            file_type: self.file_type,
            bitrate: self.bitrate,
            bit_depth: self.bit_depth,
            sampling_rate: self.sampling_rate,
            isrc: self.isrc,
            is_hot_cue_auto_load_on: self.is_hot_cue_auto_load_on,
            is_kuvo_deliver_status_on: self.is_kuvo_deliver_status_on,
            kuvo_delivery_comment: self.kuvo_delivery_comment,
            master_db_id: self.master_db_id,
            master_content_id: self.master_content_id,
            analysis_data_file_path: self.analysis_data_file_path,
            analysed_bits: self.analysed_bits,
            content_link: self.content_link,
            has_modified: self.has_modified,
            cue_update_count: self.cue_update_count,
            analysis_data_update_count: self.analysis_data_update_count,
            information_update_count: self.information_update_count,
            artist: art,
            remixer: rem,
            original_artist: orig_art,
            composer: comp,
            lyricist: lyr,
            album: alb,
            genre: gen,
            label: lab,
            key: key_,
            color: col,
            image: img,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentJoin {
    pub id: i32,
    pub title: Option<String>,
    pub title_for_search: Option<String>,
    pub subtitle: Option<String>,
    pub bpmx100: Option<i32>,
    pub length: Option<i32>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub artist_id: Option<i32>,
    pub remixer_id: Option<i32>,
    pub original_artist_id: Option<i32>,
    pub composer_id: Option<i32>,
    pub lyricist_id: Option<i32>,
    pub album_id: Option<i32>,
    pub genre_id: Option<i32>,
    pub label_id: Option<i32>,
    pub key_id: Option<i32>,
    pub color_id: Option<i32>,
    pub image_id: Option<i32>,
    pub dj_comment: Option<String>,
    pub rating: Option<i32>,
    pub release_year: Option<i32>,
    pub release_date: Option<String>,
    pub date_created: Date,
    pub date_added: Date,
    pub path: String,
    pub file_name: Option<String>,
    pub file_size: Option<i32>,
    pub file_type: Option<i32>,
    pub bitrate: Option<i32>,
    pub bit_depth: Option<i32>,
    pub sampling_rate: Option<i32>,
    pub isrc: Option<String>,
    pub is_hot_cue_auto_load_on: Option<i32>,
    pub is_kuvo_deliver_status_on: Option<i32>,
    pub kuvo_delivery_comment: Option<String>,
    pub master_db_id: Option<i32>,
    pub master_content_id: Option<i32>,
    pub analysis_data_file_path: Option<String>,
    pub analysed_bits: Option<i32>,
    pub content_link: Option<i32>,
    pub has_modified: Option<i32>,
    pub cue_update_count: Option<i32>,
    pub analysis_data_update_count: Option<i32>,
    pub information_update_count: Option<i32>,

    pub artist: Option<Artist>,
    pub remixer: Option<Remixer>,
    pub original_artist: Option<OriginalArtist>,
    pub composer: Option<Composer>,
    pub lyricist: Option<Lyricist>,
    pub album: Option<Album>,
    pub genre: Option<Genre>,
    pub label: Option<Label>,
    pub key: Option<Key>,
    pub color: Option<Color>,
    pub image: Option<Image>,
}

impl Into<Content> for ContentJoin {
    fn into(self) -> Content {
        Content {
            id: self.id,
            title: self.title,
            title_for_search: self.title_for_search,
            subtitle: self.subtitle,
            bpmx100: self.bpmx100,
            length: self.length,
            track_no: self.track_no,
            disc_no: self.disc_no,
            artist_id: self.artist_id,
            remixer_id: self.remixer_id,
            original_artist_id: self.original_artist_id,
            composer_id: self.composer_id,
            lyricist_id: self.lyricist_id,
            album_id: self.album_id,
            genre_id: self.genre_id,
            label_id: self.label_id,
            key_id: self.key_id,
            color_id: self.color_id,
            image_id: self.image_id,
            dj_comment: self.dj_comment,
            rating: self.rating,
            release_year: self.release_year,
            release_date: self.release_date,
            date_created: self.date_created,
            date_added: self.date_added,
            path: self.path,
            file_name: self.file_name,
            file_size: self.file_size,
            file_type: self.file_type,
            bitrate: self.bitrate,
            bit_depth: self.bit_depth,
            sampling_rate: self.sampling_rate,
            isrc: self.isrc,
            is_hot_cue_auto_load_on: self.is_hot_cue_auto_load_on,
            is_kuvo_deliver_status_on: self.is_kuvo_deliver_status_on,
            kuvo_delivery_comment: self.kuvo_delivery_comment,
            master_db_id: self.master_db_id,
            master_content_id: self.master_content_id,
            analysis_data_file_path: self.analysis_data_file_path,
            analysed_bits: self.analysed_bits,
            content_link: self.content_link,
            has_modified: self.has_modified,
            cue_update_count: self.cue_update_count,
            analysis_data_update_count: self.analysis_data_update_count,
            information_update_count: self.information_update_count,
        }
    }
}

impl From<Content> for ContentJoin {
    fn from(raw: Content) -> Self {
        Self {
            id: raw.id,
            title: raw.title,
            title_for_search: raw.title_for_search,
            subtitle: raw.subtitle,
            bpmx100: raw.bpmx100,
            length: raw.length,
            track_no: raw.track_no,
            disc_no: raw.disc_no,
            artist_id: raw.artist_id,
            remixer_id: raw.remixer_id,
            original_artist_id: raw.original_artist_id,
            composer_id: raw.composer_id,
            lyricist_id: raw.lyricist_id,
            album_id: raw.album_id,
            genre_id: raw.genre_id,
            label_id: raw.label_id,
            key_id: raw.key_id,
            color_id: raw.color_id,
            image_id: raw.image_id,
            dj_comment: raw.dj_comment,
            rating: raw.rating,
            release_year: raw.release_year,
            release_date: raw.release_date,
            date_created: raw.date_created,
            date_added: raw.date_added,
            path: raw.path,
            file_name: raw.file_name,
            file_size: raw.file_size,
            file_type: raw.file_type,
            bitrate: raw.bitrate,
            bit_depth: raw.bit_depth,
            sampling_rate: raw.sampling_rate,
            isrc: raw.isrc,
            is_hot_cue_auto_load_on: raw.is_hot_cue_auto_load_on,
            is_kuvo_deliver_status_on: raw.is_kuvo_deliver_status_on,
            kuvo_delivery_comment: raw.kuvo_delivery_comment,
            master_db_id: raw.master_db_id,
            master_content_id: raw.master_content_id,
            analysis_data_file_path: raw.analysis_data_file_path,
            analysed_bits: raw.analysed_bits,
            content_link: raw.content_link,
            has_modified: raw.has_modified,
            cue_update_count: raw.cue_update_count,
            analysis_data_update_count: raw.analysis_data_update_count,
            information_update_count: raw.information_update_count,
            artist: None,
            remixer: None,
            original_artist: None,
            composer: None,
            lyricist: None,
            album: None,
            genre: None,
            label: None,
            key: None,
            color: None,
            image: None,
        }
    }
}

impl ContentJoin {
    pub fn from_join(tuple: JoinTuple) -> Self {
        let mut result = ContentJoin::from(tuple.0);
        result.artist = tuple.1;
        result.remixer = tuple.2;
        result.original_artist = tuple.3;
        result.composer = tuple.4;
        result.lyricist = tuple.5;
        result.album = tuple.6;
        result.genre = tuple.7;
        result.label = tuple.8;
        result.key = tuple.9;
        result.color = tuple.10;
        result.image = tuple.11;
        result
    }
}

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = schema::content)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[cfg_attr(feature = "pyo3", pyclass(get_all, set_all, mapping))]
#[cfg_attr(feature = "pyo3", derive(PyMutableMapping))]
#[cfg_attr(feature = "napi", napi(object))]
pub struct NewContent {
    pub title: Option<String>,
    pub title_for_search: Option<String>,
    pub subtitle: Option<String>,
    pub bpmx100: Option<i32>,
    pub length: Option<i32>,
    pub track_no: Option<i32>,
    pub disc_no: Option<i32>,
    pub artist_id: Option<i32>,
    pub remixer_id: Option<i32>,
    pub original_artist_id: Option<i32>,
    pub composer_id: Option<i32>,
    pub lyricist_id: Option<i32>,
    pub album_id: Option<i32>,
    pub genre_id: Option<i32>,
    pub label_id: Option<i32>,
    pub key_id: Option<i32>,
    pub color_id: Option<i32>,
    pub image_id: Option<i32>,
    pub dj_comment: Option<String>,
    pub rating: Option<i32>,
    pub release_year: Option<i32>,
    pub release_date: Option<String>,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub date_created: Date,
    #[diesel(serialize_as = DateString)]
    #[diesel(deserialize_as = DateString)]
    pub date_added: Date,
    #[cfg_attr(feature = "pyo3", repr_field)]
    pub path: String,
    pub file_name: Option<String>,
    pub file_size: Option<i32>,
    pub file_type: Option<i32>,
    pub bitrate: Option<i32>,
    pub bit_depth: Option<i32>,
    pub sampling_rate: Option<i32>,
    pub isrc: Option<String>,
    pub is_hot_cue_auto_load_on: Option<i32>,
    pub is_kuvo_deliver_status_on: Option<i32>,
    pub kuvo_delivery_comment: Option<String>,
    pub master_db_id: Option<i32>,
    pub master_content_id: Option<i32>,
    pub analysis_data_file_path: Option<String>,
    pub analysed_bits: Option<i32>,
    pub content_link: Option<i32>,
    pub has_modified: Option<i32>,
    pub cue_update_count: Option<i32>,
    pub analysis_data_update_count: Option<i32>,
    pub information_update_count: Option<i32>,
}

impl NewContent {
    pub fn new(path: String, now: Date) -> Self {
        Self {
            date_created: now.clone(),
            date_added: now.clone(),
            path,
            ..Default::default()
        }
    }
}
