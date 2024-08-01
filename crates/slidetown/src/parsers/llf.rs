use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"LLF\0kjc\0ag\0\0")]
pub struct Header {
    #[br(assert(version_date == 20061204, "unexpected version {}", version_date))]
    pub version_date: u32,
    pub unk: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub block_index: u32,

    #[serde(skip)]
    pub file_offset: u32,

    #[serde(skip)]
    pub file_length: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Llf {
    pub header: Header,
    #[bw(calc = blocks.len() as u32)]
    pub block_count: u32,
    #[br(count = block_count)]
    pub blocks: Vec<Block>,
}

impl Llf {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}
