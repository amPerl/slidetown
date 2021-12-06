use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
#[br(magic = b"hit\0")]
pub struct Header {
    pub version_date: u32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Hit {
    pub header: Header,

    pub index_count: u32,
    #[br(count=index_count)]
    pub indices: Vec<u32>,

    pub vert_count: u32,
    #[br(count=vert_count)]
    pub verts: Vec<(f32, f32, f32)>,
}

impl Header {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

impl Hit {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}
