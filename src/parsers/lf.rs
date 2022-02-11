use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"LF\0\0kjc\0ag\0\0")]
pub struct Header {
    #[br(assert(version_date == 20061220 || version_date == 20090406, "unexpected version {}", version_date))]
    pub version_date: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub position_x: u32,
    pub position_y: u32,

    pub file_offset: u32,
    pub file_length: u32,

    pub unknown: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Lf {
    pub header: Header,

    pub unknown2: u32,
    pub block_count: u32,

    #[br(count = 13)]
    pub unknown3: Vec<u32>,

    pub size_x: u32,
    pub size_y: u32,
    pub size_idx: u32,

    #[br(count = 5)]
    pub unknown4: Vec<f32>,

    #[br(count = block_count)]
    pub blocks: Vec<Block>,
}

impl Lf {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}
