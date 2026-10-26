// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! RecommendedLike snapshots and explicit DeviceSQL queries, insertions and deletions.

use std::io::{Read, Seek};

use super::{unsupported, validate_nonzero_id};
use crate::binary::{ExtPageType, ExtendedRow, RecommendLikeRow, Row, RowRecord, TableKind};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached RecommendedLike row in the extended database.
///
/// Public-field edits affect only this snapshot. Unknown data and original framing
/// remain available through [`Self::raw_record`]. Related rows are never loaded implicitly.
/// Existing repeated rows are retained on reads; model insertion rejects duplicate
/// ordered content pairs. There is no unique scalar row ID.
#[derive(Clone, PartialEq, Eq)]
pub struct RecommendedLike {
    /// Raw eight-byte value at 0x00; not interpreted as a timestamp.
    pub unknown_00: u64,
    /// First standard export track ID; no source/target direction is inferred.
    pub content_id_1: u32,
    /// Second standard export track ID.
    pub content_id_2: u32,
    /// Raw word at 0x10.
    pub unknown_10: u32,
    /// Raw word at 0x14; flag semantics are unknown.
    pub unknown_14: u32,
    raw: RowRecord,
}

impl std::fmt::Debug for RecommendedLike {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecommendedLike")
            .field("unknown_00", &self.unknown_00)
            .field("content_id_1", &self.content_id_1)
            .field("content_id_2", &self.content_id_2)
            .field("unknown_10", &self.unknown_10)
            .field("unknown_14", &self.unknown_14)
            .finish()
    }
}

impl RecommendedLike {
    /// Database namespace and table containing these rows.
    pub const TABLE: TableKind = TableKind::Extended(ExtPageType::RecommendLike);

    /// Stream live rows in physical order; creating the iterator reads no pages.
    pub fn iter<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<impl Iterator<Item = Result<Self>> + '_> {
        Ok(conn
            .iter_raw(Self::TABLE)?
            .map(|record| Self::try_from(record?)))
    }

    /// Read all live rows in physical order without sorting or deduplication.
    pub fn all<R: Read + Seek>(conn: &mut DeviceSqlConnection<R>) -> Result<Vec<Self>> {
        Self::iter(conn)?.collect()
    }

    /// Scan rows containing either track ID, retaining physical order and repetitions.
    /// Referenced rows are not loaded or validated.
    pub fn by_content_id<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        content_id: &u32,
    ) -> Result<Vec<Self>> {
        let mut rows = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.content_id_1 == *content_id || row.content_id_2 == *content_id {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    /// Delete this original occurrence, ignoring public-field edits.
    ///
    /// Other occurrences and referenced contents are retained. Changed binary
    /// values or heap offsets return [`Error::StaleModelRow`]; missing locations
    /// return an engine error. Re-fetch after structural edits: an identical
    /// replacement at the same location cannot be distinguished. Returns one on
    /// success. Errors roll back; saving remains explicit.
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

    /// Original row window, including reserved fields. Locations are not stable
    /// mutation handles after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }
}

impl TryFrom<RowRecord> for RecommendedLike {
    type Error = Error;

    /// Decode the binary fields while retaining the original row window.
    fn try_from(record: RowRecord) -> Result<Self> {
        if record.table() != Self::TABLE {
            return Err(unsupported(&record));
        }
        let Row::Extended(ExtendedRow::RecommendLike(row)) = record.decode()? else {
            return Err(unsupported(&record));
        };
        Ok(Self {
            unknown_00: row.unknown_00,
            content_id_1: row.track_id_1,
            content_id_2: row.track_id_2,
            unknown_10: row.unknown_10,
            unknown_14: row.unknown_14,
            raw: record,
        })
    }
}

/// Insertion input for a fresh RecommendedLike row.
///
/// Creation semantics are not established. All unknown words must be supplied
/// explicitly; this input provides binary insertion, not a recommendation generator
/// or a guarantee of player behavior. Track existence and pair direction are not
/// inferred. Model insertion requires a unique ordered content pair.
/// Saving is explicit; insertion commits only to the in-memory connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRecommendedLike {
    /// Raw eight-byte value at 0x00; not interpreted as a timestamp.
    pub unknown_00: u64,
    /// First standard export track ID; no source/target direction is inferred.
    pub content_id_1: u32,
    /// Second standard export track ID.
    pub content_id_2: u32,
    /// Raw word at 0x10.
    pub unknown_10: u32,
    /// Raw word at 0x14; flag semantics are unknown.
    pub unknown_14: u32,
}

impl NewRecommendedLike {
    /// Prepare an insertion with explicit inputs.
    pub fn new(
        content_id_1: u32,
        content_id_2: u32,
        unknown_00: u64,
        unknown_10: u32,
        unknown_14: u32,
    ) -> Self {
        Self {
            content_id_1,
            content_id_2,
            unknown_00,
            unknown_10,
            unknown_14,
        }
    }

    /// Insert atomically, returning the engine-created row; failures leave the connection unchanged.
    /// Rejects an existing `(content_id_1, content_id_2)` pair regardless of its
    /// unknown values. Reversed pairs remain distinct; symmetry is not assumed.
    pub fn insert<R: Read + Seek>(
        self,
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<RecommendedLike> {
        conn.check_kind(RecommendedLike::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        for record in tx.iter_raw(RecommendedLike::TABLE)? {
            let record = record?;
            let Row::Extended(ExtendedRow::RecommendLike(existing)) = record.decode()? else {
                return Err(unsupported(&record));
            };
            if existing.track_id_1 == self.content_id_1 && existing.track_id_2 == self.content_id_2
            {
                return Err(Error::UniqueConstraintViolation {
                    table: RecommendedLike::TABLE,
                    fields: &["content_id_1", "content_id_2"],
                });
            }
        }
        let handle = tx.insert(
            RecommendedLike::TABLE,
            &Row::Extended(ExtendedRow::RecommendLike(row)),
        )?;
        let model = RecommendedLike::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; placement is assigned by the engine.
    /// Direct binary insertion bypasses the model's pair-uniqueness check.
    pub fn to_binary(&self) -> Result<RecommendLikeRow> {
        validate_nonzero_id(RecommendedLike::TABLE, "content_id_1", self.content_id_1)?;
        validate_nonzero_id(RecommendedLike::TABLE, "content_id_2", self.content_id_2)?;
        Ok(RecommendLikeRow {
            unknown_00: self.unknown_00,
            track_id_1: self.content_id_1,
            track_id_2: self.content_id_2,
            unknown_10: self.unknown_10,
            unknown_14: self.unknown_14,
        })
    }
}
