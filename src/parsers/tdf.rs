use binrw::{
    binrw,
    io::{Read, Seek},
    BinReaderExt, BinResult, BinWrite, FilePtr32, NullWideString,
};

#[binrw]
#[derive(PartialEq)]
pub struct HeaderBitmap {
    #[br(restore_position)]
    #[bw(ignore)]
    pub length: (u16, u16),
    #[br(count = length.1)]
    pub data: Vec<u8>,
}

impl std::fmt::Debug for HeaderBitmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeaderBitmap")
            .field("length", &self.length)
            .field("data.len()", &self.data.len())
            .finish()
    }
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct Tdf {
    pub bmp: HeaderBitmap,

    pub version: (u16, u16),

    pub year: u16,
    pub month: u8,
    pub day: u8,

    pub flag: u32,

    #[br(temp)]
    #[bw(calc = {
        let column_count = rows.iter().next().map(|row| row.len() as _).unwrap_or(0);
        let row_count = rows.len() as u32;
        let string_table_length_chars: usize = rows.iter().map(|row| row.iter().map(|s| s.encode_utf16().count() + 1).sum::<usize>()).sum();
        let length_with_rows = 24 + 4 * row_count * column_count;
        length_with_rows + string_table_length_chars as u32 * 2
    })]
    pub length: u32,

    #[br(temp)]
    #[bw(calc = rows.iter().next().map(|row| row.len() as _).unwrap_or(0))]
    pub column_count: u32,

    #[br(temp)]
    #[bw(calc = rows.len() as u32)]
    pub row_count: u32,

    // 24 bytes before this (excl bmp)
    // 4 * row_count * column_count bytes
    #[br(parse_with = parse_rows, args(bmp.data.len() as _, column_count, row_count))]
    #[bw(write_with = write_rows)]
    pub rows: Vec<Vec<String>>,
}

#[binrw::parser(reader, endian)]
fn parse_rows(offset: u64, column_count: u32, row_count: u32) -> BinResult<Vec<Vec<String>>> {
    let mut result = Vec::new();
    let mut string_table_length = 0;

    for _row_idx in 0..row_count {
        let mut row = Vec::new();
        for _column_idx in 0..column_count {
            let nws = FilePtr32::<NullWideString>::parse(reader, endian, binrw::args! { offset })?;
            let string = nws.to_string();
            string_table_length += string.encode_utf16().count() * 2 + 2;
            row.push(string);
        }
        result.push(row);
    }

    reader.seek(std::io::SeekFrom::Current(string_table_length as _))?;

    Ok(result)
}

#[binrw::writer(writer, endian)]
fn write_rows(rows: &Vec<Vec<String>>) -> BinResult<()> {
    let cells_count: usize = rows.iter().map(|r| r.len()).sum();
    let mut strings_offset = 24 + 4 * cells_count as u32;
    let mut strings = Vec::new();

    for row in rows {
        for column in row {
            // write where the string will be placed
            strings_offset.write_options(writer, endian, ())?;

            // store the string and advance the offset
            let str_chars = column
                .encode_utf16()
                .chain(std::iter::once(0u16))
                .collect::<Vec<_>>();
            strings_offset += str_chars.len() as u32 * 2;
            strings.push(str_chars);
        }
    }

    for string in strings {
        string.write_options(writer, endian, ())?;
    }

    Ok(())
}

impl Tdf {
    pub fn read<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        Ok(reader.read_le()?)
    }
}
