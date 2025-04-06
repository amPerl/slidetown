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
    pub collider_index: i32,
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
pub struct Collider {
    pub object_index: u32,
    pub collider_index: u32,
    pub r#type: u32, // 1-2 = Box, 4 = Capsule
    pub position: Vec3f,
    pub rotation: Mat3x3,
    pub size: Vec3f,   // Box dimensions when 1-2
    pub unknown5: f32, // Capsule height/2 when 4
}

// these are something to do with animated objects or ones that produce sound.
// they are out of bounds, so it can't be collision-related. values are object ids
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
pub struct UnknownBlock3 {
    pub block_index: u32,
    #[bw(calc = items.len() as u32)]
    pub unknown_count: u32,
    #[br(count = unknown_count)]
    pub items: Vec<u32>, // no idea. always empty in mp main loi
}

// mainly lampposts, starting from MI
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LampBlock {
    #[bw(calc = lamp_ids.len() as u32)]
    pub count: u32,
    pub unknown_3_per_lamp_id: u32, // 3 * count
    #[br(count = count)]
    pub lamp_ids: Vec<u32>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TrafficLightBlock {
    #[bw(calc = traffic_light_ids.len() as u32)]
    pub count: u32,
    #[br(count = count)]
    pub traffic_light_ids: Vec<u32>,
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

    #[bw(calc = colliders.len() as u32)]
    pub collider_count: u32,
    #[br(count = collider_count)]
    pub colliders: Vec<Collider>,

    #[br(count = total_block_count)]
    pub unknown_objects_2: Vec<UnknownObject2>,

    #[bw(calc = unknown_blocks_3.len() as u32)]
    pub unknown_block_3_count: u32,
    #[br(count = unknown_block_3_count)]
    pub unknown_blocks_3: Vec<UnknownBlock3>,

    #[br(count = total_block_count)]
    pub lamp_blocks: Vec<LampBlock>,

    #[br(count = total_block_count)]
    pub traffic_light_blocks: Vec<TrafficLightBlock>,
}

impl Loi {
    pub fn read<R: Read + Seek>(reader: &mut R, total_block_count: usize) -> anyhow::Result<Self> {
        Ok(reader.read_le_args((total_block_count,))?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}
