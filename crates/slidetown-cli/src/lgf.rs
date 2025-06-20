use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
};

use clap::{Parser, Subcommand};
use slidetown::parsers::lgf;

use crate::util::nif_obj;

#[derive(Parser)]
pub struct LgfOpts {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// display info about archive contents
    Info(InfoOpts),

    /// export preview obj with terrain guardrails
    Obj(ObjOpts),

    /// export preview gltf with terrain guardrails
    Gltf(GltfOpts),
}

#[derive(Parser)]
struct InfoOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,
}

fn process_info(info_opts: InfoOpts) -> anyhow::Result<()> {
    let mut file = File::open(info_opts.input_path)?;
    let lgf: lgf::Lgf = lgf::Lgf::read(&mut file)?;

    println!("Block count: {}", lgf.blocks.len());

    Ok(())
}

#[derive(Parser)]
struct ObjOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,

    /// output file
    #[arg(short, long)]
    output_path: String,
}

fn process_obj(obj_opts: ObjOpts) -> anyhow::Result<()> {
    let mut file = File::open(&obj_opts.input_path)?;
    let lgf: lgf::Lgf = lgf::Lgf::read(&mut file)?;

    let mut obj = nif_obj::Obj::default();

    for block in lgf.blocks {
        file.seek(SeekFrom::Start(block.file_offset as u64))?;

        let mut nif_buf = vec![0u8; block.file_length as usize];
        file.read_exact(&mut nif_buf)?;

        let mut nif_cursor = Cursor::new(nif_buf);

        let nif = match nif::Nif::parse(&mut nif_cursor) {
            Ok(nif) => nif,
            Err(e) => {
                println!(
                    "Failed to parse NIF for block index {}: {:?}",
                    block.block_index, e
                );
                continue;
            }
        };

        obj.visit_nif(&nif, Some(format!("Block{}Guardrail", block.block_index)));
    }

    let obj_path = std::path::PathBuf::from(obj_opts.output_path);
    let mtl_path = obj_path.with_extension("mtl");

    obj.write_to_files(obj_path, mtl_path)?;

    Ok(())
}

#[derive(Parser)]
struct GltfOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,

    /// output file
    #[arg(short, long)]
    output_path: String,
}

fn process_gltf(gltf_opts: GltfOpts) -> anyhow::Result<()> {
    let mut file = File::open(&gltf_opts.input_path)?;
    let lgf: lgf::Lgf = lgf::Lgf::read(&mut file)?;

    let mut gltf = nif::collectors::gltf::Gltf::new();

    for block in lgf.blocks {
        file.seek(SeekFrom::Start(block.file_offset as u64))?;

        let mut nif_buf = vec![0u8; block.file_length as usize];
        file.read_exact(&mut nif_buf)?;

        let mut nif_cursor = Cursor::new(nif_buf);

        let nif = match nif::Nif::parse(&mut nif_cursor) {
            Ok(nif) => nif,
            Err(e) => {
                println!(
                    "Failed to parse NIF for block index {} : {:?}",
                    block.block_index, e
                );
                continue;
            }
        };

        gltf.visit_nif(
            &nif,
            Some("Block Guardrails"),
            &format!("Block{}Guardrail", block.block_index),
        );
    }

    let gltf_path = std::path::PathBuf::from(gltf_opts.output_path);
    gltf.write_to_files(gltf_path)?;

    Ok(())
}

pub fn process_lgf(lgf_opts: LgfOpts) -> anyhow::Result<()> {
    match lgf_opts.cmd {
        Command::Info(info_opts) => process_info(info_opts),
        Command::Obj(obj_opts) => process_obj(obj_opts),
        Command::Gltf(gltf_opts) => process_gltf(gltf_opts),
    }
}
