#![allow(unused)]

use rbox::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut db = MasterDb::new(
        "/home/jonesdyl/Nextcloud/dev/rust/rbox/.testdata/RBv6/demo/rekordbox/master.db",
    )?;

    let albums = db.get_album()?;
    for album in albums {
        println!("{:?}", album);
    }

    Ok(())
}
