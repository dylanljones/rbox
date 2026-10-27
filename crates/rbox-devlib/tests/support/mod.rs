// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Shared row construction and decoding for transaction and persistence scenarios.

use rbox_devlib::binary::{
    DatabaseKind, ExtPageType, ExtendedRow, PageType, PlaylistEntryRow, Row, RowRecord,
    StandardRow, TableKind, TrackMyTagRow,
};
use rbox_devlib::Result;

pub fn decode_rows(records: impl IntoIterator<Item = Result<RowRecord>>) -> Result<Vec<Row>> {
    records.into_iter().map(|record| record?.decode()).collect()
}

pub fn entry_table(kind: DatabaseKind) -> TableKind {
    match kind {
        DatabaseKind::Standard => TableKind::Standard(PageType::PlaylistEntries),
        DatabaseKind::Extended => TableKind::Extended(ExtPageType::TrackMyTags),
    }
}

pub fn entry(kind: DatabaseKind, id: u32) -> Row {
    match kind {
        DatabaseKind::Standard => Row::Standard(StandardRow::PlaylistEntry(PlaylistEntryRow {
            position: id,
            track_id: id,
            playlist_id: 1,
        })),
        DatabaseKind::Extended => Row::Extended(ExtendedRow::TrackMyTag(TrackMyTagRow {
            unknown_00: 0,
            track_id: id,
            tag_id: 1,
            reserved_string_marker: 3,
            reserved_tail: [0; 3],
        })),
    }
}

pub fn association(kind: DatabaseKind, id: u32) -> (TableKind, Row) {
    (entry_table(kind), entry(kind, id))
}
