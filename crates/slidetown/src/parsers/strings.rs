#![allow(dead_code)]

use encoding_rs::EUC_KR;

use binrw::io::{Seek, SeekFrom};
use binrw::{io::Read, BinRead, BinResult, BinWrite};

/// Read size when searching for a terminator.
const CHUNK: usize = 64;

/// Read bytes before `end`, leaving the reader after it.
/// Read in blocks to avoid per-byte overhead, then seek back after the terminator.
/// At EOF, return the bytes read even if no terminator was found.
fn read_until<R: Read + Seek>(reader: &mut R, end: u8) -> BinResult<Vec<u8>> {
    let mut out = Vec::new();
    let mut buffer = [0u8; CHUNK];
    loop {
        let from = reader.stream_position()?;
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok(out);
        }
        if let Some(at) = buffer[..read].iter().position(|byte| *byte == end) {
            out.extend_from_slice(&buffer[..at]);
            reader.seek(SeekFrom::Start(from + at as u64 + 1))?;
            return Ok(out);
        }
        out.extend_from_slice(&buffer[..read]);
    }
}

#[binrw::parser(reader, endian)]
pub fn read_int_prefixed_string() -> BinResult<String> {
    let pos = reader.stream_position()?;
    let count = u32::read_options(reader, endian, ())?;

    // Read in bulk to avoid per-byte overhead across thousands of archive names.
    // Grow with the data instead of allocating from an untrusted count.
    // Propagate I/O errors; EOF may still return fewer bytes than requested.
    let mut bytes = Vec::new();
    reader.take(count as u64).read_to_end(&mut bytes)?;
    String::from_utf8(bytes).map_err(|e| binrw::Error::Custom {
        pos,
        err: Box::new(e),
    })
}

#[binrw::writer(writer, endian)]
pub fn write_int_prefixed_string(value: &String) -> BinResult<()> {
    let str_bytes = value.as_bytes();
    (str_bytes.len() as u32).write_options(writer, endian, ())?;
    str_bytes.write_options(writer, endian, ())
}

#[binrw::parser(reader)]
pub fn parse_lf_terminated_string() -> BinResult<String> {
    let pos = reader.stream_position()?;
    String::from_utf8(read_until(reader, b'\n')?).map_err(|e| binrw::Error::Custom {
        pos,
        err: Box::new(e),
    })
}

#[binrw::parser(reader)]
pub fn parse_null_terminated_euc_kr_string() -> BinResult<String> {
    let bytes = read_until(reader, 0)?;
    let (cow, _encoding_used, _had_errors) = EUC_KR.decode(&bytes);
    Ok(cow.to_string())
}

#[allow(clippy::ptr_arg)]
pub fn string_to_null_terminated_euc_kr(s: &String) -> Vec<u8> {
    let null_terminated = format!("{}\0", s);
    let (cow, _encoding_used, _had_errors) = EUC_KR.encode(&null_terminated);
    cow.to_vec()
}
