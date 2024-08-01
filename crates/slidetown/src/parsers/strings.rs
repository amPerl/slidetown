#![allow(dead_code)]

use encoding_rs::EUC_KR;

use binrw::{io::Read, BinRead, BinResult, BinWrite};

#[binrw::parser(reader, endian)]
pub fn read_int_prefixed_string() -> BinResult<String> {
    let pos = reader.stream_position()?;
    let count = u32::read_options(reader, endian, ())?;

    String::from_utf8(
        reader
            .take(count as u64)
            .bytes()
            .filter_map(Result::ok)
            .collect(),
    )
    .map_err(|e| binrw::Error::Custom {
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

    String::from_utf8(
        reader
            .bytes()
            .filter_map(Result::ok)
            .take_while(|&b| b != b'\n')
            .collect(),
    )
    .map_err(|e| binrw::Error::Custom {
        pos,
        err: Box::new(e),
    })
}

#[binrw::parser(reader)]
pub fn parse_null_terminated_euc_kr_string() -> BinResult<String> {
    let bytes: Vec<u8> = reader
        .bytes()
        .filter_map(Result::ok)
        .take_while(|&b| b != 0)
        .collect();

    let (cow, _encoding_used, _had_errors) = EUC_KR.decode(&bytes);
    Ok(cow.to_string())
}

#[allow(clippy::ptr_arg)]
pub fn string_to_null_terminated_euc_kr(s: &String) -> Vec<u8> {
    let null_terminated = format!("{}\0", s);
    let (cow, _encoding_used, _had_errors) = EUC_KR.encode(&null_terminated);
    cow.to_vec()
}
