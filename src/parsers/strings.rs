#![allow(dead_code)]

use encoding_rs::EUC_KR;
use std::io::SeekFrom;

use binrw::{
    io::{Read, Seek},
    BinRead, BinResult, ReadOptions,
};

pub fn parse_int_prefixed_string<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    _: (),
) -> BinResult<String> {
    let pos = reader.seek(SeekFrom::Current(0))?;
    let count = u32::read_options(reader, options, ())?;

    String::from_utf8(
        reader
            .take(count as u64)
            .bytes()
            .filter_map(Result::ok)
            .collect(),
    )
    .map_err(|e| binrw::Error::Custom {
        pos: pos as u64,
        err: Box::new(e),
    })
}

pub fn parse_lf_terminated_string<R: Read + Seek>(
    reader: &mut R,
    _options: &ReadOptions,
    _: (),
) -> BinResult<String> {
    let pos = reader.seek(SeekFrom::Current(0))?;

    String::from_utf8(
        reader
            .bytes()
            .filter_map(Result::ok)
            .take_while(|&b| b != b'\n')
            .collect(),
    )
    .map_err(|e| binrw::Error::Custom {
        pos: pos as u64,
        err: Box::new(e),
    })
}

pub fn parse_null_terminated_euc_kr_string<R: Read + Seek>(
    reader: &mut R,
    _options: &ReadOptions,
    _: (),
) -> BinResult<String> {
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
