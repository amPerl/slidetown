use anyhow::Context;

use super::Xlt;

#[derive(Debug, Clone)]
pub struct PvpTrackInfo {
    pub meta: PvpTrackInfoMeta,
    pub entries: Vec<PvpTrackInfoEntry>,
}

impl PvpTrackInfo {
    pub fn from_xlt(xlt: &Xlt) -> anyhow::Result<Self> {
        let mut row_iter = xlt.rows.iter();

        let _ = row_iter
            .next()
            .context("expected pvp track list meta header to be on row 1")?;
        let meta_row = row_iter
            .next()
            .context("expected pvp track list meta to be on row 2")?;
        let meta = PvpTrackInfoMeta::from_xlt_row(meta_row)
            .context("failed to parse pvp track list meta")?;

        let _ = row_iter
            .next()
            .context("expected pvp track list header to be on row 3")?;
        let mut entries = Vec::new();
        for _track_idx in 0..meta.track_count {
            let mut row = row_iter.next().context("expected pvp track list entry")?;
            let mut entry = PvpTrackInfoEntry::from_xlt_row(row)
                .context("failed to parse pvp track list entry")?;

            // The first two rows are the start and end gates, then track_count rows of gates
            for _gate_idx in 0..(entry.track_count + 1) {
                let gate = PvpTrackInfoGate::from_xlt_row(row)
                    .context("failed to parse pvp track list gate entry")?;
                entry.gates.push(gate);
                row = row_iter
                    .next()
                    .context("expected pvp track list gate entry")?;
            }

            entries.push(entry);
        }

        Ok(Self { meta, entries })
    }
}

#[derive(Debug, Clone)]
pub struct PvpTrackInfoMeta {
    pub count: usize,
    pub track_count: usize,
}

impl PvpTrackInfoMeta {
    pub fn from_xlt_row(row: &[String]) -> anyhow::Result<Self> {
        Ok(Self {
            count: row.get(1).context("expected count at column 2")?.parse()?,
            track_count: row
                .get(2)
                .context("expected track count at column 3")?
                .parse()?,
        })
    }
}

// Support	city	track	Id	type	New	AI	TrackCnt	gate	num	reward	pos_start			pos_end			minx	miny	maxx	maxy	LapCount	Traffic Preset	ReversePoint	CityType	city	tracknum	Reverse	Recommend	MissionBattle	name	Magic Number	Comment
// 1	moonpalace	Track1	0	0	0	1	5	start	0	0	-1117.9	619.9	0	-1132	657	0	-3201.639	-351.358	-700.427	1445.974	2	0	0	1	1	1	0	"111111"	"00000000"	moonpalace_track1	7777777
// 1				1	0	1	-1	end	0	0	-1117.9	619.9	0	-1132	657	0

#[derive(Debug, Clone)]
pub struct PvpTrackInfoGate {
    pub entry_type: usize, // 0 start, 1 end, 2 gate
    pub gate: String,
    pub num: usize,
    pub reward: usize,
    pub pos_start: [f32; 3],
    pub pos_end: [f32; 3],
}

impl PvpTrackInfoGate {
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
        skip_col!("support");
        skip_col!("city");
        skip_col!("track");
        skip_col!("id");
        parse_col_ty!(entry_type, usize);
        skip_col!("new");
        skip_col!("ai");
        skip_col!("track_count");
        parse_col_str!(gate);
        parse_col_ty!(num, usize);
        parse_col_ty!(reward, usize);
        parse_col_ty!(pos_start_x, f32);
        parse_col_ty!(pos_start_y, f32);
        parse_col_ty!(pos_start_z, f32);
        parse_col_ty!(pos_end_x, f32);
        parse_col_ty!(pos_end_y, f32);
        parse_col_ty!(pos_end_z, f32);

        Ok(Self {
            entry_type,
            gate,
            num,
            reward,
            pos_start: [pos_start_x, pos_start_y, pos_start_z],
            pos_end: [pos_end_x, pos_end_y, pos_end_z],
        })
    }
}

#[derive(Debug, Clone)]
pub struct PvpTrackInfoEntry {
    pub support: usize,
    pub city: String,
    pub track: String,
    pub id: usize,

    pub new: usize,
    pub ai: usize,
    pub track_count: isize,

    pub minx: f32,
    pub miny: f32,
    pub maxx: f32,
    pub maxy: f32,
    pub lap_count: usize,
    pub traffic_preset: usize,
    pub reverse_point: usize,
    pub city_type: usize,
    pub city_id: usize,
    pub track_num: usize,
    pub reverse: usize,
    pub recommend: String,
    pub mission_battle: String,
    pub name: String,
    pub magic_number: String,
    pub comment: String,

    pub gates: Vec<PvpTrackInfoGate>,
}

impl PvpTrackInfoEntry {
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
                    $name.parse().context(format!(
                        "failed to parse {} from value {} in column {}. full row: {:?}",
                        stringify!($name),
                        $name,
                        col_num,
                        row
                    ))?
                };
            };
        }

        skip_col!("index");
        parse_col_ty!(support, usize);
        parse_col_str!(city);
        parse_col_str!(track);
        parse_col_ty!(id, usize);
        skip_col!("type");
        parse_col_ty!(new, usize);
        parse_col_ty!(ai, usize);
        parse_col_ty!(track_count, isize);
        skip_col!("gate");
        skip_col!("num");
        skip_col!("reward");
        skip_col!("pos_start_x");
        skip_col!("pos_start_y");
        skip_col!("pos_start_z");
        skip_col!("pos_end_x");
        skip_col!("pos_end_y");
        skip_col!("pos_end_z");
        parse_col_ty!(minx, f32);
        parse_col_ty!(miny, f32);
        parse_col_ty!(maxx, f32);
        parse_col_ty!(maxy, f32);
        parse_col_ty!(lap_count, usize);
        parse_col_ty!(traffic_preset, usize);
        parse_col_ty!(reverse_point, usize);
        parse_col_ty!(city_type, usize);
        parse_col_ty!(city_id, usize);
        parse_col_ty!(track_num, usize);
        parse_col_ty!(reverse, usize);
        parse_col_str!(recommend);
        parse_col_str!(mission_battle);
        parse_col_str!(name);
        parse_col_str!(magic_number);
        parse_col_str!(comment);

        Ok(Self {
            support,
            city,
            track,
            id,
            new,
            ai,
            track_count,
            minx,
            miny,
            maxx,
            maxy,
            lap_count,
            traffic_preset,
            reverse_point,
            city_type,
            city_id,
            track_num,
            reverse,
            recommend,
            mission_battle,
            name,
            magic_number,
            comment,
            gates: Vec::new(),
        })
    }
}
