use crate::parsers::strings;
use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinReaderExt, BinWriterExt,
};
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[brw(magic = b"LOF\0kjc\0ag\0\0")]
pub struct Header {
    #[br(assert(version_date == 20061222, "unexpected version {}", version_date))]
    pub version_date: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub index: u32,
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u32, // since 20061216
    pub unknown4: u32,
    pub unknown5: u32,

    #[br(parse_with = strings::parse_null_terminated_euc_kr_string )]
    #[bw(map = strings::string_to_null_terminated_euc_kr)]
    pub name: String,

    #[br(parse_with = strings::parse_null_terminated_euc_kr_string )]
    #[bw(map = strings::string_to_null_terminated_euc_kr)]
    pub file_name: String,

    pub unknown6: f32,
    pub unknown7: u32, // since 20061216
    pub unknown8: u32, // since 20061216
    #[serde(skip)]
    pub file_offset: u32,
    #[serde(skip)]
    pub file_length: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Lof {
    pub header: Header,
    #[bw(calc = models.len() as u32)]
    pub model_count: u32,
    pub unknown1: u32,
    #[br(count = model_count)]
    pub models: Vec<Model>,
}

impl Lof {
    pub fn read_without_data<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }

    pub fn write_without_data<W: Write + Seek>(&self, writer: &mut W) -> anyhow::Result<()> {
        Ok(writer.write_le(self)?)
    }
}
