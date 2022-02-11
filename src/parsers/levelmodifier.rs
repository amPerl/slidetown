use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum LevelModifierVersion {
    #[brw(magic = 20070507_u32)]
    Stage1,
    #[brw(magic = 20100814_u32)]
    Stage2,
    #[brw(magic = 20110901_u32)]
    Stage3,
}

impl LevelModifierVersion {
    pub fn stat_range(&self) -> usize {
        match self {
            // Stage 1 / 0.22
            LevelModifierVersion::Stage1 => 801,
            // Stage 2 / EU
            LevelModifierVersion::Stage2 => 1001,
            // Stage 3 / GC/Dev
            LevelModifierVersion::Stage3 => 1601,
        }
    }

    pub fn dura_range(&self) -> usize {
        match self {
            LevelModifierVersion::Stage1 => self.stat_range(),
            LevelModifierVersion::Stage2 => self.stat_range(),
            LevelModifierVersion::Stage3 => 2501,
        }
    }

    pub fn boost_range(&self) -> usize {
        match self {
            LevelModifierVersion::Stage1 => self.stat_range(),
            LevelModifierVersion::Stage2 => self.stat_range(),
            LevelModifierVersion::Stage3 => 1201,
        }
    }

    pub fn has_stage2_fields(&self) -> bool {
        match self {
            LevelModifierVersion::Stage1 => false,
            LevelModifierVersion::Stage2 => true,
            LevelModifierVersion::Stage3 => false,
        }
    }
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"DPDB")]
pub struct Header {
    pub version: LevelModifierVersion,
}
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[br(import(option_size: u32))]
pub struct GroupOption {
    #[br(count = option_size)]
    pub values: Vec<f32>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelModifier {
    pub header: Header,

    #[bw(calc = speed_ids.len() as u32)]
    pub speed_length: u32,
    #[bw(calc = accel_ids.len() as u32)]
    pub accel_length: u32,
    #[bw(calc = dura_ids.len() as u32)]
    pub dura_length: u32,
    #[bw(calc = boost_ids.len() as u32)]
    pub boost_length: u32,

    #[br(count = speed_length)]
    pub speed_ids: Vec<u32>,
    #[br(count = accel_length)]
    pub accel_ids: Vec<u32>,
    #[br(count = dura_length)]
    pub dura_ids: Vec<u32>,
    #[br(count = boost_length)]
    pub boost_ids: Vec<u32>,

    #[br(args { count: header.version.stat_range(), inner: (speed_length,) })]
    pub speed: Vec<GroupOption>,
    #[br(args { count: header.version.stat_range(), inner: (accel_length,) })]
    pub accel: Vec<GroupOption>,
    #[br(args { count: header.version.dura_range(), inner: (dura_length,) })]
    pub dura: Vec<GroupOption>,
    #[br(args { count: header.version.boost_range(), inner: (boost_length,) })]
    pub boost: Vec<GroupOption>,

    #[br(if(header.version.has_stage2_fields()), args { count: 57, inner: (19,) })]
    pub stage2_unk1: Option<Vec<GroupOption>>,
    #[br(if(header.version.has_stage2_fields()))]
    pub stage2_unk2: Option<[f32; 2]>,
    #[br(if(header.version.has_stage2_fields()), args { count: 873, inner: (3,) })]
    pub stage2_unk3: Option<Vec<GroupOption>>,
}

impl LevelModifier {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}
