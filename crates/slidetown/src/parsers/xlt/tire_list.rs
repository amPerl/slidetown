use anyhow::Context;

use super::Xlt;

#[derive(Debug, Clone)]
pub struct TireList {
    pub meta: TireListMeta,
    pub entries: Vec<TireListEntry>,
}

impl TireList {
    pub fn from_xlt(xlt: &Xlt) -> anyhow::Result<Self> {
        let mut row_iter = xlt.rows.iter();

        let _ = row_iter
            .next()
            .context("expected tire list meta header to be on row 1")?;
        let meta_row = row_iter
            .next()
            .context("expected tire list meta to be on row 2")?;
        let meta =
            TireListMeta::from_xlt_row(meta_row).context("failed to parse tire list meta")?;

        let _ = row_iter
            .next()
            .context("expected tire list header to be on row 3")?;
        let mut entries = Vec::new();
        for row in row_iter.take_while(|r| r.first().is_some() && !r[0].trim().is_empty()) {
            let entry =
                TireListEntry::from_xlt_row(row).context("failed to parse tire list entry")?;
            entries.push(entry);
        }

        Ok(Self { meta, entries })
    }
}

#[derive(Debug, Clone)]
pub struct TireListMeta {
    pub count: usize,
}

impl TireListMeta {
    pub fn from_xlt_row(row: &[String]) -> anyhow::Result<Self> {
        Ok(Self {
            count: row.get(1).context("expected count at column 2")?.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TireListEntry {
    pub id: isize,
    pub name: String,
    pub listing: String,
    pub tire_group: String,
    pub car_prototype: String,
    pub description: String,
    pub file_name: String,
    pub width: usize,
    pub diameter: usize,
    pub ui_icon_id: String,
    pub new: String,
}

impl TireListEntry {
    pub fn from_xlt_row(row: &[String]) -> anyhow::Result<Self> {
        let mut row_iter = row.iter();

        let mut col_num = 0;

        macro_rules! skip_col {
            ($name:literal) => {
                col_num += 1;
                let _ = row_iter
                    .next()
                    .context(format!("expected {} at column {}", $name, col_num))?
                    .trim()
                    .to_owned();
            };
        }

        macro_rules! parse_col_str {
            ($name:ident) => {
                col_num += 1;
                let $name = row_iter
                    .next()
                    .context(format!(
                        "expected {} at column {}",
                        stringify!($name),
                        col_num
                    ))?
                    .trim()
                    .to_owned();
            };
        }
        macro_rules! parse_col_ty {
            ($name:ident, $ty:ty) => {
                let $name: $ty = {
                    parse_col_str!($name);
                    // eprintln!("{}: {:?}", stringify!($name), $name);
                    $name
                        .parse()
                        .context(format!("failed to parse {}", stringify!($name)))?
                };
            };
        }

        skip_col!("index");
        parse_col_ty!(id, isize);
        parse_col_str!(name);
        parse_col_str!(listing);
        parse_col_str!(tire_group);
        parse_col_str!(car_prototype);
        parse_col_str!(description);
        parse_col_str!(file_name);
        parse_col_ty!(width, usize);
        parse_col_ty!(diameter, usize);
        parse_col_str!(ui_icon_id);
        parse_col_str!(new);

        Ok(Self {
            id,
            name,
            listing,
            tire_group,
            car_prototype,
            description,
            file_name,
            width,
            diameter,
            ui_icon_id,
            new,
        })
    }
}
