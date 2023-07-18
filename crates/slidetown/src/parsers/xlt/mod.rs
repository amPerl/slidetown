use std::io::{Read, Write};

use anyhow::Context;

pub mod spoiler_list;
pub mod tire_list;
pub mod vehicle_list;
pub mod visual_item_list;
pub mod vshop_item_list;

#[derive(Debug, Clone)]
pub struct Xlt {
    pub rows: Vec<Vec<String>>,
}

impl Xlt {
    pub fn read<R: Read>(reader: &mut R) -> anyhow::Result<Self> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;

        let mut rows = Vec::new();

        let mut u16_iter = bytes
            .chunks_exact(2)
            .filter_map(|slice| slice.try_into().ok())
            .map(u16::from_le_bytes)
            .skip(1);

        let mut line_buf = Vec::new();
        while let Some(ch) = u16_iter.next() {
            if ch == '\r' as u16 {
                let _lf = u16_iter.next().unwrap();
                let line_str =
                    String::from_utf16(&line_buf).context("failed to parse utf16 xlt line")?;
                rows.push(line_str.split('\t').map(str::to_owned).collect());
                line_buf.clear();
            } else {
                line_buf.push(ch);
            }
        }
        if !line_buf.is_empty() {
            let line_str =
                String::from_utf16(&line_buf).context("failed to parse utf16 xlt line")?;
            rows.push(line_str.split('\t').map(str::to_owned).collect());
        }

        Ok(Self { rows })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_all(&[0xFF, 0xFE])?;
        for row in self.rows.iter() {
            let mut row_str = row.join("\t");
            row_str.push_str("\r\n");
            writer.write_all(
                &row_str
                    .encode_utf16()
                    .flat_map(u16::to_le_bytes)
                    .collect::<Vec<_>>(),
            )?;
        }
        Ok(())
    }
}
