// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! RecommendLike snapshots and explicit DeviceSQL queries and insertions.

use std::io::{Read, Seek};

use super::{unsupported, validate_nonzero_id};
use crate::binary::{ExtPageType, ExtendedRow, RecommendLikeRow, Row, RowRecord, TableKind};
use crate::devicesql::DeviceSqlConnection;
use crate::{Error, Result};

/// A detached RecommendLike row in the extended database.
///
/// Public-field edits affect only this snapshot. Unknown data and original framing
/// remain available through [`Self::raw_record`]. Related rows are never loaded implicitly.
/// No unique row ID or singleton constraint is assumed; repeated rows are retained.
#[derive(Clone, PartialEq, Eq)]
pub struct RecommendLike {
    /// Raw eight-byte value at 0x00; not interpreted as a timestamp.
    pub unknown_00: u64,
    /// First standard export track ID; no source/target direction is inferred.
    pub track_id_1: u32,
    /// Second standard export track ID.
    pub track_id_2: u32,
    /// Raw word at 0x10.
    pub unknown_10: u32,
    /// Raw word at 0x14; flag semantics are unknown.
    pub unknown_14: u32,
    raw: RowRecord,
}

impl std::fmt::Debug for RecommendLike {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecommendLike")
            .field("unknown_00", &self.unknown_00)
            .field("track_id_1", &self.track_id_1)
            .field("track_id_2", &self.track_id_2)
            .field("unknown_10", &self.unknown_10)
            .field("unknown_14", &self.unknown_14)
            .finish()
    }
}

impl RecommendLike {
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
    pub fn by_track<R: Read + Seek>(
        conn: &mut DeviceSqlConnection<R>,
        track_id: &u32,
    ) -> Result<Vec<Self>> {
        let mut rows = Vec::new();
        for row in Self::iter(conn)? {
            let row = row?;
            if row.track_id_1 == *track_id || row.track_id_2 == *track_id {
                rows.push(row);
            }
        }
        Ok(rows)
    }
    /// Original row window, including reserved fields. Locations are not stable
    /// mutation handles after other database edits.
    pub fn raw_record(&self) -> &RowRecord {
        &self.raw
    }
}

impl TryFrom<RowRecord> for RecommendLike {
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
            track_id_1: row.track_id_1,
            track_id_2: row.track_id_2,
            unknown_10: row.unknown_10,
            unknown_14: row.unknown_14,
            raw: record,
        })
    }
}

/// Insertion input for a fresh RecommendLike row.
///
/// Creation semantics are not established. All unknown words must be supplied
/// explicitly; this input provides binary insertion, not a recommendation generator
/// or a guarantee of player behavior. Track existence, pair direction and uniqueness
/// are not inferred.
/// Saving is explicit; insertion commits only to the in-memory connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRecommendLike {
    /// Raw eight-byte value at 0x00; not interpreted as a timestamp.
    pub unknown_00: u64,
    /// First standard export track ID; no source/target direction is inferred.
    pub track_id_1: u32,
    /// Second standard export track ID.
    pub track_id_2: u32,
    /// Raw word at 0x10.
    pub unknown_10: u32,
    /// Raw word at 0x14; flag semantics are unknown.
    pub unknown_14: u32,
}

impl NewRecommendLike {
    /// Prepare an insertion with explicit inputs.
    pub fn new(
        track_id_1: u32,
        track_id_2: u32,
        unknown_00: u64,
        unknown_10: u32,
        unknown_14: u32,
    ) -> Self {
        Self {
            track_id_1,
            track_id_2,
            unknown_00,
            unknown_10,
            unknown_14,
        }
    }

    /// Insert atomically, returning the engine-created row; failures leave the connection unchanged.
    /// Existing rows are not replaced or deduplicated.
    pub fn insert<R: Read + Seek>(
        self,
        conn: &mut DeviceSqlConnection<R>,
    ) -> Result<RecommendLike> {
        conn.check_kind(RecommendLike::TABLE)?;
        let row = self.to_binary()?;
        let mut tx = conn.transaction();
        let handle = tx.insert(
            RecommendLike::TABLE,
            &Row::Extended(ExtendedRow::RecommendLike(row)),
        )?;
        let model = RecommendLike::try_from(tx.read(&handle)?)?;
        tx.commit()?;
        Ok(model)
    }

    /// Encode fields for an explicit binary transaction; placement is assigned by the engine.
    pub fn to_binary(&self) -> Result<RecommendLikeRow> {
        validate_nonzero_id(RecommendLike::TABLE, "track_id_1", self.track_id_1)?;
        validate_nonzero_id(RecommendLike::TABLE, "track_id_2", self.track_id_2)?;
        Ok(RecommendLikeRow {
            unknown_00: self.unknown_00,
            track_id_1: self.track_id_1,
            track_id_2: self.track_id_2,
            unknown_10: self.unknown_10,
            unknown_14: self.unknown_14,
        })
    }
}
