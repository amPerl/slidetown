#![allow(unused_variables)]

use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt, NullString,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Chpath {
    pub header: Header,
    pub always_same_1: u32,
    #[bw(calc = (paths.iter().map(Path::size_bytes).sum::<usize>() + 20) as u16)]
    pub file_size: u16, // without header
    pub always_same_2: [u16; 5],
    pub path_count: u32,
    #[br(count = path_count)]
    pub paths: Vec<Path>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"CHPATH\0")]
pub struct Header {
    #[br(assert(version_date == 20040810, "unexpected version {}", version_date))]
    pub version_date: u32,

    #[br(assert(nhn == "NHN-AG"))]
    #[br(map = |x: NullString| x.to_string())]
    #[bw(map = |x: &String| NullString::from(x.clone()))]
    pub nhn: String,

    #[br(assert(jc == "jc"))]
    #[br(map = |x: NullString| x.to_string())]
    #[bw(map = |x: &String| NullString::from(x.clone()))]
    pub jc: String,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Path {
    #[br(count = 7)]
    pub always_same_1: Vec<u32>,
    pub unknown1: u32,
    #[br(count = 10)]
    pub always_same_2: Vec<u32>,
    pub total_distance1: f32,
    pub total_distance2: f32,
    pub always_same_3: u32,
    pub unknown2: u32,

    #[bw(calc = points.len() as u32)]
    pub point_count: u32,
    #[br(count = point_count)]
    pub points: Vec<(f32, f32, f32, f32)>,

    #[bw(ignore)]
    #[br(temp, try, restore_position)]
    pub peek: u32,

    #[br(if(peek == 20010710))]
    pub missing_in_taipei: Option<[u32; 3]>,
}

impl Path {
    const SIZE_POINT: usize = 16;
    const SIZE_REST: usize = 104;
    const SIZE_REST_TAIPEI: usize = 92;

    pub fn size_bytes(&self) -> usize {
        Self::SIZE_POINT * self.points.len()
            + if self.missing_in_taipei.is_some() {
                Self::SIZE_REST
            } else {
                Self::SIZE_REST_TAIPEI
            }
    }
}

impl Chpath {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}
