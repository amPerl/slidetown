use crate::parsers::strings;
use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
#[br(magic = b"LOF\0kjc\0")]
pub struct Header {
    pub unknown1: u32,
    pub version_date: u32,
    pub model_count: u32,
    pub unknown2: u32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Model {
    pub index: u32,
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u32, // only in newer versions
    pub unknown4: u32,
    pub unknown5: u32,

    #[br(parse_with = strings::parse_null_terminated_euc_kr_string )]
    pub name: String,

    #[br(parse_with = strings::parse_null_terminated_euc_kr_string )]
    pub file_name: String,

    pub unknown6: f32,
    pub unknown7: u32, // only in newer versions
    pub unknown8: u32, // only in newer versions
    #[serde(skip)]
    pub file_offset: u32,
    #[serde(skip)]
    pub file_length: u32,
}

#[derive(Debug, PartialEq, BinRead, Serialize, Deserialize)]
pub struct Lof {
    pub header: Header,
    #[br(count = header.model_count)]
    pub models: Vec<Model>,
}

impl Header {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

impl Model {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}

impl Lof {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}
