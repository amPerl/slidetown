use std::{collections::HashMap, fs::File};

use binrw::BinReaderExt;
use slidetown::parsers::levelmodifier::LevelModifier;

fn main() -> anyhow::Result<()> {
    let mut lm_file = File::open("resources/levelmodifier/dcrnew_levelmodifier.dat")?;
    let lm: LevelModifier = lm_file.read_le()?;

    let mut min_max_map = HashMap::new();

    for accel_option in lm.accel {
        for (value, id) in accel_option
            .values
            .into_iter()
            .zip(lm.accel_ids.clone().into_iter())
        {
            let (min, max) = min_max_map.entry(id).or_insert((f32::MAX, f32::MIN));
            if value < *min {
                *min = value;
            }
            if value > *max {
                *max = value;
            }
        }
    }

    for speed_option in lm.speed {
        for (value, id) in speed_option
            .values
            .into_iter()
            .zip(lm.speed_ids.clone().into_iter())
        {
            let (min, max) = min_max_map.entry(id).or_insert((f32::MAX, f32::MIN));
            if value < *min {
                *min = value;
            }
            if value > *max {
                *max = value;
            }
        }
    }

    for boost_option in lm.boost {
        for (value, id) in boost_option
            .values
            .into_iter()
            .zip(lm.boost_ids.clone().into_iter())
        {
            let (min, max) = min_max_map.entry(id).or_insert((f32::MAX, f32::MIN));
            if value < *min {
                *min = value;
            }
            if value > *max {
                *max = value;
            }
        }
    }

    for dura_option in lm.dura {
        for (value, id) in dura_option
            .values
            .into_iter()
            .zip(lm.dura_ids.clone().into_iter())
        {
            let (min, max) = min_max_map.entry(id).or_insert((f32::MAX, f32::MIN));
            if value < *min {
                *min = value;
            }
            if value > *max {
                *max = value;
            }
        }
    }

    let mut keys = min_max_map.keys().copied().collect::<Vec<_>>();
    keys.sort();

    for id in keys {
        let (min, max) = min_max_map.get(&id).unwrap();
        println!("{id}: ({min}, {max})");
    }

    Ok(())
}
