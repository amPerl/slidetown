use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
#[br(magic = b"DPDB")]
pub struct Header {
    pub version_date: u32,

    pub speed_length: u32,
    pub accel_length: u32,
    pub dura_length: u32,
    pub boost_length: u32,

    #[br(count=speed_length)]
    pub speed_ids: Vec<u32>,
    #[br(count=accel_length)]
    pub accel_ids: Vec<u32>,
    #[br(count=dura_length)]
    pub dura_ids: Vec<u32>,
    #[br(count=boost_length)]
    pub boost_ids: Vec<u32>,
}
#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
#[br(import(option_size: u32))]
pub struct GroupOption {
    #[br(count = option_size)]
    pub values: Vec<f32>,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct LevelModifier {
    pub header: Header,

    #[br(args { count: if header.version_date > 20070507 { 1001 } else { 801 }, inner: (header.speed_length,) })]
    pub speed: Vec<GroupOption>,
    #[br(args { count: if header.version_date > 20070507 { 1001 } else { 801 }, inner: (header.accel_length,) })]
    pub accel: Vec<GroupOption>,
    #[br(args { count: if header.version_date > 20070507 { 1001 } else { 801 }, inner: (header.dura_length,) })]
    pub dura: Vec<GroupOption>,
    #[br(args { count: if header.version_date > 20070507 { 1001 } else { 801 }, inner: (header.boost_length,) })]
    pub boost: Vec<GroupOption>,
}

impl Header {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

impl GroupOption {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

impl LevelModifier {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}
