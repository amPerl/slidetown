use binrw::io::{Read, Seek};
use std::io::SeekFrom;

pub struct XorReader<'cipher, T: Read + Seek> {
    reader: T,
    pos: u64,
    cipher_offset: usize,
    cipher: &'cipher [u8],
}

impl<'cipher, T: Read + Seek> XorReader<'cipher, T> {
    pub fn new(mut reader: T, cipher: &'cipher [u8], cipher_offset: usize) -> Self {
        let pos: u64 = reader.seek(SeekFrom::Current(0)).unwrap();
        Self {
            reader,
            pos,
            cipher,
            cipher_offset,
        }
    }
}

impl<'cipher, T: Read + Seek> Read for XorReader<'cipher, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.reader.read(buf)?;
        let cipher_len = self.cipher.len();
        (0..bytes_read).for_each(|i| {
            let pos = self.pos as usize + i;
            if pos >= self.cipher_offset {
                // Safety: cipher_len is checked before the loop
                buf[i] ^= unsafe { self.cipher.get_unchecked(pos % cipher_len) };
            }
        });
        self.pos += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl<'cipher, T: Read + Seek> Seek for XorReader<'cipher, T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}
