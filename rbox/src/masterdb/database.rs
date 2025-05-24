// Author: Dylan Jones
// Date:   2025-05-01
//
// Rekordbox master.db database handler

// #![allow(unused)]

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use diesel::dsl::{exists, select};
use diesel::{connection::SimpleConnection, prelude::*, query_dsl::RunQueryDsl, SqliteConnection};
use dunce;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use uuid::Uuid;

use super::enums::*;
use super::models::*;
use super::playlist_xml::MasterPlaylistXml;
use super::random_id::RandomIdGenerator;
use super::util::{format_datetime, sort_tree_list};
use super::{
    agentRegistry, cloudAgentRegistry, contentActiveCensor, contentCue, contentFile,
    djmdActiveCensor, djmdAlbum, djmdArtist, djmdCategory, djmdCloudProperty, djmdColor,
    djmdContent, djmdCue, djmdDevice, djmdGenre, djmdHistory, djmdHotCueBanklist, djmdKey,
    djmdLabel, djmdMenuItems, djmdMixerParam, djmdMyTag, djmdPlaylist, djmdProperty,
    djmdRecommendLike, djmdRelatedTracks, djmdSampler, djmdSongPlaylist, djmdSongTagList, djmdSort,
    imageFile, schema, settingFile, uuidIDMap,
};
use crate::anlz::{find_anlz_files, Anlz, AnlzFiles, AnlzPaths};
use crate::options::RekordboxOptions;
use crate::pathlib::NormalizePath;

const MAGIC: &str = "513ge593d49928d46ggb9ggc9d8e:4254c85:f8e426eg8b92843b2gg547195:8";

fn open_connection(path: &str) -> anyhow::Result<SqliteConnection> {
    let key = String::from_utf8(MAGIC.as_bytes().iter().map(|&b| b - 1).collect())?;

    let mut conn = SqliteConnection::establish(path)?;
    conn.batch_execute(format!("PRAGMA key = '{key}';").as_str())?;
    conn.batch_execute("PRAGMA foreign_keys = ON")?;
    conn.batch_execute("PRAGMA journal_mode = WAL")?;
    conn.batch_execute("PRAGMA synchronous = NORMAL")?;
    // conn.batch_execute("PRAGMA busy_timeout = 100")?;
    // conn.batch_execute("PRAGMA wal_autocheckpoint = 1000")?;
    // conn.batch_execute("PRAGMA wal_checkpoint(TRUNCATE)")?;
    Ok(conn)
}

pub struct MasterDb {
    /// The Sqlite database connection
    pub conn: SqliteConnection,
    /// The path of the PIONEER share directory located in the same directory as the DB
    pub share_dir: Option<PathBuf>,
    /// The path of the masterPlaylist6 XML file located in the same directory as the DB
    pub plxml_path: Option<PathBuf>,
}

impl MasterDb {
    /// Open a Rekordbox database specified by path.
    ///
    /// The path must be a valid Rekordbox database file. The function will try to locate the
    /// `share` directory and the `masterPlaylist6.xml` file in the same directory as the database
    /// file. If they are not found, the database can still be used, however, some features such as
    /// playlist management and locating analysis files will return errors.
    pub fn new<P: AsRef<OsStr>>(path: P) -> anyhow::Result<Self> {
        let path_obj = Path::new(&path);
        if !path_obj.exists() {
            return Err(anyhow::anyhow!("Database file does not exist"));
        }
        let parent_dir = path_obj.parent().expect("Failed to get parent directory");
        let share_dir_path = parent_dir.join("share");
        let share_dir_str = if share_dir_path.exists() {
            Some(share_dir_path.normalize())
        } else {
            None
        };
        let pl_xml_path = parent_dir.join("masterPlaylists6.xml");
        let pl_xml_path_str = if pl_xml_path.exists() {
            Some(pl_xml_path.normalize())
        } else {
            None
        };
        let conn = open_connection(path_obj.to_str().unwrap())?;
        Ok(Self {
            conn,
            share_dir: share_dir_str,
            plxml_path: pl_xml_path_str,
        })
    }

    /// Open the Rekordbox database specified by the options [`RekordboxOptions`]
    ///
    /// The options specified by the user must be valid. The `master.db` file, the `share` directory
    /// and the `masterPlaylist6.xml` file will be extracted from the options.
    pub fn from_options(options: &RekordboxOptions) -> anyhow::Result<Self> {
        let share_dir = options.analysis_root.normalize();
        let plxml_path = options.get_db_dir()?.normalize();
        let conn = open_connection(options.db_path.to_str().unwrap())?;

        Ok(Self {
            conn,
            share_dir: Some(share_dir),
            plxml_path: Some(plxml_path),
        })
    }

    /// Open the default Rekordbox `master.db` database.
    ///
    /// The default location of the `master.db` file is determined by the [`RekordboxOptions`] struct.
    pub fn open() -> anyhow::Result<Self> {
        let options = RekordboxOptions::open()?;
        Self::from_options(&options)
    }

    // -- AgentRegistry ----------------------------------------------------------------------------

    pub fn get_agent_registry(&mut self) -> anyhow::Result<Vec<AgentRegistry>> {
        let results = agentRegistry.load::<AgentRegistry>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_agent_registry_by_id(&mut self, id: &str) -> anyhow::Result<Option<AgentRegistry>> {
        let result = agentRegistry
            .filter(schema::agentRegistry::registry_id.eq(id))
            .first::<AgentRegistry>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_local_usn(&mut self) -> anyhow::Result<i32> {
        let result = agentRegistry
            .filter(schema::agentRegistry::registry_id.eq("lastUpdateCount"))
            .select(schema::agentRegistry::int_1)
            .first::<Option<i32>>(&mut self.conn)?;
        // Raise error if not found, otherwise return value
        if let Some(value) = result {
            Ok(value)
        } else {
            Err(anyhow::anyhow!("lastUpdateCount not found"))
        }
    }

    // fn set_local_usn(&mut self, usn: i32) -> anyhow::Result<usize> {
    //     let result = diesel::update(
    //         agentRegistry.filter(schema::agentRegistry::registry_id.eq("lastUpdateCount")),
    //     )
    //     .set(schema::agentRegistry::int_1.eq(usn))
    //     .execute(&mut self.conn)?;
    //     Ok(result)
    // }

    fn increment_local_usn(&mut self, usn: i32) -> anyhow::Result<i32> {
        let result = diesel::update(
            agentRegistry.filter(schema::agentRegistry::registry_id.eq("lastUpdateCount")),
        )
        .set(schema::agentRegistry::int_1.eq(schema::agentRegistry::int_1 + usn))
        .returning(schema::agentRegistry::int_1)
        .get_result::<Option<i32>>(&mut self.conn)?;
        let new_usn = result.expect("No new USN");
        Ok(new_usn)
    }

    // -- CloudAgentRegistry -----------------------------------------------------------------------

    pub fn get_cloud_agent_registry(&mut self) -> anyhow::Result<Vec<CloudAgentRegistry>> {
        let results = cloudAgentRegistry.load::<CloudAgentRegistry>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_cloud_agent_registry_by_id(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Option<CloudAgentRegistry>> {
        let result = cloudAgentRegistry
            .filter(schema::cloudAgentRegistry::ID.eq(id))
            .first::<CloudAgentRegistry>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ContentActiveCensor ----------------------------------------------------------------------

    pub fn get_content_active_censor(&mut self) -> anyhow::Result<Vec<ContentActiveCensor>> {
        let results = contentActiveCensor.load::<ContentActiveCensor>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_content_active_censor_by_id(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Option<ContentActiveCensor>> {
        let result = contentActiveCensor
            .filter(schema::contentActiveCensor::ID.eq(id))
            .first::<ContentActiveCensor>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ContentCue -------------------------------------------------------------------------------

    pub fn get_content_cue(&mut self) -> anyhow::Result<Vec<ContentCue>> {
        let results = contentCue.load::<ContentCue>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_content_cue_by_id(&mut self, id: &str) -> anyhow::Result<Option<ContentCue>> {
        let result = contentCue
            .filter(schema::contentCue::ID.eq(id))
            .first::<ContentCue>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ContentFile ------------------------------------------------------------------------------

    pub fn get_content_file(&mut self) -> anyhow::Result<Vec<ContentFile>> {
        let results = contentFile.load::<ContentFile>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_content_file_by_id(&mut self, id: &str) -> anyhow::Result<Option<ContentFile>> {
        let result = contentFile
            .filter(schema::contentFile::ID.eq(id))
            .first::<ContentFile>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ActiveCensor -----------------------------------------------------------------------------

    pub fn get_active_censor(&mut self) -> anyhow::Result<Vec<DjmdActiveCensor>> {
        let results = djmdActiveCensor.load::<DjmdActiveCensor>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_active_censor_by_id(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Option<DjmdActiveCensor>> {
        let result = djmdActiveCensor
            .filter(schema::djmdActiveCensor::ID.eq(id))
            .first::<DjmdActiveCensor>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Album ------------------------------------------------------------------------------------

    pub fn get_album(&mut self) -> anyhow::Result<Vec<DjmdAlbum>> {
        let results: Vec<DjmdAlbum> = djmdAlbum.load(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_album_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdAlbum>> {
        let result: Option<DjmdAlbum> = djmdAlbum
            .filter(schema::djmdAlbum::ID.eq(id))
            .first(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_album_by_name(&mut self, name: &str) -> anyhow::Result<Option<DjmdAlbum>> {
        let results: Vec<DjmdAlbum> = djmdAlbum
            .filter(schema::djmdAlbum::Name.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    fn album_exists(&mut self, id: &String) -> anyhow::Result<bool> {
        let id_exists: bool = select(exists(djmdAlbum.filter(schema::djmdAlbum::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    fn generate_album_id(&mut self) -> anyhow::Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.album_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    pub fn insert_album(
        &mut self,
        name: String,
        artist_id: Option<String>,
        image_path: Option<String>,
        compilation: Option<i32>,
    ) -> anyhow::Result<DjmdAlbum> {
        // Generate ID/UUID
        let id: String = self.generate_album_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date string
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        // Create and insert model
        let item: DjmdAlbum =
            DjmdAlbum::new(id, uuid, usn, now, name, artist_id, image_path, compilation)?;
        let result: DjmdAlbum = diesel::insert_into(djmdAlbum)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    pub fn update_album(&mut self, item: &mut DjmdAlbum) -> anyhow::Result<DjmdAlbum> {
        // Count differences
        let existing: DjmdAlbum = self.get_album_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.Name != existing.Name {
            n += 1
        }
        if item.AlbumArtistID != existing.AlbumArtistID {
            n += 1
        }
        if item.ImagePath != existing.ImagePath {
            n += 1
        }
        if item.Compilation != existing.Compilation {
            n += 1
        }
        if item.SearchStr != existing.SearchStr {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        item.updated_at = Some(now);
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdAlbum = diesel::update(&*item)
            .set(&*item)
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    pub fn delete_album(&mut self, id: &str) -> anyhow::Result<usize> {
        let result = diesel::delete(djmdAlbum.filter(schema::djmdAlbum::ID.eq(id)))
            .execute(&mut self.conn)?;

        // Remove any references to the album in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::AlbumID.eq(id)))
            .set(schema::djmdContent::AlbumID.eq(None::<String>))
            .execute(&mut self.conn)?;

        let _ = self.increment_local_usn(1);
        Ok(result)
    }

    // -- Artist -----------------------------------------------------------------------------------

    pub fn get_artist(&mut self) -> anyhow::Result<Vec<DjmdArtist>> {
        let results: Vec<DjmdArtist> = djmdArtist.load(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_artist_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdArtist>> {
        let result: Option<DjmdArtist> = djmdArtist
            .filter(schema::djmdArtist::ID.eq(id))
            .first(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_artist_by_name(&mut self, name: &str) -> anyhow::Result<Option<DjmdArtist>> {
        let results: Vec<DjmdArtist> = djmdArtist
            .filter(schema::djmdArtist::Name.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    fn artist_exists(&mut self, id: &String) -> anyhow::Result<bool> {
        let id_exists: bool = select(exists(djmdArtist.filter(schema::djmdArtist::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    fn generate_artist_id(&mut self) -> anyhow::Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.artist_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    pub fn insert_artist(&mut self, name: String) -> anyhow::Result<DjmdArtist> {
        // Generate ID/UUID
        let id: String = self.generate_artist_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date string
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        // Create and insert model
        let item: DjmdArtist = DjmdArtist::new(id, uuid, usn, now, name)?;
        let result: DjmdArtist = diesel::insert_into(djmdArtist)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    pub fn update_artist(&mut self, item: &mut DjmdArtist) -> anyhow::Result<DjmdArtist> {
        // Count differences
        let existing: DjmdArtist = self.get_artist_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.Name != existing.Name {
            n += 1
        }
        if item.SearchStr != existing.SearchStr {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        item.updated_at = Some(now);
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdArtist = diesel::update(&*item)
            .set(&*item)
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    pub fn delete_artist(&mut self, id: &str) -> anyhow::Result<usize> {
        let result: usize = diesel::delete(djmdArtist.filter(schema::djmdArtist::ID.eq(id)))
            .execute(&mut self.conn)?;
        self.increment_local_usn(1)?;

        // Remove any references to the artist in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::ArtistID.eq(id)))
            .set(schema::djmdContent::ArtistID.eq(None::<String>))
            .execute(&mut self.conn)?;
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::OrgArtistID.eq(id)))
            .set(schema::djmdContent::OrgArtistID.eq(None::<String>))
            .execute(&mut self.conn)?;

        // Remove any references to the artist in DjmdAlbum
        let _ = diesel::update(djmdAlbum.filter(schema::djmdAlbum::AlbumArtistID.eq(id)))
            .set(schema::djmdAlbum::AlbumArtistID.eq(None::<String>))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- Category ---------------------------------------------------------------------------------

    pub fn get_category(&mut self) -> anyhow::Result<Vec<DjmdCategory>> {
        let results = djmdCategory
            .order_by(schema::djmdCategory::Seq)
            .load::<DjmdCategory>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_category_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdCategory>> {
        let result = djmdCategory
            .filter(schema::djmdCategory::ID.eq(id))
            .first::<DjmdCategory>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Color ------------------------------------------------------------------------------------

    pub fn get_color(&mut self) -> anyhow::Result<Vec<DjmdColor>> {
        let results = djmdColor
            .order_by(schema::djmdColor::SortKey)
            .load::<DjmdColor>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_color_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdColor>> {
        let result = djmdColor
            .filter(schema::djmdColor::ID.eq(id))
            .first::<DjmdColor>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Content ----------------------------------------------------------------------------------

    pub fn get_content(&mut self) -> anyhow::Result<Vec<DjmdContent>> {
        let results = djmdContent.load::<DjmdContent>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_content_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdContent>> {
        let result = djmdContent
            .filter(schema::djmdContent::ID.eq(id))
            .first::<DjmdContent>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_contents_by_ids(&mut self, ids: Vec<&str>) -> anyhow::Result<Vec<DjmdContent>> {
        let mut result: Vec<DjmdContent> = Vec::new();
        for id in ids {
            let content = djmdContent
                .filter(schema::djmdContent::ID.eq(id))
                .first::<DjmdContent>(&mut self.conn)
                .optional()?;

            if let Some(content) = content {
                result.push(content);
            }
        }
        Ok(result)
    }

    pub fn get_content_by_path(&mut self, path: &str) -> anyhow::Result<Option<DjmdContent>> {
        let result = djmdContent
            .filter(schema::djmdContent::FolderPath.eq(path))
            .first::<DjmdContent>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    fn content_exists(&mut self, id: &String) -> anyhow::Result<bool> {
        let id_exists: bool = select(exists(djmdContent.filter(schema::djmdContent::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    pub fn update_content(&mut self, item: &DjmdContent) -> anyhow::Result<usize> {
        let result = diesel::update(item).set(item).execute(&mut self.conn)?;
        Ok(result)
    }

    pub fn delete_content(&mut self, id: &str) -> anyhow::Result<usize> {
        let result = diesel::delete(djmdContent.filter(schema::djmdContent::ID.eq(id)))
            .execute(&mut self.conn)?;
        Ok(result)
    }

    /// Returns the path to the corresponding ANLZxxxx.DAT file
    fn get_content_analysis_data_path(&mut self, id: &str) -> anyhow::Result<PathBuf> {
        if self.share_dir.is_none() {
            return Err(anyhow::anyhow!("Share dir not set!"));
        }
        let result = djmdContent
            .filter(schema::djmdContent::ID.eq(id))
            .select(schema::djmdContent::AnalysisDataPath)
            .first::<Option<String>>(&mut self.conn)?;
        if let Some(result) = result {
            // Strip first "/" in result
            let striped = result.strip_prefix("/");
            if let Some(striped) = striped {
                let anlz_file = self.share_dir.clone().unwrap().join(striped);
                let anlz_files_canonicalized = dunce::canonicalize(&anlz_file);
                if let Err(e) = anlz_files_canonicalized {
                    return Err(anyhow::anyhow!("Failed to canonicalize path: {}", e));
                }
                return Ok(anlz_files_canonicalized?);
            }
        }
        Err(anyhow::anyhow!("Failed to get AnalysisDataPath"))
    }

    /// Returns the path to the directory containing the ANLZxxxx.xxx files
    pub fn get_content_anlz_dir(&mut self, id: &str) -> anyhow::Result<Option<PathBuf>> {
        let anlz_file = self.get_content_analysis_data_path(id)?;
        let root = anlz_file.parent().unwrap();
        return Ok(Some(root.to_path_buf()));
    }

    /// Returns a struct containing the  ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 paths
    pub fn get_content_anlz_paths(&mut self, id: &str) -> anyhow::Result<Option<AnlzPaths>> {
        let root = self.get_content_anlz_dir(id)?;
        if root.is_none() {
            return Ok(None);
        }
        find_anlz_files(root.unwrap())
    }

    /// Returns a struct containing the loaded ANLZxxxx.DAT, ANLZxxxx.EXT, and ANLZxxxx.EX2 files
    pub fn get_content_anlz_files(&mut self, id: &str) -> anyhow::Result<Option<AnlzFiles>> {
        let paths = self.get_content_anlz_paths(id)?;
        if paths.is_none() {
            return Ok(None);
        }
        let paths = paths.unwrap();
        let mut files = AnlzFiles {
            dat: Anlz::load(paths.dat)?,
            ext: None,
            ex2: None,
        };
        if let Some(ext) = paths.ext {
            files.ext = Some(Anlz::load(ext)?);
        }
        if let Some(ex2) = paths.ex2 {
            files.ex2 = Some(Anlz::load(ex2)?);
        }
        Ok(Some(files))
    }

    // -- Cue --------------------------------------------------------------------------------------

    pub fn get_cue(&mut self) -> anyhow::Result<Vec<DjmdCue>> {
        let results = djmdCue.load::<DjmdCue>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_cue_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdCue>> {
        let result = djmdCue
            .filter(schema::djmdCue::ID.eq(id))
            .first::<DjmdCue>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Device -----------------------------------------------------------------------------------

    pub fn get_device(&mut self) -> anyhow::Result<Vec<DjmdDevice>> {
        let results = djmdDevice.load::<DjmdDevice>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_device_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdDevice>> {
        let result = djmdDevice
            .filter(schema::djmdDevice::ID.eq(id))
            .first::<DjmdDevice>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Genre ------------------------------------------------------------------------------------

    pub fn get_genre(&mut self) -> anyhow::Result<Vec<DjmdGenre>> {
        let results = djmdGenre.load::<DjmdGenre>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_genre_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdGenre>> {
        let result = djmdGenre
            .filter(schema::djmdGenre::ID.eq(id))
            .first::<DjmdGenre>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_genre_by_name(&mut self, name: &str) -> anyhow::Result<Option<DjmdGenre>> {
        let results: Vec<DjmdGenre> = djmdGenre
            .filter(schema::djmdGenre::Name.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    fn genre_exists(&mut self, id: &String) -> anyhow::Result<bool> {
        let id_exists: bool = select(exists(djmdGenre.filter(schema::djmdGenre::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    fn generate_genre_id(&mut self) -> anyhow::Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.genre_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    pub fn insert_genre(&mut self, name: String) -> anyhow::Result<DjmdGenre> {
        // Generate ID/UUID
        let id: String = self.generate_genre_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date string
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        // Create and insert model
        let item: DjmdGenre = DjmdGenre::new(id, uuid, usn, now, name)?;
        let result: DjmdGenre = diesel::insert_into(djmdGenre)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    pub fn update_genre(&mut self, item: &mut DjmdGenre) -> anyhow::Result<DjmdGenre> {
        // Count differences
        let existing: DjmdGenre = self.get_genre_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.Name != existing.Name {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        item.updated_at = Some(now);
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdGenre = diesel::update(&*item)
            .set(&*item)
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    pub fn delete_genre(&mut self, id: &str) -> anyhow::Result<usize> {
        let result = diesel::delete(djmdGenre.filter(schema::djmdGenre::ID.eq(id)))
            .execute(&mut self.conn)?;
        self.increment_local_usn(1)?;

        // Remove any references to the genre in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::GenreID.eq(id)))
            .set(schema::djmdContent::GenreID.eq(None::<String>))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- History ----------------------------------------------------------------------------------

    pub fn get_history(&mut self) -> anyhow::Result<Vec<DjmdHistory>> {
        let results = djmdHistory
            .order_by(schema::djmdHistory::Seq)
            .load::<DjmdHistory>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_history_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdHistory>> {
        let result = djmdHistory
            .filter(schema::djmdHistory::ID.eq(id))
            .first::<DjmdHistory>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_history_songs(&mut self, id: &str) -> anyhow::Result<Vec<DjmdSongHistory>> {
        let results = schema::djmdSongHistory::table
            .filter(schema::djmdSongHistory::HistoryID.eq(id))
            .order_by(schema::djmdSongHistory::TrackNo)
            .load::<DjmdSongHistory>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_history_contents(&mut self, id: &str) -> anyhow::Result<Vec<DjmdContent>> {
        let songs = self.get_history_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    // -- HotCueBanklist ---------------------------------------------------------------------------

    pub fn get_hot_cue_banklist(&mut self) -> anyhow::Result<Vec<DjmdHotCueBanklist>> {
        let results = djmdHotCueBanklist.load::<DjmdHotCueBanklist>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_hot_cue_banklist_by_id(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Option<DjmdHotCueBanklist>> {
        let result = djmdHotCueBanklist
            .filter(schema::djmdHotCueBanklist::ID.eq(id))
            .first::<DjmdHotCueBanklist>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_hot_cue_banklist_children(
        &mut self,
        parent_id: &str,
    ) -> anyhow::Result<Vec<DjmdHotCueBanklist>> {
        let results = djmdHotCueBanklist
            .filter(schema::djmdHotCueBanklist::ParentID.eq(parent_id))
            .order_by(schema::djmdHotCueBanklist::Seq)
            .load::<DjmdHotCueBanklist>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_hot_cue_banklist_songs(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Vec<DjmdSongHotCueBanklist>> {
        let results = schema::djmdSongHotCueBanklist::table
            .filter(schema::djmdSongHotCueBanklist::CueID.eq(id))
            .order_by(schema::djmdSongHotCueBanklist::TrackNo)
            .load::<DjmdSongHotCueBanklist>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_hot_cue_banklist_contents(&mut self, id: &str) -> anyhow::Result<Vec<DjmdContent>> {
        let songs = self.get_hot_cue_banklist_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    pub fn get_hot_cue_banklist_cues(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Vec<HotCueBanklistCue>> {
        let results = schema::hotCueBanklistCue::table
            .filter(schema::hotCueBanklistCue::HotCueBanklistID.eq(id))
            .load::<HotCueBanklistCue>(&mut self.conn)?;
        Ok(results)
    }

    // -- Key --------------------------------------------------------------------------------------

    pub fn get_key(&mut self) -> anyhow::Result<Vec<DjmdKey>> {
        let results = djmdKey
            .order_by(schema::djmdKey::Seq)
            .load::<DjmdKey>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_key_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdKey>> {
        let result = djmdKey
            .filter(schema::djmdKey::ID.eq(id))
            .first::<DjmdKey>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_key_by_name(&mut self, name: &str) -> anyhow::Result<Option<DjmdKey>> {
        let results: Vec<DjmdKey> = djmdKey
            .filter(schema::djmdKey::ScaleName.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    fn key_exists(&mut self, id: &String) -> anyhow::Result<bool> {
        let id_exists: bool = select(exists(djmdKey.filter(schema::djmdKey::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    fn generate_key_id(&mut self) -> anyhow::Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.key_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    pub fn insert_key(&mut self, name: String) -> anyhow::Result<DjmdKey> {
        // Generate ID/UUID
        let id: String = self.generate_key_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date string
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        // Count existing items for inserting at end
        let count = djmdKey.count().get_result::<i64>(&mut self.conn)? as i32;
        let seq: i32 = count + 1;

        // Create and insert model
        let item: DjmdKey = DjmdKey::new(id, uuid, usn, now, name, seq)?;
        let result: DjmdKey = diesel::insert_into(djmdKey)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    pub fn update_key(&mut self, item: &mut DjmdKey) -> anyhow::Result<DjmdKey> {
        // Count differences
        let existing: DjmdKey = self.get_key_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.ScaleName != existing.ScaleName {
            n += 1
        }
        if item.Seq != existing.Seq {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        item.updated_at = Some(now);
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdKey = diesel::update(&*item)
            .set(&*item)
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    pub fn delete_key(&mut self, id: &str) -> anyhow::Result<usize> {
        let result =
            diesel::delete(djmdKey.filter(schema::djmdKey::ID.eq(id))).execute(&mut self.conn)?;
        self.increment_local_usn(1)?;

        // Remove any references to the key in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::KeyID.eq(id)))
            .set(schema::djmdContent::KeyID.eq(None::<String>))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- Label ------------------------------------------------------------------------------------

    pub fn get_label(&mut self) -> anyhow::Result<Vec<DjmdLabel>> {
        let results = djmdLabel.load::<DjmdLabel>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_label_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdLabel>> {
        let result = djmdLabel
            .filter(schema::djmdLabel::ID.eq(id))
            .first::<DjmdLabel>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_label_by_name(&mut self, name: &str) -> anyhow::Result<Option<DjmdLabel>> {
        let results: Vec<DjmdLabel> = djmdLabel
            .filter(schema::djmdLabel::Name.eq(name))
            .load(&mut self.conn)?;
        let n = results.len();
        if n == 0 {
            Ok(None)
        } else if n == 1 {
            let result = results[0].clone();
            Ok(Some(result))
        } else {
            // More than one item, return error
            Err(anyhow::anyhow!("More than one element found!"))
        }
    }

    fn label_exists(&mut self, id: &String) -> anyhow::Result<bool> {
        let id_exists: bool = select(exists(djmdLabel.filter(schema::djmdLabel::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    fn generate_label_id(&mut self) -> anyhow::Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.label_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    pub fn insert_label(&mut self, name: String) -> anyhow::Result<DjmdLabel> {
        // Generate ID/UUID
        let id: String = self.generate_label_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        // Get next USN
        let usn: i32 = self.increment_local_usn(1)?;
        // Generate date string
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        // Create and insert model
        let item: DjmdLabel = DjmdLabel::new(id, uuid, usn, now, name)?;
        let result: DjmdLabel = diesel::insert_into(djmdLabel)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    pub fn update_label(&mut self, item: &mut DjmdLabel) -> anyhow::Result<DjmdLabel> {
        // Count differences
        let existing: DjmdLabel = self.get_label_by_id(&item.ID)?.unwrap();
        let mut n: i32 = 0;
        if item.Name != existing.Name {
            n += 1
        }
        if n == 0 {
            return Ok(existing);
        }
        // Update update-time
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        item.updated_at = Some(now);
        // Update USN
        let usn: i32 = self.increment_local_usn(n)?;
        item.rb_local_usn = Some(usn);

        let result: DjmdLabel = diesel::update(&*item)
            .set(&*item)
            .get_result(&mut self.conn)?;
        Ok(result)
    }

    pub fn delete_label(&mut self, id: &str) -> anyhow::Result<usize> {
        let result = diesel::delete(djmdLabel.filter(schema::djmdLabel::ID.eq(id)))
            .execute(&mut self.conn)?;
        self.increment_local_usn(1)?;

        // Remove any references to the key in DjmdContent
        let _ = diesel::update(djmdContent.filter(schema::djmdContent::LabelID.eq(id)))
            .set(schema::djmdContent::LabelID.eq(None::<String>))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- MenuItems --------------------------------------------------------------------------------

    pub fn get_menu_item(&mut self) -> anyhow::Result<Vec<DjmdMenuItems>> {
        let results = djmdMenuItems.load::<DjmdMenuItems>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_menu_item_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdMenuItems>> {
        let result = djmdMenuItems
            .filter(schema::djmdMenuItems::ID.eq(id))
            .first::<DjmdMenuItems>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- MixerParam -------------------------------------------------------------------------------

    pub fn get_mixer_param(&mut self) -> anyhow::Result<Vec<DjmdMixerParam>> {
        let results = djmdMixerParam.load::<DjmdMixerParam>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_mixer_param_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdMixerParam>> {
        let result = djmdMixerParam
            .filter(schema::djmdMixerParam::ID.eq(id))
            .first::<DjmdMixerParam>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- MyTag ------------------------------------------------------------------------------------

    pub fn get_my_tag(&mut self) -> anyhow::Result<Vec<DjmdMyTag>> {
        let results = djmdMyTag.load::<DjmdMyTag>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_my_tag_children(&mut self, parent_id: &str) -> anyhow::Result<Vec<DjmdMyTag>> {
        let results = djmdMyTag
            .filter(schema::djmdMyTag::ParentID.eq(parent_id))
            .order_by(schema::djmdMyTag::Seq)
            .load::<DjmdMyTag>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_my_tag_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdMyTag>> {
        let result = djmdMyTag
            .filter(schema::djmdMyTag::ID.eq(id))
            .first::<DjmdMyTag>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_my_tag_songs(&mut self, id: &str) -> anyhow::Result<Vec<DjmdSongMyTag>> {
        let results = schema::djmdSongMyTag::table
            .filter(schema::djmdSongMyTag::MyTagID.eq(id))
            .order_by(schema::djmdSongMyTag::TrackNo)
            .load::<DjmdSongMyTag>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_my_tag_contents(&mut self, id: &str) -> anyhow::Result<Vec<DjmdContent>> {
        let songs = self.get_my_tag_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    // -- Playlist ---------------------------------------------------------------------------------

    pub fn get_playlist(&mut self) -> anyhow::Result<Vec<DjmdPlaylist>> {
        let results = djmdPlaylist.load::<DjmdPlaylist>(&mut self.conn)?;
        Ok(results)
    }

    /// Return a sorted tree of playlists
    pub fn get_playlist_tree(&mut self) -> anyhow::Result<Vec<Rc<RefCell<PlaylistTreeItem>>>> {
        let playlists: Vec<PlaylistTreeItem> = self
            .get_playlist()?
            .iter()
            .map(|p: &DjmdPlaylist| PlaylistTreeItem::from_playlist(p.clone()))
            .collect();

        let mut map = HashMap::<String, Rc<RefCell<PlaylistTreeItem>>>::new();
        let mut roots = Vec::new();

        // Populate the map with nodes
        for pl in playlists.iter() {
            let item = Rc::new(RefCell::new(pl.clone()));
            map.insert(pl.ID.clone(), item.clone());
            if pl.ParentID.is_none() || pl.ParentID == Some("root".to_string()) {
                roots.push(item.clone());
            }
        }

        // Build the tree structure
        for id in map.keys() {
            let node = map.get(id).unwrap();
            if let Some(parent_id) = node.borrow().ParentID.clone() {
                if let Some(parent_node) = map.get(&parent_id) {
                    parent_node.borrow_mut().Children.push(node.clone());
                }
            }
        }
        sort_tree_list(&mut roots);

        Ok(roots)
    }

    pub fn get_playlist_children(&mut self, parent_id: &str) -> anyhow::Result<Vec<DjmdPlaylist>> {
        let results = djmdPlaylist
            .filter(schema::djmdPlaylist::ParentID.eq(parent_id))
            .order_by(schema::djmdPlaylist::Seq)
            .load::<DjmdPlaylist>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_playlist_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdPlaylist>> {
        let result = djmdPlaylist
            .filter(schema::djmdPlaylist::ID.eq(id))
            .first::<DjmdPlaylist>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_playlist_by_path(
        &mut self,
        path: Vec<&str>,
    ) -> anyhow::Result<Option<DjmdPlaylist>> {
        let mut parent_id: String = "root".to_string();
        let mut playlist: Option<DjmdPlaylist> = None;
        for name in path {
            let result = djmdPlaylist
                .filter(schema::djmdPlaylist::ParentID.eq(&parent_id))
                .filter(schema::djmdPlaylist::Name.eq(name))
                .first::<DjmdPlaylist>(&mut self.conn)
                .optional()?;
            playlist = result.clone();
            if let Some(result) = result {
                parent_id = result.ID.clone();
            } else {
                return Ok(None);
            }
        }
        Ok(playlist)
    }

    pub fn get_playlist_songs(&mut self, id: &str) -> anyhow::Result<Vec<DjmdSongPlaylist>> {
        let results = schema::djmdSongPlaylist::table
            .filter(schema::djmdSongPlaylist::PlaylistID.eq(id))
            .order_by(schema::djmdSongPlaylist::TrackNo)
            .load::<DjmdSongPlaylist>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_playlist_contents(&mut self, id: &str) -> anyhow::Result<Vec<DjmdContent>> {
        let songs = self.get_playlist_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    pub fn get_playlist_song_by_id(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Option<DjmdSongPlaylist>> {
        let result = djmdSongPlaylist
            .filter(schema::djmdSongPlaylist::ID.eq(id))
            .first::<DjmdSongPlaylist>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    fn playlist_type(&mut self, id: &String) -> anyhow::Result<PlaylistType> {
        if id == "root" {
            return Ok(PlaylistType::Folder);
        }
        let result = djmdPlaylist
            .filter(schema::djmdPlaylist::ID.eq(id))
            .select(schema::djmdPlaylist::Attribute)
            .first::<Option<i32>>(&mut self.conn)?;
        let attr = result.expect("Playlist not found");
        let ptype = PlaylistType::try_from(attr).expect("Invalid playlist Attribute");
        Ok(ptype)
    }

    fn playlist_exists(&mut self, id: &String) -> anyhow::Result<bool> {
        let id_exists: bool = select(exists(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id))))
            .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    fn playlist_song_exists(&mut self, id: &String) -> anyhow::Result<bool> {
        let id_exists: bool = select(exists(
            djmdSongPlaylist.filter(schema::djmdSongPlaylist::ID.eq(id)),
        ))
        .get_result(&mut self.conn)?;
        Ok(id_exists)
    }

    fn generate_playlist_id(&mut self) -> anyhow::Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.playlist_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    fn generate_playlist_song_id(&mut self) -> anyhow::Result<String> {
        let generator = RandomIdGenerator::new(true);
        let mut id: String = String::new();
        for id_result in generator {
            if let Ok(tmp_id) = id_result {
                let id_exists: bool = self.playlist_song_exists(&tmp_id)?;
                if !id_exists {
                    id = tmp_id;
                    break;
                }
            }
        }
        Ok(id)
    }

    pub fn insert_playlist(
        &mut self,
        name: String,
        attribute: PlaylistType,
        parent_id: Option<String>,
        seq: Option<i32>,
        image_path: Option<String>,
        smart_list: Option<String>,
    ) -> anyhow::Result<DjmdPlaylist> {
        // Handle defaults
        let parent_id = parent_id.unwrap_or("root".to_string());
        if self.playlist_type(&parent_id)? != PlaylistType::Folder {
            return Err(anyhow!("Parent playlist must be a Folder!"));
        };
        if parent_id.as_str() != "root" && !self.playlist_exists(&parent_id)? {
            return Err(anyhow!("Parent playlist does not exist!"));
        };
        // Generate ID/UUID for new playlist
        let id: String = self.generate_playlist_id()?;
        let uuid: String = Uuid::new_v4().to_string();

        // Generate date string
        let utcnow = Utc::now();
        let now: String = format_datetime(utcnow.clone());
        // Count existing playlists with same parent
        let count = djmdPlaylist
            .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
            .count()
            .get_result::<i64>(&mut self.conn)? as i32;

        // Handle seq
        let seq = if let Some(seq) = seq {
            // Playlist is not inserted at end
            if seq <= 0 {
                return Err(anyhow!("seq must be greater than 0!"));
            } else if seq > count + 1 {
                return Err(anyhow!("seq is too high!"));
            };
            // Shift playlists with seq higher than the new seq number by one
            // Shifting the other playlists increments the USN by 1 for *all* moved playlists
            let move_usn = self.increment_local_usn(1)?;
            diesel::update(
                djmdPlaylist
                    .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                    .filter(schema::djmdPlaylist::Seq.ge(seq)),
            )
            .set((
                schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq + 1),
                schema::djmdPlaylist::rb_local_usn.eq(move_usn.clone()),
                schema::djmdPlaylist::updated_at.eq(now.clone()),
            ))
            .execute(&mut self.conn)?;
            seq
        } else {
            // Insert at end (count + 1)
            count + 1
        };

        // Get next USN: We increment by 2 (1 for creating, 1 for renaming from 'New Playlist')
        let usn: i32 = self.increment_local_usn(2)?;

        // Create and insert model
        let item: DjmdPlaylist = DjmdPlaylist::new(
            id.clone(),
            uuid,
            usn,
            now,
            name,
            attribute.clone() as i32,
            parent_id.clone(),
            seq,
            image_path,
            smart_list,
        )?;
        let result: DjmdPlaylist = diesel::insert_into(djmdPlaylist)
            .values(item)
            .get_result(&mut self.conn)?;

        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            pl_xml.add(
                id.clone(),
                parent_id.clone(),
                attribute as i32,
                utcnow.naive_utc(),
            );
            let _ = pl_xml.dump();
        }

        Ok(result)
    }

    pub fn rename_playlist(&mut self, id: &String, name: String) -> anyhow::Result<DjmdPlaylist> {
        let utcnow = Utc::now();
        let now: String = format_datetime(utcnow.clone());
        let usn: i32 = self.increment_local_usn(1)?;
        let result: DjmdPlaylist =
            diesel::update(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id)))
                .set((
                    schema::djmdPlaylist::Name.eq(name),
                    schema::djmdPlaylist::rb_local_usn.eq(usn),
                    schema::djmdPlaylist::updated_at.eq(now),
                ))
                .get_result(&mut self.conn)?;

        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            pl_xml.update(id.to_string(), utcnow.naive_utc());
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }

        Ok(result)
    }

    pub fn move_playlist(
        &mut self,
        id: &String,
        seq: Option<i32>,
        parent_id: Option<String>,
    ) -> anyhow::Result<DjmdPlaylist> {
        // Get playlist
        let playlist: DjmdPlaylist = self.get_playlist_by_id(id)?.expect("Playlist not found");

        // Get parentID (old parentID if None)
        let old_parent_id = playlist.ParentID.expect("No ParentID set");
        let old_seq = playlist.Seq.expect("No Seq set");
        let parent_id: String = parent_id.unwrap_or(old_parent_id.clone());

        // Moving increments USN by 1 for all changes
        let usn = self.increment_local_usn(1)?;
        // Generate date string
        let utcnow = Utc::now();
        let now: String = format_datetime(utcnow.clone());

        if parent_id != old_parent_id {
            let sequence: i32;
            // parent changed, move to new parent
            if self.playlist_type(&parent_id)? != PlaylistType::Folder {
                return Err(anyhow!("Parent playlist must be a Folder!"));
            };
            if parent_id.as_str() != "root" && !self.playlist_exists(&parent_id)? {
                return Err(anyhow!("Parent playlist does not exist!"));
            };

            // Update seq of playlists in old parent:
            // Decrease seq of playlists with seq higher than old seq
            diesel::update(
                djmdPlaylist
                    .filter(schema::djmdPlaylist::ParentID.eq(old_parent_id.clone()))
                    .filter(schema::djmdPlaylist::Seq.gt(old_seq)),
            )
            .set((
                schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq - 1),
                schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                schema::djmdPlaylist::updated_at.eq(now.clone()),
            ))
            .execute(&mut self.conn)?;
            if let Some(seq) = seq {
                // Move to seq in new parent
                diesel::update(
                    djmdPlaylist
                        .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                        .filter(schema::djmdPlaylist::Seq.ge(seq)),
                )
                .set((
                    schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq + 1),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(now.clone()),
                ))
                .execute(&mut self.conn)?;

                sequence = seq;
            } else {
                // If no seq given, move to end of new parent:
                // No seq in new parent have to be updated
                let count = djmdPlaylist
                    .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                    .count()
                    .get_result::<i64>(&mut self.conn)? as i32;
                sequence = count + 1;
            };
            // Update Seq and parent of actual playlist
            diesel::update(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id)))
                .set((
                    schema::djmdPlaylist::Seq.eq(sequence),
                    schema::djmdPlaylist::ParentID.eq(parent_id.clone()),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(now.clone()),
                ))
                .execute(&mut self.conn)?;
        } else {
            let seq = seq.unwrap_or(old_seq.clone());
            // Move to seq in current parent
            if seq > old_seq {
                // Seq is increased (move down in playlist):
                // Decrease seq of all playlists with seq between old seq and new seq
                diesel::update(
                    djmdPlaylist
                        .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                        .filter(schema::djmdPlaylist::Seq.gt(old_seq))
                        .filter(schema::djmdPlaylist::Seq.le(seq)),
                )
                .set((
                    schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq - 1),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(now.clone()),
                ))
                .execute(&mut self.conn)?;
            } else if seq < old_seq {
                // Seq is decreased (moved up in playlist):
                // Increase seq of all playlists with seq between old seq and new seq
                diesel::update(
                    djmdPlaylist
                        .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                        .filter(schema::djmdPlaylist::Seq.lt(old_seq))
                        .filter(schema::djmdPlaylist::Seq.ge(seq)),
                )
                .set((
                    schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq + 1),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(now.clone()),
                ))
                .execute(&mut self.conn)?;
            };
            // Update Seq of actual playlist
            diesel::update(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id)))
                .set((
                    schema::djmdPlaylist::Seq.eq(seq),
                    schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
                    schema::djmdPlaylist::updated_at.eq(now.clone()),
                ))
                .execute(&mut self.conn)?;
        };

        let playlist: DjmdPlaylist = self.get_playlist_by_id(id)?.expect("Playlist not found");

        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            pl_xml.update_parent(id.to_string(), parent_id.clone(), utcnow.naive_utc());
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }

        Ok(playlist)
    }

    pub fn delete_playlist(&mut self, id: &str) -> anyhow::Result<usize> {
        let usn = self.increment_local_usn(1)?;
        // Generate date string
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);

        // Get parentID and seq number
        let playlist: DjmdPlaylist = self.get_playlist_by_id(id)?.expect("Playlist not found");
        let attr = playlist.Attribute.unwrap();
        let playlist_type = PlaylistType::try_from(attr).expect("Invalid playlist Attribute");
        let parent_id = playlist.ParentID.unwrap();
        let seq = playlist.Seq.unwrap();

        // Decrease seq of all playlists with seq higher then seq of deleted playlist
        diesel::update(
            djmdPlaylist
                .filter(schema::djmdPlaylist::ParentID.eq(parent_id.clone()))
                .filter(schema::djmdPlaylist::Seq.gt(seq)),
        )
        .set((
            schema::djmdPlaylist::Seq.eq(schema::djmdPlaylist::Seq - 1),
            schema::djmdPlaylist::rb_local_usn.eq(usn.clone()),
            schema::djmdPlaylist::updated_at.eq(now.clone()),
        ))
        .execute(&mut self.conn)?;

        // Delete playlist
        let mut result = diesel::delete(djmdPlaylist.filter(schema::djmdPlaylist::ID.eq(id)))
            .execute(&mut self.conn)?;

        // Cascade delete references
        if playlist_type == PlaylistType::Playlist {
            // Delete all songs in playlist
            let n = diesel::delete(
                djmdSongPlaylist.filter(schema::djmdSongPlaylist::PlaylistID.eq(id)),
            )
            .execute(&mut self.conn)?;
            result += n;
        } else if playlist_type == PlaylistType::Folder {
            // Select child ids
            let children: Vec<String> = djmdPlaylist
                .filter(schema::djmdPlaylist::ParentID.eq(id))
                .select(schema::djmdPlaylist::ID)
                .load::<String>(&mut self.conn)?;
            // Delete all children recursively
            for child_id in children {
                let n = self.delete_playlist(&child_id)?;
                result += n;
            }
        }

        if let Some(pl_xml_path) = self.plxml_path.clone() {
            let mut pl_xml = MasterPlaylistXml::load(pl_xml_path);
            pl_xml.remove(id.to_string());
            let _ = pl_xml.dump();
        } else {
            eprintln!("WARNING: Coulnd't update playlist XML, file not found!");
        }

        Ok(result)
    }

    pub fn insert_playlist_song(
        &mut self,
        playlist_id: &String,
        content_id: &String,
        seq: Option<i32>,
    ) -> anyhow::Result<DjmdSongPlaylist> {
        if self.playlist_type(&playlist_id)? != PlaylistType::Playlist {
            return Err(anyhow!(
                "Playlist with ID {} must be a Playlist!",
                playlist_id
            ));
        };
        if !self.playlist_exists(&playlist_id)? {
            return Err(anyhow!("Playlist with ID {} does not exist!", playlist_id));
        };
        if !self.content_exists(&content_id)? {
            return Err(anyhow!("Content with ID {} does not exist!", content_id));
        };

        // Generate ID/UUID/Date for new playlist song
        let id: String = self.generate_playlist_song_id()?;
        let uuid: String = Uuid::new_v4().to_string();
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);

        // Count existing songs in playlist
        let count = djmdSongPlaylist
            .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
            .count()
            .get_result::<i64>(&mut self.conn)? as i32;

        // Get next USN: We increment by 1 *before* moving others
        let usn: i32 = self.increment_local_usn(1)?;

        // Handle seq
        let seq = if let Some(seq) = seq {
            // Song is not inserted at end
            if seq <= 0 {
                return Err(anyhow!("seq must be greater than 0!"));
            } else if seq > count + 1 {
                return Err(anyhow!("seq is too high!"));
            };
            // Shift songs with seq higher than the new seq number by one
            // Shifting the other songs increments the USN by 1 for *all* moved songs
            let move_usn = self.increment_local_usn(1)?;
            diesel::update(
                djmdSongPlaylist
                    .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
                    .filter(schema::djmdSongPlaylist::TrackNo.ge(seq)),
            )
            .set((
                schema::djmdSongPlaylist::TrackNo.eq(schema::djmdSongPlaylist::TrackNo + 1),
                schema::djmdSongPlaylist::rb_local_usn.eq(move_usn.clone()),
                schema::djmdSongPlaylist::updated_at.eq(now.clone()),
            ))
            .execute(&mut self.conn)?;
            seq
        } else {
            // Insert at end (count + 1)
            count + 1
        };

        let item = DjmdSongPlaylist::new(
            id,
            uuid,
            usn,
            now,
            playlist_id.clone(),
            content_id.clone(),
            seq,
        )?;
        let result: DjmdSongPlaylist = diesel::insert_into(djmdSongPlaylist)
            .values(item)
            .get_result(&mut self.conn)?;

        Ok(result)
    }

    pub fn move_playlist_song(
        &mut self,
        id: &String,
        seq: i32,
    ) -> anyhow::Result<DjmdSongPlaylist> {
        // Get playlistID and seq number
        let song: DjmdSongPlaylist = self.get_playlist_song_by_id(id)?.expect("Song not found");
        if song.TrackNo.unwrap() == seq {
            // Nothing to change
            return Ok(song);
        }

        let playlist_id = song.PlaylistID.unwrap();
        let old_seq = song.TrackNo.unwrap();

        let usn = self.increment_local_usn(1)?;
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);
        if seq > old_seq {
            // Seq is increased (move down in playlist):
            // Decrease seq of all songs with seq between old seq and new seq
            diesel::update(
                djmdSongPlaylist
                    .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
                    .filter(schema::djmdSongPlaylist::TrackNo.gt(old_seq))
                    .filter(schema::djmdSongPlaylist::TrackNo.le(seq)),
            )
            .set((
                schema::djmdSongPlaylist::TrackNo.eq(schema::djmdSongPlaylist::TrackNo - 1),
                schema::djmdSongPlaylist::rb_local_usn.eq(usn.clone()),
                schema::djmdSongPlaylist::updated_at.eq(now.clone()),
            ))
            .execute(&mut self.conn)?;
        } else {
            // Seq is decreased (moved up in playlist):
            // Increase seq of all songs with seq between old seq and new seq
            diesel::update(
                djmdSongPlaylist
                    .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
                    .filter(schema::djmdSongPlaylist::TrackNo.lt(old_seq))
                    .filter(schema::djmdSongPlaylist::TrackNo.ge(seq)),
            )
            .set((
                schema::djmdSongPlaylist::TrackNo.eq(schema::djmdSongPlaylist::TrackNo + 1),
                schema::djmdSongPlaylist::rb_local_usn.eq(usn.clone()),
                schema::djmdSongPlaylist::updated_at.eq(now.clone()),
            ))
            .execute(&mut self.conn)?;
        };
        // Update Seq of actual song
        diesel::update(djmdSongPlaylist.filter(schema::djmdSongPlaylist::ID.eq(id)))
            .set((
                schema::djmdSongPlaylist::TrackNo.eq(seq),
                schema::djmdSongPlaylist::rb_local_usn.eq(usn.clone()),
                schema::djmdSongPlaylist::updated_at.eq(now.clone()),
            ))
            .execute(&mut self.conn)?;

        let result: DjmdSongPlaylist = self.get_playlist_song_by_id(id)?.expect("Song not found");

        Ok(result)
    }

    pub fn delete_playlist_song(&mut self, id: &str) -> anyhow::Result<usize> {
        let usn = self.increment_local_usn(1)?;
        let utcnow: DateTime<Utc> = Utc::now();
        let now: String = format_datetime(utcnow);

        // Get playlist and seq number
        let song: DjmdSongPlaylist = self.get_playlist_song_by_id(id)?.expect("Song not found");
        let playlist_id = song.PlaylistID.unwrap();
        let seq = song.TrackNo.unwrap();

        // Decrease seq of all songs with seq higher then seq of deleted song
        diesel::update(
            djmdSongPlaylist
                .filter(schema::djmdSongPlaylist::PlaylistID.eq(playlist_id.clone()))
                .filter(schema::djmdSongPlaylist::TrackNo.gt(seq)),
        )
        .set((
            schema::djmdSongPlaylist::TrackNo.eq(schema::djmdSongPlaylist::TrackNo - 1),
            schema::djmdSongPlaylist::rb_local_usn.eq(usn.clone()),
            schema::djmdSongPlaylist::updated_at.eq(now.clone()),
        ))
        .execute(&mut self.conn)?;

        let result = diesel::delete(djmdSongPlaylist.filter(schema::djmdSongPlaylist::ID.eq(id)))
            .execute(&mut self.conn)?;

        Ok(result)
    }

    // -- Property ---------------------------------------------------------------------------------

    pub fn get_property(&mut self) -> anyhow::Result<Vec<DjmdProperty>> {
        let results = djmdProperty.load::<DjmdProperty>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_property_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdProperty>> {
        let result = djmdProperty
            .filter(schema::djmdProperty::DBID.eq(id))
            .first::<DjmdProperty>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- CloudProperty ----------------------------------------------------------------------------

    pub fn get_cloud_property(&mut self) -> anyhow::Result<Vec<DjmdCloudProperty>> {
        let results = djmdCloudProperty.load::<DjmdCloudProperty>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_cloud_property_by_id(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Option<DjmdCloudProperty>> {
        let result = djmdCloudProperty
            .filter(schema::djmdCloudProperty::ID.eq(id))
            .first::<DjmdCloudProperty>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- RecommendLike ----------------------------------------------------------------------------

    pub fn get_recommend_like(&mut self) -> anyhow::Result<Vec<DjmdRecommendLike>> {
        let results = djmdRecommendLike.load::<DjmdRecommendLike>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_recommend_like_by_id(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Option<DjmdRecommendLike>> {
        let result = djmdRecommendLike
            .filter(schema::djmdRecommendLike::ID.eq(id))
            .first::<DjmdRecommendLike>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- RelatedTracks ----------------------------------------------------------------------------

    pub fn get_related_tracks(&mut self) -> anyhow::Result<Vec<DjmdRelatedTracks>> {
        let results = djmdRelatedTracks.load::<DjmdRelatedTracks>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_related_tracks_children(
        &mut self,
        parent_id: &str,
    ) -> anyhow::Result<Vec<DjmdRelatedTracks>> {
        let results = djmdRelatedTracks
            .filter(schema::djmdRelatedTracks::ParentID.eq(parent_id))
            .order_by(schema::djmdRelatedTracks::Seq)
            .load::<DjmdRelatedTracks>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_related_tracks_by_id(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Option<DjmdRelatedTracks>> {
        let result = djmdRelatedTracks
            .filter(schema::djmdRelatedTracks::ID.eq(id))
            .first::<DjmdRelatedTracks>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_related_tracks_songs(
        &mut self,
        id: &str,
    ) -> anyhow::Result<Vec<DjmdSongRelatedTracks>> {
        let results = schema::djmdSongRelatedTracks::table
            .filter(schema::djmdSongRelatedTracks::RelatedTracksID.eq(id))
            .order_by(schema::djmdSongRelatedTracks::TrackNo)
            .load::<DjmdSongRelatedTracks>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_related_tracks_contents(&mut self, id: &str) -> anyhow::Result<Vec<DjmdContent>> {
        let songs = self.get_related_tracks_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    // -- Sampler ----------------------------------------------------------------------------------

    pub fn get_sampler(&mut self) -> anyhow::Result<Vec<DjmdSampler>> {
        let results = djmdSampler.load::<DjmdSampler>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_sampler_children(&mut self, parent_id: &str) -> anyhow::Result<Vec<DjmdSampler>> {
        let results = djmdSampler
            .filter(schema::djmdSampler::ParentID.eq(parent_id))
            .order_by(schema::djmdSampler::Seq)
            .load::<DjmdSampler>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_sampler_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdSampler>> {
        let result = djmdSampler
            .filter(schema::djmdSampler::ID.eq(id))
            .first::<DjmdSampler>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    pub fn get_sampler_songs(&mut self, id: &str) -> anyhow::Result<Vec<DjmdSongSampler>> {
        let results = schema::djmdSongSampler::table
            .filter(schema::djmdSongSampler::SamplerID.eq(id))
            .order_by(schema::djmdSongSampler::TrackNo)
            .load::<DjmdSongSampler>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_sampler_contents(&mut self, id: &str) -> anyhow::Result<Vec<DjmdContent>> {
        let songs = self.get_sampler_songs(id)?;
        let ids: Vec<&str> = songs
            .iter()
            .map(|s| s.ContentID.as_ref().unwrap().as_str())
            .collect();
        let result = self.get_contents_by_ids(ids)?;
        Ok(result)
    }

    // -- SongTagList ------------------------------------------------------------------------------

    pub fn get_song_tag_list(&mut self) -> anyhow::Result<Vec<DjmdSongTagList>> {
        let results = djmdSongTagList
            .order_by(schema::djmdSongTagList::TrackNo)
            .load::<DjmdSongTagList>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_song_tag_list_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdSongTagList>> {
        let result = djmdSongTagList
            .filter(schema::djmdSongTagList::ID.eq(id))
            .first::<DjmdSongTagList>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- Sort -------------------------------------------------------------------------------------

    pub fn get_sort(&mut self) -> anyhow::Result<Vec<DjmdSort>> {
        let results = djmdSort
            .order_by(schema::djmdSort::Seq)
            .load::<DjmdSort>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_sort_by_id(&mut self, id: &str) -> anyhow::Result<Option<DjmdSort>> {
        let result = djmdSort
            .filter(schema::djmdSort::ID.eq(id))
            .first::<DjmdSort>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- ImageFile --------------------------------------------------------------------------------

    pub fn get_image_file(&mut self) -> anyhow::Result<Vec<ImageFile>> {
        let results = imageFile.load::<ImageFile>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_image_file_by_id(&mut self, id: &str) -> anyhow::Result<Option<ImageFile>> {
        let result = imageFile
            .filter(schema::imageFile::ID.eq(id))
            .first::<ImageFile>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- SettingFile ------------------------------------------------------------------------------

    pub fn get_setting_file(&mut self) -> anyhow::Result<Vec<SettingFile>> {
        let results = settingFile.load::<SettingFile>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_setting_file_by_id(&mut self, id: &str) -> anyhow::Result<Option<SettingFile>> {
        let result = settingFile
            .filter(schema::settingFile::ID.eq(id))
            .first::<SettingFile>(&mut self.conn)
            .optional()?;
        Ok(result)
    }

    // -- UuidIDMap --------------------------------------------------------------------------------

    pub fn get_uuid_id_map(&mut self) -> anyhow::Result<Vec<UuidIDMap>> {
        let results = uuidIDMap.load::<UuidIDMap>(&mut self.conn)?;
        Ok(results)
    }

    pub fn get_uuid_id_map_by_id(&mut self, id: &str) -> anyhow::Result<Option<UuidIDMap>> {
        let result = uuidIDMap
            .filter(schema::uuidIDMap::ID.eq(id))
            .first::<UuidIDMap>(&mut self.conn)
            .optional()?;
        Ok(result)
    }
}
