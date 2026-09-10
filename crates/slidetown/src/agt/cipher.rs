use binrw::io::{Read, Seek, Write};

pub struct XorReader<'cipher, 'reader, T: Read + Seek> {
    reader: &'reader mut T,
    pos: u64,
    cipher_offset: usize,
    cipher: &'cipher [u8],
}

impl<'cipher, 'reader, T: Read + Seek> XorReader<'cipher, 'reader, T> {
    pub fn new(reader: &'reader mut T, cipher: &'cipher [u8], cipher_offset: usize) -> Self {
        let pos: u64 = reader.stream_position().unwrap();
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
        let from = self.pos as usize;
        self.pos += bytes_read as u64;
        let cipher_len = self.cipher.len();
        if cipher_len == 0 {
            return Ok(bytes_read);
        }

        // XOR contiguous slices up to each cipher boundary.
        // This avoids per-byte modulo and indexing checks without unsafe access.
        let start = self.cipher_offset.saturating_sub(from).min(bytes_read);
        let mut at = (from + start) % cipher_len;
        let mut rest = &mut buf[start..bytes_read];
        while !rest.is_empty() {
            let run = rest.len().min(cipher_len - at);
            let (now, later) = rest.split_at_mut(run);
            for (byte, key) in now.iter_mut().zip(&self.cipher[at..at + run]) {
                *byte ^= *key;
            }
            at = (at + run) % cipher_len;
            rest = later;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;

    /// Byte-at-a-time reference implementation.
    fn one_at_a_time(plain: &[u8], cipher: &[u8], offset: usize) -> Vec<u8> {
        let mut out = plain.to_vec();
        for (i, byte) in out.iter_mut().enumerate() {
            if i >= offset {
                *byte ^= cipher[i % cipher.len()];
            }
        }
        out
    }

    /// Check stream-relative cipher alignment across read sizes, including reads
    /// that end mid-cipher or before the cipher offset.
    #[test]
    fn a_run_at_a_time_matches_a_byte_at_a_time() {
        let plain: Vec<u8> = (0..1000u32).map(|i| (i * 7 % 251) as u8).collect();
        for cipher_len in [1usize, 3, 16, 64, 255] {
            let cipher: Vec<u8> = (0..cipher_len).map(|i| (i * 31 % 256) as u8).collect();
            for offset in [0usize, 1, 17, 512, 1000, 2000] {
                let want = one_at_a_time(&plain, &cipher, offset);
                for chunk in [1usize, 5, 64, 999, 4096] {
                    let mut source = Cursor::new(plain.clone());
                    let mut reader = XorReader::new(&mut source, &cipher, offset);
                    let mut got = Vec::new();
                    let mut buffer = vec![0u8; chunk];
                    loop {
                        let read = reader.read(&mut buffer).unwrap();
                        if read == 0 {
                            break;
                        }
                        got.extend_from_slice(&buffer[..read]);
                    }
                    assert_eq!(
                        got, want,
                        "cipher {cipher_len}, offset {offset}, chunk {chunk}"
                    );
                }
            }
        }
    }
}
