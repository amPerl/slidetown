use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct WorldOpts {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "display info about world")]
    Info(InfoOpts),
    #[command(about = "print object density map for world")]
    Map(InfoOpts),
}

#[derive(Parser)]
struct InfoOpts {
    /// input directory
    #[arg(short, long)]
    input_path: String,
}

fn try_parse_nifs<I, T>(file: &mut File, named_offsets: I) -> anyhow::Result<()>
where
    I: Iterator<Item = (T, u32, u32)>,
    T: std::fmt::Debug,
{
    let mut nif_errors = Vec::new();
    let mut count = 0;

    for (idx, pos, len) in named_offsets {
        count += 1;

        file.seek(SeekFrom::Start(pos as u64))?;
        let mut buf = vec![0u8; len as usize];
        file.read_exact(&mut buf)?;

        let mut nif_cursor = Cursor::new(buf);

        if let Err(e) = nif::Nif::parse(&mut nif_cursor) {
            println!("Failed to parse nif id {:?} - {:#?}", idx, e);
            nif_errors.push(e);

            // dump for debug
            file.seek(SeekFrom::Start(pos as u64))
                .expect("failed to seek for dump");
            let mut buf = vec![0u8; len as usize];
            file.read_exact(&mut buf).expect("failed to read for dump");
            let mut dump_file =
                std::fs::File::create("dump.nif").expect("failed to create file for dump");
            dump_file.write_all(&buf).expect("failed to write for dump");
            panic!("dumped file");
        }
    }

    let parsed_nifs = count - nif_errors.len();

    println!(
        "[try_parse_nifs] Parsed {} nifs of {} ({}%)",
        parsed_nifs,
        count,
        (parsed_nifs as f32) / (count as f32) * 100.0
    );

    Ok(())
}

fn process_info(info_opts: InfoOpts) -> anyhow::Result<()> {
    let input_path = Path::new(&info_opts.input_path);

    let mut lf_file = File::open(input_path.join("terrain0.lf"))?;
    let lf = slidetown::parsers::lf::Lf::read_without_data(&mut lf_file)?;

    println!(
        "[lf] Terrain dimensions: {:?}",
        (lf.size_x, lf.size_y, lf.size_idx)
    );
    println!("[lf] Blocks in terrain header: {}", lf.block_count);
    println!("[lf] Blocks in terrain: {}", lf.blocks.len());

    println!("[lf] Parsing nifs..");
    try_parse_nifs(
        &mut lf_file,
        lf.blocks
            .iter()
            .filter(|b| b.file_length > 0)
            .map(|b| (b.index, b.file_offset, b.file_length)),
    )?;

    let mut lbf_file = File::open(input_path.join("blockObj0.lbf"))?;
    let lbf = slidetown::parsers::lbf::Lbf::parse(&mut lbf_file)?;

    println!(
        "[lbf] Blocks in blockObj header: {}",
        lbf.header.block_count
    );
    println!("[lbf] Blocks in blockObj: {}", lbf.blocks.len());
    println!(
        "[lbf] Block objects in blockObj header: {}",
        lbf.header.block_object_count
    );
    println!(
        "[lbf] Block objects in blockObj: {}",
        lbf.blocks
            .iter()
            .map(|b: &slidetown::parsers::lbf::Block| b.object_count)
            .sum::<u32>()
    );

    println!("[lbf] Parsing nifs..");
    try_parse_nifs(
        &mut lbf_file,
        lbf.blocks.iter().flat_map(|b| {
            b.objects
                .iter()
                .map(|bo| (bo.unk, bo.file_offset, bo.file_length))
        }),
    )?;

    let mut lof_file = File::open(input_path.join("modeltable0.lof"))?;
    let lof = slidetown::parsers::lof::Lof::read_without_data(&mut lof_file)?;

    println!("[lof] Models in table: {}", lof.models.len());

    println!("[lof] Parsing nifs..");
    try_parse_nifs(
        &mut lof_file,
        lof.models
            .iter()
            .map(|m| (&m.file_name, m.file_offset, m.file_length)),
    )?;

    let mut loi_file = File::open(input_path.join("Main\\object0.loI"))?;
    let loi = slidetown::parsers::loi::Loi::read(&mut loi_file, lf.block_count as _)?;

    println!("[loi] Blocks in object index: {}", loi.blocks.len());
    println!(
        "[loi] Blocks with 1 or more objects: {}",
        loi.blocks.iter().filter(|b| !b.objects.is_empty()).count()
    );
    println!(
        "[loi] Objects in all blocks combined: {}",
        loi.blocks.iter().map(|b| b.objects.len()).sum::<usize>()
    );

    Ok(())
}

fn process_map(info_opts: InfoOpts) -> anyhow::Result<()> {
    let input_path = Path::new(&info_opts.input_path);

    let lf = {
        let mut file = File::open(input_path.join("terrain0.lf"))?;
        slidetown::parsers::lf::Lf::read_without_data(&mut file)?
    };

    let loi = {
        let mut file = File::open(input_path.join("Main\\object0.loI"))?;
        slidetown::parsers::loi::Loi::read(&mut file, lf.block_count as _)?
    };

    println!("Object count by block:");
    print!("    ");
    for x in 0..lf.size_x {
        match x % 10 {
            0 => print!("{}", x / 10),
            _ => print!(" "),
        }
    }
    println!();
    print!("    ");
    for x in 0..lf.size_x {
        print!("{}", x % 10);
    }
    println!();
    for y in 0..lf.size_y {
        let row_offset = y * lf.size_x;

        let row_counts: Vec<usize> = (row_offset..row_offset + lf.size_x)
            .map(|i| loi.blocks.get(i as usize).map_or(0, |b| b.objects.len()))
            .collect();

        if row_counts.iter().sum::<usize>() == 0 {
            continue;
        }

        print!("{:03} ", y);
        for x in 0..lf.size_x {
            let object_count = row_counts.get(x as usize).unwrap();
            match object_count {
                0 => print!("_"),
                1..=9 => print!("{}", object_count),
                _ => print!("+"),
            };
        }
        println!();
    }

    Ok(())
}

pub fn process_world(world_opts: WorldOpts) -> anyhow::Result<()> {
    match world_opts.cmd {
        Command::Info(info_opts) => process_info(info_opts),
        Command::Map(info_opts) => process_map(info_opts),
    }
}
