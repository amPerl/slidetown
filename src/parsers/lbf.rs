use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
#[br(magic = b"LBF\0kjc\0")]
pub struct Header {
    pub unknown1: u32,

    #[serde(skip)]
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
    pub index: u32,

    #[serde(skip)]
    pub file_offset: u32,

    #[serde(skip)]
    pub file_length: u32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Lbf {
    pub header: Header,

    #[br(count = header.block_count)]
    pub blocks: Vec<Block>,
}

impl Header {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
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
