#![allow(unused)]

use rbox::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut db = MasterDb::open()?;
    Ok(())
}
