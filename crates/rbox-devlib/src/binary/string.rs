// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Framed DeviceSQL strings with lossless binary I/O and explicit text decoding.
//!
//! Parsing retains framing, padding, terminators, and invalid Unicode. New frames
//! require an explicit encoding, avoiding an implicit device-compatibility policy.
//! The unusual ISRC representation is interpreted only through `isrc_text`.

use binrw::{BinRead, BinResult, BinWrite, Endian};
use std::io::{Cursor, Read, Seek, Write};

use crate::{Error, Result};

/// Representation to use when explicitly authoring a string frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    /// One-byte length marker, at most 126 ASCII bytes.
    ShortAscii,
    /// Marker 0x40, little-endian total length, one opaque byte, ASCII payload.
    LongAscii,
    /// Marker 0x90 and UTF-16LE payload without a trailing NUL.
    Utf16Le,
    /// Marker 0x90 and UTF-16LE payload with one trailing NUL code unit.
    Utf16LeTerminated,
}

/// One immutable frame. No text normalization occurs on reads or writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSqlString {
    bytes: Vec<u8>,
}

impl DeviceSqlString {
    /// Parse a single frame from the beginning of a bounded slice.
    /// Following bytes are left out of the returned frame.
    /// Unknown even markers are preserved using the long-frame envelope; their
    /// text encoding is not assumed to be known. Invalid text does not fail parsing.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Ok(Self::read_le(&mut Cursor::new(bytes))?)
    }

    /// Create text using an explicit framing choice. Row alignment is separate.
    /// Length limits describe the wire representation, not available row space
    /// or a guarantee that every player accepts strings of that length.
    pub fn encode(text: &str, encoding: StringEncoding) -> Result<Self> {
        if text.contains('\0') {
            return Err(Error::EmbeddedNul);
        }
        let mut bytes = Vec::new();
        match encoding {
            StringEncoding::ShortAscii => {
                if !text.is_ascii() {
                    return Err(Error::NonAscii);
                }
                if text.len() > 126 {
                    return Err(Error::StringTooLong);
                }
                bytes.push((text.len() * 2 + 3) as u8);
                bytes.extend_from_slice(text.as_bytes());
            }
            StringEncoding::LongAscii => {
                if !text.is_ascii() {
                    return Err(Error::NonAscii);
                }
                if text.len() > usize::from(u16::MAX) - 4 {
                    return Err(Error::StringTooLong);
                }
                bytes.push(0x40);
                bytes.extend_from_slice(&((text.len() + 4) as u16).to_le_bytes());
                bytes.push(0);
                bytes.extend_from_slice(text.as_bytes());
            }
            StringEncoding::Utf16Le | StringEncoding::Utf16LeTerminated => {
                let terminated = encoding == StringEncoding::Utf16LeTerminated;
                let units = text.encode_utf16().count();
                let max = (usize::from(u16::MAX) - 4) / 2 - usize::from(terminated);
                if units > max {
                    return Err(Error::StringTooLong);
                }
                bytes.push(0x90);
                let len = (4 + 2 * (units + usize::from(terminated))) as u16;
                bytes.extend_from_slice(&len.to_le_bytes());
                bytes.push(0);
                for unit in text.encode_utf16() {
                    bytes.extend_from_slice(&unit.to_le_bytes());
                }
                if terminated {
                    bytes.extend_from_slice(&[0, 0]);
                }
            }
        }
        Ok(Self { bytes })
    }

    /// Author the special ISRC frame used in a track's ISRC slot.
    ///
    /// Empty input produces the ordinary empty short frame (`03`). Otherwise,
    /// exactly twelve ASCII alphanumeric bytes are required. This checks the
    /// representation only, not whether an ISRC was assigned by a registry.
    /// Normal text, even if it looks like an ISRC, should use [`Self::encode`].
    pub fn encode_isrc(text: &str) -> Result<Self> {
        if text.is_empty() {
            return Self::encode(text, StringEncoding::ShortAscii);
        }
        if text.len() != 12 || !text.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return Err(Error::InvalidIsrc);
        }
        let mut bytes = Vec::with_capacity(18);
        bytes.extend_from_slice(&[0x90, 18, 0, 0, 3]);
        bytes.extend_from_slice(text.as_bytes());
        bytes.push(0);
        Ok(Self { bytes })
    }

    /// Original complete frame, including any opaque fourth byte.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Original first byte, without interpreting unknown long markers.
    pub fn marker(&self) -> u8 {
        self.bytes[0]
    }

    /// Payload without the one-byte or four-byte framing header.
    pub fn payload(&self) -> &[u8] {
        &self.bytes[if self.marker() & 1 != 0 { 1 } else { 4 }..]
    }

    /// Decode strictly, removing one trailing UTF-16 NUL if present.
    /// This never guesses that UTF-16 beginning with byte 0x03 is an ISRC.
    /// ASCII payloads retain NUL bytes verbatim. Neither form stops at an embedded
    /// NUL; decoding uses the declared frame length.
    pub fn text(&self) -> Result<String> {
        let payload = self.payload();
        match self.marker() {
            m if m & 1 != 0 || m == 0x40 => {
                if !payload.is_ascii() {
                    return Err(Error::NonAscii);
                }
                Ok(payload.iter().map(|&b| char::from(b)).collect())
            }
            0x90 => {
                if !payload.len().is_multiple_of(2) {
                    return Err(Error::InvalidUtf16);
                }
                let mut units: Vec<_> = payload
                    .chunks_exact(2)
                    .map(|b| u16::from_le_bytes([b[0], b[1]]))
                    .collect();
                if units.last() == Some(&0) {
                    units.pop();
                }
                String::from_utf16(&units).map_err(|_| Error::InvalidUtf16)
            }
            m => Err(Error::UnsupportedStringMarker(m)),
        }
    }

    /// Decode the special ISRC framing in the track's ISRC slot explicitly.
    /// The ordinary empty short frame represents an absent ISRC.
    pub fn isrc_text(&self) -> Result<String> {
        if self.bytes == [3] {
            return Ok(String::new());
        }
        let p = self.payload();
        if self.marker() != 0x90
            || p.len() != 14
            || p[0] != 3
            || p[13] != 0
            || !p[1..13].iter().all(u8::is_ascii_alphanumeric)
        {
            return Err(Error::InvalidIsrc);
        }
        Ok(p[1..13].iter().map(|&b| char::from(b)).collect())
    }

    /// Explicit display-only conversion. Unsupported markers remain an error.
    /// ASCII-designated payloads use UTF-8 replacement decoding, allowing display
    /// of nonconforming UTF-8. UTF-16 replaces unpaired surrogates and an odd final
    /// byte. As in [`Self::text`], one trailing UTF-16 NUL is removed.
    pub fn text_lossy(&self) -> Result<String> {
        let p = self.payload();
        match self.marker() {
            m if m & 1 != 0 || m == 0x40 => Ok(String::from_utf8_lossy(p).into_owned()),
            0x90 => {
                let mut units: Vec<_> = p
                    .chunks_exact(2)
                    .map(|b| u16::from_le_bytes([b[0], b[1]]))
                    .collect();
                if p.len().is_multiple_of(2) && units.last() == Some(&0) {
                    units.pop();
                }
                let mut text = String::from_utf16_lossy(&units);
                if !p.len().is_multiple_of(2) {
                    text.push('\u{fffd}');
                }
                Ok(text)
            }
            m => Err(Error::UnsupportedStringMarker(m)),
        }
    }
}

impl BinRead for DeviceSqlString {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(r: &mut R, _: Endian, (): ()) -> BinResult<Self> {
        let pos = r.stream_position()?;
        let marker = u8::read_le(r)?;
        let mut bytes = vec![marker];
        if marker & 1 != 0 {
            if marker < 3 {
                return Err(binrw::Error::AssertFail {
                    pos,
                    message: "invalid short string length".into(),
                });
            }
            bytes.resize(usize::from(marker >> 1), 0);
            r.read_exact(&mut bytes[1..])?;
        } else {
            let len = u16::read_le(r)?;
            if len < 4 {
                return Err(binrw::Error::AssertFail {
                    pos: pos + 1,
                    message: "long string length is smaller than its header".into(),
                });
            }
            bytes.extend_from_slice(&len.to_le_bytes());
            // The u16 frame length bounds this allocation to 65535 bytes.
            bytes.resize(usize::from(len), 0);
            r.read_exact(&mut bytes[3..])?;
        }
        Ok(Self { bytes })
    }
}

impl BinWrite for DeviceSqlString {
    type Args<'a> = ();
    fn write_options<W: Write + Seek>(&self, w: &mut W, _: Endian, (): ()) -> BinResult<()> {
        w.write_all(&self.bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! String wire examples, encoding boundaries, and lossless malformed-data handling.
    //!
    //! Independent tests informed by Rekordcrate's empty/ASCII/Unicode/ISRC cases,
    //! rbxport's native unterminated Unicode and aligned artist samples, and Baken's
    //! browse-column example. Expected frames below are literal bytes, not output
    //! obtained from the codec under test. Tests use only in-memory byte arrays
    //! and cursors; no database rows, external fixtures, or shared test crate.

    use super::*;

    fn assert_preserved(raw: &[u8]) -> DeviceSqlString {
        let parsed = DeviceSqlString::parse(raw).unwrap();
        assert_eq!(parsed.as_bytes(), raw);
        let mut output = Cursor::new(Vec::new());
        parsed.write_le(&mut output).unwrap();
        assert_eq!(output.into_inner(), raw);
        parsed
    }

    fn assert_encoding(text: &str, encoding: StringEncoding, raw: &[u8]) {
        let parsed = assert_preserved(raw);
        assert_eq!(parsed.text().unwrap(), text);
        assert_eq!(parsed.text_lossy().unwrap(), text);
        assert_eq!(DeviceSqlString::encode(text, encoding).unwrap(), parsed);
    }

    #[test]
    fn ascii_and_empty_frames_match_wire_bytes() {
        assert_encoding("", StringEncoding::ShortAscii, &[3]);
        assert_encoding("1A", StringEncoding::ShortAscii, b"\x071A");
        assert_encoding("abc", StringEncoding::ShortAscii, b"\x09abc");
        assert_encoding("", StringEncoding::LongAscii, &[0x40, 4, 0, 0]);
        assert_encoding("abc", StringEncoding::LongAscii, b"\x40\x07\0\0abc");
        assert_encoding("", StringEncoding::Utf16Le, &[0x90, 4, 0, 0]);
        assert_encoding(
            "",
            StringEncoding::Utf16LeTerminated,
            &[0x90, 6, 0, 0, 0, 0],
        );
        // ASCII includes control characters; only NUL is forbidden when authoring.
        assert_encoding("\t\n\r\x7f", StringEncoding::ShortAscii, b"\x0b\t\n\r\x7f");
    }

    #[test]
    fn every_valid_short_marker_and_the_127_byte_boundary() {
        for len in 0..=126 {
            let text = "x".repeat(len);
            let mut wire = vec![(2 * len + 3) as u8];
            wire.extend_from_slice(text.as_bytes());
            assert_encoding(&text, StringEncoding::ShortAscii, &wire);
        }
        let text = "x".repeat(127);
        assert!(matches!(
            DeviceSqlString::encode(&text, StringEncoding::ShortAscii),
            Err(Error::StringTooLong)
        ));
        let mut wire = vec![0x40, 0x83, 0, 0];
        wire.extend_from_slice(text.as_bytes());
        assert_encoding(&text, StringEncoding::LongAscii, &wire);
    }

    #[test]
    fn native_unicode_frames_have_no_implicit_terminator() {
        assert_encoding(
            "L\u{dc}RUM",
            StringEncoding::Utf16Le,
            &[0x90, 14, 0, 0, 0x4c, 0, 0xdc, 0, 0x52, 0, 0x55, 0, 0x4d, 0],
        );
        assert_encoding(
            "DJ Ti\u{eb}sto",
            StringEncoding::Utf16Le,
            &[
                0x90, 22, 0, 0, 0x44, 0, 0x4a, 0, 0x20, 0, 0x54, 0, 0x69, 0, 0xeb, 0, 0x73, 0,
                0x74, 0, 0x6f, 0,
            ],
        );
        // Native browse-column decoration, not Unicode text to normalize away.
        assert_encoding(
            "\u{fffa}GENRE\u{fffb}",
            StringEncoding::Utf16Le,
            &[
                0x90, 18, 0, 0, 0xfa, 0xff, 0x47, 0, 0x45, 0, 0x4e, 0, 0x52, 0, 0x45, 0, 0xfb, 0xff,
            ],
        );
    }

    #[test]
    fn utf16_handles_surrogate_pairs_and_does_not_guess_isrc_from_byte_three() {
        assert_encoding(
            "\u{103}\u{1f3b5}",
            StringEncoding::Utf16Le,
            &[0x90, 10, 0, 0, 3, 1, 0x3c, 0xd8, 0xb5, 0xdf],
        );
        let string = DeviceSqlString::parse(&[0x90, 6, 0, 0, 3, 1]).unwrap();
        assert_eq!(string.text().unwrap(), "\u{103}");
        assert!(matches!(string.isrc_text(), Err(Error::InvalidIsrc)));
        assert_encoding(
            "\u{1f3b5}",
            StringEncoding::Utf16LeTerminated,
            &[0x90, 10, 0, 0, 0x3c, 0xd8, 0xb5, 0xdf, 0, 0],
        );
    }

    #[test]
    fn long_lengths_count_bytes_and_include_the_header() {
        for len in [252, 256, 65_531] {
            let text = "a".repeat(len);
            let string = DeviceSqlString::encode(&text, StringEncoding::LongAscii).unwrap();
            assert_eq!(string.as_bytes().len(), len + 4);
            assert_eq!(&string.as_bytes()[1..3], &((len + 4) as u16).to_le_bytes());
            assert_eq!(assert_preserved(string.as_bytes()).text().unwrap(), text);
        }
        assert!(matches!(
            DeviceSqlString::encode(&"a".repeat(65_532), StringEncoding::LongAscii),
            Err(Error::StringTooLong)
        ));
        for (encoding, units) in [
            (StringEncoding::Utf16Le, 32_765),
            (StringEncoding::Utf16LeTerminated, 32_764),
        ] {
            // Three UTF-8 bytes, one UTF-16 unit: UTF-8 length is not the wire limit.
            let text = "\u{800}".repeat(units);
            let string = DeviceSqlString::encode(&text, encoding).unwrap();
            assert_eq!(&string.as_bytes()[..4], &[0x90, 0xfe, 0xff, 0]);
            assert_eq!(assert_preserved(string.as_bytes()).text().unwrap(), text);
            assert!(matches!(
                DeviceSqlString::encode(&(text + "a"), encoding),
                Err(Error::StringTooLong)
            ));
        }
        // A supplementary code point needs two units, not one character slot.
        let text = "\u{1f3b5}".repeat(16_382) + "a";
        assert_eq!(
            DeviceSqlString::encode(&text, StringEncoding::Utf16Le)
                .unwrap()
                .as_bytes()
                .len(),
            65_534
        );
        assert!(matches!(
            DeviceSqlString::encode(&(text + "\u{1f3b5}"), StringEncoding::Utf16Le),
            Err(Error::StringTooLong)
        ));
    }

    #[test]
    fn authored_text_rejects_non_ascii_and_embedded_nul() {
        for encoding in [StringEncoding::ShortAscii, StringEncoding::LongAscii] {
            assert!(matches!(
                DeviceSqlString::encode("\u{e9}", encoding),
                Err(Error::NonAscii)
            ));
        }
        for encoding in [
            StringEncoding::ShortAscii,
            StringEncoding::LongAscii,
            StringEncoding::Utf16Le,
            StringEncoding::Utf16LeTerminated,
        ] {
            for text in ["\0", "a\0b", "a\0"] {
                assert!(matches!(
                    DeviceSqlString::encode(text, encoding),
                    Err(Error::EmbeddedNul)
                ));
            }
        }
    }

    #[test]
    fn existing_nuls_and_opaque_header_bytes_are_preserved() {
        for raw in [&b"\x09a\0b"[..], &b"\x40\x07\0\xa5a\0b"[..]] {
            assert_eq!(assert_preserved(raw).text().unwrap(), "a\0b");
        }
        assert_eq!(
            assert_preserved(b"\x40\x06\0\xa5a\0").text().unwrap(),
            "a\0"
        );
        let raw = [0x90, 12, 0, 0xa5, b'a', 0, 0, 0, b'b', 0, 0, 0];
        let string = assert_preserved(&raw);
        assert_eq!(string.payload(), &raw[4..]);
        assert_eq!(string.text().unwrap(), "a\0b");
        assert_eq!(string.text_lossy().unwrap(), "a\0b");
        // Strip exactly one final UTF-16 NUL, not all trailing NULs.
        let string = assert_preserved(&[0x90, 10, 0, 0, b'a', 0, 0, 0, 0, 0]);
        assert_eq!(string.text().unwrap(), "a\0");
        assert_eq!(string.text_lossy().unwrap(), "a\0");
    }

    #[test]
    fn invalid_ascii_is_lossless_and_strict_decoding_is_distinct_from_display() {
        for (raw, lossy) in [
            (&[5, 0xff][..], "\u{fffd}"),
            (&[7, 0xc3, 0xa9][..], "\u{e9}"),
            (&[0x40, 6, 0, 0, 0xc3, 0xa9][..], "\u{e9}"),
        ] {
            let string = assert_preserved(raw);
            assert!(matches!(string.text(), Err(Error::NonAscii)));
            assert_eq!(string.text_lossy().unwrap(), lossy);
        }
    }

    #[test]
    fn malformed_utf16_is_lossless_with_explicit_replacement_decoding() {
        for (payload, display) in [
            (&[0, 0xd8][..], "\u{fffd}"), // Lone high surrogate.
            (&[0, 0xdc][..], "\u{fffd}"), // Lone low surrogate.
            (&[0, 0xdc, 0, 0xd8][..], "\u{fffd}\u{fffd}"),
            (&[0x61][..], "\u{fffd}"), // Incomplete unit must not silently disappear.
            (&[0x61, 0, 0xff][..], "a\u{fffd}"),
            (&[0, 0xd8, 0xff][..], "\u{fffd}\u{fffd}"),
            (&[0, 0, 0xff][..], "\0\u{fffd}"), // This NUL is not the final unit.
        ] {
            let mut raw = vec![0x90, (4 + payload.len()) as u8, 0, 0xa5];
            raw.extend_from_slice(payload);
            let string = assert_preserved(&raw);
            assert!(matches!(string.text(), Err(Error::InvalidUtf16)));
            assert_eq!(string.text_lossy().unwrap(), display);
        }
    }

    #[test]
    fn isrc_authors_and_reads_the_special_frame_and_empty_value() {
        let raw = b"\x90\x12\0\0\x03GBAYE6700149\0";
        let string = assert_preserved(raw);
        assert_eq!(string.isrc_text().unwrap(), "GBAYE6700149");
        assert_eq!(
            DeviceSqlString::encode_isrc("GBAYE6700149").unwrap(),
            string
        );
        assert_eq!(DeviceSqlString::encode_isrc("").unwrap().as_bytes(), &[3]);
        assert_eq!(assert_preserved(&[3]).isrc_text().unwrap(), "");
        // Unknown fourth header byte is not part of the ISRC text.
        let mut opaque = raw.to_vec();
        opaque[3] = 0xa5;
        assert_eq!(
            assert_preserved(&opaque).isrc_text().unwrap(),
            "GBAYE6700149"
        );
        let ordinary = DeviceSqlString::encode("GBAYE6700149", StringEncoding::ShortAscii).unwrap();
        assert!(matches!(ordinary.isrc_text(), Err(Error::InvalidIsrc)));
        assert_ne!(string.text().unwrap(), "GBAYE6700149"); // Interpretation requires slot context.
    }

    #[test]
    fn malformed_isrc_frames_are_retained_but_not_interpreted() {
        for input in [
            "GBAYE670014",
            "GBAYE67001499",
            "GBAYE670014!",
            "GBAYE670014\0",
            "GBAYE67001\u{e9}",
        ] {
            assert!(matches!(
                DeviceSqlString::encode_isrc(input),
                Err(Error::InvalidIsrc)
            ));
        }
        let valid = b"\x90\x12\0\0\x03GBAYE6700149\0";
        for (offset, value) in [(0, 0x40), (4, 2), (5, 0xff), (5, 0), (16, b'!'), (17, 1)] {
            let mut raw = valid.to_vec();
            raw[offset] = value;
            assert!(matches!(
                assert_preserved(&raw).isrc_text(),
                Err(Error::InvalidIsrc)
            ));
        }
        let short = b"\x90\x11\0\0\x03GBAYE670014\0";
        assert!(matches!(
            assert_preserved(short).isrc_text(),
            Err(Error::InvalidIsrc)
        ));
    }

    #[test]
    fn unknown_even_markers_preserve_the_long_envelope_without_guessing_text() {
        for marker in (0..=254).step_by(2).filter(|m| ![0x40, 0x90].contains(m)) {
            let string = assert_preserved(&[marker, 5, 0, 0xab, 0xff]);
            assert_eq!(string.marker(), marker);
            assert_eq!(string.payload(), &[0xff]);
            assert!(matches!(string.text(), Err(Error::UnsupportedStringMarker(m)) if m == marker));
            assert!(
                matches!(string.text_lossy(), Err(Error::UnsupportedStringMarker(m)) if m == marker)
            );
        }
    }

    #[test]
    fn truncated_frames_and_impossible_lengths_fail() {
        for frame in [
            &b"\x09abc"[..],
            &b"\x40\x07\0\0abc"[..],
            &[0x90, 6, 0, 0, 0x61, 0][..],
            &b"\x90\x12\0\0\x03GBAYE6700149\0"[..],
            &[0x42, 5, 0, 0xa5, 0xff][..],
        ] {
            for end in 0..frame.len() {
                assert!(
                    matches!(DeviceSqlString::parse(&frame[..end]), Err(Error::Binary(_))),
                    "prefix {end} of {frame:?}"
                );
            }
        }
        assert!(matches!(
            DeviceSqlString::parse(&[1]),
            Err(Error::Binary(_))
        ));
        for marker in [0x40, 0x90, 0x42] {
            for len in 0..4 {
                assert!(DeviceSqlString::parse(&[marker, len, 0, 0]).is_err());
            }
        }
    }

    #[test]
    fn streams_consume_one_frame_at_nonzero_positions_and_stay_little_endian() {
        let frames: &[&[u8]] = &[
            b"\x09abc",
            b"\x40\x07\0\xa5abc",
            &[0x90, 6, 0, 0, 3, 1],
            &[0x42, 4, 0, 0x99],
        ];
        for frame in frames {
            let mut input = vec![0xcc, 0xdd];
            input.extend_from_slice(frame);
            input.extend_from_slice(&[3, 0xee]);
            assert_eq!(
                DeviceSqlString::parse(&input[2..]).unwrap().as_bytes(),
                *frame
            );
            let mut reader = Cursor::new(&input);
            reader.set_position(2);
            let string = DeviceSqlString::read_options(&mut reader, Endian::Big, ()).unwrap();
            assert_eq!(reader.position(), (2 + frame.len()) as u64);
            assert_eq!(
                DeviceSqlString::read_le(&mut reader)
                    .unwrap()
                    .text()
                    .unwrap(),
                ""
            );
            assert_eq!(reader.position(), (3 + frame.len()) as u64);
            let mut writer = Cursor::new(input.clone());
            writer.set_position(2);
            string.write_options(&mut writer, Endian::Big, ()).unwrap();
            assert_eq!(writer.position(), (2 + frame.len()) as u64);
            assert_eq!(writer.into_inner(), input);
        }
    }

    #[test]
    fn string_framing_leaves_surrounding_alignment_bytes_to_the_caller() {
        // Native LURUM string bytes, with unrelated prefix and trailing padding.
        let frame = [0x90, 14, 0, 0, 0x4c, 0, 0xdc, 0, 0x52, 0, 0x55, 0, 0x4d, 0];
        let mut raw = vec![0xaa, 0xbb, 0, 0];
        raw.extend_from_slice(&frame);
        raw.extend_from_slice(&[0, 0, 3]); // Padding to the next frame, then empty text.
        let mut reader = Cursor::new(&raw);
        reader.set_position(4);
        let string = DeviceSqlString::read_le(&mut reader).unwrap();
        assert_eq!(string.as_bytes(), frame);
        assert_eq!(string.text().unwrap(), "L\u{dc}RUM");
        assert_eq!(reader.position(), 18);
        assert_eq!(&raw[reader.position() as usize..], &[0, 0, 3]);

        // Neither reading nor writing the frame skips or emits alignment bytes.
        let mut writer = Cursor::new(raw.clone());
        writer.set_position(4);
        string.write_le(&mut writer).unwrap();
        assert_eq!(writer.position(), 18);
        assert_eq!(writer.into_inner(), raw);
        reader.set_position(20);
        assert_eq!(
            DeviceSqlString::read_le(&mut reader)
                .unwrap()
                .text()
                .unwrap(),
            ""
        );
    }
}
