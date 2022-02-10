use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"LOI\0kjc\0ag\0\0")]
pub struct Header {
    #[br(assert(version_date == 20061222 || version_date == 20090403, "unexpected version {}", version_date))]
    pub version_date: u32,
}

pub type Vec3f = (f32, f32, f32);
pub type Mat3x3 = (Vec3f, Vec3f, Vec3f);

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockObject {
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: f32,
    pub unknown4: f32,
    pub object_index: u32,
    pub block_index: u32,
    pub model_table_index: u32,
    pub position: Vec3f,
    pub rotation: Mat3x3,
    pub scale: f32,
    pub unknown8: u32,
    pub unknown9: u32,
    pub object_extra_index: i32,
    pub unknown11: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub block_index: u32,
    #[bw(calc = objects.len() as u32)]
    pub object_count: u32,
    #[br(count = object_count)]
    pub objects: Vec<BlockObject>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectExtra {
    pub object_index: u32,
    pub object_extra_index: u32,
    pub unknown3: u32,
    pub position: Vec3f,
    pub rotation: Mat3x3,
    pub unknown4: (f32, f32, f32),
    pub unknown5: f32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnknownObject2 {
    #[bw(calc = items.len() as u32)]
    pub unknown_count: u32,
    #[br(count = unknown_count)]
    pub items: Vec<u32>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnknownObject3 {
    pub unknown1: u32,
    #[bw(calc = items.len() as u32)]
    pub unknown_count: u32,
    #[br(count = unknown_count)]
    pub items: Vec<u32>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnknownObject3Section {
    #[bw(calc = unknown_objects_3.len() as u32)]
    pub unknown_object_3_count: u32,
    #[br(count = unknown_object_3_count)]
    pub unknown_objects_3: Vec<UnknownObject3>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnknownObject4 {
    #[bw(calc = items.len() as u32)]
    pub unknown_count: u32,
    pub unknown1: u32,
    #[br(count = unknown_count)]
    pub items: Vec<u32>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnknownObject5 {
    #[bw(calc = object_indices.len() as u32)]
    pub object_count: u32,
    #[br(count = object_count)]
    pub object_indices: Vec<u32>,
}

#[binrw]
#[br(import(total_block_count: usize))]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Loi {
    pub header: Header,

    #[bw(calc = blocks.len() as u32)]
    pub block_count: u32,
    #[br(count = block_count)]
    pub blocks: Vec<Block>,

    #[bw(calc = object_extras.len() as u32)]
    pub object_extra_count: u32,
    #[br(count = object_extra_count)]
    pub object_extras: Vec<ObjectExtra>,

    #[br(count = total_block_count)]
    pub unknown_objects_2: Vec<UnknownObject2>,

    pub unknown_object_3_section: UnknownObject3Section,

    #[br(count = total_block_count)]
    pub unknown_objects_4: Vec<UnknownObject4>,

    #[br(count = total_block_count)]
    pub unknown_objects_5: Vec<UnknownObject5>,
}

impl Loi {
    pub fn read<R: Read + Seek>(reader: &mut R, total_block_count: usize) -> anyhow::Result<Self> {
        Ok(reader.read_le_args((total_block_count,))?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}
