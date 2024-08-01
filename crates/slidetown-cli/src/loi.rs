use std::{fs::File, io::BufWriter, path::Path};

use clap::{Parser, Subcommand};
use slidetown::parsers::loi;

#[derive(Parser)]
pub struct LoiOpts {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// display info about object list
    Info(InfoOpts),

    /// unpack object list and create manifest
    Unpack(UnpackOpts),

    /// pack object list using manifest
    Pack(PackOpts),

    /// export preview gltf with instanced objects
    Gltf(GltfOpts),
}

#[derive(Parser)]
struct InfoOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,

    /// total block count, as specified by the LF
    #[arg(short, long)]
    total_block_count: usize,
}

fn process_info(info_opts: InfoOpts) -> anyhow::Result<()> {
    let mut file = File::open(info_opts.input_path)?;
    let loi: loi::Loi = loi::Loi::read(&mut file, info_opts.total_block_count)?;

    println!("Block count: {}", loi.blocks.len());
    println!(
        "Blocks with any objects in them {}",
        loi.blocks
            .iter()
            .filter(|block| !block.objects.is_empty())
            .count()
    );
    println!(
        "Object count sum over blocks {}",
        loi.blocks
            .iter()
            .map(|block| block.objects.len())
            .sum::<usize>()
    );

    let object_indices = loi
        .blocks
        .iter()
        .flat_map(|block| block.objects.iter().map(|object| object.object_index))
        .collect::<Vec<u32>>();

    let object_indices_max = *object_indices.iter().max().unwrap_or(&0);

    println!("Highest object_index {}", object_indices_max);

    // println!("Blocks with objects:");
    // for block in loi.blocks.iter() {
    //     if block.object_count <= 0 {
    //         continue;
    //     }
    //     println!(
    //         "Block {} with object ids: {:?}",
    //         block.block_index,
    //         block
    //             .objects
    //             .iter()
    //             .map(|object| object.object_index)
    //             .collect::<Vec<u32>>()
    //     )
    // }

    // for potential_object_index in 0..object_indices_max {
    //     if object_indices.contains(&potential_object_index) {
    //         continue;
    //     }
    //     println!(
    //         "Missing object index in sequence: {}",
    //         potential_object_index
    //     );
    // }

    Ok(())
}

#[derive(Parser)]
struct UnpackOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,

    /// output file
    #[arg(short, long)]
    output_path: String,

    /// total block count, as specified by the LF
    #[arg(short, long)]
    total_block_count: usize,
}

fn process_unpack(unpack_opts: UnpackOpts) -> anyhow::Result<()> {
    let mut file = File::open(&unpack_opts.input_path).expect("Failed to open source file");

    let loi_archive: loi::Loi = loi::Loi::read(&mut file, unpack_opts.total_block_count)
        .expect("Failed to parse source file");

    let out_path = Path::new(&unpack_opts.output_path);

    {
        let json_file = File::create(out_path).expect("Failed to open target file");
        serde_json::to_writer_pretty(json_file, &loi_archive)
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

    let loi: loi::Loi = {
        let manifest_file = File::open(input_path)?;
        serde_json::from_reader(manifest_file)?
    };

    let mut out_file = BufWriter::new(File::create(pack_opts.output_path)?);
    loi.write(&mut out_file)?;

    Ok(())
}

#[derive(Parser)]
struct GltfOpts {
    /// path to object0.loI
    #[arg(short, long)]
    loi_path: String,

    /// path to modeltable0.LOF
    #[arg(short, long)]
    lof_path: String,

    /// output file
    #[arg(short, long)]
    output_path: String,

    /// total block count, as specified by the LF
    #[arg(short, long)]
    total_block_count: usize,
}

fn process_gltf(gltf_opts: GltfOpts) -> anyhow::Result<()> {
    let mut file = File::open(&gltf_opts.loi_path)?;
    let loi: loi::Loi = loi::Loi::read(&mut file, gltf_opts.total_block_count)?;

    let (mut gltf, model_indices) =
        crate::lof::process_gltf_inner(&gltf_opts.lof_path, None).expect("failed to process lof");

    let mut instance_indices = Vec::new();

    for block in loi.blocks {
        for block_object in block.objects {
            let &model_node_index = model_indices
                .get(&block_object.model_table_index)
                .expect("couldn't find model");
            instance_indices.push(gltf.clone_node(
                model_node_index,
                Some([
                    block_object.position.0,
                    block_object.position.1,
                    block_object.position.2,
                ]),
                Some([
                    block_object.rotation.0 .0,
                    block_object.rotation.0 .1,
                    block_object.rotation.0 .2,
                    block_object.rotation.1 .0,
                    block_object.rotation.1 .1,
                    block_object.rotation.1 .2,
                    block_object.rotation.2 .0,
                    block_object.rotation.2 .1,
                    block_object.rotation.2 .2,
                ]),
                Some(block_object.scale),
            ));
        }
    }

    gltf.get_or_create_scene("Instanced Objects", Some(instance_indices));

    let gltf_path = std::path::PathBuf::from(gltf_opts.output_path);
    gltf.write_to_files(gltf_path)?;

    Ok(())
}

pub fn process_loi(loi_opts: LoiOpts) -> anyhow::Result<()> {
    match loi_opts.cmd {
        Command::Info(info_opts) => process_info(info_opts),
        Command::Unpack(unpack_opts) => process_unpack(unpack_opts),
        Command::Pack(pack_opts) => process_pack(pack_opts),
        Command::Gltf(gltf_opts) => process_gltf(gltf_opts),
    }
}
