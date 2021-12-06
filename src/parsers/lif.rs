use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
#[br(magic = b"LIF\0kjc\0")]
pub struct Header {
    pub unknown1: u32,
    pub version_date: u32,
    pub block_count: u32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub unk: u32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Lif {
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

impl Lif {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}
