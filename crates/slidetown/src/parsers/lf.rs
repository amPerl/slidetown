use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt,
};
use serde::{Deserialize, Serialize};

use super::{archives::record_entry_offset, EntryOffsets};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"LF\0\0kjc\0ag\0\0")]
pub struct Header {
    #[br(assert(version_date == 20061220 || version_date == 20090406, "unexpected version {}", version_date))]
    pub version_date: u32,
}

/// Terrain block grid position and archive geometry location.
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[bw(import(entry_offsets: Option<EntryOffsets>))]
pub struct Block {
    /// Grid index: `position_y * size_x + position_x`.
    pub index: u32,
    /// Cell coordinates relative to the grid origin.
    pub position_x: u32,
    pub position_y: u32,

    #[bw(args(entry_offsets), write_with = record_entry_offset)]
    #[serde(skip)]
    pub file_offset: u32,
    #[serde(skip)]
    pub file_length: u32,

    pub unknown: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[bw(import(entry_offsets: Option<EntryOffsets>))]
pub struct Lf {
    pub header: Header,

    pub unknown2: u32,
    pub block_count: u32,

    #[br(count = 13)]
    pub unknown3: Vec<u32>,

    /// Grid width and height in cells.
    pub size_x: u32,
    pub size_y: u32,
    /// Total cells: `size_x * size_y`.
    pub size_idx: u32,

    /// Cell width in world units.
    pub cell_size: f32,
    /// World-space origin at the corner of cell (0, 0).
    pub origin_x: f32,
    pub origin_y: f32,
    /// World extents from the origin.
    /// Each axis has `floor(extent / cell_size) + 1` cells.
    pub extent_x: f32,
    pub extent_y: f32,

    #[br(count = block_count)]
    #[bw(args(entry_offsets))]
    pub blocks: Vec<Block>,
}

impl Lf {
    /// Return the block at world coordinates, or `None` outside the grid.
    /// All layers share this grid; use it to reassign moved objects to blocks.
    pub fn block_at(&self, x: f32, y: f32) -> Option<u32> {
        let across = ((x - self.origin_x) / self.cell_size).floor();
        let down = ((y - self.origin_y) / self.cell_size).floor();
        if across < 0.0 || down < 0.0 {
            return None;
        }
        let (across, down) = (across as u32, down as u32);
        if across >= self.size_x || down >= self.size_y {
            return None;
        }
        Some(down * self.size_x + across)
    }

    pub fn read_without_data<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write_without_data<W: Write + Seek>(
        &self,
        writer: &mut W,
        entry_offsets: EntryOffsets,
    ) -> anyhow::Result<()> {
        Ok(writer.write_le_args(self, (Some(entry_offsets),))?)
    }
}
