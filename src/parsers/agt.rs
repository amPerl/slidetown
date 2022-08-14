use crate::parsers::strings;
use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};

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
        entry_count: u32,
    ) -> anyhow::Result<Vec<Entry>> {
        let entries = (0..entry_count)
            .into_iter()
            .map(|_| reader.read_le::<Entry>())
            .collect::<Result<Vec<Entry>, _>>()?;

        Ok(entries)
    }
}
