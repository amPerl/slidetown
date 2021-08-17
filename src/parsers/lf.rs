use binread::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
#[br(magic = b"LF\0\0kjc\0")]
pub struct Header {
    pub unknown1: u32,

    #[serde(skip)]
    pub version_date: u32,

    pub unknown2: u32,
    pub block_count: u32,

    #[br(count = 13)]
    pub unknown3: Vec<u32>,

    pub size_x: u32,
    pub size_y: u32,
    pub size_idx: u32,

    #[br(count = 5)]
    pub unknown4: Vec<f32>,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub position_x: u32,
    pub position_y: u32,

    #[serde(skip)]
    pub file_offset: u32,

    #[serde(skip)]
    pub file_length: u32,

    pub unknown: u32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Lf {
    pub header: Header,

    #[br(count=header.block_count)]
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

impl Lf {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}
