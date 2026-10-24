// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Binary creation profiles, opt-in initialization defaults, and empty index pages.
//!
//! Constants describe one supported creation profile, not parser invariants.

use crate::binary::{
    BrowseCategoryRow, BrowseColumnRow, BrowseSortRow, ColorRow, CommonPageHeader,
    DatabasePropertyHeader, DatabasePropertyRow, DeviceSqlString, ExportPropertyRow, ExtPageType,
    ExtendedRow, IndexEntry, IndexPageHeader, PackedRowCounts, PageFlags, PageHeader,
    PageHeaderKind, PageIndex, PageType, Row, StandardRow, StringEncoding, StringOffsets,
    TableKind, TablePointer, PAGE_SIZE,
};
use crate::Result;

// This is one explicit creation profile, not a parser default or a claim that
// other page sizes/unknown-header values are invalid when reading existing files.
const INDEX_TAIL_SIZE: usize = 20;

pub(super) const INDEX_CAPACITY: u16 = ((PAGE_SIZE
    - CommonPageHeader::BINARY_SIZE as usize
    - IndexPageHeader::BINARY_SIZE as usize
    - INDEX_TAIL_SIZE)
    / IndexEntry::BINARY_SIZE as usize) as u16;

pub(super) const DATA_PAGE_FLAGS: PageFlags = PageFlags(0x24);
const INDEX_PAGE_FLAGS: PageFlags = PageFlags(DATA_PAGE_FLAGS.0 | PageFlags::INDEX);

/// Explicit binary inputs for a newly initialized 4096-byte-page database.
///
/// Defaults produce the same bare container as [`super::DeviceSqlConnection::create`].
/// No configuration rows, dates or library IDs are inferred. Table entries are
/// initialized in the supplied order, which determines candidate-page allocation.
/// The directory itself remains in ascending table-code order.
/// Use [`Self::standard`] or [`Self::extended`] for populated initialization
/// defaults; neither changes how existing database rows are read or updated.
#[derive(Debug, Clone)]
pub struct CreateOptions {
    /// Uninterpreted file-header word at 0x10; rbxport uses 1, native fixtures use 5.
    pub unknown_10: u32,
    /// Committed file sequence, strictly greater than every initial page sequence.
    pub sequence: u32,
    /// Initial binary rows and data-page sequences; omitted tables remain empty.
    pub tables: Vec<InitialTable>,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self {
            unknown_10: 1,
            sequence: 2,
            tables: Vec::new(),
        }
    }
}

impl CreateOptions {
    /// Initialize a standard database with colors, browse settings and export properties.
    ///
    /// Includes eight colors, 27 browse columns, 22 category settings, 17 sort
    /// settings and one property row with zero tracks. Content tables stay empty.
    /// Values follow the Baken creation policy documented in this module's source;
    /// unknown fields remain configurable binary values, not parser invariants.
    ///
    /// Supply the export date (conventionally `YYYY-MM-DD`) and device name.
    /// The date is stored as supplied ASCII text; no calendar validation or clock
    /// lookup is performed. Names use short/long ASCII or UTF-16 as appropriate.
    /// Invalid strings return the binary string codec's typed errors.
    ///
    /// The header word is 1; index/data sequences start at 1 and the committed
    /// sequence is 2. The returned public fields and initial rows can be customized
    /// before calling [`super::DeviceSqlConnection::create_with_options`].
    /// Track count is an initial value, not automatically maintained by CRUD.
    ///
    /// ```
    /// use rbox_pdb::devicesql::{CreateOptions, DeviceSqlConnection};
    /// use rbox_pdb::binary::DatabaseKind;
    ///
    /// # fn main() -> rbox_pdb::Result<()> {
    /// let options = CreateOptions::standard("2026-10-24", "My USB")?;
    /// let database = DeviceSqlConnection::create_with_options(DatabaseKind::Standard, &options)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn standard(date: &str, name: &str) -> Result<Self> {
        let text = |s: &str| DeviceSqlString::encode(s, StringEncoding::ShortAscii);
        let property = StandardRow::ExportProperty(ExportPropertyRow {
            subtype: ExportPropertyRow::SUBTYPE,
            index_shift: 0,
            track_count: 0,
            unknown_08: 0,
            background_color: 0,
            unknown_0a: 0,
            date: text(date)?,
            separator: 0x1e19,
            version: text("1000")?,
            name_padding: Vec::new(),
            name: DeviceSqlString::encode(
                name,
                if !name.is_ascii() {
                    StringEncoding::Utf16Le
                } else if name.len() <= 126 {
                    StringEncoding::ShortAscii
                } else {
                    StringEncoding::LongAscii
                },
            )?,
        });
        let colors = [
            "Pink", "Red", "Orange", "Yellow", "Green", "Aqua", "Blue", "Purple",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            Ok(StandardRow::Color(ColorRow {
                unknown_00: 0,
                unknown_04: (i + 1) as u8,
                id: (i + 1) as u16,
                unknown_07: 0,
                name: text(name)?,
            }))
        })
        .collect::<Result<Vec<_>>>()?;
        let columns = [
            (0x80, "GENRE"),
            (0x81, "ARTIST"),
            (0x82, "ALBUM"),
            (0x83, "TRACK"),
            (0x85, "BPM"),
            (0x86, "RATING"),
            (0x87, "YEAR"),
            (0x88, "REMIXER"),
            (0x89, "LABEL"),
            (0x8a, "ORIGINAL ARTIST"),
            (0x8b, "KEY"),
            (0x8d, "CUE"),
            (0x8e, "COLOR"),
            (0x92, "TIME"),
            (0x93, "BITRATE"),
            (0x94, "FILE NAME"),
            (0x84, "PLAYLIST"),
            (0x98, "HOT CUE BANK"),
            (0x95, "HISTORY"),
            (0x91, "SEARCH"),
            (0x96, "COMMENTS"),
            (0x8c, "DATE ADDED"),
            (0x97, "DJ PLAY COUNT"),
            (0x90, "FOLDER"),
            (0xa1, "DEFAULT"),
            (0xa2, "ALPHABET"),
            (0xaa, "MATCHING"),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (code, name))| {
            Ok(StandardRow::BrowseColumn(BrowseColumnRow {
                id: (i + 1) as u16,
                code,
                name: DeviceSqlString::encode(
                    &format!("\u{fffa}{name}\u{fffb}"),
                    StringEncoding::Utf16Le,
                )?,
            }))
        })
        .collect::<Result<Vec<_>>>()?;
        // (column ID, selector, raw attribute, raw state, position), in physical order.
        let categories = [
            (1, 1, 99, 1, 0),
            (5, 6, 5, 1, 0),
            (6, 7, 99, 1, 0),
            (7, 8, 99, 1, 0),
            (8, 9, 99, 1, 0),
            (9, 10, 99, 1, 0),
            (10, 11, 99, 1, 0),
            (13, 15, 99, 1, 0),
            (14, 19, 4, 1, 0),
            (15, 20, 6, 1, 0),
            (16, 21, 99, 1, 0),
            (18, 23, 99, 1, 0),
            (2, 2, 2, 0, 1),
            (3, 3, 3, 0, 2),
            (4, 4, 1, 0, 3),
            (11, 12, 99, 0, 4),
            (17, 5, 99, 0, 5),
            (19, 22, 99, 0, 6),
            (20, 18, 99, 0, 7),
            (27, 26, 99, 2, 8),
            (24, 17, 99, 0, 9),
            (22, 27, 99, 0, 10),
        ]
        .into_iter()
        .map(|(column_id, selector, unknown_04, state, position)| {
            StandardRow::BrowseCategory(BrowseCategoryRow {
                column_id,
                selector,
                unknown_04,
                state,
                position,
            })
        })
        .collect();
        // Retain the complete tail word; its semantics are not inferred here.
        let sorts = [
            (1, 6, 1),
            (21, 7, 1),
            (14, 8, 1),
            (8, 9, 1),
            (9, 10, 1),
            (10, 11, 1),
            (15, 13, 1),
            (13, 15, 1),
            (23, 16, 1),
            (22, 17, 1),
            (25, 0, 0x100),
            (26, 1, 0x200),
            (2, 2, 0x300),
            (3, 3, 0x400),
            (5, 4, 0x500),
            (6, 5, 0x600),
            (11, 12, 0x700),
        ]
        .into_iter()
        .map(|(column_id, selector, raw_tail)| {
            StandardRow::BrowseSort(BrowseSortRow {
                column_id,
                selector,
                raw_tail,
            })
        })
        .collect();
        let tables = [
            (PageType::ExportProperties, vec![property]),
            (PageType::ColorLabels, colors),
            (PageType::BrowseColumns, columns),
            (PageType::BrowseCategorySettings, categories),
            (PageType::BrowseSortSettings, sorts),
        ]
        .into_iter()
        .map(|(table, rows)| InitialTable {
            table: TableKind::Standard(table),
            first_sequence: 1,
            rows: rows.into_iter().map(Row::Standard).collect(),
        })
        .collect();
        Ok(Self {
            tables,
            ..Self::default()
        })
    }

    /// Initialize an extended database with its My Tag database-property row.
    ///
    /// All content tables remain empty. The property uses subtype 0x0700, five
    /// zero reserved words and five empty strings.
    ///
    /// The identity is supplied verbatim, including zero; no source library ID,
    /// random identity, or DBID + 1 conversion is inferred. A companion OneLibrary
    /// database, if produced by the caller, should use the same My Tag identity.
    /// The header word is 1 and the committed sequence is 2, as in
    /// [`Self::standard`]. This does not create any files or modify existing rows.
    ///
    /// ```
    /// use rbox_pdb::devicesql::{CreateOptions, DeviceSqlConnection};
    /// use rbox_pdb::binary::DatabaseKind;
    ///
    /// # fn main() -> rbox_pdb::Result<()> {
    /// let options = CreateOptions::extended(1234)?;
    /// let database = DeviceSqlConnection::create_with_options(DatabaseKind::Extended, &options)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn extended(my_tag_master_db_id: u32) -> Result<Self> {
        let empty = DeviceSqlString::encode("", StringEncoding::ShortAscii)?;
        let property = DatabasePropertyRow {
            header: DatabasePropertyHeader {
                subtype: DatabasePropertyHeader::SUBTYPE,
                index_shift: 0,
                unknown_words: [0; 5],
                my_tag_master_db_id,
            },
            strings: StringOffsets::Near {
                marker: 3,
                offsets: [0; 5],
            },
            unknown_strings: std::array::from_fn(|_| empty.clone()),
        };
        Ok(Self {
            tables: vec![InitialTable {
                table: TableKind::Extended(ExtPageType::DatabasePropertyOptions),
                first_sequence: 1,
                rows: vec![Row::Extended(ExtendedRow::DatabaseProperty(property))],
            }],
            ..Self::default()
        })
    }
}

/// Initial rows of one table, using the same supported codecs as transactions.
#[derive(Debug, Clone)]
pub struct InitialTable {
    /// Table in the selected database namespace; each table may occur only once.
    pub table: TableKind,
    /// Sequence of the first data page. Further data pages increment this value
    /// in chain order. Index pages always start at sequence 1.
    pub first_sequence: u32,
    /// Rows in insertion order, with page-local offsets/slots assigned by the engine.
    pub rows: Vec<Row>,
}

pub(super) fn empty_index(table: &TablePointer) -> Result<PageHeader> {
    Ok(PageHeader {
        common: CommonPageHeader {
            signature: 0,
            page_index: table.first_page,
            table_type: table.table_type,
            next_page: table.empty_candidate,
            sequence: 1,
            unknown_14: 0,
            counts: PackedRowCounts::new(0, 0)?,
            flags: INDEX_PAGE_FLAGS,
            free_size: 0,
            used_size: 0,
        },
        kind: PageHeaderKind::Index(IndexPageHeader {
            unknown_20: 0x1fff,
            unknown_22: 0x1fff,
            entry_capacity: INDEX_CAPACITY,
            next_entry: 0,
            page_index: table.first_page,
            data_page: PageIndex::NONE,
            unknown_30: PageIndex::NONE.0,
            unknown_34: 0,
            entry_count: 0,
            first_empty: IndexPageHeader::NO_EMPTY_SLOT,
        }),
    })
}
