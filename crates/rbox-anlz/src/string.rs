// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

//! Length-prefixed UTF-16 string encoding used by ANLZ sections.

use binrw::io::{Read, Seek, Write};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use std::fmt;

/// A byte-length-prefixed UTF-16BE ANLZ comment.
///
/// Text is validated as UTF-16. Original trailing NUL padding is retained, so
/// absent, unterminated, and padded comments round-trip without normalization.
/// New nonempty strings get one NUL; new empty comments have a zero-byte payload.
/// PPTH paths retain their separate, always-terminated representation.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AnlzString {
    text: String,
    trailing_nuls: u32,
}

impl AnlzString {
    /// Encoded payload size, excluding the four-byte length prefix.
    pub fn payload_byte_len(&self) -> u32 {
        self.checked_payload_len()
            .expect("ANLZ string exceeds u32 length")
    }

    fn checked_payload_len(&self) -> Option<u32> {
        u32::try_from(self.text.encode_utf16().count())
            .ok()?
            .checked_add(self.trailing_nuls)?
            .checked_mul(2)
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Replace the text while retaining its padding convention.
    pub fn set(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.text.is_empty() && self.trailing_nuls == 0 && !text.is_empty() {
            self.trailing_nuls = 1;
        }
        self.text = text;
    }

    fn read_payload<R: Read + Seek>(reader: &mut R, maximum: Option<u32>) -> BinResult<Self> {
        let pos = reader.stream_position()?;
        let len = u32::read_options(reader, Endian::Big, ())?;
        if len % 2 != 0 || maximum.is_some_and(|max| len > max) {
            return Err(binrw::Error::AssertFail {
                pos,
                message: "UTF-16 payload has an odd length or exceeds its ANLZ entry".into(),
            });
        }
        // Grow only as bytes are read; an untrusted prefix must not trigger
        // a huge up-front allocation on a truncated input.
        let mut units = Vec::new();
        for _ in 0..len / 2 {
            units.push(u16::read_options(reader, Endian::Big, ())?);
        }
        let trailing_nuls = units.iter().rev().take_while(|unit| **unit == 0).count();
        units.truncate(units.len() - trailing_nuls);
        let text = String::from_utf16(&units).map_err(|err| binrw::Error::Custom {
            pos,
            err: Box::new(err),
        })?;
        Ok(Self {
            text,
            trailing_nuls: trailing_nuls as u32,
        })
    }

    /// Parse within the payload space left in an enclosing ANLZ entry.
    pub(crate) fn read_bounded<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        (maximum,): (u32,),
    ) -> BinResult<Self> {
        Self::read_payload(reader, Some(maximum))
    }
}

impl From<String> for AnlzString {
    fn from(text: String) -> Self {
        let trailing_nuls = u32::from(!text.is_empty());
        Self {
            text,
            trailing_nuls,
        }
    }
}

impl From<&str> for AnlzString {
    fn from(text: &str) -> Self {
        text.to_owned().into()
    }
}

impl AsRef<str> for AnlzString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::ops::Deref for AnlzString {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AnlzString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl BinRead for AnlzString {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        Self::read_payload(reader, None)
    }
}

impl BinWrite for AnlzString {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<()> {
        let len = self
            .checked_payload_len()
            .ok_or_else(|| binrw::Error::AssertFail {
                pos: writer.stream_position().unwrap_or(0),
                message: "ANLZ string exceeds u32 length".into(),
            })?;
        len.write_options(writer, Endian::Big, ())?;
        for unit in self.text.encode_utf16() {
            unit.write_options(writer, Endian::Big, ())?;
        }
        for _ in 0..self.trailing_nuls {
            0u16.write_options(writer, Endian::Big, ())?;
        }
        Ok(())
    }
}
