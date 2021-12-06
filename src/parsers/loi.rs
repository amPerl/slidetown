use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
#[br(magic = b"LOI\0kjc\0")]
pub struct Header {
    pub unknown1: u32,
    pub version_date: u32,
    pub block_count: u32,
}

pub type Vec3f = (f32, f32, f32);
pub type Mat3x3 = (Vec3f, Vec3f, Vec3f);

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Block {
    pub block_index: u32,
    pub object_count: u32,
    #[br(count=object_count)]
    pub objects: Vec<BlockObject>,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct ObjectExtra {
    pub object_index: u32,
    pub object_extra_index: u32,
    pub unknown3: u32,
    pub position: Vec3f,
    pub rotation: Mat3x3,
    pub unknown4: (f32, f32, f32),
    pub unknown5: f32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct UnknownObject2 {
    pub unknown_count: u32,
    #[br(count=unknown_count)]
    pub items: Vec<u32>,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct UnknownObject3 {
    pub unknown1: u32,
    pub unknown_count: u32,
    #[br(count=unknown_count)]
    pub items: Vec<u32>,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct UnknownObject3Section {
    pub unknown_object_3_count: u32,
    #[br(count=unknown_object_3_count)]
    pub unknown_objects_3: Vec<UnknownObject3>,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct UnknownObject4 {
    pub unknown_count: u32,
    pub unknown1: u32,
    #[br(count=unknown_count)]
    pub items: Vec<u32>,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct UnknownObject5 {
    pub object_count: u32,
    #[br(count=object_count)]
    pub object_indices: Vec<u32>,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Loi {
    pub header: Header,
    #[br(count=header.block_count)]
    pub blocks: Vec<Block>,

    pub object_extra_count: u32,
    #[br(count=object_extra_count)]
    pub object_extras: Vec<ObjectExtra>,

    #[br(count=header.block_count)]
    pub unknown_objects_2: Vec<UnknownObject2>,

    #[br(if(header.version_date >= 20061222))]
    pub unknown_object_3_section: Option<UnknownObject3Section>,

    #[br(count=header.block_count)]
    pub unknown_objects_4: Vec<UnknownObject4>,

    #[br(count=header.block_count)]
    pub unknown_objects_5: Vec<UnknownObject5>,
}

impl Header {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

impl BlockObject {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

impl Block {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

impl Loi {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}
