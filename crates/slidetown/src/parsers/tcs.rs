use binrw::{
    binrw,
    io::{Read, Seek, SeekFrom, Write},
    BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, Endian, NullString,
};
use serde::{Deserialize, Serialize};

/// adds node types, multi-road paths and a start node per road
const VERSION_WITH_NODE_TYPES: u32 = 20061102;
/// road widths replaced by a hit width and walkways on both sides
const VERSION_WITH_WALKWAYS: u32 = 20080327;

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"TCS\0")]
pub struct Header {
    /// `B` for binary, text files aren't supported
    #[br(assert(mode == 0x42, "unexpected mode {:#x}", mode))]
    pub mode: u16,
    pub version_date: u32,

    #[br(assert(description == "TRAFFIC-CONTROL-SYSTEM"))]
    #[br(map = |x: NullString| x.to_string())]
    #[bw(map = |x: &String| NullString::from(x.clone()))]
    pub description: String,

    #[br(assert(nhn == "NHN-AG"))]
    #[br(map = |x: NullString| x.to_string())]
    #[bw(map = |x: &String| NullString::from(x.clone()))]
    pub nhn: String,

    #[br(assert(jc == "JC"))]
    #[br(map = |x: NullString| x.to_string())]
    #[bw(map = |x: &String| NullString::from(x.clone()))]
    pub jc: String,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Tcs {
    pub header: Header,

    #[br(parse_with = read_chunk, args(0x5, (header.version_date,)))]
    #[bw(write_with = write_chunk, args(0x5, ()))]
    pub main: Main,
}

impl Tcs {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[br(import(version: u32))]
pub struct Main {
    #[br(parse_with = read_chunk, args(0x1000, (version,)))]
    #[bw(write_with = write_chunk, args(0x1000, ()))]
    pub graph: Graph,

    pub time_keys: RawChunk,
    pub signals: RawChunk,
    pub signal_control: RawChunk,
    pub paths: RawChunk,
    pub cross_signals: RawChunk,
    pub signal4cls: RawChunk,
    pub roads: RawChunk,
    pub cross_roads: RawChunk,
    pub joints: RawChunk,
    pub crosses: RawChunk,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[br(import(version: u32))]
pub struct Graph {
    #[br(parse_with = read_chunk, args(0x1100, ()))]
    #[bw(write_with = write_chunk, args(0x1100, ()))]
    pub info: i32,

    #[br(temp, assert(node_count.id == 0x2000, "expected chunk 0x2000, found {:#x}", node_count.id))]
    #[bw(calc = Count::of(0x2000, nodes.len()))]
    node_count: Count,
    #[br(parse_with = read_chunks, args(0x2100, node_count.value as usize, (version,)))]
    #[bw(write_with = write_chunks, args(0x2100, ()))]
    pub nodes: Vec<NdNode>,

    #[br(temp, assert(path_count.id == 0x3000, "expected chunk 0x3000, found {:#x}", path_count.id))]
    #[bw(calc = Count::of(0x3000, paths.len()))]
    path_count: Count,
    #[br(parse_with = read_chunks, args(0x3100, path_count.value as usize, (version,)))]
    #[bw(write_with = write_chunks, args(0x3100, ()))]
    pub paths: Vec<NdPath>,

    #[br(temp, assert(arc_count.id == 0x4000, "expected chunk 0x4000, found {:#x}", arc_count.id))]
    #[bw(calc = Count::of(0x4000, arcs.len()))]
    arc_count: Count,
    #[br(parse_with = read_chunks, args(0x4100, arc_count.value as usize, ()))]
    #[bw(write_with = write_chunks, args(0x4100, ()))]
    pub arcs: Vec<NdArc>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[br(import(version: u32))]
pub struct NdNode {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub path_count: i32,
    pub arc_count: i32,
    #[br(if(version >= VERSION_WITH_NODE_TYPES))]
    pub node_type: Option<i32>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NdArc {
    pub from: i32,
    pub to: i32,
    pub path_id: i32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[br(import(version: u32))]
pub struct NdPath {
    pub from: i32,
    /// 0 single, 1 multi, always single in older files
    #[br(if(version >= VERSION_WITH_NODE_TYPES))]
    pub path_type: Option<i32>,

    #[br(temp, if(path_type == Some(1)))]
    #[bw(calc = (*path_type == Some(1)).then_some(roads.len() as i32))]
    road_count: Option<i32>,
    #[br(count = road_count.unwrap_or(1), args { inner: (version,) })]
    pub roads: Vec<NdRoad>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[br(import(version: u32))]
pub struct NdRoad {
    pub path_id: i32,
    /// in older files, the path's `from`
    #[br(if(version >= VERSION_WITH_NODE_TYPES))]
    pub from: Option<i32>,
    pub to: i32,
    pub path_dist: f32,
    pub weight: f32,
    pub road_type: i32,
    pub min_speed: f32,
    pub max_speed: f32,
    #[br(if(version < VERSION_WITH_NODE_TYPES))]
    pub data_type: Option<i32>,

    #[br(if(version < VERSION_WITH_WALKWAYS))]
    pub widths: Option<Widths>,
    #[br(if(version >= VERSION_WITH_WALKWAYS))]
    pub walks: Option<Walks>,

    pub lane_width: f32,
    pub middle_width: f32,
    pub right_count: i32,
    pub left_count: i32,
    pub one_way: i32,
    pub start: f32,
    pub end: f32,

    #[bw(calc = points.len() as i32)]
    pub point_count: i32,
    #[br(count = point_count)]
    pub points: Vec<NdPoint>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Widths {
    pub road: f32,
    pub side: f32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Walks {
    pub hit_width: f32,
    pub left_side_way: f32,
    pub left_walk_width: f32,
    pub left_walk_height: f32,
    pub right_side_way: f32,
    pub right_walk_width: f32,
    pub right_walk_height: f32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NdPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub right_dist: f32,
    pub left_dist: f32,
}

/// count of the chunks that follow
#[binrw]
#[derive(Debug, PartialEq)]
struct Count {
    id: u32,
    #[br(temp, assert(length == 12, "count chunk {:#x} is {} bytes", id, length))]
    #[bw(calc = 12)]
    length: u32,
    value: i32,
}

impl Count {
    fn of(id: u32, count: usize) -> Count {
        Count {
            id,
            value: count as i32,
        }
    }
}

/// a chunk we don't parse yet
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RawChunk {
    pub id: u32,
    #[bw(calc = data.len() as u32 + 8)]
    pub length: u32,
    #[br(count = length.saturating_sub(8))]
    pub data: Vec<u8>,
}

/// reads `{u32 id, u32 length}` and a `T` that fills the chunk exactly
fn read_chunk<'a, T, R>(
    reader: &mut R,
    endian: Endian,
    (id, args): (u32, T::Args<'a>),
) -> BinResult<T>
where
    T: BinRead,
    R: Read + Seek,
{
    let at = reader.stream_position()?;
    let found = u32::read_options(reader, endian, ())?;
    if found != id {
        return Err(binrw::Error::AssertFail {
            pos: at,
            message: format!("expected chunk {:#x}, found {:#x}", id, found),
        });
    }
    let length = u32::read_options(reader, endian, ())?;
    let value = T::read_options(reader, endian, args)?;
    let read = reader.stream_position()? - at;
    if read != length as u64 {
        return Err(binrw::Error::AssertFail {
            pos: at,
            message: format!("chunk {:#x} is {} bytes, read {}", id, length, read),
        });
    }
    Ok(value)
}

fn read_chunks<'a, T, R>(
    reader: &mut R,
    endian: Endian,
    (id, count, args): (u32, usize, T::Args<'a>),
) -> BinResult<Vec<T>>
where
    T: BinRead,
    T::Args<'a>: Clone,
    R: Read + Seek,
{
    (0..count)
        .map(|_| read_chunk(reader, endian, (id, args.clone())))
        .collect()
}

/// writes a chunk of `T`, filling in the length afterwards
fn write_chunk<'a, T, W>(
    value: &T,
    writer: &mut W,
    endian: Endian,
    (id, args): (u32, T::Args<'a>),
) -> BinResult<()>
where
    T: BinWrite,
    W: Write + Seek,
{
    let at = writer.stream_position()?;
    id.write_options(writer, endian, ())?;
    0u32.write_options(writer, endian, ())?;
    value.write_options(writer, endian, args)?;
    let end = writer.stream_position()?;
    writer.seek(SeekFrom::Start(at + 4))?;
    ((end - at) as u32).write_options(writer, endian, ())?;
    writer.seek(SeekFrom::Start(end))?;
    Ok(())
}

// binrw passes the field as &Vec
#[allow(clippy::ptr_arg)]
fn write_chunks<'a, T, W>(
    values: &Vec<T>,
    writer: &mut W,
    endian: Endian,
    (id, args): (u32, T::Args<'a>),
) -> BinResult<()>
where
    T: BinWrite,
    T::Args<'a>: Clone,
    W: Write + Seek,
{
    values
        .iter()
        .try_for_each(|value| write_chunk(value, writer, endian, (id, args.clone())))
}
