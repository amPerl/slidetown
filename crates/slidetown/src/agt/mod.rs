use binrw::{
    io::{Read, Seek},
    BinWriterExt,
};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Cursor, SeekFrom};
use std::{collections::BTreeMap, io::Write};

mod cipher;

use crate::parsers::agt::Entry;
use crate::parsers::agt::Header;

use self::cipher::XorReader;
use self::cipher::XorWriter;

pub struct AgtReader<'cipher, 'reader, T: Read + Seek> {
    reader: XorReader<'cipher, 'reader, T>,
}

impl<'cipher, 'reader, T: Read + Seek> AgtReader<'cipher, 'reader, T> {
    pub fn new(reader: &'reader mut T, cipher: &'cipher [u8]) -> Self {
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
            .seek(SeekFrom::Start(entry.chunks_offset as u64))?;

        let mut chunk_lengths = Vec::new();

        for _ in 0..entry.chunk_count {
            let mut len_buf = [0u8; 2];
            self.reader.read_exact(&mut len_buf)?;
            chunk_lengths.push(u16::from_le_bytes(len_buf));
        }

        let mut result = Vec::new();

        for len in chunk_lengths {
            let mut compressed_buf = vec![0u8; len as _];
            self.reader.read_exact(&mut compressed_buf)?;
            let mut decoder = ZlibDecoder::new(Cursor::new(compressed_buf));
            decoder.read_to_end(&mut result)?;
        }

        Ok(result)
    }
}

#[derive(Debug)]
enum AgtBuilderEntrySource {
    // /// The compressed data for this entry will be copied from an existing AGT file
    // AgtFile { path: String, entry: Entry },
    // /// The uncompressed data for this entry will be read from a file
    // File {
    //     /// Path to the file
    //     path: String,
    //     /// Offset within the file
    //     offset: usize,
    //     /// Length of the data
    //     size: usize,
    // },
    /// The entry will be dumped from memory
    Memory {
        /// Raw, uncompressed data
        data: Vec<u8>,
    },
}

impl AgtBuilderEntrySource {
    pub fn entry(&self, path: String) -> Entry {
        match self {
            // AgtBuilderEntrySource::AgtFile { entry, .. } => Entry {
            //     chunks_offset: 0,
            //     chunk_count: (entry.decompressed_length as f64 / 16384.0).ceil() as u32,
            //     decompressed_length: entry.decompressed_length,
            //     path,
            // },
            AgtBuilderEntrySource::Memory { data } => Entry {
                chunks_offset: 0,
                chunk_count: (data.len() as f64 / 16384.0).ceil() as u32,
                decompressed_length: data.len() as _,
                path,
            },
        }
    }
}

pub struct AgtBuilder {
    entry_sources: BTreeMap<String, AgtBuilderEntrySource>,
}

impl AgtBuilder {
    pub fn new() -> Self {
        Self {
            entry_sources: Default::default(),
        }
    }

    // /// Add an entry that will be read from an agt file
    // pub fn add_agt_entry(&mut self, agt_path: String, entry: Entry) {
    //     self.entry_sources.insert(
    //         entry.path.clone(),
    //         AgtBuilderEntrySource::AgtFile {
    //             path: agt_path,
    //             entry,
    //         },
    //     );
    // }
    // /// Add an entry that will be read from a file
    // pub fn add_entry_file(
    //     &mut self,
    //     entry_path: String,
    //     file_path: String,
    //     offset: usize,
    //     size: usize,
    // ) {
    //     self.entry_sources.insert(
    //         entry_path,
    //         AgtBuilderEntrySource::File {
    //             path: file_path,
    //             offset,
    //             size,
    //         },
    //     );
    // }

    /// Add an entry from memory
    pub fn add_entry_memory(&mut self, path: String, data: &[u8]) {
        self.entry_sources.insert(
            path,
            AgtBuilderEntrySource::Memory {
                data: data.to_vec(),
            },
        );
    }

    pub fn write<W: Write + Seek>(self, writer: &mut W, cipher: &[u8]) -> anyhow::Result<()> {
        let mut writer = XorWriter::new(writer, cipher, 32);

        writer.write_le(&Header {
            file_count: self.entry_sources.len() as u32,
            ..Default::default()
        })?;

        // Offsets to entries in the table of contents, for backfilling chunk offsets
        let mut entry_offsets = Vec::new();
        // Chunk counts, to skip chunk lengths segment later
        let mut chunk_counts = Vec::new();

        for (new_entry_path, entry_source) in self.entry_sources.iter() {
            // Record location where we're going to write this entry
            entry_offsets.push(writer.stream_position()?);
            // Create an incomplete entry, store the chunk count and write the entry
            let entry = entry_source.entry(new_entry_path.to_owned());
            chunk_counts.push(entry.chunk_count);
            writer.write_le(&entry)?;
        }

        // Offsets to [chunk lengths, chunks], to fill in chunk offsets
        let mut data_offsets = Vec::new();

        // Go through all the entry data sources, compress and write the chunks, then backfill the chunk lengths
        for (entry_source, entry_chunks_count) in self
            .entry_sources
            .into_values()
            .zip(chunk_counts.into_iter())
        {
            // Record the position and skip the chunk lengths
            let data_offset = writer.stream_position()?;
            data_offsets.push(data_offset);
            writer.seek(SeekFrom::Current(entry_chunks_count as i64 * 2))?;

            // Compressed chunk lengths, for backfilling the chunk header
            let mut entry_chunks_lengths = Vec::new();
            match entry_source {
                // AgtBuilderEntrySource::AgtFile { path, entry } => todo!(),
                AgtBuilderEntrySource::Memory { mut data } => {
                    for chunk in data.chunks_mut(16384) {
                        let compressed_chunk = {
                            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                            encoder.write_all(chunk)?;
                            encoder.finish()?
                        };

                        let compressed_len = compressed_chunk.len() as u16;
                        entry_chunks_lengths.push(compressed_len);
                        writer.write_le(&compressed_chunk)?;
                    }
                }
            }
            // Backfill chunk header then return to position
            let post_data_offset = writer.stream_position()?;
            writer.seek(SeekFrom::Start(data_offset))?;
            writer.write_le(&entry_chunks_lengths)?;
            writer.seek(SeekFrom::Start(post_data_offset))?;
        }

        // Backfill data offsets in entries
        for (entry_offset, data_offset) in entry_offsets.into_iter().zip(data_offsets.into_iter()) {
            writer.seek(SeekFrom::Start(entry_offset))?;
            writer.write_le(&(data_offset as u32))?;
        }

        Ok(())
    }
}

impl Default for AgtBuilder {
    fn default() -> Self {
        Self::new()
    }
}
