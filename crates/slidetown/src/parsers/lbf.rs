use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
#[br(magic = b"LBF\0kjc\0ag\0\0")]
pub struct Header {
    pub version_date: u32,

    pub unknown2: u32,
    pub block_count: u32,

    pub block_object_count: u32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Block {
    pub object_count: u32,
    #[br(count = object_count)]
    pub objects: Vec<BlockObject>,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct BlockObject {
    pub unk: u32,
    pub block_index: u32,

    #[serde(skip)]
    pub file_offset: u32,

    #[serde(skip)]
    pub file_length: u32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Lbf {
    pub header: Header,

    #[br(count = if header.is_empty_stub() { 0 } else { header.block_count as usize })]
    pub blocks: Vec<Block>,
}

impl Header {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    /// NeoOros and Taipei's `blockObj1.LBF` use 28-byte empty stubs with valid magic
    /// and version, but 0xFFFFFFFF counts. Treat these as empty before reading blocks.
    pub fn is_empty_stub(&self) -> bool {
        self.block_count == u32::MAX
    }
}

impl Block {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

impl Lbf {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// NeoOros/Taipei empty stub: magic, version, and all-ones counts.
    const EMPTY_STUB: &[u8] = &[
        b'L', b'B', b'F', 0, b'k', b'j', b'c', 0, b'a', b'g', 0, 0, // magic
        0x59, 0x8F, 0x32, 0x01, // version_date = 20090713
        0xFF, 0xFF, 0xFF, 0xFF, // unknown2
        0xFF, 0xFF, 0xFF, 0xFF, // block_count
        0xFF, 0xFF, 0xFF, 0xFF, // block_object_count
    ];

    #[test]
    fn empty_stub_parses_as_zero_blocks() {
        let lbf = Lbf::parse(&mut Cursor::new(EMPTY_STUB)).expect("stub should parse");
        assert!(lbf.header.is_empty_stub());
        assert_eq!(lbf.header.version_date, 20090713);
        assert!(lbf.blocks.is_empty());
    }

    #[test]
    fn ordinary_header_is_not_a_stub() {
        let mut bytes = EMPTY_STUB.to_vec();
        bytes[20..24].copy_from_slice(&0u32.to_le_bytes()); // block_count = 0
        let lbf = Lbf::parse(&mut Cursor::new(bytes)).expect("should parse");
        assert!(!lbf.header.is_empty_stub());
        assert!(lbf.blocks.is_empty());
    }
}
