use std::time::Instant;

use clap::{Parser, Subcommand};

mod agt;
mod lbf;
mod levelmodifier;
mod lf;
mod lof;
mod loi;
mod world;

pub mod nif_obj;

#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"), author = "amPerl")]
struct Opts {
    /// archive type
    #[command(subcommand)]
    archive: Archive,
}

#[derive(Subcommand)]
enum Archive {
    /// AGT archives
    Agt(agt::AgtOpts),

    /// LF terrain blocks
    Lf(lf::LfOpts),

    /// LF terrain block objects
    Lbf(lbf::LbfOpts),

    /// LOF model table
    Lof(lof::LofOpts),

    /// LOI object list
    Loi(loi::LoiOpts),

    /// World/city
    World(world::WorldOpts),

    /// LevelModifier variables
    #[command(name = "levelmodifier")]
    LevelModifier(levelmodifier::LevelModifierOpts),
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    let before_process = Instant::now();

    let result = match opts.archive {
        Archive::Agt(agt_opts) => agt::process_agt(agt_opts),
        Archive::Lf(lf_opts) => lf::process_lf(lf_opts),
        Archive::Lbf(lbf_opts) => lbf::process_lbf(lbf_opts),
        Archive::Lof(lof_opts) => lof::process_lof(lof_opts),
        Archive::Loi(loi_opts) => loi::process_loi(loi_opts),
        Archive::World(world_opts) => world::process_world(world_opts),
        Archive::LevelModifier(levelmodifier_opts) => {
            levelmodifier::process_levelmodifier(levelmodifier_opts)
        }
    };

    println!("Done in {}ms", before_process.elapsed().as_millis());
    result
}
