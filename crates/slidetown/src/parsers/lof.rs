use crate::parsers::strings;
use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt,
};
use serde::{Deserialize, Serialize};

use super::archives::{record_entry_offset, EntryOffsets};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"LOF\0kjc\0ag\0\0")]
pub struct Header {
    #[br(assert(version_date == 20061222, "unexpected version {}", version_date))]
    pub version_date: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[bw(import(entry_offsets: Option<EntryOffsets>))]
pub struct Model {
    pub index: u32,
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u32, // since 20061216, almost inverse of column N
    pub lighting: u32, // turns on light at night?
    pub effect_id: u32,

    #[br(parse_with = strings::parse_null_terminated_euc_kr_string )]
    #[bw(map = strings::string_to_null_terminated_euc_kr)]
    pub name: String,

    #[br(parse_with = strings::parse_null_terminated_euc_kr_string )]
    #[bw(map = strings::string_to_null_terminated_euc_kr)]
    pub file_name: String,

    pub animation_duration: f32,
    pub r#loop: u32,        // since 20061216
    pub random_offset: u32, // since 20061216
    #[bw(args(entry_offsets), write_with = record_entry_offset)]
    #[serde(skip)]
    pub file_offset: u32,
    #[serde(skip)]
    pub file_length: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[bw(import(entry_offsets: Option<EntryOffsets>))]
pub struct Lof {
    pub header: Header,
    #[bw(calc = models.len() as u32)]
    pub model_count: u32,
    pub max_file_size: u32,
    #[br(count = model_count)]
    #[bw(args(entry_offsets))]
    pub models: Vec<Model>,
}

impl Lof {
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
