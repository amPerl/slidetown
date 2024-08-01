use clap::{Parser, Subcommand};
use slidetown::parsers::agt;
use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
};

#[derive(Parser)]
pub struct AgtOpts {
    #[command(subcommand)]
    cmd: Command,
    // #[arg(short, long, about = "optional custom key file")]
    // key_path: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "display info about archive contents")]
    Info(InfoOpts),
}

#[derive(Parser)]
struct InfoOpts {
    /// input file
    #[arg(short, long)]
    input_path: String,
}

static SPOOKY_KEY: &[u8] = &[
    0x01, 0x05, 0x06, 0x02, 0x04, 0x03, 0x07, 0x08, 0x01, 0x05, 0x06, 0x0F, 0x04, 0x03, 0x07, 0x0C,
    0x31, 0x85, 0x76, 0x39, 0x34, 0x3D, 0x30, 0xE8, 0x67, 0x36, 0x36, 0x32, 0x3E, 0x33, 0x34, 0x3B,
    0x11, 0x15, 0x16, 0x16, 0x14, 0x13, 0x1D, 0x18, 0x11, 0x03, 0x06, 0x0C, 0x04, 0x03, 0x06, 0x08,
    0x2E, 0x55, 0x26, 0x23, 0x2A, 0x23, 0x2E, 0x28, 0x21, 0x21, 0x26, 0x27, 0x2E, 0x00, 0x2D, 0x2D,
    0xCF, 0xA5, 0x06, 0x02, 0x04, 0x0F, 0x07, 0x18, 0xE1, 0x15, 0x36, 0x18, 0x60, 0x13, 0x1A, 0x19,
    0x11, 0x15, 0x16, 0x10, 0x12, 0x13, 0x17, 0x38, 0xF1, 0x25,
];

fn process_info(info_opts: InfoOpts) -> anyhow::Result<()> {
    let mut file = File::open(info_opts.input_path)?;
    let header = agt::Header::parse(&mut file)?;

    let entry_count = header.file_count as usize;
    let max_entry_path_length = 260;
    let max_entry_size = 4 * 4 + max_entry_path_length;

    let entries_buffer_size = entry_count * max_entry_size;
    let mut entries_buffer = vec![0u8; entries_buffer_size];

    file.seek(SeekFrom::Start(32))?;
    file.read_exact(&mut entries_buffer)?;

    #[allow(clippy::needless_range_loop)]
    for i in 0..entries_buffer.len() {
        let file_index = i + 32;
        entries_buffer[i] ^= SPOOKY_KEY[file_index % SPOOKY_KEY.len()];
    }

    let mut entries_cursor = Cursor::new(entries_buffer);

    let entries = agt::Entry::parse_entries(&mut entries_cursor, entry_count as _)?;

    println!("Version: {:?}", header.version);

    if header.file_count == 0 {
        println!("No files");
    } else {
        println!("{} files:", header.file_count);
        for archive_file in entries {
            println!(
                "- {} ({} chunk(s), {} bytes decompressed)",
                archive_file.path, archive_file.chunk_count, archive_file.decompressed_length
            );
        }
    }

    Ok(())
}

pub fn process_agt(agt_opts: AgtOpts) -> anyhow::Result<()> {
    match agt_opts.cmd {
        Command::Info(info_opts) => process_info(info_opts),
    }
}
