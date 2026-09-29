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

    #[br(parse_with = read_collection, args(0x90000, 0x90010, 0x90020, ()))]
    #[bw(write_with = write_collection, args(0x90000, 0x90010, 0x90020, ()))]
    pub time_keys: Collection<TimeKeys>,

    #[br(parse_with = read_collection, args(0xA0000, 0xA0010, 0xA0030, ()))]
    #[bw(write_with = write_collection, args(0xA0000, 0xA0010, 0xA0030, ()))]
    pub signals: Collection<Signal>,

    #[br(parse_with = read_counted_chunk, args(0xA0040, ()))]
    #[bw(write_with = write_counted_chunk, args(0xA0040, ()))]
    pub controlled_signal_ids: Vec<i32>,

    #[br(parse_with = read_paths)]
    #[bw(write_with = write_paths)]
    pub paths: Paths,

    #[br(parse_with = read_collection, args(0xA5000, 0xA5010, 0xA5030, ()))]
    #[bw(write_with = write_collection, args(0xA5000, 0xA5010, 0xA5030, ()))]
    pub cross_signals: Collection<CrossSignal>,

    #[br(parse_with = read_chunk, args(0xA6000, ()))]
    #[bw(write_with = write_chunk, args(0xA6000, ()))]
    pub four_light_signals: FourLightSignals,

    #[br(parse_with = read_collection, args(0x20000, 0x20100, 0x20200, ()))]
    #[bw(write_with = write_collection, args(0x20000, 0x20100, 0x20200, ()))]
    pub roads: Collection<Road>,

    #[br(parse_with = read_collection, args(0x50000, 0x50100, 0x50200, ()))]
    #[bw(write_with = write_collection, args(0x50000, 0x50100, 0x50200, ()))]
    pub cross_roads: Collection<CrossRoad>,

    #[br(parse_with = read_collection, args(0x75000, 0x75100, 0x75200, ()))]
    #[bw(write_with = write_collection, args(0x75000, 0x75100, 0x75200, ()))]
    pub joints: Collection<Joint>,

    #[br(parse_with = read_collection, args(0x10000, 0x11000, 0x12200, ()))]
    #[bw(write_with = write_collection, args(0x10000, 0x11000, 0x12200, ()))]
    pub crosses: Collection<Cross>,
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

/// how a car is posed along a path over time, one per path
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeKeys {
    pub id: i32,
    #[bw(calc = keys.len() as i32)]
    pub key_count: i32,
    #[br(count = key_count)]
    pub keys: Vec<TimeKey>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeKey {
    pub time: f32,
    pub position: [f32; 3],
    pub right: [f32; 3],
    pub direction: [f32; 3],
    pub normal: [f32; 3],
    pub rotation: [f32; 4],
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    pub area_id: i32,
    pub id: i32,
    pub pos_time: f32,
    pub dist: f32,
    /// 0 stop, 1 intermediate, 2 go, 3 left max
    pub default_state: i32,
    pub path_id: i32,
}

/// an intersection's signal controller, which turns one road's signals green at a time
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CrossSignal {
    pub area_id: i32,
    pub id: i32,
    #[bw(calc = roads.len() as i32)]
    pub road_count: i32,
    #[br(count = road_count)]
    pub roads: Vec<CrossSignalRoad>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CrossSignalRoad {
    #[bw(calc = signal_ids.len() as i32)]
    pub signal_count: i32,
    #[br(count = signal_count)]
    pub signal_ids: Vec<i32>,
}

/// the client's 4-light signals
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FourLightSignals {
    #[br(parse_with = read_counted_chunk, args(0xA6010, ()))]
    #[bw(write_with = write_counted_chunk, args(0xA6010, ()))]
    pub controls: Vec<FourLightControl>,
    #[br(parse_with = read_counted_chunk, args(0xA6030, ()))]
    #[bw(write_with = write_counted_chunk, args(0xA6030, ()))]
    pub signals: Vec<FourLightSignal>,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FourLightControl {
    pub id: i32,
    pub signal_id: i32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FourLightSignal {
    pub id: i32,
    /// 0 stop, 1 intermediate, 2 go, 3 left max
    pub state: i32,
    /// index into the controls
    pub control: i32,
    pub signal_id: i32,
}

/// the lanes cars drive: every path's record, then every path's points
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Paths {
    pub count: i32,
    pub paths: Vec<Path>,
    pub points: Vec<PathPoints>,
}

/// one lane
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Path {
    pub area_id: i32,
    pub id: i32,
    /// 0 forward, 1 right, 2 left, 3 left forward, 4 right forward
    pub path_type: i32,
    pub start_pos_time: f32,
    pub end_pos_time: f32,
    pub max_speed: f32,
    pub stop_r_dist: f32,
    pub cruise_dist: f32,
    pub total_dist: f32,
    pub time_keys_id: i32,
    pub dec: i32,
    pub inc: i32,
    /// 0 path, 1 joint, 2 unknown
    pub before_type: i32,
    pub before_id: i32,
    /// 0 path, 1 joint, 2 unknown
    pub next_type: i32,
    pub next_id: i32,

    #[bw(calc = signal_ids.len() as i32)]
    pub signal_count: i32,
    #[br(count = signal_count)]
    pub signal_ids: Vec<i32>,
}

/// the points of the path with the same id
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PathPoints {
    pub area_id: i32,
    pub id: i32,
    pub delta: f32,
    #[bw(calc = points.len() as i32)]
    pub point_count: i32,
    #[br(count = point_count)]
    pub points: Vec<(f32, f32, f32)>,
}

/// a road, with the graph path it belongs to
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Road {
    pub base: RoadBase,
    pub nd_node: i32,
    pub nd_path: i32,
}

/// the lanes each way along a road, and its arcs in the graph
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RoadBase {
    pub area_id: i32,
    pub id: i32,
    pub lane_count: i32,

    #[bw(calc = right_lane_ids.len() as i32)]
    pub right_count: i32,
    #[bw(calc = left_lane_ids.len() as i32)]
    pub left_count: i32,
    #[br(count = right_count)]
    pub right_lane_ids: Vec<i32>,
    #[br(count = left_count)]
    pub left_lane_ids: Vec<i32>,

    pub node0: i32,
    pub arc0: i32,
    pub node1: i32,
    pub arc1: i32,
}

/// a road through an intersection
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CrossRoad {
    pub base: RoadBase,
    pub cross_road_id: i32,
    pub node2: i32,
    pub arc2: i32,
    pub node3: i32,
    pub arc3: i32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Cross {
    pub area_id: i32,
    pub id: i32,
    pub nd_node: i32,

    #[bw(calc = cross_road_ids.len() as i32)]
    pub cross_road_count: i32,
    #[br(count = cross_road_count)]
    pub cross_road_ids: Vec<i32>,

    #[bw(calc = path_ids.len() as i32)]
    pub path_count: i32,
    /// the paths turning through it
    #[br(count = path_count)]
    pub path_ids: Vec<i32>,

    /// -1 when unsignalled
    pub cross_signal_id: i32,
}

/// where lanes run into each other
#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Joint {
    pub area_id: i32,
    pub id: i32,
    /// 0 in, 1 out, 2 none
    pub joint_type: i32,

    #[bw(calc = in_path_ids.len() as i32)]
    pub in_count: i32,
    #[bw(calc = out_path_ids.len() as i32)]
    pub out_count: i32,
    #[br(count = in_count)]
    pub in_path_ids: Vec<i32>,
    #[br(count = out_count)]
    pub out_path_ids: Vec<i32>,

    pub nd_node: i32,
    pub arc: i32,
}

/// a container's count chunk and the chunks after it, which fill the container. the count isn't
/// always how many there are: cras says 128 roads and has 126
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Collection<T> {
    pub count: i32,
    pub items: Vec<T>,
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

/// reads `{u32 id, u32 length}`, checking the id, and says where the chunk started and how long
/// it is
fn read_head<R: Read + Seek>(reader: &mut R, endian: Endian, id: u32) -> BinResult<(u64, u32)> {
    let at = reader.stream_position()?;
    let found = u32::read_options(reader, endian, ())?;
    if found != id {
        return Err(binrw::Error::AssertFail {
            pos: at,
            message: format!("expected chunk {:#x}, found {:#x}", id, found),
        });
    }
    Ok((at, u32::read_options(reader, endian, ())?))
}

/// checks what was read of a chunk filled it exactly
fn read_all<R: Read + Seek>(reader: &mut R, id: u32, (at, length): (u64, u32)) -> BinResult<()> {
    let read = reader.stream_position()? - at;
    if read != length as u64 {
        return Err(binrw::Error::AssertFail {
            pos: at,
            message: format!("chunk {:#x} is {} bytes, read {}", id, length, read),
        });
    }
    Ok(())
}

/// writes a head with no length yet, and says where it is
fn write_head<W: Write + Seek>(writer: &mut W, endian: Endian, id: u32) -> BinResult<u64> {
    let at = writer.stream_position()?;
    id.write_options(writer, endian, ())?;
    0u32.write_options(writer, endian, ())?;
    Ok(at)
}

/// fills in the length of the chunk whose head is at `at`
fn write_length<W: Write + Seek>(writer: &mut W, endian: Endian, at: u64) -> BinResult<()> {
    let end = writer.stream_position()?;
    writer.seek(SeekFrom::Start(at + 4))?;
    ((end - at) as u32).write_options(writer, endian, ())?;
    writer.seek(SeekFrom::Start(end))?;
    Ok(())
}

/// reads a chunk of `T` that fills it exactly
fn read_chunk<'a, T, R>(
    reader: &mut R,
    endian: Endian,
    (id, args): (u32, T::Args<'a>),
) -> BinResult<T>
where
    T: BinRead,
    R: Read + Seek,
{
    let head = read_head(reader, endian, id)?;
    let value = T::read_options(reader, endian, args)?;
    read_all(reader, id, head)?;
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

/// reads a container holding a count chunk and chunks of `T` up to its end
fn read_collection<'a, T, R>(
    reader: &mut R,
    endian: Endian,
    (id, count_id, item_id, args): (u32, u32, u32, T::Args<'a>),
) -> BinResult<Collection<T>>
where
    T: BinRead,
    T::Args<'a>: Clone,
    R: Read + Seek,
{
    let head = read_head(reader, endian, id)?;
    let count = read_chunk(reader, endian, (count_id, ()))?;
    let end = head.0 + head.1 as u64;
    let mut items = Vec::new();
    while reader.stream_position()? < end {
        items.push(read_chunk(reader, endian, (item_id, args.clone()))?);
    }
    read_all(reader, id, head)?;
    Ok(Collection { count, items })
}

/// reads a chunk holding a count and that many `T`
fn read_counted_chunk<'a, T, R>(
    reader: &mut R,
    endian: Endian,
    (id, args): (u32, T::Args<'a>),
) -> BinResult<Vec<T>>
where
    T: BinRead,
    T::Args<'a>: Clone,
    R: Read + Seek,
{
    let head = read_head(reader, endian, id)?;
    let count = i32::read_options(reader, endian, ())?;
    let values = (0..count)
        .map(|_| T::read_options(reader, endian, args.clone()))
        .collect::<BinResult<_>>()?;
    read_all(reader, id, head)?;
    Ok(values)
}

/// the id of the chunk coming up, leaving the reader where it was
fn peek_id<R: Read + Seek>(reader: &mut R, endian: Endian) -> BinResult<u32> {
    let at = reader.stream_position()?;
    let id = u32::read_options(reader, endian, ())?;
    reader.seek(SeekFrom::Start(at))?;
    Ok(id)
}

#[binrw::parser(reader, endian)]
fn read_paths() -> BinResult<Paths> {
    let head = read_head(reader, endian, 0x70000)?;
    let count = read_chunk(reader, endian, (0x70100, ()))?;
    let end = head.0 + head.1 as u64;
    let mut paths = Vec::new();
    while reader.stream_position()? < end && peek_id(reader, endian)? == 0x70200 {
        paths.push(read_chunk(reader, endian, (0x70200, ()))?);
    }
    let mut points = Vec::new();
    while reader.stream_position()? < end {
        points.push(read_chunk(reader, endian, (0x70300, ()))?);
    }
    read_all(reader, 0x70000, head)?;
    Ok(Paths {
        count,
        paths,
        points,
    })
}

#[binrw::writer(writer, endian)]
fn write_paths(paths: &Paths) -> BinResult<()> {
    let at = write_head(writer, endian, 0x70000)?;
    write_chunk(&paths.count, writer, endian, (0x70100, ()))?;
    write_chunks(&paths.paths, writer, endian, (0x70200, ()))?;
    write_chunks(&paths.points, writer, endian, (0x70300, ()))?;
    write_length(writer, endian, at)
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
    let at = write_head(writer, endian, id)?;
    value.write_options(writer, endian, args)?;
    write_length(writer, endian, at)
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

// binrw passes the field as &Vec
#[allow(clippy::ptr_arg)]
fn write_counted_chunk<'a, T, W>(
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
    let at = write_head(writer, endian, id)?;
    (values.len() as i32).write_options(writer, endian, ())?;
    values
        .iter()
        .try_for_each(|value| value.write_options(writer, endian, args.clone()))?;
    write_length(writer, endian, at)
}

fn write_collection<'a, T, W>(
    collection: &Collection<T>,
    writer: &mut W,
    endian: Endian,
    (id, count_id, item_id, args): (u32, u32, u32, T::Args<'a>),
) -> BinResult<()>
where
    T: BinWrite,
    T::Args<'a>: Clone,
    W: Write + Seek,
{
    let at = write_head(writer, endian, id)?;
    write_chunk(&collection.count, writer, endian, (count_id, ()))?;
    write_chunks(&collection.items, writer, endian, (item_id, args))?;
    write_length(writer, endian, at)
}
