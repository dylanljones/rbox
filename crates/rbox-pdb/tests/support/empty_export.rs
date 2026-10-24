// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Test-owned creation profiles for the approved empty USB fixture, with no fixture reads.
//! Browse settings and tag identities belong to this library, not engine defaults.

use rbox_pdb::binary::{
    DeviceSqlString, ExtPageType, ExtendedRow, MyTagHeader, MyTagRow, Row, StandardRow,
    StringEncoding, StringOffsets, TableKind,
};
use rbox_pdb::devicesql::{CreateOptions, InitialTable};
use rbox_pdb::Result;

/// Reproduce the fresh USB's standard settings, order, and historical sequences.
pub fn standard() -> Result<CreateOptions> {
    let mut options = CreateOptions::standard("2026-10-24", "")?;
    options.unknown_10 = 5;
    options.sequence = 7;
    for (seed, sequence) in options.tables.iter_mut().zip(1..) {
        seed.first_sequence = sequence;
        for row in &mut seed.rows {
            match row {
                Row::Standard(StandardRow::BrowseCategory(row)) if row.column_id == 22 => {
                    // Observed value; the meaning of state 5 remains unqualified.
                    row.state = 5;
                }
                Row::Standard(StandardRow::BrowseSort(row)) if row.column_id == 2 => {
                    row.raw_tail = 0x0302;
                }
                Row::Standard(StandardRow::BrowseSort(row)) if row.column_id == 21 => {
                    row.raw_tail = 0x0800;
                }
                _ => {}
            }
        }
        const CATEGORIES: [u16; 22] = [
            15, 16, 18, 8, 9, 10, 13, 14, 1, 5, 6, 7, 2, 3, 4, 11, 17, 19, 20, 27, 24, 22,
        ];
        const SORTS: [u16; 17] = [22, 14, 8, 9, 10, 15, 13, 23, 1, 25, 26, 2, 3, 5, 6, 11, 21];
        seed.rows.sort_by_key(|row| match row {
            Row::Standard(StandardRow::BrowseCategory(row)) => {
                CATEGORIES.iter().position(|&id| id == row.column_id)
            }
            Row::Standard(StandardRow::BrowseSort(row)) => {
                SORTS.iter().position(|&id| id == row.column_id)
            }
            _ => None,
        });
    }
    Ok(options)
}

/// Reproduce the library's tag definitions even though no tracks were exported.
pub fn extended() -> Result<CreateOptions> {
    let mut options = CreateOptions::extended(668_155_178)?;
    options.unknown_10 = 5;
    options.sequence = 4;
    // Explicit (id, parent id, position, name) inputs; no ID-generation rule inferred.
    let tags = [
        (1, 0, 0, "Genre"),
        (3_390_897_981, 1, 0, "clean"),
        (1_612_344_835, 1, 1, "halfstep"),
        (2_627_717_087, 1, 2, "roller"),
        (1_210_473_204, 1, 3, "liquid"),
        (1_188_290_843, 1, 4, "neuro"),
        (1_296_105_431, 1, 5, "oldschool"),
        (4_163_676_879, 1, 6, "jungle"),
        (2, 0, 1, "Components"),
        (3_465_307_957, 2, 0, "deep"),
        (2_188_641_323, 2, 1, "fast"),
        (1_086_746_924, 2, 2, "slow"),
        (1_516_819_250, 2, 3, "hard"),
        (3_625_116_075, 2, 4, "soft"),
        (766_214_018, 2, 5, "offbeat"),
        (2_901_589_990, 2, 6, "breaks"),
        (228_356_307, 2, 7, "steppy"),
        (3, 0, 2, "Situation"),
        (1_641_388_739, 3, 0, "vocals"),
        (1_350_832_133, 3, 1, "lead"),
        (4, 0, 3, "Untitled Column"),
        (3_267_870_107, 4, 0, "Main Floor"),
        (3_265_890_185, 4, 1, "Second Floor"),
        (4_246_176_701, 4, 2, "Lounge"),
        (1_432_388_175, 4, 3, "Mid Night"),
        (2_550_246_366, 4, 4, "Morning"),
        (4_216_162_052, 4, 5, "Build up"),
        (2_934_612_046, 4, 6, "Peak Time"),
        (1_398_671_310, 4, 7, "Build down"),
    ];
    let rows = tags
        .into_iter()
        .map(|(id, parent_id, position, name)| {
            Ok(Row::Extended(ExtendedRow::MyTag(MyTagRow {
                header: MyTagHeader {
                    subtype: MyTagHeader::NEAR_SUBTYPE,
                    index_shift: 0,
                    unknown_04: 0,
                    unknown_08: 0,
                    parent_id,
                    position,
                    id,
                    unknown_18: 0,
                    unknown_1a: 0,
                    attribute: u8::from(parent_id == 0),
                },
                strings: StringOffsets::Near {
                    marker: 3,
                    offsets: [0; 2],
                },
                name: DeviceSqlString::encode(name, StringEncoding::ShortAscii)?,
                unknown_string: DeviceSqlString::encode("", StringEncoding::ShortAscii)?,
            })))
        })
        .collect::<Result<Vec<_>>>()?;
    // Property first reserves candidate 19; tags then reserve candidate 20.
    options.tables.push(InitialTable {
        table: TableKind::Extended(ExtPageType::MyTags),
        first_sequence: 2,
        rows,
    });
    Ok(options)
}
