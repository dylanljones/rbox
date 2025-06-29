#![allow(unused)]

use rbox::prelude::*;
use rbox::settings::OnAirDisplay;

fn main() -> anyhow::Result<()> {
    // let mut db = MasterDb::open()?;

    let p = r"D:\Dropbox\Dev\Music\rbox\.testdata\RBv6\demo\rekordbox6\MYSETTING.DAT";
    let mut sett = Setting::new_mysetting(p)?;
    let item = sett.get_phase_meter()?;
    let itemstr: String = item.to_string();
    println!("{:#?}", item);
    println!("{:#?}", itemstr);
    // sett.dump()?;

    Ok(())
}
