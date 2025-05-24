// Author: Dylan Jones
// Date:   2025-05-08

use rbox::xml::RekordboxXml;

mod common;

#[test]
fn test_read_xml() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path()?;
    let _xml = RekordboxXml::load(path);
    Ok(())
}

#[test]
fn test_write_xml() -> anyhow::Result<()> {
    let path = common::setup_rekordbox_xml_path()?;
    let out = common::testdata_demo_dir()?.join("database-out.xml");
    let xml = RekordboxXml::load(path);
    // Write to input file
    xml.dump()?;
    // Write to new file
    xml.dump_copy(out)?;
    Ok(())
}
