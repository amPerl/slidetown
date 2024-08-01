use std::{
    convert::TryInto,
    fs::File,
    io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
};

use clap::{Parser, Subcommand};
use slidetown::parsers::{lf, EntryOffsets};

use crate::nif_obj;

#[derive(Parser)]
pub struct LfOpts {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// display info about archive contents
    Info(InfoOpts),

    /// unpack terrain block nifs and create manifest
    Unpack(UnpackOpts),

    /// pack terrain block nifs using manifest
    Pack(PackOpts),

    /// export preview obj with terrain blocks
    Obj(ObjOpts),

    /// export preview gltf with terrain blocks
    Gltf(GltfOpts),
}

#[derive(Parser)]
struct InfoOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,

    /// only output block count
    #[arg(short, long, default_value = "false")]
    blocks: bool,
}

fn process_info(info_opts: InfoOpts) -> anyhow::Result<()> {
    let mut file = File::open(info_opts.input_path)?;
    let lf: lf::Lf = lf::Lf::read_without_data(&mut file)?;

    if info_opts.blocks {
        println!("{}", lf.block_count);
        return Ok(());
    }

    println!("Dimensions: {}x{}", lf.size_x, lf.size_y);
    println!("Block count: {}", lf.block_count);

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
    let lf: lf::Lf = lf::Lf::read_without_data(&mut file)?;

    let mut obj = nif_obj::Obj::default();

    for block in lf.blocks {
        file.seek(SeekFrom::Start(block.file_offset as u64))?;

        let mut nif_buf = vec![0u8; block.file_length as usize];
        file.read_exact(&mut nif_buf)?;

        let mut nif_cursor = Cursor::new(nif_buf);

        let nif = match nif::Nif::parse(&mut nif_cursor) {
            Ok(nif) => nif,
            Err(e) => {
                println!(
                    "Failed to parse NIF for block x{} y{}: {:?}",
                    block.position_x, block.position_y, e
                );
                continue;
            }
        };

        obj.visit_nif(&nif, Some(format!("Block{}", block.index)));
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
    let lf: lf::Lf = lf::Lf::read_without_data(&mut file)?;

    let mut gltf = nif::collectors::gltf::Gltf::new();

    for block in lf.blocks {
        file.seek(SeekFrom::Start(block.file_offset as u64))?;

        let mut nif_buf = vec![0u8; block.file_length as usize];
        file.read_exact(&mut nif_buf)?;

        let mut nif_cursor = Cursor::new(nif_buf);

        let nif = match nif::Nif::parse(&mut nif_cursor) {
            Ok(nif) => nif,
            Err(e) => {
                println!(
                    "Failed to parse NIF for block x{} y{}: {:?}",
                    block.position_x, block.position_y, e
                );
                continue;
            }
        };

        gltf.visit_nif(&nif, Some("Terrain"), &format!("Block{}", block.index));
    }

    let gltf_path = std::path::PathBuf::from(gltf_opts.output_path);

    gltf.write_to_files(gltf_path)?;

    Ok(())
}

#[derive(Parser)]
struct UnpackOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,

    /// output manifest
    #[arg(short, long)]
    output_path: String,
}

fn process_unpack(unpack_opts: UnpackOpts) -> anyhow::Result<()> {
    let mut file = File::open(&unpack_opts.input_path)?;

    let lf_archive: lf::Lf = lf::Lf::read_without_data(&mut file)?;

    let out_dir_path = Path::new(&unpack_opts.output_path);
    std::fs::create_dir_all(out_dir_path)?;

    {
        let manifest_file = File::create(out_dir_path.join("manifest.json"))?;
        serde_json::to_writer_pretty(manifest_file, &lf_archive)?;
    }

    for lf_block in lf_archive.blocks {
        println!("Writing block {}", lf_block.index);

        let nif_position: u64 = lf_block.file_offset.into();
        let nif_length: usize = lf_block
            .file_length
            .try_into()
            .expect("Block file size too high");

        let mut nif_buffer = vec![0u8; nif_length];

        file.seek(SeekFrom::Start(nif_position))?;
        file.read_exact(&mut nif_buffer)?;

        let mut nif_file = File::create(out_dir_path.join(format!("{}.nif", lf_block.index)))?;
        nif_file.write_all(&nif_buffer)?;
    }

    Ok(())
}

#[derive(Parser)]
struct PackOpts {
    /// input manifest
    #[arg(short, long)]
    input_path: String,

    /// output file
    #[arg(short, long)]
    output_path: String,
}

fn process_pack(pack_opts: PackOpts) -> anyhow::Result<()> {
    let input_path = Path::new(&pack_opts.input_path);

    let mut lf_archive: lf::Lf = {
        let manifest_file = File::open(input_path)?;
        serde_json::from_reader(manifest_file)?
    };

    lf_archive.header.version_date = 20090406;

    let offsets = EntryOffsets::default();

    let mut out_file = BufWriter::new(File::create(pack_opts.output_path)?);
    lf_archive.write_without_data(&mut out_file, offsets.clone())?;

    for (block, &header_offset) in lf_archive.blocks.iter().zip(offsets.borrow().iter()) {
        let block_file_path = input_path.with_file_name(format!("{}.nif", block.index));

        let file_offset = out_file.stream_position()? as u32;

        let mut block_file = File::open(block_file_path)?;
        let file_length = std::io::copy(&mut block_file, &mut out_file)? as u32;

        // Go back and fill in header offsets
        out_file.seek(SeekFrom::Start(header_offset))?;
        out_file.write_all(&file_offset.to_le_bytes())?;
        out_file.write_all(&file_length.to_le_bytes())?;
        out_file.seek(SeekFrom::Start((file_offset + file_length).into()))?;
    }

    Ok(())
}

pub fn process_lf(lf_opts: LfOpts) -> anyhow::Result<()> {
    match lf_opts.cmd {
        Command::Info(info_opts) => process_info(info_opts),
        Command::Unpack(unpack_opts) => process_unpack(unpack_opts),
        Command::Pack(pack_opts) => process_pack(pack_opts),
        Command::Obj(obj_opts) => process_obj(obj_opts),
        Command::Gltf(gltf_opts) => process_gltf(gltf_opts),
    }
}
