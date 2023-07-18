use std::str::FromStr;

use anyhow::Context;

use super::Xlt;

#[derive(Debug, Clone)]
pub struct VehicleList {
    pub meta: VehicleListMeta,
    pub entries: Vec<VehicleListEntry>,
}

impl VehicleList {
    pub fn from_xlt(xlt: &Xlt) -> anyhow::Result<Self> {
        let mut row_iter = xlt.rows.iter();

        let _ = row_iter
            .next()
            .context("expected vehicle list meta header to be on row 1")?;
        let meta_row = row_iter
            .next()
            .context("expected vehicle list meta to be on row 2")?;
        let meta =
            VehicleListMeta::from_xlt_row(meta_row).context("failed to parse vehicle list meta")?;

        let _ = row_iter
            .next()
            .context("expected vehicle list header 1 to be on row 3")?;
        let _ = row_iter
            .next()
            .context("expected vehicle list header 2 to be on row 4")?;
        let mut entries = Vec::new();
        for row in row_iter.take_while(|r| r.get(0).is_some() && !r[0].trim().is_empty()) {
            let entry = VehicleListEntry::from_xlt_row(row)
                .context("failed to parse vehicle list entry")?;
            entries.push(entry);
        }

        Ok(Self { meta, entries })
    }
}

#[derive(Debug, Clone)]
pub struct VehicleListMeta {
    pub player_car_base: usize,
    pub huv_base: usize,
    pub traffic_base: usize,
}

impl VehicleListMeta {
    pub fn from_xlt_row(row: &[String]) -> anyhow::Result<Self> {
        Ok(Self {
            player_car_base: row
                .get(2)
                .context("expected player_car_base at column 3")?
                .parse()?,
            huv_base: row
                .get(4)
                .context("expected huv_base at column 5")?
                .parse()?,
            traffic_base: row
                .get(6)
                .context("expected traffic_base at column 7")?
                .parse()?,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VehicleKind {
    PlayerCar,
    Player,
    Huv,
    HuvBoss,
    Traffic,
    RacingBattle,
}

impl FromStr for VehicleKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "PlayerCar" => Self::PlayerCar,
            "Player" => Self::Player,
            "HUV" => Self::Huv,
            "HUV BOSS" => Self::HuvBoss,
            "Traffic" => Self::Traffic,
            "RacingBattle" => Self::RacingBattle,
            _ => anyhow::bail!("{} is not a known VehicleKind", s),
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GradeKind {
    V,
    R,
}

impl FromStr for GradeKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "V" => Self::V,
            "R" => Self::R,
            _ => anyhow::bail!("{} is not a known GradeKind", s),
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VehicleRequirement {
    Gotcha,
    Event,
}

impl FromStr for VehicleRequirement {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "GOTCHA" => Self::Gotcha,
            "EVENT" => Self::Event,
            _ => anyhow::bail!("{} is not a known VehicleRequirement", s),
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CarFlag {
    G,
    H,
    B,
    M,
    C,
}

impl FromStr for CarFlag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "G" => Self::G,
            "H" => Self::H,
            "B" => Self::B,
            "M" => Self::M,
            "C" => Self::C,
            _ => anyhow::bail!("{} is not a known CarFlag", s),
        })
    }
}

#[derive(Debug, Clone)]
pub struct VehicleListEntry {
    pub enabled: bool,
    pub kind: VehicleKind,
    pub maker: String,
    pub real_name: String,
    pub name: String,
    pub file_name: String,
    pub old_file_name: String,
    pub id: usize,
    pub sellable: bool,
    pub make_type: bool,      // ? 1 on some, otherwise 0
    pub close_stage: bool,    // ? always 0
    pub display_order: isize, // 101, 102, 301, 302, etc..
    pub grade_kind: Option<GradeKind>,
    pub aero_set: bool, // ?
    pub accel: Option<usize>,
    pub speed: Option<usize>,
    pub crash: Option<usize>,
    pub boost: Option<usize>,
    pub required_level: Option<usize>,
    pub required_condition: Option<VehicleRequirement>, // N/A if none
    pub grade: GradeKind,
    pub grade_level: usize,
    pub length: Option<usize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub f_ratio: f64,
    pub f_length: Option<usize>,
    pub r_length: Option<usize>,
    pub shadow1: Option<usize>,
    pub shadow2: Option<usize>,
    pub body_window_pairs: [[usize; 2]; 4],
    pub tire_scale_front: usize,
    pub tire_scale_rear: usize,
    pub tire_width_front: f64,
    pub tire_width_rear: f64,
    pub tire_count: usize,
    pub tire_count_rear: usize,
    pub tire_ids: [Option<usize>; 10],
    pub tire_group_id: String, // enum?
    pub spoiler: Option<usize>,
    pub number_plate: Option<usize>,
    pub speed_slot: Option<usize>,
    pub accel_slot: Option<usize>,
    pub crash_slot: Option<usize>,
    pub boost_slot: Option<usize>,
    pub weight: usize,
    pub turbo_weaken_factor: f64,
    pub no_slip_time: f64,
    pub jump_scale: f64,
    pub max_height_diff: f64,
    pub cam_n: f64,
    pub cam_h: f64,
    pub acceleration: f64,
    pub deacceleration: f64,
    pub jump_car: bool, // ?
    pub car_flag: Option<CarFlag>,
    pub car_assist: Option<String>,
}

impl VehicleListEntry {
    pub fn from_xlt_row(row: &[String]) -> anyhow::Result<Self> {
        let mut row_iter = row.iter();

        macro_rules! parse_col {
            ($name:ident, $col:literal) => {
                let $name = {
                    let s = row_iter
                        .next()
                        .context(format!("expected {} at column {}", stringify!($name), $col))?
                        .trim();
                    // dbg!(stringify!($name), &s);
                    s.parse()
                        .context(format!("failed to parse {}", stringify!($name)))?
                };
            };
        }
        macro_rules! parse_col_ty {
            ($name:ident, $ty:ty, $col:literal) => {
                let $name: $ty = {
                    let s = row_iter
                        .next()
                        .context(format!("expected {} at column {}", stringify!($name), $col))?
                        .trim();
                    // dbg!(stringify!($name), &s);
                    s.parse()
                        .context(format!("failed to parse {}", stringify!($name)))?
                };
            };
        }
        macro_rules! skip_col {
            ($name:literal, $col:literal) => {
                let _ = row_iter
                    .next()
                    .context(format!("expected {} at column {}", $name, $col))?;
            };
        }

        skip_col!("index", 1);
        parse_col_ty!(enabled, usize, 2);
        skip_col!("vehicle kind number", 3);
        parse_col!(kind, 4);
        parse_col!(maker, 5);
        parse_col!(real_name, 6);
        parse_col!(name, 7);
        parse_col!(file_name, 8);
        parse_col!(old_file_name, 9);
        parse_col!(id, 10);
        parse_col_ty!(sellable, usize, 11);
        parse_col_ty!(make_type, usize, 12);
        parse_col_ty!(close_stage, usize, 13);
        parse_col!(display_order, 14);

        let grade_kind_str = row_iter
            .next()
            .context("expected grade_kind at column 15")?;
        let mut grade_kind = None;
        if grade_kind_str != "N/A" {
            grade_kind = Some(
                grade_kind_str
                    .parse()
                    .context("failed to parse grade_kind")?,
            );
        }

        parse_col_ty!(aero_set, usize, 16);

        let accel_str = row_iter
            .next()
            .context("expected accel at column 17")?
            .trim();
        let mut accel = None;
        if accel_str != "-1" {
            accel = Some(accel_str.parse().context("failed to parse accel")?);
        }

        let speed_str = row_iter
            .next()
            .context("expected speed at column 18")?
            .trim();
        let mut speed = None;
        if speed_str != "-1" {
            speed = Some(speed_str.parse().context("failed to parse speed")?);
        }

        let crash_str = row_iter
            .next()
            .context("expected crash at column 19")?
            .trim();
        let mut crash = None;
        if crash_str != "-1" {
            crash = Some(crash_str.parse().context("failed to parse crash")?);
        }

        let boost_str = row_iter
            .next()
            .context("expected boost at column 20")?
            .trim();
        let mut boost = None;
        if boost_str != "-1" {
            boost = Some(boost_str.parse().context("failed to parse boost")?);
        }

        let required_level_str = row_iter
            .next()
            .context("expected required_level at column 21")?
            .trim();
        let mut required_level = None;
        if required_level_str != "-1" {
            required_level = Some(
                required_level_str
                    .parse()
                    .context("failed to parse required_level")?,
            );
        }

        let required_condition_str = row_iter
            .next()
            .context("expected required condition at column 22")?;
        let mut required_condition = None;
        if required_condition_str != "N/A" {
            required_condition = Some(
                required_condition_str
                    .parse()
                    .context("failed to parse required condition")?,
            );
        }
        parse_col!(grade, 23);
        parse_col!(grade_level, 24);

        let length_str = row_iter
            .next()
            .context("expected length at column 25")?
            .trim();
        let mut length = None;
        if length_str != "-1" {
            length = Some(length_str.parse().context("failed to parse length")?);
        }

        let width_str = row_iter
            .next()
            .context("expected width at column 26")?
            .trim();
        let mut width = None;
        if width_str != "-1" {
            width = Some(width_str.parse().context("failed to parse width")?);
        }

        let height_str = row_iter
            .next()
            .context("expected height at column 27")?
            .trim();
        let mut height = None;
        if height_str != "-1" {
            height = Some(height_str.parse().context("failed to parse height")?);
        }

        parse_col!(f_ratio, 28);

        let f_length_str = row_iter
            .next()
            .context("expected flength at column 29")?
            .trim();
        let mut f_length = None;
        if f_length_str != "-1" {
            f_length = Some(f_length_str.parse().context("failed to parse flength")?);
        }

        let r_length_str = row_iter
            .next()
            .context("expected rlength at column 30")?
            .trim();
        let mut r_length = None;
        if r_length_str != "-1" {
            r_length = Some(r_length_str.parse().context("failed to parse rlength")?);
        }

        let shadow1_str = row_iter
            .next()
            .context("expected shadow1 at column 31")?
            .trim();
        let mut shadow1 = None;
        if shadow1_str != "-1" {
            shadow1 = Some(shadow1_str.parse().context("failed to parse shadow1")?);
        }

        let shadow2_str = row_iter
            .next()
            .context("expected shadow2 at column 32")?
            .trim();
        let mut shadow2 = None;
        if shadow2_str != "-1" {
            shadow2 = Some(shadow2_str.parse().context("failed to parse shadow2")?);
        }

        let body_window_pairs = [
            [
                row_iter
                    .next()
                    .context("expected body0 at column 33")?
                    .trim()
                    .parse()
                    .context("failed to parse body0")?,
                row_iter
                    .next()
                    .context("expected window at column 34")?
                    .trim()
                    .parse()
                    .context("failed to parse window")?,
            ],
            [
                row_iter
                    .next()
                    .context("expected body0 at column 35")?
                    .trim()
                    .parse()
                    .context("failed to parse body0")?,
                row_iter
                    .next()
                    .context("expected window at column 36")?
                    .trim()
                    .parse()
                    .context("failed to parse window")?,
            ],
            [
                row_iter
                    .next()
                    .context("expected body0 at column 37")?
                    .trim()
                    .parse()
                    .context("failed to parse body0")?,
                row_iter
                    .next()
                    .context("expected window at column 38")?
                    .trim()
                    .parse()
                    .context("failed to parse window")?,
            ],
            [
                row_iter
                    .next()
                    .context("expected body0 at column 39")?
                    .trim()
                    .parse()
                    .context("failed to parse body0")?,
                row_iter
                    .next()
                    .context("expected window at column 40")?
                    .trim()
                    .parse()
                    .context("failed to parse window")?,
            ],
        ];

        parse_col!(tire_scale_front, 41);
        parse_col!(tire_scale_rear, 42);
        parse_col!(tire_width_front, 43);
        parse_col!(tire_width_rear, 44);
        parse_col!(tire_count, 45);
        parse_col!(tire_count_rear, 46);

        let tire_ids = [
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 1 at column 47")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 2 at column 48")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 3 at column 49")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 4 at column 50")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 5 at column 51")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 6 at column 52")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 7 at column 53")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 8 at column 54")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 9 at column 55")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
            {
                let s = row_iter
                    .next()
                    .context("expected tire id 10 at column 56")?
                    .trim();
                if s != "-1" {
                    Some(s.parse().context("failed to parse tire id")?)
                } else {
                    None
                }
            },
        ];

        parse_col!(tire_group_id, 57);

        let spoiler_str = row_iter
            .next()
            .context("expected spoiler at column 58")?
            .trim();
        let mut spoiler = None;
        if spoiler_str != "-1" {
            spoiler = Some(spoiler_str.parse().context("failed to parse spoiler")?);
        }

        let number_plate_str = row_iter
            .next()
            .context("expected number_plate at column 59")?
            .trim();
        let mut number_plate = None;
        if number_plate_str != "-1" {
            number_plate = Some(
                number_plate_str
                    .parse()
                    .context("failed to parse number_plate")?,
            );
        }

        let speed_slot_str = row_iter
            .next()
            .context("expected speed_slot at column 60")?
            .trim();
        let mut speed_slot = None;
        if speed_slot_str != "-1" {
            speed_slot = Some(
                speed_slot_str
                    .parse()
                    .context("failed to parse speed_slot")?,
            );
        }

        let accel_slot_str = row_iter
            .next()
            .context("expected accel_slot at column 61")?
            .trim();
        let mut accel_slot = None;
        if accel_slot_str != "-1" {
            accel_slot = Some(
                accel_slot_str
                    .parse()
                    .context("failed to parse accel_slot")?,
            );
        }

        let crash_slot_str = row_iter
            .next()
            .context("expected crash_slot at column 62")?
            .trim();
        let mut crash_slot = None;
        if crash_slot_str != "-1" {
            crash_slot = Some(
                crash_slot_str
                    .parse()
                    .context("failed to parse crash_slot")?,
            );
        }

        let boost_slot_str = row_iter
            .next()
            .context("expected boost_slot at column 63")?
            .trim();
        let mut boost_slot = None;
        if boost_slot_str != "-1" {
            boost_slot = Some(
                boost_slot_str
                    .parse()
                    .context("failed to parse boost_slot")?,
            );
        }

        parse_col!(weight, 64);
        parse_col!(turbo_weaken_factor, 65);
        parse_col!(no_slip_time, 66);
        parse_col!(jump_scale, 67);
        parse_col!(max_height_diff, 68);
        parse_col!(cam_n, 69);
        parse_col!(cam_h, 70);
        parse_col!(acceleration, 71);
        parse_col!(deacceleration, 72);
        parse_col_ty!(jump_car, usize, 73);

        let car_flag = {
            let s = row_iter
                .next()
                .context("expected car_flag 10 at column 74")?
                .trim();
            if s != "0" && s != "-" {
                Some(s.parse().context("failed to parse car_flag")?)
            } else {
                None
            }
        };

        let car_assist = {
            let s = row_iter
                .next()
                .context("expected car_assist 10 at column 75")?
                .trim();
            if s != "0" && s != "-" {
                Some(s.parse().context("failed to parse car_assist")?)
            } else {
                None
            }
        };

        skip_col!("magic number", 76);

        Ok(Self {
            enabled: enabled != 0,
            kind,
            maker,
            real_name,
            name,
            file_name,
            old_file_name,
            id,
            sellable: sellable != 0,
            make_type: make_type != 0,
            close_stage: close_stage != 0,
            display_order,
            grade_kind,
            aero_set: aero_set != 0,
            accel,
            speed,
            crash,
            boost,
            required_level,
            required_condition,
            grade,
            grade_level,
            length,
            width,
            height,
            f_ratio,
            f_length,
            r_length,
            shadow1,
            shadow2,
            body_window_pairs,
            tire_scale_front,
            tire_scale_rear,
            tire_width_front,
            tire_width_rear,
            tire_count,
            tire_count_rear,
            tire_ids,
            tire_group_id,
            spoiler,
            number_plate,
            speed_slot,
            accel_slot,
            crash_slot,
            boost_slot,
            weight,
            turbo_weaken_factor,
            no_slip_time,
            jump_scale,
            max_height_diff,
            cam_n,
            cam_h,
            acceleration,
            deacceleration,
            jump_car: jump_car != 0,
            car_flag,
            car_assist,
        })
    }
}
