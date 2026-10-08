// Copyright (C) 2026 Dylan Jones
// SPDX-License-Identifier: GPL-3.0-only

use binrw::io::{Read, Result, Seek, SeekFrom, Write};
use std::iter::Cycle;

/// Stream cipher wrapper around another [`Read`]- or [`Write`]-able stream that XOR's all bytes
/// after reading/before writing.
#[derive(Debug)]
pub struct XorStream<T> {
    stream: T,
    key: Cycle<std::vec::IntoIter<u8>>,
    key_size: u64,
}

impl<T> XorStream<T> {
    /// Create a new XOR wrapper around `stream` that passes XOR's all data with `key` before
    /// forwarding it.
    pub fn with_key(stream: T, key: Vec<u8>) -> Self {
        let key = if key.is_empty() { vec![0] } else { key };
        let key_size = key.len() as u64;
        let key = key.into_iter().cycle();
        Self {
            stream,
            key,
            key_size,
        }
    }
}

impl<R: Read> Read for XorStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes_read = self.stream.read(buf)?;

        for byte in &mut buf[..bytes_read] {
            let key_byte = self.key.next().unwrap_or(0);
            *byte ^= key_byte;
        }

        Ok(bytes_read)
    }
}

impl<W: Write> Write for XorStream<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let encrypted_buffer: Vec<u8> = buf
            .iter()
            .zip(self.key.clone())
            .map(|(byte, key)| byte ^ key)
            .collect();
        let written = self.stream.write(&encrypted_buffer)?;
        if written != 0 {
            self.key.nth(written - 1);
        }
        Ok(written)
    }

    fn flush(&mut self) -> Result<()> {
        self.stream.flush()
    }
}

impl<S: Seek> Seek for XorStream<S> {
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        let old_position = self.stream.stream_position()?;
        let new_position = self.stream.seek(position)?;

        // Calculate how many bytes we need to move forward in the key stream to match the seek in
        // the actual buffer.
        let offset = if new_position > old_position {
            (new_position - old_position) % self.key_size
        } else {
            self.key_size - ((old_position - new_position) % self.key_size)
        };

        for _ in 0..offset {
            self.key.next();
        }

        Ok(new_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Error, ErrorKind};

    struct PartialStream {
        inner: Cursor<Vec<u8>>,
        fail_write: bool,
    }

    impl Read for PartialStream {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let len = buf.len().min(2);
            self.inner.read(&mut buf[..len])
        }
    }

    impl Write for PartialStream {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            if self.fail_write {
                self.fail_write = false;
                return Err(Error::from(ErrorKind::Interrupted));
            }
            self.inner.write(&buf[..buf.len().min(2)])
        }

        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }

    impl Seek for PartialStream {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
            self.inner.seek(pos)
        }
    }

    #[test]
    fn short_reads_leave_unused_buffer_and_key_untouched() {
        let plain = b"partial stream data";
        let key = vec![11, 22, 33];
        let encrypted: Vec<_> = plain
            .iter()
            .zip(key.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect();
        let inner = PartialStream {
            inner: Cursor::new(encrypted),
            fail_write: false,
        };
        let mut stream = XorStream::with_key(inner, key);
        let mut first = [0xff; 7];
        assert_eq!(stream.read(&mut first).unwrap(), 2);
        assert_eq!(&first[..2], &plain[..2]);
        assert_eq!(&first[2..], &[0xff; 5]);
        let mut rest = Vec::new();
        stream.read_to_end(&mut rest).unwrap();
        assert_eq!(rest, plain[2..]);
        stream.seek(SeekFrom::Start(1)).unwrap();
        let mut next = [0; 6];
        stream.read_exact(&mut next).unwrap();
        assert_eq!(&next, &plain[1..7]);
    }

    #[test]
    fn short_and_interrupted_writes_advance_only_written_bytes() {
        let plain = b"partial stream data";
        let key = vec![11, 22, 33];
        let expected: Vec<_> = plain
            .iter()
            .zip(key.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect();
        let inner = PartialStream {
            inner: Cursor::new(Vec::new()),
            fail_write: true,
        };
        let mut stream = XorStream::with_key(inner, key);
        stream.write_all(plain).unwrap();
        assert_eq!(stream.stream.inner.into_inner(), expected);
    }
}
