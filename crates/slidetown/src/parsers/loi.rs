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

/// A model instance placed in the world.
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockObject {
    /// Always zero.
    pub unknown1: u32,
    /// Always one.
    pub unknown2: u32,
    pub unknown3: f32,
    pub unknown4: f32,
    /// ID used by colliders and the three trailing lists.
    /// Sparse, unordered, and unique only within this file; other placement sets may reuse it.
    pub object_index: u32,
    /// Terrain block containing this object; matches the enclosing block.
    pub block_index: u32,
    pub model_table_index: u32,
    pub position: Vec3f,
    pub rotation: Mat3x3,
    /// Uniform scale for all three axes.
    pub scale: f32,
    pub unknown8: u32,
    /// Zero or one.
    pub unknown9: u32,
    /// Collider group ID, or -1 for no reference. Unreferenced colliders may still exist.
    pub collider_index: i32,
    /// Seconds added to the clock before the model's animation is sampled. Zero on most copies.
    pub anim_time_offset: f32,
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

/// A collision shape in world coordinates.
/// An object's shapes share a `collider_index`, which identifies the group, not the row.
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Collider {
    /// Owning object's `object_index`.
    pub object_index: u32,
    /// Group ID shared by all of this object's colliders.
    pub collider_index: u32,
    pub r#type: u32, // 1-2 = Box, 4 = Capsule
    pub position: Vec3f,
    pub rotation: Mat3x3,
    pub size: Vec3f,   // Box dimensions when 1-2
    pub unknown5: f32, // Capsule height/2 when 4
}

/// Animated or sound-producing `object_index` values, grouped by terrain block.
/// One of three lists with a row per block. IDs may reference objects outside this file,
/// such as a track referencing its underlying world.
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

/// Lamps in a terrain block and their total model glow-plane count (not light count).
/// Plane counts vary by model, so `3 * count` fails in mixed-model blocks:
/// 50 in koinonia, 6 in oros, 2 in cras, and 36 in taipei; none in moonpalace.
/// Summing model planes matches every moonpalace, koinonia, cras, and oros block,
/// and 7,776 of taipei's 7,800 blocks.
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LampBlock {
    #[bw(calc = lamp_ids.len() as u32)]
    pub count: u32,
    pub glow_planes: u32,
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

    /// Lamp IDs grouped by terrain block.
    #[br(count = total_block_count)]
    pub lamp_blocks: Vec<LampBlock>,

    /// Traffic light IDs grouped by terrain block.
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
