use binrw::{binread, binrw, helpers::until_eof, NullString};

#[binread]
#[derive(Debug, Clone, Copy)]
pub enum NtxVersion {
    New {
        #[br(temp)]
        padding: [u8; 72],
        #[br(temp, magic = b"DDS ")]
        dds: (),
    },
    Old {
        #[br(temp)]
        padding: [u8; 20],
        #[br(temp, magic = b"DDS ")]
        dds: (),
    },
}

impl NtxVersion {
    pub fn path_length(&self) -> usize {
        match self {
            NtxVersion::New {} => 64,
            NtxVersion::Old {} => 16,
        }
    }
    pub fn size_length(&self) -> usize {
        match self {
            NtxVersion::New {} => 8,
            NtxVersion::Old {} => 4,
        }
    }
}

#[binrw]
#[derive(Debug, Clone)]
pub struct Ntx {
    #[br(restore_position)]
    #[bw(ignore)]
    version: NtxVersion,
    #[br(parse_with = until_eof)]
    #[brw(args(version.clone(),))]
    entries: Vec<NtxEntry>,
}

#[binrw]
#[derive(Clone)]
#[brw(import(version: NtxVersion))]
pub struct NtxEntry {
    #[br(count = version.path_length(), map = |b: Vec<u8>| {
        let s = NullString(b).to_string();
        s.split('\0').next().unwrap().to_string()
    })]
    #[bw(map = |s| {
        let mut result = s.clone().into_bytes();
        result.resize(version.path_length(), 0);
        result
    })]
    pub path: String,
    #[brw(pad_size_to = version.size_length())]
    #[br(temp)]
    #[bw(calc = data.len() as _)]
    pub size: u32,
    #[br(count = size)]
    pub data: Vec<u8>,
}

impl std::fmt::Debug for NtxEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NtxEntry")
            .field(&self.path)
            .field(&self.data.len())
            .finish()
    }
}
