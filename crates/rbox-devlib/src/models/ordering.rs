// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! In-memory plans for scoped DeviceSQL ordering; callers apply plans transactionally.

use std::collections::{BTreeSet, HashSet};

use crate::binary::TableKind;
use crate::{Error, Result};

pub(super) struct Position {
    pub parent: u32,
    pub seq: u32,
}

pub(super) struct Change {
    pub index: usize,
    pub parent: u32,
    pub seq: u32,
}

pub(super) fn invalid(table: TableKind, field: &'static str, reason: &'static str) -> Error {
    Error::InvalidModelField {
        table,
        field,
        reason,
    }
}

fn offset(table: TableKind, seq: Option<u32>, start: u32, len: usize) -> Result<usize> {
    let len = u32::try_from(len).map_err(|_| Error::MutationOverflow)?;
    let end = start.checked_add(len).ok_or(Error::MutationOverflow)?;
    let seq = seq.unwrap_or(end);
    if seq < start || seq > end {
        return Err(invalid(table, "seq", "position is outside the collection"));
    }
    Ok((seq - start) as usize)
}

pub(super) fn ordered(rows: &[Position], parent: u32) -> Vec<usize> {
    let mut indices: Vec<_> = (0..rows.len())
        .filter(|&i| rows[i].parent == parent)
        .collect();
    // Stable sorting retains physical order for imported ties.
    indices.sort_by_key(|&i| rows[i].seq);
    indices
}

fn changes(rows: &[Position], indices: &[usize], parent: u32, start: u32) -> Result<Vec<Change>> {
    let mut result = Vec::new();
    for (i, &index) in indices.iter().enumerate() {
        let seq = start
            .checked_add(u32::try_from(i).map_err(|_| Error::MutationOverflow)?)
            .ok_or(Error::MutationOverflow)?;
        if rows[index].parent != parent || rows[index].seq != seq {
            result.push(Change { index, parent, seq });
        }
    }
    Ok(result)
}

pub(super) fn renumber(rows: &[Position], parent: u32, start: u32) -> Result<Vec<Change>> {
    changes(rows, &ordered(rows, parent), parent, start)
}

pub(super) fn insert(
    table: TableKind,
    rows: &[Position],
    parent: u32,
    seq: Option<u32>,
    start: u32,
) -> Result<(u32, Vec<Change>)> {
    let mut extended: Vec<_> = rows
        .iter()
        .map(|r| Position {
            parent: r.parent,
            seq: r.seq,
        })
        .collect();
    let mut indices = ordered(rows, parent);
    let index = offset(table, seq, start, indices.len())?;
    let position = start
        .checked_add(u32::try_from(index).map_err(|_| Error::MutationOverflow)?)
        .ok_or(Error::MutationOverflow)?;
    indices.insert(index, extended.len());
    extended.push(Position {
        parent,
        seq: position,
    });
    Ok((position, changes(&extended, &indices, parent, start)?))
}

pub(super) fn move_rows(
    table: TableKind,
    rows: &[Position],
    selected: &[usize],
    parent: u32,
    seq: Option<u32>,
    start: u32,
) -> Result<Vec<Change>> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    let selected_set: HashSet<_> = selected.iter().copied().collect();
    if selected_set.len() != selected.len() {
        return Err(invalid(table, "id", "duplicate move identity"));
    }
    let mut scopes = BTreeSet::from([parent]);
    scopes.extend(selected.iter().map(|&i| rows[i].parent));
    let mut result = Vec::new();
    for scope in scopes {
        let mut members = ordered(rows, scope);
        members.retain(|i| !selected_set.contains(i));
        if scope == parent {
            let index = offset(table, seq, start, members.len())?;
            members.splice(index..index, selected.iter().copied());
        }
        result.extend(changes(rows, &members, scope, start)?);
    }
    Ok(result)
}

pub(super) fn valid(seqs: &[u32], start: u32) -> bool {
    seqs.iter()
        .enumerate()
        .all(|(i, &seq)| u64::from(seq) == u64::from(start) + i as u64)
}
