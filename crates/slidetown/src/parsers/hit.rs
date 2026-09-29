use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Header {
    #[br(assert(magic.eq_ignore_ascii_case(b"hit\0"), "unexpected magic {:?}", magic))]
    pub magic: [u8; 4],
    #[br(assert(
        version_date == 20051005 || version_date == 20060720 || version_date == 20090629,
        "unexpected version {}",
        version_date
    ))]
    pub version_date: u32,
}

impl Header {
    pub fn has_w(&self) -> bool {
        self.version_date != 20060720
    }
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[br(import(has_w: bool))]
pub struct Vert {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Always 0 in 20051005, a small unknown int in 20090629.
    #[br(if(has_w))]
    pub w: Option<f32>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub header: Header,

    #[bw(calc = indices.len() as u32)]
    pub index_count: u32,
    #[br(count = index_count)]
    pub indices: Vec<u32>,

    #[bw(calc = verts.len() as u32)]
    pub vert_count: u32,
    #[br(count = vert_count, args { inner: (header.has_w(),) })]
    pub verts: Vec<Vert>,
}

impl Hit {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}
