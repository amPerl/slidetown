use anyhow::Context;

use super::Xlt;

#[derive(Debug, Clone)]
pub struct VShopItemList {
    pub meta: VShopItemListMeta,
    pub entries: Vec<VShopItemListEntry>,
}

impl VShopItemList {
    pub fn from_xlt(xlt: &Xlt) -> anyhow::Result<Self> {
        let mut row_iter = xlt.rows.iter();

        let _ = row_iter
            .next()
            .context("expected vshop item list meta header to be on row 1")?;
        let meta_row = row_iter
            .next()
            .context("expected vshop item list meta to be on row 2")?;
        let meta = VShopItemListMeta::from_xlt_row(meta_row)
            .context("failed to parse vshop item list meta")?;

        let _ = row_iter
            .next()
            .context("expected vshop item list header to be on row 3")?;
        let mut entries = Vec::new();
        for row in row_iter.take_while(|r| r.get(0).is_some() && !r[0].trim().is_empty()) {
            let entry = VShopItemListEntry::from_xlt_row(row)
                .context("failed to parse vshop item list entry")?;
            entries.push(entry);
        }

        Ok(Self { meta, entries })
    }
}

#[derive(Debug, Clone)]
pub struct VShopItemListMeta {
    pub count: usize,
    pub bonus_mito_rate: usize,
    pub max_unique_id: usize,
}

impl VShopItemListMeta {
    pub fn from_xlt_row(row: &[String]) -> anyhow::Result<Self> {
        Ok(Self {
            count: row.get(1).context("expected count at column 2")?.parse()?,
            bonus_mito_rate: row
                .get(2)
                .context("expected bonus_mito_rate at column 3")?
                .parse()?,
            max_unique_id: row
                .get(3)
                .context("expected max_unique_id at column 4")?
                .parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct VShopItemListEntry {
    pub enabled: String,
    pub id: String,
    pub item_id: String,
    pub name: String,
    pub description: String,
    pub top: String,
    pub top_category: String,
    pub main: String,
    pub main_category: String,
    pub sub: String,
    pub sub_category: String,
    pub sub_category_name: String,
    pub refer_visualitem: String,
    pub sell_stage: String,
    pub close_stage: String,
    pub unequipable: String,
    pub pc_room_part: String,
    pub feature: String,
    pub default_part: String,
    pub display_order: String,
    pub instant: String,
    pub hot: String,
    pub car_shop_hot: String,
    pub event: String,
    pub rebate: String,
    pub use_mito: String,
    pub mito_price: String,
    pub sell_mito: String,
    pub use_gcoin: String,
    pub use_mileage: String,
    pub period_7d: String,
    pub price_7d: String,
    pub mito_price_7d: String,
    pub mile_price_7d: String,
    pub bonus_mito_7d: String,
    pub period_30d: String,
    pub price_30d: String,
    pub mito_price_30d: String,
    pub mile_price_30d: String,
    pub bonus_mito_30d: String,
    pub period_90d: String,
    pub price_90d: String,
    pub mito_price_90d: String,
    pub mile_price_90d: String,
    pub bonus_mito_90d: String,
    pub period_365d: String,
    pub price_365d: String,
    pub mito_price_365d: String,
    pub mile_price_365d: String,
    pub bonus_mito_365d: String,
    pub period_0d: String,
    pub price_0d: String,
    pub mito_price_0d: String,
    pub mile_price_0d: String,
    pub bonus_mito_0d: String,
    pub bonus_speed: String,
    pub bonus_accel: String,
    pub bonus_boost: String,
    pub bonus_crash: String,
    pub refund: String,
}

impl VShopItemListEntry {
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

        skip_col!("index");
        parse_col_str!(enabled);
        parse_col_str!(id);
        parse_col_str!(item_id);
        parse_col_str!(name);
        parse_col_str!(description);
        parse_col_str!(top);
        parse_col_str!(top_category);
        parse_col_str!(main);
        parse_col_str!(main_category);
        parse_col_str!(sub);
        parse_col_str!(sub_category);
        parse_col_str!(sub_category_name);
        parse_col_str!(refer_visualitem);
        parse_col_str!(sell_stage);
        parse_col_str!(close_stage);
        parse_col_str!(unequipable);
        parse_col_str!(pc_room_part);
        parse_col_str!(feature);
        parse_col_str!(default_part);
        parse_col_str!(display_order);
        parse_col_str!(instant);
        parse_col_str!(hot);
        parse_col_str!(car_shop_hot);
        parse_col_str!(event);
        parse_col_str!(rebate);
        parse_col_str!(use_mito);
        parse_col_str!(mito_price);
        parse_col_str!(sell_mito);
        parse_col_str!(use_gcoin);
        parse_col_str!(use_mileage);
        parse_col_str!(period_7d);
        parse_col_str!(price_7d);
        parse_col_str!(mito_price_7d);
        parse_col_str!(mile_price_7d);
        parse_col_str!(bonus_mito_7d);
        parse_col_str!(period_30d);
        parse_col_str!(price_30d);
        parse_col_str!(mito_price_30d);
        parse_col_str!(mile_price_30d);
        parse_col_str!(bonus_mito_30d);
        parse_col_str!(period_90d);
        parse_col_str!(price_90d);
        parse_col_str!(mito_price_90d);
        parse_col_str!(mile_price_90d);
        parse_col_str!(bonus_mito_90d);
        parse_col_str!(period_365d);
        parse_col_str!(price_365d);
        parse_col_str!(mito_price_365d);
        parse_col_str!(mile_price_365d);
        parse_col_str!(bonus_mito_365d);
        parse_col_str!(period_0d);
        parse_col_str!(price_0d);
        parse_col_str!(mito_price_0d);
        parse_col_str!(mile_price_0d);
        parse_col_str!(bonus_mito_0d);
        parse_col_str!(bonus_speed);
        parse_col_str!(bonus_accel);
        parse_col_str!(bonus_boost);
        parse_col_str!(bonus_crash);
        parse_col_str!(refund);

        Ok(Self {
            enabled,
            id,
            item_id,
            name,
            description,
            top,
            top_category,
            main,
            main_category,
            sub,
            sub_category,
            sub_category_name,
            refer_visualitem,
            sell_stage,
            close_stage,
            unequipable,
            pc_room_part,
            feature,
            default_part,
            display_order,
            instant,
            hot,
            car_shop_hot,
            event,
            rebate,
            use_mito,
            mito_price,
            sell_mito,
            use_gcoin,
            use_mileage,
            period_7d,
            price_7d,
            mito_price_7d,
            mile_price_7d,
            bonus_mito_7d,
            period_30d,
            price_30d,
            mito_price_30d,
            mile_price_30d,
            bonus_mito_30d,
            period_90d,
            price_90d,
            mito_price_90d,
            mile_price_90d,
            bonus_mito_90d,
            period_365d,
            price_365d,
            mito_price_365d,
            mile_price_365d,
            bonus_mito_365d,
            period_0d,
            price_0d,
            mito_price_0d,
            mile_price_0d,
            bonus_mito_0d,
            bonus_speed,
            bonus_accel,
            bonus_boost,
            bonus_crash,
            refund,
        })
    }
}
