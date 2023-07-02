use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
};

use binrw::{BinRead, BinReaderExt, BinWrite, BinWriterExt};

pub fn test_full_rewrite<'ra, 'wa, S: BinWrite + BinRead>(
    path: &str,
    read_args: <S as BinRead>::Args<'ra>,
    write_args: <S as BinWrite>::Args<'wa>,
) -> anyhow::Result<S>
where
    <S as BinRead>::Args<'ra>: Clone,
    <S as BinWrite>::Args<'wa>: Clone,
    S: std::fmt::Debug,
{
    // Open input file for reading
    let mut in_file = BufReader::new(File::open(path)?);
    // Get file length and seek back to beginning
    let in_file_length = in_file.seek(SeekFrom::End(0))? as usize;
    in_file.seek(SeekFrom::Start(0))?;
    // Read the entire file into a buffer for parsing and rewrite validation
    let mut in_buf = vec![0u8; in_file_length];
    in_file.read_exact(&mut in_buf)?;
    // Just make sure we don't use the file directly anymore
    drop(in_file);

    let mut in_cursor = Cursor::new(&in_buf);
    let result: S = in_cursor.read_le_args(read_args)?;

    // dbg!(&result);

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    out_file.write_le_args(&result, write_args)?;

    // std::fs::write("tmp.bin", &out_buf)?;

    assert_eq!(
        in_buf.len(),
        in_cursor.position() as usize,
        "input was {} bytes, cursor stopped at {} bytes",
        in_buf.len(),
        in_cursor.position()
    );

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }

    Ok(result)
}
