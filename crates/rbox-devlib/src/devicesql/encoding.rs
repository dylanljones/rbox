// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Binary row layouts, preserved raw string frames, and checked slot fixups.

use binrw::{BinRead, BinWrite};
use std::io::Cursor;

use crate::binary::{
    AlbumHeader, ArtistHeader, DatabasePropertyHeader, DeviceSqlString, ExportPropertyRow,
    MyTagHeader, RowPrefix, TrackHeader,
};
use crate::binary::{ExtPageType as X, PageType as P, TableKind};
use crate::binary::{ExtendedRow as E, Row, StandardRow as S, StringOffsets, TrackStrings};
use crate::{Error, Result};

pub(super) fn unsupported(table: TableKind, reason: &'static str) -> Error {
    Error::UnsupportedMutation { table, reason }
}

pub(super) fn table(row: &Row) -> Option<TableKind> {
    Some(match row {
        Row::Standard(row) => TableKind::Standard(match row {
            S::Track(_) => P::Tracks,
            S::Artist(_) => P::Artists,
            S::Album(_) => P::Albums,
            S::Key(_) => P::MusicalKeys,
            S::Color(_) => P::ColorLabels,
            S::PlaylistTree(_) => P::PlaylistTree,
            S::PlaylistEntry(_) => P::PlaylistEntries,
            S::HistoryEntry(_) => P::HistoryEntries,
            S::BrowseColumn(_) => P::BrowseColumns,
            S::BrowseCategory(_) => P::BrowseCategorySettings,
            S::BrowseSort(_) => P::BrowseSortSettings,
            S::ExportProperty(_) => P::ExportProperties,
            S::Genre(_) => P::Genres,
            S::Label(_) => P::Labels,
            S::HistoryPlaylist(_) => P::HistoryPlaylists,
            S::Artwork(_) => P::ArtworkReferences,
            S::Opaque => return None,
        }),
        Row::Extended(row) => TableKind::Extended(match row {
            E::MyTag(_) => X::MyTags,
            E::TrackMyTag(_) => X::TrackMyTags,
            E::RecommendLike(_) => X::RecommendLike,
            E::DatabaseProperty(_) => X::DatabasePropertyOptions,
            E::Opaque => return None,
        }),
    })
}

fn encode<T: for<'a> BinWrite<Args<'a> = ()>>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = Cursor::new(Vec::new());
    value.write_le(&mut bytes)?;
    Ok(bytes.into_inner())
}
/// Owned bytes and allocation requirements, before page-local slot assignment.
pub(super) struct RowLayout {
    pub(super) bytes: Vec<u8>,
    pub(super) allocation: usize,
    slot_header: bool,
}

impl RowLayout {
    pub(super) fn fits_slot(&self, slot: u16) -> bool {
        !self.slot_header || slot <= RowPrefix::MAX_SLOT
    }

    pub(super) fn at_slot(&self, slot: u16) -> Result<Vec<u8>> {
        if !self.fits_slot(slot) {
            return Err(Error::MutationOverflow);
        }
        let mut bytes = self.bytes.clone();
        if self.slot_header {
            let mut prefix = RowPrefix::read_le(&mut Cursor::new(&bytes))?;
            prefix.set_slot(slot)?;
            prefix.write_le(&mut Cursor::new(&mut bytes[..]))?;
        }
        Ok(bytes)
    }
}

fn align4(n: usize) -> Result<usize> {
    Ok(n.checked_add(3).ok_or(Error::MutationOverflow)? & !3)
}

fn track_frames(t: &TrackStrings) -> [&DeviceSqlString; 21] {
    [
        &t.isrc,
        &t.lyricist,
        &t.unknown_02,
        &t.unknown_03,
        &t.unknown_04,
        &t.message,
        &t.publish_track_info,
        &t.hot_cue_auto_load,
        &t.unknown_08,
        &t.unknown_09,
        &t.date_added,
        &t.release_date,
        &t.mix_name,
        &t.unknown_13,
        &t.analysis_path,
        &t.analysis_date,
        &t.comment,
        &t.title,
        &t.unknown_18,
        &t.filename,
        &t.file_path,
    ]
}

// Offset values are placement metadata. Preserve the marker, rebuild offsets,
// keep raw frames, and align long frames relative to the start of the row.
fn string_row<const N: usize>(
    table: TableKind,
    mut prefix: Vec<u8>,
    offsets: &StringOffsets<N>,
    frames: [&DeviceSqlString; N],
) -> Result<Vec<u8>> {
    let start = prefix.len();
    let mut placed = offsets.clone();
    prefix.resize(start + placed.binary_size(), 0);
    for (index, frame) in frames.into_iter().enumerate() {
        if frame.marker() & 1 == 0 {
            prefix.resize(align4(prefix.len())?, 0);
        }
        let offset = u16::try_from(prefix.len()).map_err(|_| Error::MutationOverflow)?;
        match &mut placed {
            StringOffsets::Far { offsets, .. } => offsets[index] = offset,
            StringOffsets::Near { offsets, .. } => {
                offsets[index] = u8::try_from(offset).map_err(|_| {
                    unsupported(
                        table,
                        "near string offsets overflow; this far layout is not qualified",
                    )
                })?
            }
        }
        prefix.extend_from_slice(frame.as_bytes());
    }
    placed.write_le(&mut Cursor::new(&mut prefix[start..]))?;
    Ok(prefix)
}

/// Build supported row families without inventing application IDs or unknown fields.
pub(super) fn layout(table: TableKind, row: &Row) -> Result<RowLayout> {
    if self::table(row) != Some(table) {
        return Err(unsupported(table, "row variant does not match table"));
    }
    let invalid = || unsupported(table, "unsupported subtype or string allocation profile");
    let (bytes, allocation) = match row {
        Row::Standard(S::Artist(r)) => {
            if r.header.subtype
                != if r.strings.is_far() {
                    ArtistHeader::FAR_SUBTYPE
                } else {
                    ArtistHeader::NEAR_SUBTYPE
                }
            {
                return Err(invalid());
            }
            let header = encode(&r.header)?;
            let prefix = header.len() + r.strings.binary_size();
            let b = string_row(table, header, &r.strings, [&r.name])?;
            let n = align4(prefix + align4(r.name.as_bytes().len())?)?
                .checked_add(4)
                .ok_or(Error::MutationOverflow)?;
            (b, n)
        }
        Row::Standard(S::Album(r)) => {
            if r.header.subtype
                != if r.strings.is_far() {
                    AlbumHeader::FAR_SUBTYPE
                } else {
                    AlbumHeader::NEAR_SUBTYPE
                }
            {
                return Err(invalid());
            }
            let header = encode(&r.header)?;
            let prefix = header.len() + r.strings.binary_size();
            let b = string_row(table, header, &r.strings, [&r.name])?;
            let n = align4(prefix + align4(r.name.as_bytes().len())?)?
                .checked_add(4)
                .ok_or(Error::MutationOverflow)?;
            (b, n)
        }
        Row::Standard(S::Track(r)) => {
            if r.header.subtype != TrackHeader::SUBTYPE || !r.strings.is_far() {
                return Err(invalid());
            }
            let frames = track_frames(&r.text);
            let header = encode(&r.header)?;
            let mut n = header.len() + r.strings.binary_size() + 4;
            let b = string_row(table, header, &r.strings, frames)?;
            for frame in frames {
                n = n
                    .checked_add(align4(frame.as_bytes().len())?)
                    .ok_or(Error::MutationOverflow)?;
            }
            (b, n)
        }
        Row::Extended(E::MyTag(r)) => {
            // The near layout and its trailing allocation are observed natively.
            // Far tags remain readable but need independent allocation evidence.
            if r.header.subtype != MyTagHeader::NEAR_SUBTYPE
                || r.strings.is_far()
                || r.unknown_string.as_bytes() != [3]
            {
                return Err(invalid());
            }
            let b = string_row(
                table,
                encode(&r.header)?,
                &r.strings,
                [&r.name, &r.unknown_string],
            )?;
            let n = align4(
                align4(MyTagHeader::BINARY_SIZE + r.strings.binary_size())?
                    + r.name.as_bytes().len()
                    + 8,
            )?;
            (b, n)
        }
        Row::Extended(E::DatabaseProperty(r)) => {
            if r.header.subtype != DatabasePropertyHeader::SUBTYPE
                || r.strings.is_far()
                || r.unknown_strings.iter().any(|s| s.as_bytes() != [3])
            {
                return Err(invalid());
            }
            let b = string_row(
                table,
                encode(&r.header)?,
                &r.strings,
                std::array::from_fn(|i| &r.unknown_strings[i]),
            )?;
            (b, 60)
        }
        Row::Standard(S::ExportProperty(r)) => {
            if r.subtype != ExportPropertyRow::SUBTYPE {
                return Err(invalid());
            }
            let mut r = r.clone();
            if r.name_padding.iter().any(|&b| b != 0) {
                return Err(invalid());
            }
            r.name_padding.clear();
            let before_name = encode(&r)?.len() - r.name.as_bytes().len();
            r.name_padding = if r.name.marker() == 0x90 {
                vec![0; align4(before_name)? - before_name]
            } else {
                Vec::new()
            };
            let b = encode(&r)?;
            // Native empty.pdb and rbxport's property encoder reserve eight
            // trailing zero bytes before rounding the allocation to four bytes.
            let n = align4(b.len().checked_add(8).ok_or(Error::MutationOverflow)?)?;
            (b, n)
        }
        _ => {
            let bytes = inline(table, row)?;
            let allocation = align4(bytes.len())?;
            return Ok(RowLayout {
                bytes,
                allocation,
                slot_header: false,
            });
        }
    };
    let allocation = allocation.max(align4(bytes.len())?);
    Ok(RowLayout {
        bytes,
        allocation,
        slot_header: true,
    })
}

/// Reject unmodeled nonzero bytes between referenced frames before rebuilding.
/// Retired bytes after the known content are never copied or overwritten.
pub(super) fn check_relocation(table: TableKind, row: &Row, original: &[u8]) -> Result<()> {
    fn check<const N: usize>(
        table: TableKind,
        original: &[u8],
        prefix: usize,
        offsets: &StringOffsets<N>,
        frames: [&DeviceSqlString; N],
    ) -> Result<()> {
        let end = prefix + offsets.binary_size();
        let mut covered = vec![false; original.len()];
        covered
            .get_mut(..end)
            .ok_or(Error::MutationOverflow)?
            .fill(true);
        let mut known_end = end;
        for (i, frame) in frames.iter().enumerate() {
            let start = usize::from(offsets.offset(i).ok_or(Error::MutationOverflow)?);
            let end = start
                .checked_add(frame.as_bytes().len())
                .ok_or(Error::MutationOverflow)?;
            covered
                .get_mut(start..end)
                .ok_or(Error::MutationOverflow)?
                .fill(true);
            known_end = known_end.max(end);
        }
        if original[..known_end]
            .iter()
            .zip(&covered)
            .any(|(&b, &known)| !known && b != 0)
        {
            return Err(unsupported(
                table,
                "relocation would discard unmodeled bytes between strings",
            ));
        }
        Ok(())
    }
    match row {
        Row::Standard(S::Artist(r)) => check(
            table,
            original,
            ArtistHeader::BINARY_SIZE,
            &r.strings,
            [&r.name],
        ),
        Row::Standard(S::Album(r)) => check(
            table,
            original,
            AlbumHeader::BINARY_SIZE,
            &r.strings,
            [&r.name],
        ),
        Row::Standard(S::Track(r)) => check(
            table,
            original,
            TrackHeader::BINARY_SIZE,
            &r.strings,
            track_frames(&r.text),
        ),
        Row::Extended(E::MyTag(r)) => check(
            table,
            original,
            MyTagHeader::BINARY_SIZE,
            &r.strings,
            [&r.name, &r.unknown_string],
        ),
        Row::Extended(E::DatabaseProperty(r)) => check(
            table,
            original,
            DatabasePropertyHeader::BINARY_SIZE,
            &r.strings,
            std::array::from_fn(|i| &r.unknown_strings[i]),
        ),
        _ => Ok(()),
    }
}

/// Repack within known content, without guessing allocation or consuming tails.
pub(super) fn repack_existing(
    table: TableKind,
    old: &Row,
    new: &Row,
    original: &[u8],
) -> Result<(Vec<u8>, Vec<u8>)> {
    let invalid = || {
        unsupported(
            table,
            "replacement does not fit the existing known content extent",
        )
    };
    if self::table(new) != Some(table) {
        return Err(invalid());
    }
    check_relocation(table, old, original)?;
    macro_rules! repack {
        ($a:expr, $b:expr, $old:expr, $new:expr) => {{
            let (a, b) = ($a, $b);
            if a.header.subtype != b.header.subtype
                || a.header.index_shift != b.header.index_shift
                || a.strings != b.strings
            {
                return Err(invalid());
            }
            repack_frames(
                table,
                original,
                encode(&b.header)?,
                &a.strings,
                $old,
                &b.strings,
                $new,
            )
        }};
    }
    match (old, new) {
        (Row::Standard(S::Track(a)), Row::Standard(S::Track(b))) => {
            repack!(a, b, track_frames(&a.text), track_frames(&b.text))
        }
        (Row::Standard(S::Artist(a)), Row::Standard(S::Artist(b))) => {
            repack!(a, b, [&a.name], [&b.name])
        }
        (Row::Standard(S::Album(a)), Row::Standard(S::Album(b))) => {
            repack!(a, b, [&a.name], [&b.name])
        }
        (Row::Extended(E::DatabaseProperty(a)), Row::Extended(E::DatabaseProperty(b))) => repack!(
            a,
            b,
            std::array::from_fn(|i| &a.unknown_strings[i]),
            std::array::from_fn(|i| &b.unknown_strings[i])
        ),
        (Row::Extended(E::MyTag(a)), Row::Extended(E::MyTag(b))) => {
            let result = repack!(
                a,
                b,
                [&a.name, &a.unknown_string],
                [&b.name, &b.unknown_string]
            );
            if matches!(result, Err(Error::UnsupportedMutation { .. })) {
                if let StringOffsets::Near { marker, .. } = a.strings {
                    // The readable far layout changes offset width, not scalar fields.
                    // Only use it within already-owned content bytes.
                    let mut header = b.header.clone();
                    header.subtype = MyTagHeader::FAR_SUBTYPE;
                    let far = StringOffsets::Far {
                        marker: u16::from(marker),
                        offsets: [0; 2],
                    };
                    return repack_frames(
                        table,
                        original,
                        encode(&header)?,
                        &a.strings,
                        [&a.name, &a.unknown_string],
                        &far,
                        [&b.name, &b.unknown_string],
                    );
                }
            }
            result
        }
        _ => Err(invalid()),
    }
}

fn repack_frames<const N: usize>(
    table: TableKind,
    original: &[u8],
    header: Vec<u8>,
    old_offsets: &StringOffsets<N>,
    old: [&DeviceSqlString; N],
    new_offsets: &StringOffsets<N>,
    new: [&DeviceSqlString; N],
) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut end = header.len() + old_offsets.binary_size();
    for (i, frame) in old.iter().enumerate() {
        let start = usize::from(old_offsets.offset(i).ok_or(Error::MutationOverflow)?);
        end = end.max(
            start
                .checked_add(frame.as_bytes().len())
                .ok_or(Error::MutationOverflow)?,
        );
    }
    let mut bytes = string_row(table, header, new_offsets, new)?;
    if bytes.len() > end {
        return Err(unsupported(
            table,
            "repacked strings exceed existing content",
        ));
    }
    let before = original.get(..end).ok_or(Error::MutationOverflow)?.to_vec();
    bytes.resize(end, 0);
    Ok((before, bytes))
}

pub(super) fn inline(table: TableKind, row: &Row) -> Result<Vec<u8>> {
    if self::table(row) != Some(table) {
        return Err(unsupported(table, "row variant does not match table"));
    }
    match row {
        Row::Standard(row) => match row {
            S::Genre(r) | S::Label(r) | S::HistoryPlaylist(r) | S::Artwork(r) => encode(r),
            S::Key(r) => encode(r),
            S::Color(r) => encode(r),
            S::PlaylistTree(r) => encode(r),
            S::PlaylistEntry(r) => encode(r),
            S::HistoryEntry(r) => encode(r),
            S::BrowseColumn(r) => encode(r),
            S::BrowseCategory(r) => encode(r),
            S::BrowseSort(r) => encode(r),
            _ => Err(unsupported(
                table,
                "row requires a placement-aware string layout",
            )),
        },
        Row::Extended(E::TrackMyTag(r)) => encode(r),
        Row::Extended(E::RecommendLike(r)) => encode(r),
        _ => Err(unsupported(
            table,
            "row requires a placement-aware string layout",
        )),
    }
}

/// Return original and replacement owned field bytes; never serialize a row window.
pub(super) fn update(
    table: TableKind,
    old: &Row,
    new: &Row,
    original: &[u8],
) -> Result<(Vec<u8>, Vec<u8>)> {
    if self::table(new) != Some(table) {
        return Err(unsupported(table, "row variant does not match table"));
    }
    match (old, new) {
        (Row::Standard(S::Track(a)), Row::Standard(S::Track(b)))
            if a.strings == b.strings
                && a.text == b.text
                && a.header.subtype == b.header.subtype
                && a.header.index_shift == b.header.index_shift =>
        {
            Ok((encode(&a.header)?, encode(&b.header)?))
        }
        (Row::Standard(S::Artist(a)), Row::Standard(S::Artist(b)))
            if a.strings == b.strings
                && a.name == b.name
                && a.header.subtype == b.header.subtype
                && a.header.index_shift == b.header.index_shift =>
        {
            Ok((encode(&a.header)?, encode(&b.header)?))
        }
        (Row::Standard(S::Album(a)), Row::Standard(S::Album(b)))
            if a.strings == b.strings
                && a.name == b.name
                && a.header.subtype == b.header.subtype
                && a.header.index_shift == b.header.index_shift =>
        {
            Ok((encode(&a.header)?, encode(&b.header)?))
        }
        (Row::Extended(E::MyTag(a)), Row::Extended(E::MyTag(b)))
            if a.strings == b.strings
                && a.name == b.name
                && a.unknown_string == b.unknown_string
                && a.header.subtype == b.header.subtype
                && a.header.index_shift == b.header.index_shift =>
        {
            Ok((encode(&a.header)?, encode(&b.header)?))
        }
        (Row::Extended(E::DatabaseProperty(a)), Row::Extended(E::DatabaseProperty(b)))
            if a.strings == b.strings
                && a.unknown_strings == b.unknown_strings
                && a.header.subtype == b.header.subtype
                && a.header.index_shift == b.header.index_shift =>
        {
            Ok((encode(&a.header)?, encode(&b.header)?))
        }
        (Row::Standard(S::ExportProperty(a)), Row::Standard(S::ExportProperty(b)))
            if a.subtype == b.subtype
                && a.index_shift == b.index_shift
                && a.date == b.date
                && a.separator == b.separator
                && a.version == b.version
                && a.name_padding == b.name_padding
                && a.name == b.name =>
        {
            Ok((
                encode(a)?[..ExportPropertyRow::PREFIX_SIZE].to_vec(),
                encode(b)?[..ExportPropertyRow::PREFIX_SIZE].to_vec(),
            ))
        }
        _ => match fitting_strings(table, old, new, original) {
            Some(result) => result,
            None => Ok((inline(table, old)?, inline(table, new)?)),
        },
    }
}

// Only bytes already owned by a decoded frame are writable. In particular, a
// row window, padding, or a tombstone is not evidence of available allocation.
fn fitting_strings(
    table: TableKind,
    old: &Row,
    new: &Row,
    original: &[u8],
) -> Option<Result<(Vec<u8>, Vec<u8>)>> {
    macro_rules! patch {
        ($a:expr, $b:expr, $old:expr, $new:expr) => {{
            let (a, b) = ($a, $b);
            if a.strings != b.strings
                || a.header.subtype != b.header.subtype
                || a.header.index_shift != b.header.index_shift
            {
                return None;
            }
            Some((|| {
                patch_frames(
                    table,
                    original,
                    encode(&a.header)?,
                    encode(&b.header)?,
                    &a.strings,
                    $old,
                    $new,
                )
            })())
        }};
    }
    match (old, new) {
        (Row::Standard(S::Track(a)), Row::Standard(S::Track(b))) => {
            patch!(a, b, track_frames(&a.text), track_frames(&b.text))
        }
        (Row::Standard(S::Artist(a)), Row::Standard(S::Artist(b))) => {
            patch!(a, b, [&a.name], [&b.name])
        }
        (Row::Standard(S::Album(a)), Row::Standard(S::Album(b))) => {
            patch!(a, b, [&a.name], [&b.name])
        }
        (Row::Extended(E::MyTag(a)), Row::Extended(E::MyTag(b))) => patch!(
            a,
            b,
            [&a.name, &a.unknown_string],
            [&b.name, &b.unknown_string]
        ),
        (Row::Extended(E::DatabaseProperty(a)), Row::Extended(E::DatabaseProperty(b))) => patch!(
            a,
            b,
            std::array::from_fn(|i| &a.unknown_strings[i]),
            std::array::from_fn(|i| &b.unknown_strings[i])
        ),
        _ => None,
    }
}

fn patch_frames<const N: usize>(
    table: TableKind,
    original: &[u8],
    old_header: Vec<u8>,
    new_header: Vec<u8>,
    offsets: &StringOffsets<N>,
    old: [&DeviceSqlString; N],
    new: [&DeviceSqlString; N],
) -> Result<(Vec<u8>, Vec<u8>)> {
    let invalid = || {
        unsupported(
            table,
            "string edit needs relocation or has overlapping frames",
        )
    };
    if !original.starts_with(&old_header) || old_header.len() != new_header.len() {
        return Err(invalid());
    }
    let prefix = old_header.len() + offsets.binary_size();
    let mut ranges = Vec::with_capacity(N);
    let mut end = prefix;
    for i in 0..N {
        let start = usize::from(offsets.offset(i).ok_or_else(invalid)?);
        let stop = start
            .checked_add(old[i].as_bytes().len())
            .ok_or(Error::MutationOverflow)?;
        if start < prefix
            || original.get(start..stop) != Some(old[i].as_bytes())
            || new[i].as_bytes().len() > old[i].as_bytes().len()
            || (old[i] != new[i] && new[i].marker() & 1 == 0 && start % 4 != 0)
            || ranges
                .iter()
                .any(|r: &std::ops::Range<usize>| start < r.end && stop > r.start)
        {
            return Err(invalid());
        }
        ranges.push(start..stop);
        end = end.max(stop);
    }
    let before = original.get(..end).ok_or_else(invalid)?.to_vec();
    let mut after = before.clone();
    after[..new_header.len()].copy_from_slice(&new_header);
    for (i, range) in ranges.into_iter().enumerate() {
        let bytes = new[i].as_bytes();
        // These bytes belong to the replaced frame, not to an unknown gap.
        // Clear its retired suffix so later relocation does not misidentify old
        // text as unmodeled nonzero data between the new frame and the next one.
        if old[i] != new[i] {
            after[range.clone()].fill(0);
        }
        after[range.start..range.start + bytes.len()].copy_from_slice(bytes);
    }
    Ok((before, after))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{ArtistRow, MyTagRow, StringEncoding};

    #[test]
    fn slot_fixups_preserve_table_bits_and_long_strings_align() -> Result<()> {
        // Near artist, name starts at ten; the lower five index_shift bits are
        // deliberately nonzero to catch accidental loss of internal table bits.
        let mut row = ArtistRow::parse(&[0x60, 0, 7, 0, 42, 0, 0, 0, 3, 10, 5, b'a'])?;
        row.name = DeviceSqlString::encode("artist", StringEncoding::Utf16Le)?;
        let table = TableKind::Standard(P::Artists);
        let layout = layout(table, &Row::Standard(S::Artist(row.clone())))?;
        let bytes = layout.at_slot(17)?;
        assert_eq!(&bytes[2..4], &((17u16 << 5) | 7).to_le_bytes());
        assert_eq!(bytes[9], 12);
        assert_eq!(&bytes[10..12], &[0, 0]);
        assert_eq!(ArtistRow::parse(&bytes)?.name, row.name);
        assert!(layout.at_slot(2048).is_err());
        assert!(layout.at_slot(2047).is_ok());
        let mut opaque_gap = bytes.clone();
        opaque_gap[10] = 0x55;
        let decoded = Row::Standard(S::Artist(ArtistRow::parse(&opaque_gap)?));
        assert!(matches!(
            check_relocation(table, &decoded, &opaque_gap),
            Err(Error::UnsupportedMutation { .. })
        ));
        Ok(())
    }

    #[test]
    fn near_tag_offset_boundary_is_checked_without_truncation() -> Result<()> {
        let mut bytes = vec![0; 33];
        bytes[..2].copy_from_slice(&0x680u16.to_le_bytes());
        bytes[28..].copy_from_slice(&[3, 31, 32, 3, 3]);
        let mut row = MyTagRow::parse(&bytes)?;
        let table = TableKind::Extended(X::MyTags);
        // Long ASCII starts at 32; 219 bytes + four-byte frame ends at 255.
        row.name = DeviceSqlString::encode(&"a".repeat(219), StringEncoding::LongAscii)?;
        let encoded = layout(table, &Row::Extended(E::MyTag(row.clone())))?;
        assert_eq!(encoded.bytes[30], 255);
        assert_eq!(MyTagRow::parse(&encoded.bytes)?.name, row.name);
        row.name = DeviceSqlString::encode(&"a".repeat(220), StringEncoding::LongAscii)?;
        assert!(matches!(
            layout(table, &Row::Extended(E::MyTag(row))),
            Err(Error::UnsupportedMutation { .. })
        ));
        Ok(())
    }

    #[test]
    fn fitting_frames_reject_aliases_and_new_unaligned_long_frames() -> Result<()> {
        let table = TableKind::Extended(X::MyTags);
        let mut bytes = vec![0; 64];
        bytes[..2].copy_from_slice(&0x684u16.to_le_bytes());
        bytes[28..34].copy_from_slice(&[3, 0, 36, 0, 36, 0]);
        let name = DeviceSqlString::encode("original", StringEncoding::ShortAscii)?;
        bytes[36..36 + name.as_bytes().len()].copy_from_slice(name.as_bytes());
        let old = MyTagRow::parse(&bytes)?;
        let mut new = old.clone();
        new.name = DeviceSqlString::encode("new", StringEncoding::ShortAscii)?;
        assert!(matches!(
            update(
                table,
                &Row::Extended(E::MyTag(old.clone())),
                &Row::Extended(E::MyTag(new)),
                &bytes
            ),
            Err(Error::UnsupportedMutation { .. })
        ));
        // Scalar-only edits need not rewrite shared frames.
        let mut new = old.clone();
        new.header.position = 1;
        update(
            table,
            &Row::Extended(E::MyTag(old)),
            &Row::Extended(E::MyTag(new)),
            &bytes,
        )?;

        bytes[..2].copy_from_slice(&0x680u16.to_le_bytes());
        bytes[28..31].copy_from_slice(&[3, 31, 48]);
        bytes[31..31 + name.as_bytes().len()].copy_from_slice(name.as_bytes());
        bytes[48] = 3;
        let old = MyTagRow::parse(&bytes)?;
        let mut new = old.clone();
        new.name = DeviceSqlString::encode("a", StringEncoding::LongAscii)?;
        assert!(matches!(
            update(
                table,
                &Row::Extended(E::MyTag(old)),
                &Row::Extended(E::MyTag(new)),
                &bytes
            ),
            Err(Error::UnsupportedMutation { .. })
        ));
        Ok(())
    }
}
