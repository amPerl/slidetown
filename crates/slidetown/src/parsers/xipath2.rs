use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt, NullString,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"XiPATH2\0")]
pub struct Header {
    #[br(assert(version_date == 20061013, "unexpected version {}", version_date))]
    pub version_date: u32,

    #[br(assert(nhn == "NHN-AG"))]
    #[br(map = |x: NullString| x.to_string())]
    #[bw(map = |x: &String| NullString::from(x.clone()))]
    pub nhn: String,

    #[br(assert(jc == "jc"))]
    #[br(map = |x: NullString| x.to_string())]
    #[bw(map = |x: &String| NullString::from(x.clone()))]
    pub jc: String,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Xipath2 {
    pub header: Header,

    #[bw(calc = 5)]
    #[br(temp, assert(main_id == 5, "unexpected chunk {:#x}", main_id))]
    main_id: u32,
    #[bw(calc = (Self::SIZE_MAIN + courses.iter().map(Course::size_bytes).sum::<usize>()) as u32)]
    #[br(temp)]
    main_length: u32,

    #[bw(calc = 0x1000)]
    #[br(temp, assert(count_id == 0x1000, "unexpected chunk {:#x}", count_id))]
    count_id: u32,
    #[bw(calc = 12)]
    #[br(temp)]
    count_length: u32,
    #[bw(calc = courses.len() as u32)]
    pub course_count: u32,

    #[br(count = course_count)]
    pub courses: Vec<Course>,
}

impl Xipath2 {
    /// The main chunk's own header and the count chunk.
    const SIZE_MAIN: usize = 8 + 12;

    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = 0x1100_u32)]
pub struct Course {
    #[bw(calc = self.size_bytes() as u32)]
    #[br(temp)]
    length: u32,

    #[bw(calc = points.len() as u32)]
    pub point_count: u32,
    #[br(count = point_count)]
    pub points: Vec<(f32, f32, f32)>,
}

impl Course {
    const SIZE_POINT: usize = 12;
    const SIZE_REST: usize = 12;

    pub fn size_bytes(&self) -> usize {
        Self::SIZE_REST + Self::SIZE_POINT * self.points.len()
    }
}
