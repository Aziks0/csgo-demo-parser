use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};

pub struct BitReader<'a> {
    bytes: &'a [u8],
    /// Position in bits
    position: usize,
    /// Length in bits
    length: usize,
    /// Length in bytes
    length_bytes: usize,
}

impl<'a> BitReader<'a> {
    /// Create a new bit reader.
    pub fn new(bytes: &'a [u8]) -> BitReader {
        BitReader {
            position: 0,
            length_bytes: bytes.len(),
            length: bytes.len() * 8,
            bytes,
        }
    }

    /// Read `size` bytes.
    pub fn read_bytes_exact(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; size];
        self.read_exact(buf.as_mut_slice())?;
        Ok(buf)
    }

    /// Read a [`u16`].
    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    /// Read a single byte.
    pub fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Read a single bit.
    pub fn read_bit(&mut self) -> Result<bool> {
        let new_position = self.position + 1;
        if new_position > self.length {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }
        let bit = self.peak_bit();
        self.position = new_position;
        Ok(bit)
    }

    // Copyright 2015 Ilkka Rauta
    //
    // Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
    // http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
    // <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
    // option. This file may not be copied, modified, or distributed
    // except according to those terms.
    //
    // https://github.com/irauta/bitreader/blob/af674130489109495733ea007813643e2a3eb988/src/lib.rs#L352
    /// Read a single bit, without moving the cursor of the underlying reader.
    fn peak_bit(&mut self) -> bool {
        let byte_index = self.position / 8;
        let byte = self.bytes[byte_index];
        let shift = 7 - (self.position % 8);
        let bit = byte >> shift & 1;
        bit == 1
    }

    /// Read a single byte from an unaligned position.
    ///
    /// It also works for aligned byte, but requires more computation.
    fn read_byte_unaligned(&mut self) -> u8 {
        let end_position = self.position + 8;
        let mut value: u8 = 0;

        for i in self.position..end_position {
            let byte_index = i / 8;
            let byte = self.bytes[byte_index];
            let shift = 7 - (i % 8);
            let bit = byte >> shift & 1;
            value = (value << 1) | bit;
        }

        self.position = end_position;
        value
    }

    /// Seek to an offset, in bits.
    fn seek_bits(&mut self, pos: SeekFrom) -> Result<u64> {
        let new_position = match pos {
            SeekFrom::Current(offset) => self.position as i64 + offset,
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => self.length as i64 + offset,
        };

        if new_position < 0 || new_position > self.length as i64 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            ));
        }

        self.position = new_position as usize;
        Ok(new_position as u64)
    }
}

impl Read for BitReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let start_position = self.position;
        // The length of our BitReader's buffer is the upper bound
        let end_position = (self.position + (buf.len() * 8)).min(self.length);
        // EOF reached
        if start_position == end_position {
            return Ok(0);
        }
        // Not enough data left to read a byte
        if end_position - start_position < 8 {
            return Ok(0);
        }

        let is_aligned = self.position % 8 == 0;
        let position_bytes = self.position / 8;
        let bytes_to_read = buf.len().min(self.length_bytes - position_bytes);

        for (i, buf_value) in buf.iter_mut().enumerate().take(bytes_to_read) {
            // Let the compiler/branching do its thing
            *buf_value = if is_aligned {
                self.position += 8;
                self.bytes[position_bytes + i]
            } else {
                self.read_byte_unaligned()
            };
        }

        Ok(bytes_to_read)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let bytes_read = self.read(buf)?;
        if bytes_read != buf.len() {
            Err(Error::from(ErrorKind::UnexpectedEof))
        } else {
            Ok(())
        }
    }

    fn read_to_end(&mut self, _buf: &mut Vec<u8>) -> Result<usize> {
        unimplemented!();
    }

    fn read_to_string(&mut self, _buf: &mut String) -> Result<usize> {
        unimplemented!();
    }

    fn read_vectored(&mut self, _buf: &mut [std::io::IoSliceMut<'_>]) -> Result<usize> {
        unimplemented!();
    }
}

impl Seek for BitReader<'_> {
    /// Seek to an offset, in bytes, in a stream.
    ///
    /// If the seek operation completed successfully, this method returns the
    /// new position, in bytes, from the start of the stream.
    /// In cases where the stream is unaligned, the returned value does not
    /// correspond to the _real_ new position of the stream. **So the returned value should be
    /// considered unreliable.**
    ///
    /// # Errors
    ///
    /// Seeking to a negative offset or beyond the end of a stream is
    /// considered an error.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let new_seek = match pos {
            SeekFrom::Current(offset) => SeekFrom::Current(offset * 8),
            SeekFrom::Start(offset) => SeekFrom::Start(offset * 8),
            SeekFrom::End(offset) => SeekFrom::End(offset * 8),
        };

        Ok(self.seek_bits(new_seek)? / 8)
    }

    fn stream_position(&mut self) -> Result<u64> {
        Ok(self.position as u64)
    }

    fn rewind(&mut self) -> Result<()> {
        self.position = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_bytes() {
        let mut buf: [u8; 4] = [0; 4];
        let bytes: &[u8] = &[0, 1, 2, 3];
        let mut reader = BitReader::new(bytes);
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(bytes[0..], buf);
    }

    #[test]
    fn read_bits() {
        let bytes: &[u8] = &[0b1010_1100];
        let mut reader = BitReader::new(bytes);

        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());

        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
    }

    #[test]
    fn read_bytes_unaligned() {
        let mut buf: [u8; 1] = [0];
        let bytes: &[u8] = &[0b1100_0010, 0b1001_1000];
        let expected_byte: [u8; 1] = [0b1000_0101];
        let mut reader = BitReader::new(bytes);

        assert!(reader.read_bit().unwrap());

        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, expected_byte);

        assert!(!reader.read_bit().unwrap());
    }

    #[test]
    fn error_on_overflow_read_bytes() {
        let mut buf: [u8; 5] = [0; 5];
        let bytes: &[u8] = &[0, 1, 2, 3];
        let mut reader = BitReader::new(bytes);
        assert_eq!(
            reader.read_exact(&mut buf).map_err(|e| e.kind()),
            Err(ErrorKind::UnexpectedEof)
        );
    }

    #[test]
    fn error_on_overflow_read_bits() {
        let mut buf = [0];
        let bytes: &[u8] = &[0];
        let mut reader = BitReader::new(bytes);
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(
            reader.read_bit().map_err(|e| e.kind()),
            Err(ErrorKind::UnexpectedEof)
        );
    }

    #[test]
    fn seek_bytes() {
        let mut buf: [u8; 2] = [0; 2];
        let bytes: &[u8] = &[0, 1, 2, 3, 4, 5];
        let mut reader = BitReader::new(bytes);

        assert_eq!(reader.seek(SeekFrom::Current(2)).unwrap(), 2);
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(bytes[2..4], buf);

        assert_eq!(reader.seek(SeekFrom::End(-2)).unwrap(), 4);
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(bytes[4..], buf);

        assert_eq!(reader.seek(SeekFrom::Start(1)).unwrap(), 1);
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(bytes[1..3], buf);
    }

    #[test]
    fn seek_bits() {
        let bytes: &[u8] = &[0b1010_1100];
        let mut reader = BitReader::new(bytes);

        assert!(reader.read_bit().unwrap());
        assert_eq!(reader.seek_bits(SeekFrom::Current(2)).unwrap(), 3);
        assert!(!reader.read_bit().unwrap());

        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());

        assert_eq!(reader.seek_bits(SeekFrom::End(-2)).unwrap(), 6);
        assert!(!reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());

        assert_eq!(reader.seek_bits(SeekFrom::Start(4)).unwrap(), 4);
        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
    }

    #[test]
    fn seek_bytes_unaligned() {
        let mut buf: [u8; 1] = [0];
        let bytes: &[u8] = &[0b1100_1111, 0b1001_0000];
        let mut reader = BitReader::new(bytes);

        assert!(reader.read_bit().unwrap());

        reader.seek(SeekFrom::Current(1)).unwrap();
        assert!(!reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());

        reader.seek(SeekFrom::End(-1)).unwrap();
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());

        reader.seek(SeekFrom::Start(1)).unwrap();
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(bytes[1..], buf);
    }

    #[test]
    fn error_on_overflow_seek_bytes() {
        let bytes: &[u8] = &[0, 1, 2, 3];
        let mut reader = BitReader::new(bytes);
        assert_eq!(
            reader.seek(SeekFrom::Start(6)).map_err(|e| e.kind()),
            Err(ErrorKind::InvalidInput)
        );
    }

    #[test]
    fn error_on_overflow_seek_bits() {
        let bytes: &[u8] = &[0, 1, 2, 3];
        let mut reader = BitReader::new(bytes);
        assert_eq!(
            reader.seek(SeekFrom::Start(36)).map_err(|e| e.kind()),
            Err(ErrorKind::InvalidInput)
        );
    }
}
