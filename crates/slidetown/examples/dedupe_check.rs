//! Check `(path, decompressed_length)` deduplication against NIF content hashes
//! across archive variants.
//!
//! False merges undercount: one key matches different contents.
//! False splits overcount: identical content appears under different paths.

use slidetown::agt::AgtReader;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::{Hash, Hasher},
};

static SPOOKY_KEY: &[u8] = include_bytes!("../resources/agt/spooky_key.bin");

fn content_hash(bytes: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

fn main() -> anyhow::Result<()> {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        anyhow::bail!("usage: dedupe_check <agt>...");
    }

    // (path, len) -> set of content hashes
    let mut key_to_hashes: HashMap<(String, u32), HashSet<u64>> = HashMap::new();
    // content hash -> set of normalised paths
    let mut hash_to_paths: HashMap<u64, HashSet<String>> = HashMap::new();
    let mut total_nifs = 0usize;

    for agt_path in &paths {
        let mut file = File::open(agt_path)?;
        let mut reader = AgtReader::new(&mut file, SPOOKY_KEY);
        let header = reader.read_header()?;
        let entries = reader.read_entries(header.file_count)?;

        let mut n = 0usize;
        for entry in entries.iter() {
            if !entry.path.to_lowercase().ends_with(".nif") {
                continue;
            }
            let data = reader.read_entry_data(entry)?;
            let norm = entry.path.replace('\\', "/").to_lowercase();
            let h = content_hash(&data);

            key_to_hashes
                .entry((norm.clone(), entry.decompressed_length))
                .or_default()
                .insert(h);
            hash_to_paths.entry(h).or_default().insert(norm);
            n += 1;
            total_nifs += 1;
        }
        println!("  {} -> {} nifs", agt_path, n);
    }

    let distinct_by_key = key_to_hashes.len();
    let distinct_by_content = hash_to_paths.len();

    let false_merge_keys: usize = key_to_hashes.values().filter(|s| s.len() > 1).count();
    let lost_to_merge: usize = key_to_hashes
        .values()
        .filter(|s| s.len() > 1)
        .map(|s| s.len() - 1)
        .sum();
    let false_split: usize = hash_to_paths.values().filter(|s| s.len() > 1).count();

    println!();
    println!("nif instances read          {}", total_nifs);
    println!("distinct by (path, len)     {}", distinct_by_key);
    println!("distinct by content hash    {}", distinct_by_content);
    println!();
    println!(
        "false merges: {} key(s) covered >1 content, hiding {} distinct file(s)",
        false_merge_keys, lost_to_merge
    );
    println!(
        "false splits: {} content hash(es) appeared under >1 path",
        false_split
    );

    let err = distinct_by_key as f64 / distinct_by_content.max(1) as f64;
    println!();
    println!(
        "(path,len) estimate is {:.1}% of the true distinct count",
        err * 100.0
    );

    Ok(())
}
