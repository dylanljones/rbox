// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Discovery tests using isolated application profiles and program directories.

use rbox_core::{
    error::Error,
    locate::{find_programs_in, LocationSource, LookupPaths},
    RekordboxOptions,
};
use std::{fs, path::Path};

fn write(path: &Path, content: impl AsRef<[u8]>) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn settings(paths: &LookupPaths, directory: &Path) {
    let value = quick_xml::escape::escape(directory.to_str().unwrap());
    write(
        &paths.settings_file,
        format!("<PROPERTIES><VALUE val=\"{value}\" name=\"masterDbDirectory\"/></PROPERTIES>"),
    );
}

fn options(paths: &LookupPaths, database: &Path) {
    write(
        &paths.options_file,
        serde_json::to_vec(&serde_json::json!({"options": [
            ["db-path", database],
            ["analysis-data-root-path", database.parent().unwrap().join("custom-analysis")],
            ["settings-root-path", database.parent().unwrap().join("custom-settings")]
        ]}))
        .unwrap(),
    );
}

#[test]
fn missing_profile_is_reported_without_creating_anything() {
    let root = tempfile::tempdir().unwrap();
    let paths = LookupPaths::under(root.path());
    let found = paths.locate().unwrap();
    assert!(!found.available);
    assert_eq!(found.source, LocationSource::Default);
    assert_eq!(
        found.db_path,
        root.path().join("Pioneer/rekordbox/master.db")
    );
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn defaults_try_each_candidate_and_reject_directories_as_databases() {
    let root = tempfile::tempdir().unwrap();
    let mut paths = LookupPaths::under(root.path());
    fs::create_dir_all(paths.default_library_dirs[0].join("master.db")).unwrap();
    paths
        .default_library_dirs
        .push(root.path().join("fallback"));
    let db = paths.default_library_dirs[1].join("master.db");
    write(&db, b"fixture");
    let found = paths.locate().unwrap();
    assert!(found.available);
    assert_eq!(found.db_path, db);
}

#[test]
fn options_preserve_configured_analysis_and_settings_paths() {
    let root = tempfile::tempdir().unwrap();
    let paths = LookupPaths::under(root.path());
    let db = root.path().join("external/master.db");
    options(&paths, &db);
    write(&db, b"fixture");
    let found = paths.locate().unwrap();
    assert!(found.available);
    assert_eq!(found.source, LocationSource::Options);
    assert_eq!(found.db_path, db);
    assert_eq!(
        found.analysis_root,
        root.path().join("external/custom-analysis")
    );
    assert_eq!(
        found.settings_root,
        root.path().join("external/custom-settings")
    );
}

#[test]
fn settings_win_over_stale_options_and_decode_xml_entities() {
    let root = tempfile::tempdir().unwrap();
    let paths = LookupPaths::under(root.path());
    let directory = root.path().join("Music & DJ's tracks");
    settings(&paths, &directory);
    options(&paths, &root.path().join("stale/master.db"));
    write(&directory.join("master.db"), b"fixture");
    let before = fs::read(&paths.settings_file).unwrap();
    let found = paths.locate().unwrap();
    assert_eq!(found.source, LocationSource::Settings);
    assert_eq!(found.db_path, directory.join("master.db"));
    assert_eq!(found.analysis_root, directory.join("share"));
    assert_eq!(found.settings_root, paths.settings_file.parent().unwrap());
    assert_eq!(fs::read(&paths.settings_file).unwrap(), before);
}

#[test]
fn matching_options_supply_custom_analysis_for_settings_database() {
    let root = tempfile::tempdir().unwrap();
    let paths = LookupPaths::under(root.path());
    let directory = root.path().join("library");
    settings(&paths, &directory);
    options(&paths, &directory.join("master.db"));
    let found = paths.locate().unwrap();
    assert_eq!(found.source, LocationSource::Settings);
    assert_eq!(found.analysis_root, directory.join("custom-analysis"));
}

#[test]
fn missing_configured_drive_does_not_fall_back_to_existing_default() {
    for use_settings in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let paths = LookupPaths::under(root.path());
        let directory = root.path().join("disconnected");
        write(
            &paths.default_library_dirs[0].join("master.db"),
            b"other library",
        );
        if use_settings {
            settings(&paths, &directory);
        } else {
            options(&paths, &directory.join("master.db"));
        }
        let found = paths.locate().unwrap();
        assert!(!found.available);
        assert_eq!(found.db_path, directory.join("master.db"));
        assert!(!directory.exists());
    }
}

#[test]
fn absent_or_relative_setting_uses_options() {
    for value in [
        "<PROPERTIES/>",
        "<PROPERTIES><VALUE name=\"masterDbDirectory\" val=\"relative\"/></PROPERTIES>",
    ] {
        let root = tempfile::tempdir().unwrap();
        let paths = LookupPaths::under(root.path());
        write(&paths.settings_file, value);
        options(&paths, &root.path().join("library/master.db"));
        assert_eq!(paths.locate().unwrap().source, LocationSource::Options);
    }
}

#[test]
fn corrupt_configuration_is_not_silently_ignored() {
    let root = tempfile::tempdir().unwrap();
    let paths = LookupPaths::under(root.path());
    write(&paths.options_file, b"{broken");
    assert!(matches!(paths.locate(), Err(Error::Json(_))));
    fs::remove_file(&paths.options_file).unwrap();
    write(&paths.settings_file, b"<PROPERTIES><VALUE");
    assert!(matches!(paths.locate(), Err(Error::Xml(_))));
}

#[test]
fn invalid_options_and_empty_default_list_are_errors() {
    let root = tempfile::tempdir().unwrap();
    let mut paths = LookupPaths::under(root.path());
    write(&paths.options_file, br#"{"options":[["db-path", ""],["analysis-data-root-path","share"],["settings-root-path","settings"]]}"#);
    assert!(matches!(paths.locate(), Err(Error::OptionError(_))));
    paths.default_library_dirs.clear();
    assert!(matches!(paths.locate(), Err(Error::OptionError(_))));
    let options = RekordboxOptions::new("master.db", "share", "settings").unwrap();
    assert_eq!(options.get_db_dir(), Path::new("."));
}

#[test]
fn program_versions_sort_numerically_and_duplicate_roots_are_deduplicated() {
    let root = tempfile::tempdir().unwrap();
    for version in ["6.9.0", "6.10.0", "7.0.1"] {
        write(
            &root
                .path()
                .join(format!("Pioneer/rekordbox {version}/rekordbox.exe")),
            b"fixture",
        );
    }
    write(&root.path().join("unrelated/rekordbox.exe"), b"ignored");
    let programs = find_programs_in([root.path().to_owned(), root.path().to_owned()]).unwrap();
    assert_eq!(programs.len(), 3);
    assert_eq!(
        programs
            .iter()
            .map(|p| p.version.clone())
            .collect::<Vec<_>>(),
        vec![vec![7, 0, 1], vec![6, 10, 0], vec![6, 9, 0]]
    );
}

#[test]
fn macos_bundles_require_an_executable_and_support_direct_roots() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("rekordbox 7/rekordbox.app");
    let executable = bundle.join("Contents/MacOS/rekordbox");
    write(&executable, b"fixture");
    fs::create_dir_all(root.path().join("rekordbox 8.app")).unwrap();
    let programs = find_programs_in([root.path().to_owned(), bundle]).unwrap();
    assert_eq!(programs.len(), 1);
    assert_eq!(programs[0].executable, executable);
    assert_eq!(programs[0].version, vec![7]);
}
