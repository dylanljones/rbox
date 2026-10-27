// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Track relocation preserves each analysis file's embedded path convention.

use rbox_anlz::Anlz;
use rbox_core::{locate::LocationSource, LibraryLocation, PathExt};
use rbox_db::Model;
use rbox_masterdb::{DjmdContent, MasterDb};
use rbox_testing::data::setup_master_db_path;

#[rstest::rstest]
#[case(6)]
#[case(7)]
fn preserves_anlz_path_formats(#[case] version: usize) -> anyhow::Result<()> {
    let fixture = setup_master_db_path(version);
    let root = tempfile::tempdir()?;
    let location = LibraryLocation {
        db_path: fixture.path().into(),
        analysis_root: root.path().into(),
        settings_root: root.path().into(),
        source: LocationSource::Default,
        available: true,
    };
    let mut db = MasterDb::from_location(&location)?;
    // All writes target our temporary database and analysis files.
    db.set_unsafe_writes(true);
    let mut conn = db.pool.get()?;
    let mut content = DjmdContent::all(&mut conn)?.remove(0);
    content.analysis_data_path = Some("/ANLZ0000.DAT".into());
    let content = content.update(&mut conn)?;
    drop(conn);

    let dat_path = root.path().join("ANLZ0000.DAT");
    let ext_path = root.path().join("ANLZ0000.EXT");
    let ex2_path = root.path().join("ANLZ0000.2EX");
    let mut dat = Anlz::new_dat(&dat_path)?;
    dat.set_path("?/old.mp3")?;
    dat.dump()?;
    let mut ext = Anlz::new_ext(&ext_path)?;
    ext.set_path("C:/Music/old.mp3")?;
    ext.dump()?;
    let mut ex2 = Anlz::new_2ex(&ex2_path)?;
    ex2.set_path("?/old.mp3")?;
    ex2.dump()?;
    drop((dat, ext, ex2));

    // Exercise both a rename and a subsequent directory-only relocation.
    for directory in ["first", "second"] {
        let folder = root.path().join(directory);
        std::fs::create_dir(&folder)?;
        let audio = folder.join("renamed track.mp3");
        std::fs::write(&audio, [])?;
        let full_path = audio.normalize_unix_string().unwrap();
        assert_eq!(db.update_content_path(&content.id, &full_path)?, 1);
        assert_eq!(
            Anlz::load(&dat_path)?.get_path()?.as_deref(),
            Some("?/renamed track.mp3")
        );
        assert_eq!(
            Anlz::load(&ext_path)?.get_path()?.as_deref(),
            Some(full_path.as_str())
        );
        assert_eq!(
            Anlz::load(&ex2_path)?.get_path()?.as_deref(),
            Some("?/renamed track.mp3")
        );
        let mut conn = db.pool.get()?;
        assert_eq!(
            DjmdContent::find(&mut conn, &content.id)?
                .unwrap()
                .folder_path
                .as_deref(),
            Some(full_path.as_str())
        );
    }
    Ok(())
}
