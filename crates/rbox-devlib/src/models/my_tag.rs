// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! My Tags, categories and track associations in the extended DeviceSQL database.
//!
//! [MyTag] and [NewMyTag] describe tags and categories in table 3.
//! [MyTagContent] and [NewMyTagContent] describe associations in table 4, linking
//! extended tags to tracks in the accompanying standard database. Queries do not
//! load related rows automatically and retain repeated associations.
//! Updates and deletions commit in memory; saving is explicit. Deleting a category
//! also removes descendants and their associations, without deleting contents.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};

use super::{read_fields, text, text_error, unsupported, validate_id, validate_nonzero_id};
use crate::binary::{
    ExtPageType, ExtendedRow, MyTagHeader, MyTagRow, Row, RowRecord, StringOffsets, TableKind,
    TrackMyTagRow,
};
use crate::devicesql::{DeviceSqlConnection, Transaction};
use crate::{Error, Result};

/// A detached My Tag or category from the extended database.
///
/// IDs are preserved verbatim, without remapping or assuming sequential numbering.
/// Public-field edits affect only this snapshot. Parents and tracks are not loaded.
#[derive(Clone, PartialEq, Eq)]
pub struct MyTag {
    /// Export tag/category ID.
    pub id: u32,
    /// Parent category ID; zero denotes a root in observed exports.
    pub parent_id: u32,
    /// Zero-based position within the parent/category list.
    pub seq: u32,
    /// Decoded display name.
    pub name: String,
    /// Raw attribute: zero for a tag, one for a category; other values are retained.
    pub attribute: u8,
    raw: RowRecord,
}

impl std::fmt::Debug for MyTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyTag")
            .field("id", &self.id)
            .field("parent_id", &self.parent_id)
            .field("seq", &self.seq)
            .field("name", &self.name)
            .field("attribute", &self.attribute)
            .finish()
    }
}

impl MyTag {
    /// Extended database table containing tags and categories.
    pub const TABLE: TableKind = TableKind::Extended(ExtPageType::MyTags);

    /// Stream live rows in physical order. Creating the iterator reads no pages.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live tags/categories without resolving related rows.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Find by export ID using the engine index; duplicate IDs are an error.
    pub fn find<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        id: &u32,
    ) -> Result<Option<Self>> {
        let location = conn
            .index_rows(Self::TABLE, Self::read_id)?
            .get(id)
            .copied();
        location
            .map(|location| Self::try_from(conn.read_indexed_row(Self::TABLE, location)?))
            .transpose()
    }

    /// Check an ID without decoding name text or resolving string offsets.
    pub fn id_exists<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<bool> {
        Ok(conn
            .index_rows(Self::TABLE, Self::read_id)?
            .contains_key(id))
    }

    /// Read immediate children sorted by stored position; zero selects roots.
    /// This scans the table. Ties retain physical order; hierarchy validation and
    /// recursive traversal are not performed.
    pub fn by_parent<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        parent_id: &u32,
    ) -> Result<Vec<Self>> {
        let mut children = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.parent_id == *parent_id {
                children.push(row);
            }
        }
        children.sort_by_key(|row| row.seq);
        Ok(children)
    }

    /// Known category status, or `None` for an unknown raw attribute.
    pub fn is_category(&self) -> Option<bool> {
        match self.attribute {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    /// Persist edited name, parent and position; ID and attribute are immutable.
    ///
    /// Unedited fields, reserved values and the second string frame are preserved.
    /// Edited fields use last-write-wins. A changed nonzero parent must exist and
    /// be a category; self-parenting and cyclic or broken ancestry are rejected.
    /// Zero selects the root. Siblings are not renumbered. Commits in memory only;
    /// errors roll back and saving remains explicit.
    /// String edits remain subject to the engine's supported allocation layouts;
    /// unsupported growth returns an error without discarding unknown data.
    pub fn update<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<Self> {
        conn.check_kind(Self::TABLE)?;
        let original = Self::try_from(self.raw)?;
        validate_id(Self::TABLE, self.id)?;
        for (field, changed) in [
            ("id", self.id != original.id),
            ("attribute", self.attribute != original.attribute),
        ] {
            if changed {
                return Err(Error::InvalidModelField {
                    table: Self::TABLE,
                    field,
                    reason: "existing tag identity and category status cannot be changed",
                });
            }
        }
        let mut tx = conn.transaction();
        let location = tx
            .index_rows(Self::TABLE, Self::read_id)?
            .get(&original.id)
            .copied()
            .ok_or(Error::ModelNotFound {
                table: Self::TABLE,
                id: original.id,
            })?;
        let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
        let record = tx.read(&handle)?;
        let Row::Extended(ExtendedRow::MyTag(mut row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if self.parent_id != original.parent_id {
            let headers = tag_headers(&mut tx)?;
            let mut visited = HashSet::from([original.id]);
            let mut parent = self.parent_id;
            while parent != 0 {
                if !visited.insert(parent) {
                    return Err(invalid_parent("parent ancestry must not contain a cycle"));
                }
                let header = headers
                    .get(&parent)
                    .ok_or_else(|| invalid_parent("parent category does not exist"))?;
                if header.attribute != 1 {
                    return Err(invalid_parent("parent must be a category"));
                }
                parent = header.parent_id;
            }
            row.header.parent_id = self.parent_id;
        }
        if self.seq != original.seq {
            row.header.position = self.seq;
        }
        if self.name != original.name {
            row.name = text(&self.name)?;
        }
        let handle = tx.update(&handle, &Row::Extended(ExtendedRow::MyTag(row)))?;
        let model = Self::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Delete a tag/category, its descendants and all their content associations.
    ///
    /// Descendants are followed by parent ID, even in malformed or cyclic trees;
    /// each row is removed once. Contents and surviving sequence values are untouched.
    /// Returns the total deleted row count. Missing IDs return zero without cleaning
    /// dangling associations; zero is rejected. One transaction covers both extended
    /// tables. Errors roll back; saving remains explicit.
    pub fn delete<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>, id: &u32) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        validate_id(Self::TABLE, *id)?;
        let mut tx = conn.transaction();
        let index = tx.index_rows(Self::TABLE, Self::read_id)?.clone();
        if !index.contains_key(id) {
            return Ok(0);
        }
        let headers = tag_headers(&mut tx)?;
        let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
        for header in headers.values() {
            children
                .entry(header.parent_id)
                .or_default()
                .push(header.id);
        }
        let mut removed = HashSet::new();
        let mut pending = vec![*id];
        while let Some(id) = pending.pop() {
            if removed.insert(id) {
                if let Some(ids) = children.get(&id) {
                    pending.extend(ids);
                }
            }
        }
        let mut count = remove_associations(&mut tx, |row| removed.contains(&row.tag_id))?;
        for id in removed {
            let location = index.get(&id).ok_or(Error::ModelNotFound {
                table: Self::TABLE,
                id,
            })?;
            let handle = tx.locate(Self::TABLE, location.page, location.slot)?;
            tx.delete(&handle)?;
            count = count.checked_add(1).ok_or(Error::MutationOverflow)?;
        }
        tx.commit()?;
        Ok(count)
    }

    /// Original row window, including reserved fields and the second string frame.
    /// Its location is not a stable mutation handle after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }

    fn read_id(record: &RowRecord) -> Result<u32> {
        if record.table() != Self::TABLE {
            return Err(unsupported(record));
        }
        let header: MyTagHeader = read_fields(record)?;
        if !matches!(
            header.subtype,
            MyTagHeader::NEAR_SUBTYPE | MyTagHeader::FAR_SUBTYPE
        ) {
            return Err(unsupported(record));
        }
        Ok(header.id)
    }
}

fn invalid_parent(reason: &'static str) -> Error {
    Error::InvalidModelField {
        table: MyTag::TABLE,
        field: "parent_id",
        reason,
    }
}

fn tag_headers<R: Read + Seek>(tx: &mut Transaction<'_, R>) -> Result<HashMap<u32, MyTagHeader>> {
    let mut headers = HashMap::new();
    for record in tx.iter_raw(MyTag::TABLE)? {
        let record = record?;
        let id = MyTag::read_id(&record)?;
        if headers.insert(id, read_fields(&record)?).is_some() {
            return Err(Error::DuplicateId {
                table: MyTag::TABLE,
                id,
            });
        }
    }
    Ok(headers)
}

fn remove_associations<R: Read + Seek>(
    tx: &mut Transaction<'_, R>,
    matches: impl Fn(&TrackMyTagRow) -> bool,
) -> Result<usize> {
    let mut locations = Vec::new();
    for record in tx.iter_raw(MyTagContent::TABLE)? {
        let record = record?;
        let Row::Extended(ExtendedRow::TrackMyTag(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        if matches(&row) {
            locations.push((record.page(), record.slot().slot));
        }
    }
    let count = locations.len();
    for (page, slot) in locations {
        let handle = tx.locate(MyTagContent::TABLE, page, slot)?;
        tx.delete(&handle)?;
    }
    Ok(count)
}

impl TryFrom<RowRecord> for MyTag {
    type Error = Error;

    /// Decode a tag/category while retaining its original bytes and framing.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Extended(ExtendedRow::MyTag(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            id: row.header.id,
            parent_id: row.header.parent_id,
            seq: row.header.position,
            attribute: row.header.attribute,
            name: row
                .name
                .text()
                .map_err(|source| text_error(&record, "name", source))?,
            raw: record,
        })
    }
}

/// Input for a fresh My Tag/category, with explicit ID and zero-based position.
///
/// The default reserved values are zero, with an empty second string frame, as in
/// the established writer profile. Near-offset encoding is used; the engine rejects
/// strings that cannot fit that layout. Parent existence and category status are
/// not validated. Insertion commits in memory only; saving remains explicit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMyTag {
    /// Unused, nonzero export tag/category ID.
    pub id: u32,
    /// Parent category ID; zero denotes a root.
    pub parent_id: u32,
    /// Explicit zero-based position; other rows are not renumbered.
    pub seq: u32,
    /// Display name.
    pub name: String,
    /// Raw attribute; constructors use zero for tags and one for categories.
    pub attribute: u8,
    /// Reserved word at 0x04, defaulting to zero.
    pub unknown_04: u32,
    /// Reserved word at 0x08, defaulting to zero.
    pub unknown_08: u32,
    /// Reserved word at 0x18, defaulting to zero.
    pub unknown_18: u16,
    /// Reserved byte at 0x1a, defaulting to zero.
    pub unknown_1a: u8,
}

impl NewMyTag {
    /// Prepare a tag with an explicit parent and position.
    pub fn new(id: u32, parent_id: u32, seq: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            parent_id,
            seq,
            name: name.into(),
            attribute: 0,
            unknown_04: 0,
            unknown_08: 0,
            unknown_18: 0,
            unknown_1a: 0,
        }
    }

    /// Prepare a root category with an explicit position.
    pub fn category(id: u32, seq: u32, name: impl Into<String>) -> Self {
        Self {
            attribute: 1,
            ..Self::new(id, 0, seq, name)
        }
    }

    /// Insert atomically, rejecting duplicate IDs. Errors leave the connection unchanged.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<MyTag> {
        conn.check_kind(MyTag::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        if tx
            .index_rows(MyTag::TABLE, MyTag::read_id)?
            .contains_key(&self.id)
        {
            return Err(Error::DuplicateId {
                table: MyTag::TABLE,
                id: self.id,
            });
        }
        let handle = tx.insert(MyTag::TABLE, &Row::Extended(ExtendedRow::MyTag(row)))?;
        let model = MyTag::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; the engine assigns offsets and slots.
    /// Direct binary insertion does not check model-ID uniqueness.
    pub fn to_binary(&self) -> Result<MyTagRow> {
        validate_id(MyTag::TABLE, self.id)?;
        Ok(MyTagRow {
            header: MyTagHeader {
                subtype: MyTagHeader::NEAR_SUBTYPE,
                index_shift: 0,
                unknown_04: self.unknown_04,
                unknown_08: self.unknown_08,
                parent_id: self.parent_id,
                position: self.seq,
                id: self.id,
                unknown_18: self.unknown_18,
                unknown_1a: self.unknown_1a,
                attribute: self.attribute,
            },
            strings: StringOffsets::Near {
                marker: 3,
                offsets: [0; 2],
            },
            name: text(&self.name)?,
            unknown_string: text("")?,
        })
    }
}

/// A detached association between a standard database track and an extended tag.
///
/// There is no assumed unique row ID. Queries preserve repeated associations and
/// never load related rows. Editing fields changes only this snapshot.
#[derive(Clone, PartialEq, Eq)]
pub struct MyTagContent {
    /// Export track ID in the separate standard database.
    pub content_id: u32,
    /// Tag ID in the extended database.
    pub my_tag_id: u32,
    raw: RowRecord,
}

impl std::fmt::Debug for MyTagContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyTagContent")
            .field("content_id", &self.content_id)
            .field("my_tag_id", &self.my_tag_id)
            .finish()
    }
}

impl MyTagContent {
    /// Extended database table containing track/tag associations.
    pub const TABLE: TableKind = TableKind::Extended(ExtPageType::TrackMyTags);

    /// Stream live associations in physical order; creating the iterator reads no pages.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read every live association, preserving repeated pairs.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Scan associations for an export track ID, retaining physical order and duplicates.
    /// The track in the separate standard database is not loaded or validated.
    pub fn by_content<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        content_id: &u32,
    ) -> Result<Vec<Self>> {
        let mut rows = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.content_id == *content_id {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    /// Scan associations for a tag ID, retaining physical order and duplicates.
    /// The tag row is not loaded or validated.
    pub fn by_my_tag<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        my_tag_id: &u32,
    ) -> Result<Vec<Self>> {
        let mut rows = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.my_tag_id == *my_tag_id {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    /// Delete this one original association, ignoring edits to public fields.
    ///
    /// Duplicate pairs are retained. Changed binary values or heap offsets return
    /// [`Error::StaleModelRow`]; missing locations return an engine error. Re-fetch
    /// after structural edits: an identical replacement at the same location cannot
    /// be distinguished. Returns one on success; saving remains explicit.
    pub fn delete<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        let mut tx = conn.transaction();
        let handle = tx.locate(Self::TABLE, self.raw.page(), self.raw.slot().slot)?;
        let current = tx.read(&handle)?;
        if current.slot().heap_offset != self.raw.slot().heap_offset
            || current.decode()? != self.raw.decode()?
        {
            return Err(Error::StaleModelRow {
                table: Self::TABLE,
                page: self.raw.page(),
                slot: self.raw.slot().slot,
            });
        }
        tx.delete(&handle)?;
        tx.commit()?;
        Ok(1)
    }

    /// Remove every matching content/tag pair, including duplicates.
    ///
    /// IDs must be nonzero; referenced rows need not exist, allowing dangling
    /// associations to be cleaned up. Returns the deleted count. Contents and tags
    /// are retained. Errors roll back; changes remain in memory until saved.
    pub fn remove<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        content_id: &u32,
        my_tag_id: &u32,
    ) -> Result<usize> {
        conn.check_kind(Self::TABLE)?;
        validate_nonzero_id(Self::TABLE, "content_id", *content_id)?;
        validate_nonzero_id(Self::TABLE, "my_tag_id", *my_tag_id)?;
        let mut tx = conn.transaction();
        let count = remove_associations(&mut tx, |row| {
            row.track_id == *content_id && row.tag_id == *my_tag_id
        })?;
        tx.commit()?;
        Ok(count)
    }

    /// Original row window, including reserved fields. Its location is not a stable
    /// mutation handle after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }
}

impl TryFrom<RowRecord> for MyTagContent {
    type Error = Error;

    /// Decode an association while retaining the original row window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Extended(ExtendedRow::TrackMyTag(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            content_id: row.track_id,
            my_tag_id: row.tag_id,
            raw: record,
        })
    }
}

/// Input for a fresh track/tag association, without a synthetic row ID.
///
/// References must be nonzero but their existence is not checked. Duplicate pairs
/// are allowed; no uniqueness constraint is inferred from the binary layout.
/// Reserved fields default to zero with the observed empty-string marker 3.
/// Insertion commits in memory only; saving is explicit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMyTagContent {
    /// Nonzero export track ID in the standard database.
    pub content_id: u32,
    /// Nonzero tag ID in the extended database.
    pub my_tag_id: u32,
    /// Reserved word at 0x00, defaulting to zero.
    pub unknown_00: u32,
    /// Raw marker at 0x0c; defaults to the observed empty short-string frame, 3.
    pub reserved_string_marker: u8,
    /// Reserved trailing bytes, defaulting to zero.
    pub reserved_tail: [u8; 3],
}

impl NewMyTagContent {
    /// Prepare an association with explicit cross-database IDs.
    pub fn new(content_id: u32, my_tag_id: u32) -> Self {
        Self {
            content_id,
            my_tag_id,
            unknown_00: 0,
            reserved_string_marker: 3,
            reserved_tail: [0; 3],
        }
    }

    /// Insert one association atomically without deduplicating pairs or loading references.
    pub fn insert<R: Read + Seek>(self, conn: &mut DeviceSqlConnection<R>) -> Result<MyTagContent> {
        conn.check_kind(MyTagContent::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            MyTagContent::TABLE,
            &Row::Extended(ExtendedRow::TrackMyTag(row)),
        )?;
        let model = MyTagContent::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode the binary row for insertion inside an explicit transaction.
    pub fn to_binary(&self) -> Result<TrackMyTagRow> {
        validate_nonzero_id(MyTagContent::TABLE, "content_id", self.content_id)?;
        validate_nonzero_id(MyTagContent::TABLE, "my_tag_id", self.my_tag_id)?;
        Ok(TrackMyTagRow {
            unknown_00: self.unknown_00,
            track_id: self.content_id,
            tag_id: self.my_tag_id,
            reserved_string_marker: self.reserved_string_marker,
            reserved_tail: self.reserved_tail,
        })
    }
}
