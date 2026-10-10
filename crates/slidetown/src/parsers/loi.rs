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
    /// Always 1 on disk.
    pub visible: u32,
    /// Added to `base_height`. Almost always: `base_height + height_offset = position.z`.
    pub height_offset: f32,
    pub base_height: f32,
    /// ID used by colliders and the trailing lists. Unique within this file; order and gaps
    /// don't matter, and other placement sets may reuse it.
    pub object_index: u32,
    /// Terrain block containing this object; matches the enclosing block.
    pub block_index: u32,
    pub model_table_index: u32,
    pub position: Vec3f,
    pub rotation: Mat3x3,
    /// Uniform scale for all three axes.
    pub scale: f32,
    /// Traffic lights only: the light's row in the client signal table, plus 10.
    pub signal_id: u32,
    /// Traffic lights only, 0 or 1.
    pub signal_phase: u32,
    /// Position of this object's first row in `colliders`, or -1 for none.
    pub collider_index: i32,
    /// Seconds added to the clock before the model's animation is sampled. Zero on most copies.
    pub anim_time_offset: f32,
}

/// The objects in one terrain chunk. Only chunks with objects are listed, and each must also be
/// in the set's `terrain0.lif`.
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub block_index: u32,
    #[bw(calc = objects.len() as u32)]
    pub object_count: u32,
    #[br(count = object_count)]
    pub objects: Vec<BlockObject>,
}

/// A collision shape in world coordinates. An object's rows are contiguous.
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Collider {
    /// Owning object's `object_index`.
    pub object_index: u32,
    /// Position of the owner's first row in `colliders`; the same on every row of an owner.
    pub collider_index: u32,
    /// 1-2 = box, 3 = sphere, 4 = capsule, 5 = the model's own col_wall/col_floor meshes.
    pub r#type: u32,
    pub position: Vec3f,
    pub rotation: Mat3x3,
    /// Box dimensions when 1-2.
    pub size: Vec3f,
    /// Capsule half-height, sphere radius. Set on boxes too, but unused there.
    pub half_height: f32,
}

/// Objects animated on the chunk clock, one list per terrain chunk. Ids may be in other chunks,
/// and one that doesn't resolve is skipped.
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AniObjectList {
    #[bw(calc = object_ids.len() as u32)]
    pub count: u32,
    #[br(count = count)]
    pub object_ids: Vec<u32>,
}

/// One per chunk of the set: objects animated every frame without the per-object camera check,
/// for models without time controllers. Unlike `AniObjectList`, every id must resolve.
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AniObjectBlock {
    pub block_index: u32,
    #[bw(calc = object_ids.len() as u32)]
    pub count: u32,
    #[br(count = count)]
    pub object_ids: Vec<u32>,
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

/// A placement set. A track's file is complete on its own, not added on top of main's.
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

    /// One per terrain chunk of the city, including chunks without objects.
    #[br(count = total_block_count)]
    pub ani_object_lists: Vec<AniObjectList>,

    #[bw(calc = ani_object_blocks.len() as u32)]
    pub ani_object_block_count: u32,
    #[br(count = ani_object_block_count)]
    pub ani_object_blocks: Vec<AniObjectBlock>,

    /// Lamp IDs grouped by terrain block, one per chunk of the city.
    #[br(count = total_block_count)]
    pub lamp_blocks: Vec<LampBlock>,

    /// Traffic light IDs grouped by terrain block, one per chunk of the city.
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
