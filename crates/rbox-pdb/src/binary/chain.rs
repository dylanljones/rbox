// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Header-only traversal and ownership validation of DeviceSQL table page chains.

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

use binrw::BinRead;

use super::core::{supported_page_size, FileHeader, PageHeader, PageIndex, TablePointer};
use crate::{Error, Result};

/// A table directory entry and its page headers in linked-list order.
///
/// Includes index and data pages, even when they contain no live rows.
/// Physical page order can differ from this order. No payloads are loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableChain {
    /// Original table descriptor, including its allocation candidate.
    pub table: TablePointer,
    /// Headers from `first_page` through `last_page`, inclusive.
    pub pages: Vec<PageHeader>,
}

/// Read and validate all table chains from a complete PDB stream.
///
/// `header` must be the file header from this stream. Offsets are absolute from
/// byte zero. Works identically for standard and extended files: table codes
/// remain raw, including unknown codes. Results retain table-directory order.
///
/// Reads only the 40- or 60-byte headers of linked pages, skipping their payloads
/// and all unlinked pages. Stops at each descriptor's inclusive `last_page`;
/// its successor, `empty_candidate`, and `next_unused_page` are not traversed.
/// Memory and read volume scale with the number of linked pages.
///
/// Rejects incomplete physical pages, invalid endpoints/successors, cycles,
/// shared pages, self-index mismatches, and table-code mismatches. It does not
/// validate index entries, index-specific references, row directories, or rows.
/// The stream position is not restored, including on error.
///
/// ```no_run
/// use std::fs::File;
/// use binrw::BinRead;
/// use rbox_pdb::binary::{read_table_chains, FileHeader};
///
/// # fn main() -> rbox_pdb::Result<()> {
/// let mut input = File::open("export.pdb")?;
/// let header = FileHeader::read_le(&mut input)?;
/// for chain in read_table_chains(&mut input, &header)? {
///     println!("table {}: {} linked pages", chain.table.table_type, chain.pages.len());
/// }
/// # Ok(())
/// # }
/// ```
pub fn read_table_chains<R: Read + Seek>(
    input: &mut R,
    header: &FileHeader,
) -> Result<Vec<TableChain>> {
    let page_size = header.prefix.page_size;
    let file_size = input.seek(SeekFrom::End(0))?;
    if !supported_page_size(page_size)
        || file_size < u64::from(page_size)
        || file_size % u64::from(page_size) != 0
    {
        return Err(Error::InvalidFileLayout {
            file_size,
            page_size,
        });
    }
    let page_count = file_size / u64::from(page_size);
    let mut owners = HashMap::new();
    let mut chains = Vec::new();
    for (table_index, table) in header.tables.iter().enumerate() {
        validate_reference(table.first_page, page_count, table_index)?;
        validate_reference(table.last_page, page_count, table_index)?;
        let mut pages = Vec::new();
        let mut current = table.first_page;
        loop {
            validate_reference(current, page_count, table_index)?;
            if let Some(&owner) = owners.get(&current) {
                return Err(if owner == table_index {
                    Error::PageCycle {
                        table_index,
                        page: current,
                    }
                } else {
                    Error::SharedPage {
                        page: current,
                        first_table: owner,
                        second_table: table_index,
                    }
                });
            }
            let page = read_chain_page(input, page_size, current, table_index, table.table_type)?;
            owners.insert(current, table_index);
            let next = page.common.next_page;
            pages.push(page);
            if current == table.last_page {
                break;
            }
            current = next;
        }
        chains.push(TableChain {
            table: *table,
            pages,
        });
    }
    Ok(chains)
}

pub fn validate_reference(page: PageIndex, page_count: u64, table_index: usize) -> Result<()> {
    if page.0 == 0 || page.is_none() || u64::from(page.0) >= page_count {
        return Err(Error::InvalidPageReference { table_index, page });
    }
    Ok(())
}

/// Shared checks for eager header traversal and lazy database iteration.
pub fn read_chain_page<R: Read + Seek>(
    input: &mut R,
    page_size: u32,
    current: PageIndex,
    table_index: usize,
    table_code: u32,
) -> Result<PageHeader> {
    input.seek(SeekFrom::Start(u64::from(current.0) * u64::from(page_size)))?;
    let page = PageHeader::read_le(input).map_err(|source| Error::PageRead {
        table_index,
        page: current,
        source,
    })?;
    if page.common.page_index != current {
        return Err(Error::PageIndexMismatch {
            table_index,
            expected: current,
            actual: page.common.page_index,
        });
    }
    if page.common.table_type != table_code {
        return Err(Error::PageTableMismatch {
            table_index,
            page: current,
            expected: table_code,
            actual: page.common.table_type,
        });
    }
    Ok(page)
}
