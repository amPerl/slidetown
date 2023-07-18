use binrw::io::{Read, Seek, Write};
use std::io::SeekFrom;

pub struct XorReader<'cipher, 'reader, T: Read + Seek> {
    reader: &'reader mut T,
    pos: u64,
    cipher_offset: usize,
    cipher: &'cipher [u8],
}

impl<'cipher, 'reader, T: Read + Seek> XorReader<'cipher, 'reader, T> {
    pub fn new(reader: &'reader mut T, cipher: &'cipher [u8], cipher_offset: usize) -> Self {
        let pos: u64 = reader.seek(SeekFrom::Current(0)).unwrap();
        Self {
            reader,
            pos,
            cipher,
            cipher_offset,
        }
    }
}

impl<'cipher, 'reader, T: Read + Seek> Read for XorReader<'cipher, 'reader, T> {
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

impl<'cipher, 'reader, T: Read + Seek> Seek for XorReader<'cipher, 'reader, T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

pub struct XorWriter<'cipher, 'writer, T: Write + Seek> {
    writer: &'writer mut T,
    pos: u64,
    cipher_offset: usize,
    cipher: &'cipher [u8],
}

impl<'cipher, 'writer, T: Write + Seek> XorWriter<'cipher, 'writer, T> {
    pub fn new(writer: &'writer mut T, cipher: &'cipher [u8], cipher_offset: usize) -> Self {
        Self {
            writer,
            pos: 0,
            cipher,
            cipher_offset,
        }
    }
}

impl<'cipher, 'writer, T: Write + Seek> Write for XorWriter<'cipher, 'writer, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // apply cipher to buffer to be written
        let cipher_len = self.cipher.len();
        let mut ciphered_buf = buf.to_owned();
        (0..ciphered_buf.len()).for_each(|i| {
            let pos = self.pos as usize + i;
            if pos >= self.cipher_offset {
                // Safety: cipher_len is checked before the loop
                ciphered_buf[i] ^= unsafe { self.cipher.get_unchecked(pos % cipher_len) };
            }
        });
        // write buffer to underlying writer and advance position (potentially partially)
        let bytes_written = self.writer.write(&ciphered_buf)?;
        self.pos += bytes_written as u64;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<'cipher, 'writer, T: Write + Seek> Seek for XorWriter<'cipher, 'writer, T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
