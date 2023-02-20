use std::io;

pub(crate) trait ReadExt: io::Read {
    /// Read `size` bytes and convert them to a [`String`].
    /// Every trailing null terminator will be removed.
    fn read_string_limited(&mut self, size: usize) -> io::Result<String>;

    /// Read bytes until a null terminator (`\0`) is found. Bytes are then
    /// converted to a String and returned.
    ///
    /// # Errors
    ///
    /// If `limit` is reached and no null terminator has been found, an error
    /// is returned.
    fn read_string_to_terminator(&mut self, limit: usize) -> io::Result<String>;
}

impl<R: io::Read> ReadExt for R {
    fn read_string_limited(&mut self, size: usize) -> io::Result<String> {
        let mut buf: Vec<u8> = vec![0; size];
        self.read_exact(buf.as_mut_slice())?;

        let s = String::from_utf8_lossy(&buf).into_owned();
        Ok(s.trim_end_matches('\0').to_string())
    }

    fn read_string_to_terminator(&mut self, limit: usize) -> io::Result<String> {
        let mut bytes: Vec<u8> = Vec::with_capacity(limit.min(256));
        let mut buf: [u8; 1] = [0];

        for _ in 0..limit {
            self.read_exact(&mut buf)?;

            // Check for null terminator
            if buf[0] == 0 {
                return Ok(String::from_utf8_lossy(&bytes).into_owned());
            }

            bytes.push(buf[0]);
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "limit has been reached without finding a null terminator",
        ))
    }
}
