use std::io;

pub(crate) trait ReadExt: io::Read {
    /// Read `size` bytes and convert them to a [`String`].
    /// Every trailing null terminator will be removed.
    fn read_string_limited(&mut self, size: usize) -> io::Result<String>;
}

impl<R: io::Read> ReadExt for R {
    fn read_string_limited(&mut self, size: usize) -> io::Result<String> {
        let mut buf: Vec<u8> = vec![0; size];
        self.read_exact(buf.as_mut_slice())?;

        let s = String::from_utf8_lossy(&buf).into_owned();
        Ok(s.trim_end_matches('\0').to_string())
    }
}
