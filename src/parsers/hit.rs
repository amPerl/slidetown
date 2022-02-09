use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"hit\0")]
pub struct Header {
    #[br(assert(version_date == 20060720, "unexpected version {}", version_date))]
    pub version_date: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub header: Header,

    #[bw(calc = indices.len() as u32)]
    pub index_count: u32,
    #[br(count=index_count)]
    pub indices: Vec<u32>,

    #[bw(calc = verts.len() as u32)]
    pub vert_count: u32,
    #[br(count=vert_count)]
    pub verts: Vec<(f32, f32, f32)>,
}

impl Hit {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}
