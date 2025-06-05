// Author: Dylan Jones
// Date:   2025-05-15

use super::models::DjmdPlaylistTreeItem;
use chrono::{DateTime, TimeZone, Utc};
use std::cell::RefCell;
use std::rc::Rc;

const DATEFMT: &str = "%Y-%m-%d %H:%M:%S%.3f %:z";

pub fn parse_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
    // If string contains additional info in brackets (...), remove it
    let idx = s.find('(');
    let s = if let Some(idx) = idx { &s[..idx] } else { s };
    DateTime::parse_from_str(s, DATEFMT)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| anyhow::anyhow!("Failed to parse datetime: {}", e))
}

pub fn format_datetime<Tz: TimeZone>(dt: &DateTime<Tz>) -> String {
    dt.with_timezone(&Utc).format(DATEFMT).to_string()
}

pub trait RekordboxDateString {
    fn from_date<Tz: TimeZone>(dt: DateTime<Tz>) -> Self;
    fn into_date(self) -> anyhow::Result<DateTime<Utc>>;
}

impl RekordboxDateString for String {
    fn from_date<Tz: TimeZone>(dt: DateTime<Tz>) -> Self {
        format_datetime(&dt)
    }

    fn into_date(self) -> anyhow::Result<DateTime<Utc>> {
        parse_datetime(&self)
    }
}

fn sort_tree(item: &mut Rc<RefCell<DjmdPlaylistTreeItem>>) {
    // Sort the current node's children by a chosen criterion (e.g., ID or name)
    item.borrow_mut().Children.sort_by(|a, b| {
        let a_seq = a.borrow().Seq;
        let b_seq = b.borrow().Seq;
        a_seq.cmp(&b_seq)
    });
    // Recursively sort each child
    for child in &mut item.borrow_mut().Children {
        sort_tree(child);
    }
}

pub fn sort_tree_list(tree: &mut Vec<Rc<RefCell<DjmdPlaylistTreeItem>>>) {
    // Sort the root nodes
    tree.sort_by(|a, b| {
        let a_seq = a.borrow().Seq;
        let b_seq = b.borrow().Seq;
        a_seq.cmp(&b_seq)
    });
    // Sort each tree item recursively
    for item in tree {
        sort_tree(item);
    }
}
