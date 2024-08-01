#![allow(clippy::get_first)]

use super::Xlt;
use anyhow::Context;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct VisualItemList {
    pub meta: VisualItemListMeta,
    pub vehicle_definitions: VehicleDefinitions,
    pub entries: Vec<VisualItemListEntry>,
}

impl VisualItemList {
    pub fn from_xlt(xlt: &Xlt) -> anyhow::Result<Self> {
        let mut row_iter = xlt.rows.iter();

        let _ = row_iter
            .next()
            .context("expected visual item list meta header to be on row 1")?;
        let meta_row = row_iter
            .next()
            .context("expected visual item list meta to be on row 2")?;
        let meta = VisualItemListMeta::from_xlt_row(meta_row)
            .context("failed to parse visual item list meta")?;

        let vehicle_defs_rows = row_iter.by_ref().take(3).collect::<Vec<_>>();
        let vehicle_definitions = VehicleDefinitions::from_xlt_rows(&vehicle_defs_rows)
            .context("failed to parse visual item list vehicle definitions")?;

        let _ = row_iter
            .next()
            .context("expected visual item list header to be on row 6")?;
        let mut entries = Vec::new();
        for row in row_iter.take_while(|r| r.get(1).is_some() && !r[1].trim().is_empty()) {
            let entry = VisualItemListEntry::from_xlt_row(row)
                .context("failed to parse visual item list entry")?;
            entries.push(entry);
        }

        Ok(Self {
            meta,
            vehicle_definitions,
            entries,
        })
    }
}

#[derive(Debug, Clone)]
pub struct VisualItemListMeta {
    pub list_count: usize,
    pub car_type_count: usize,
    pub max_unique_id: usize,
}

impl VisualItemListMeta {
    pub fn from_xlt_row(row: &[String]) -> anyhow::Result<Self> {
        Ok(Self {
            list_count: row
                .get(1)
                .context("expected list_count at column 2")?
                .parse()?,
            car_type_count: row
                .get(2)
                .context("expected car_type_count at column 3")?
                .parse()?,
            max_unique_id: row
                .get(3)
                .context("expected max_unique_id at column 4")?
                .parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct VehicleDefinitions {
    pub ids: Vec<usize>,
    pub names: Vec<String>,
    pub file_names: Vec<String>,
}

impl VehicleDefinitions {
    pub fn from_xlt_rows(rows: &[&Vec<String>]) -> anyhow::Result<Self> {
        let ids_row = rows.get(0).context("expected vehicle ids row")?;
        let names_row = rows.get(1).context("expected vehicle names row")?;
        let file_names_row = rows.get(2).context("expected vehicle file names row")?;
        Ok(Self {
            ids: ids_row
                .iter()
                .skip(7)
                .take_while(|id| !id.trim().is_empty())
                .flat_map(|id_str| id_str.parse().ok())
                .collect(),
            names: names_row
                .iter()
                .skip(7)
                .take_while(|s| !s.trim().is_empty())
                .cloned()
                .collect(),
            file_names: file_names_row
                .iter()
                .skip(7)
                .take_while(|s| !s.trim().is_empty())
                .cloned()
                .collect(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(usize)]
pub enum VisualItemCategory {
    Paint = 1,
    Neon = 2,
    WindowColor = 3,
    AeroBumper = 4,
    AeroHood = 5,
    AeroKit = 6,
    Spoiler = 7,
    Tire = 8,
    NumberPlate = 9,
    MufflerFlame = 10,
    Decal = 11,
    Horn = 12,
    Engine = 13,
    Upgrade = 14,
    ExpDrink = 15,
    InventoryExpansion = 16,
    MitoDrink = 17,
    RepairGirl = 18,
    PartsBox = 19,
    StickerGlue = 20,
    TurboMan = 21,
    GarageExtension = 22,
    DoubleUpgrade = 23,
    HalfCharge = 24,
    SetLubeAlpha = 25,
    SetLubeBeta = 26,
    MitoPrize = 27,
    ItemDrink = 28,
    PortableModKit = 29,
    CarRental = 30,
    RentItem = 31,
    GlossyPaint = 32,
    UseItem = 33,
    AutoFillup = 34,
    AutoQuickslot = 35,
    NameChange = 36,
    UpgradeTimed = 38,
    AutoDrive = 39,
    BuffDummy = 40,
    PassiveAssist = 41,
    Hongpao = 42,
    CarTransformationEx = 43,
    ChannelReplayPass = 44,
    ChannelGhostPass = 45,
    RoofItem = 47,
    GlossyTire = 48,
    Package = 49,
    DiscountCoupon = 51,
    AirDuct = 52,
    GachaCoin = 53,
    GCoinBox = 54,
}

impl FromStr for VisualItemCategory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num: usize = s
            .parse()
            .context("could not parse visual item category as number")?;

        Ok(match num {
            1 => Self::Paint,
            2 => Self::Neon,
            3 => Self::WindowColor,
            4 => Self::AeroBumper,
            5 => Self::AeroHood,
            6 => Self::AeroKit,
            7 => Self::Spoiler,
            8 => Self::Tire,
            9 => Self::NumberPlate,
            10 => Self::MufflerFlame,
            11 => Self::Decal,
            12 => Self::Horn,
            13 => Self::Engine,
            14 => Self::Upgrade,
            15 => Self::ExpDrink,
            16 => Self::InventoryExpansion,
            17 => Self::MitoDrink,
            18 => Self::RepairGirl,
            19 => Self::PartsBox,
            20 => Self::StickerGlue,
            21 => Self::TurboMan,
            22 => Self::GarageExtension,
            23 => Self::DoubleUpgrade,
            24 => Self::HalfCharge,
            25 => Self::SetLubeAlpha,
            26 => Self::SetLubeBeta,
            27 => Self::MitoPrize,
            28 => Self::ItemDrink,
            29 => Self::PortableModKit,
            30 => Self::CarRental,
            31 => Self::RentItem,
            32 => Self::GlossyPaint,
            33 => Self::UseItem,
            34 => Self::AutoFillup,
            35 => Self::AutoQuickslot,
            36 => Self::NameChange,
            38 => Self::UpgradeTimed,
            39 => Self::AutoDrive,
            40 => Self::BuffDummy,
            41 => Self::PassiveAssist,
            42 => Self::Hongpao,
            43 => Self::CarTransformationEx,
            44 => Self::ChannelReplayPass,
            45 => Self::ChannelGhostPass,
            47 => Self::RoofItem,
            48 => Self::GlossyTire,
            49 => Self::Package,
            51 => Self::DiscountCoupon,
            52 => Self::AirDuct,
            53 => Self::GachaCoin,
            54 => Self::GCoinBox,
            _ => anyhow::bail!("invalid visual item category: {}", num),
        })
    }
}

#[derive(Debug, Clone)]
pub struct VisualItemListEntry {
    pub category: VisualItemCategory,
    pub category_item_id: usize,
    pub item_id: String,
    pub id: usize,
    pub name: String,
    pub parameters: String,
    pub enabled_vehicle_id_indices: Vec<usize>,
}

impl VisualItemListEntry {
    pub fn from_xlt_row(row: &[String]) -> anyhow::Result<Self> {
        let mut row_iter = row.iter().skip(1);
        let category = row_iter
            .next()
            .context("expected category id at column 2")?
            .trim()
            .parse()
            .context("failed to parse visual item category")?;
        let category_item_id = row_iter
            .next()
            .context("expected category item id at column 3")?
            .trim()
            .parse()
            .context("failed to parse category item id")?;
        let item_id = row_iter
            .next()
            .context("expected item id at column 4")?
            .clone();
        let id = row_iter
            .next()
            .context("expected numeric item id at column 5")?
            .trim()
            .parse()
            .context("failed to parse numeric item id")?;
        let name = row_iter
            .next()
            .context("expected name at column 6")?
            .clone();
        let parameters = row_iter
            .next()
            .context("expected parameters at column 7")?
            .clone();
        let enabled_vehicle_id_indices = row_iter
            .map(|s| s.trim())
            .take_while(|s| !s.is_empty())
            .enumerate()
            .filter_map(|(i, s)| s.parse().map(|n: usize| (i, n)).ok())
            .filter(|&(_i, n)| (n != 0))
            .map(|(i, _n)| i)
            .collect();
        Ok(Self {
            category,
            category_item_id,
            item_id,
            id,
            name,
            parameters,
            enabled_vehicle_id_indices,
        })
    }
}
