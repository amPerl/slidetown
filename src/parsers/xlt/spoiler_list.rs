use anyhow::Context;

use super::Xlt;

#[derive(Debug, Clone)]
pub struct SpoilerList {
    pub meta: SpoilerListMeta,
    pub entries: Vec<SpoilerListEntry>,
}

impl SpoilerList {
    pub fn from_xlt(xlt: &Xlt) -> anyhow::Result<Self> {
        let mut row_iter = xlt.rows.iter();

        let _ = row_iter
            .next()
            .context("expected spoiler list meta header to be on row 1")?;
        let meta_row = row_iter
            .next()
            .context("expected spoiler list meta to be on row 2")?;
        let meta =
            SpoilerListMeta::from_xlt_row(meta_row).context("failed to parse spoiler list meta")?;

        let _ = row_iter
            .next()
            .context("expected spoiler list header to be on row 3")?;
        let mut entries = Vec::new();
        for row in row_iter.take_while(|r| r.get(0).is_some() && !r[0].trim().is_empty()) {
            let entry = SpoilerListEntry::from_xlt_row(row)
                .context("failed to parse spoiler list entry")?;
            entries.push(entry);
        }

        Ok(Self { meta, entries })
    }
}

#[derive(Debug, Clone)]
pub struct SpoilerListMeta {
    pub count: usize,
}

impl SpoilerListMeta {
    pub fn from_xlt_row(row: &[String]) -> anyhow::Result<Self> {
        Ok(Self {
            count: row.get(1).context("expected count at column 2")?.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SpoilerListEntry {
    pub id: usize,
    pub name: String,
    pub listing: String,
    pub group: String,
    pub animation: String,
    pub kind: String,
    pub icon_id: String,
    pub new: String,
}

impl SpoilerListEntry {
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
        parse_col_ty!(id, usize);
        parse_col_str!(name);
        parse_col_str!(listing);
        parse_col_str!(group);
        parse_col_str!(animation);
        parse_col_str!(kind);
        parse_col_str!(icon_id);
        parse_col_str!(new);

        Ok(Self {
            id,
            name,
            listing,
            group,
            animation,
            kind,
            icon_id,
            new,
        })
    }
}
