use binrw::io::{Read, Seek};
use inflate::InflateWriter;
use std::io::SeekFrom;
use std::io::Write;

mod cipher;

use crate::parsers::agt::Entry;
use crate::parsers::agt::Header;

use self::cipher::XorReader;

pub struct AgtReader<'cipher, T: Read + Seek> {
    reader: XorReader<'cipher, T>,
}

impl<'cipher, T: Read + Seek> AgtReader<'cipher, T> {
    pub fn new(reader: T, cipher: &'cipher [u8]) -> Self {
        Self {
            reader: XorReader::new(reader, cipher, 32),
        }
    }

    pub fn read_header(&mut self) -> anyhow::Result<Header> {
        self.reader.seek(SeekFrom::Start(0))?;
        Header::parse(&mut self.reader)
    }

    pub fn read_entries(&mut self, file_count: u32) -> anyhow::Result<Vec<Entry>> {
        self.reader.seek(SeekFrom::Start(32))?;
        Entry::parse_entries(&mut self.reader, file_count)
    }

    /// Read and decrypt the data for the given entry
    pub fn read_entry_data(&mut self, entry: &Entry) -> anyhow::Result<Vec<u8>> {
        self.reader
            .seek(SeekFrom::Start(entry.header_offset as u64))?;

        let mut chunk_lengths = Vec::new();

        for _ in 0..entry.chunk_count {
            let mut len_buf = [0u8; 2];
            self.reader.read_exact(&mut len_buf)?;
            chunk_lengths.push(u16::from_le_bytes(len_buf));
        }

        let mut result = Vec::new();

        for len in chunk_lengths {
            let mut compressed_buf = vec![0u8; (len - 2) as usize];
            self.reader.seek(SeekFrom::Current(2))?;
            self.reader.read_exact(&mut compressed_buf)?;

            let mut decoder = InflateWriter::new(Vec::new());
            decoder.write_all(&compressed_buf)?;
            result.append(&mut decoder.finish()?);
        }

        Ok(result)
    }
}
