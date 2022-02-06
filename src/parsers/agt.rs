use std::io::SeekFrom;
use std::io::Write;

use crate::parsers::strings;
use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use inflate::InflateWriter;

#[derive(Debug, PartialEq, BinRead)]
#[br(magic = b"NayaPack")]
pub struct Header {
    pub what: u32,
    pub version: (u16, u16),
    pub file_count: u32,
    pub what2: u32,
    pub what3: u32,
    pub what4: u32,
}

#[derive(Debug, PartialEq, BinRead)]
pub struct Entry {
    pub header_offset: u32,
    pub chunk_count: u32,
    pub decompressed_length: u32,

    #[br(parse_with = strings::parse_int_prefixed_string )]
    pub path: String,
}

impl Header {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Header> {
        Ok(reader.read_le()?)
    }
}

impl Entry {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Entry> {
        Ok(reader.read_le()?)
    }

    pub fn parse_entries<R: Read + Seek>(
        reader: &mut R,
        entry_count: usize,
    ) -> anyhow::Result<Vec<Entry>> {
        let entries = (0..entry_count)
            .into_iter()
            .map(|_| reader.read_le::<Entry>())
            .collect::<Result<Vec<Entry>, _>>()?;

        Ok(entries)
    }
}

pub struct AgtReader<'cipher, T: Read + Seek> {
    inner: T,
    pos: u64,
    cipher: &'cipher [u8],
}

impl<'cipher, T: Read + Seek> AgtReader<'cipher, T> {
    pub fn new(mut inner: T, cipher: &'cipher [u8]) -> Self {
        let pos: u64 = inner.seek(SeekFrom::Current(0)).unwrap();
        Self { inner, pos, cipher }
    }

    pub fn read_entry(&mut self, entry: &Entry) -> anyhow::Result<Vec<u8>> {
        self.seek(SeekFrom::Start(entry.header_offset as u64))?;

        let mut block_lengths = Vec::new();

        for _ in 0..entry.chunk_count {
            let mut len_buf = [0u8; 2];
            self.read_exact(&mut len_buf)?;
            block_lengths.push(u16::from_le_bytes(len_buf));
        }

        let mut result = Vec::new();

        for len in block_lengths {
            let mut compressed_buf = vec![0u8; (len - 2) as usize];
            self.seek(SeekFrom::Current(2))?;
            self.read_exact(&mut compressed_buf)?;

            let mut decoder = InflateWriter::new(Vec::new());
            decoder.write_all(&compressed_buf)?;
            result.append(&mut decoder.finish()?);
        }

        Ok(result)
    }
}

impl<'cipher, T: Read + Seek> Read for AgtReader<'cipher, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.inner.read(buf) {
            Ok(bytes_read) => {
                let cipher_len = self.cipher.len();
                (0..bytes_read).for_each(|i| {
                    let pos = self.pos as usize + i;
                    if pos >= 32 {
                        buf[i] ^= self.cipher[pos % cipher_len];
                    }
                });
                self.pos += bytes_read as u64;
                Ok(bytes_read)
            }
            Err(e) => Err(e),
        }
    }
}

impl<'cipher, T: Read + Seek> Seek for AgtReader<'cipher, T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match self.inner.seek(pos) {
            Ok(new_pos) => {
                self.pos = new_pos;
                Ok(new_pos)
            }
            Err(e) => Err(e),
        }
    }
}
