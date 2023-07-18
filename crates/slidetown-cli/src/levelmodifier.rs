use std::{fs::File, io::BufWriter, path::Path};

use clap::{Parser, Subcommand};
use slidetown::parsers::levelmodifier;

#[derive(Parser)]
pub struct LevelModifierOpts {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// unpack levelmodifier and create manifest
    Unpack(UnpackOpts),

    /// pack levelmodifier using manifest
    Pack(PackOpts),
}

#[derive(Parser)]
struct UnpackOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,

    /// output file
    #[arg(short, long)]
    output_path: String,
}

fn process_unpack(unpack_opts: UnpackOpts) -> anyhow::Result<()> {
    let mut file = File::open(&unpack_opts.input_path).expect("Failed to open source file");

    let levelmodifier: levelmodifier::LevelModifier =
        levelmodifier::LevelModifier::read(&mut file).expect("Failed to parse source file");

    let out_path = Path::new(&unpack_opts.output_path);

    {
        let json_file = File::create(out_path).expect("Failed to open target file");
        serde_json::to_writer_pretty(json_file, &levelmodifier)
            .expect("Failed to write to target file");
    }

    Ok(())
}

#[derive(Parser)]
struct PackOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,

    /// output file
    #[arg(short, long)]
    output_path: String,
}

fn process_pack(pack_opts: PackOpts) -> anyhow::Result<()> {
    let input_path = Path::new(&pack_opts.input_path);

    let levelmodifier: levelmodifier::LevelModifier = {
        let levelmodifier_json = File::open(input_path)?;
        serde_json::from_reader(levelmodifier_json)?
    };

    let mut out_file = BufWriter::new(File::create(pack_opts.output_path)?);
    levelmodifier.write(&mut out_file)?;

    Ok(())
}

pub fn process_levelmodifier(levelmodifier_opts: LevelModifierOpts) -> anyhow::Result<()> {
    match levelmodifier_opts.cmd {
        // Command::Info(info_opts) => process_info(info_opts),
        Command::Unpack(unpack_opts) => process_unpack(unpack_opts),
        Command::Pack(pack_opts) => process_pack(pack_opts),
    }
}
