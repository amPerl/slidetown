//! Inventory NIFs across client roots, with optional container indexing and extraction.
//!
//! Group AGTs by lowercase filename and byte length, then read one entry table per group.
//! Record table fingerprints (paths, offsets, lengths); other copies are only tried on failure,
//! so matching names and sizes do not prove identical contents.
//! The default pass avoids decompression; container indexing and extraction read payloads.
//!
//! Usage:
//!   cargo run --release --example nif_gather -- <root>... [--out <dir>]

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    hash::{Hash, Hasher},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use slidetown::{
    agt::AgtReader,
    parsers::{agt::Entry, lbf::Lbf, lf::Lf, lof::Lof},
};

/// An unextracted NIF in an LF, LBF, or LOF container.
struct Embedded {
    /// Deduplication ID, used like `Entry::path` for loose NIFs.
    /// Example: `moonpalace.agt/moonpalace/terrain0.lf#block42`.
    id: String,
    /// Absolute offset within the decompressed container.
    offset: u32,
    length: u32,
}

/// Lowercase hex matching `sha256sum` filenames.
fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Store each NIF once by content hash and record its sources.
/// `(path, length)` merges same-length edits, finding only 66% of distinct game.agt NIFs.
#[allow(clippy::too_many_arguments)]
fn store_nif(
    extract_dir: &Option<PathBuf>,
    index: &mut HashMap<String, u64>,
    sources: &mut Option<BufWriter<File>>,
    data: &[u8],
    agt_name: &str,
    agt_len: u64,
    kind: &str,
    source_id: &str,
) -> anyhow::Result<()> {
    use sha2::{Digest, Sha256};

    let Some(dir) = extract_dir else {
        return Ok(());
    };

    let digest = hex(&Sha256::digest(data));

    if !index.contains_key(&digest) {
        // Shard by the first hash byte to avoid a directory with 50k files.
        let shard = dir.join("store").join(&digest[0..2]);
        std::fs::create_dir_all(&shard)?;
        let path = shard.join(format!("{}.nif", digest));
        if !path.exists() {
            std::fs::write(&path, data)?;
        }
        index.insert(digest.clone(), data.len() as u64);
    }

    if let Some(w) = sources.as_mut() {
        writeln!(
            w,
            "{}\t{}\t{}\t{}\t{}\t{}",
            digest,
            data.len(),
            agt_name,
            agt_len,
            kind,
            source_id
        )?;
    }

    Ok(())
}

fn container_kind(path: &str) -> Option<&'static str> {
    match path.rsplit('.').next()?.to_lowercase().as_str() {
        "lf" => Some("lf"),
        "lbf" => Some("lbf"),
        "lof" => Some("lof"),
        _ => None,
    }
}

/// List embedded NIFs using index offsets and lengths, without reading their payloads.
fn index_container(kind: &str, data: &[u8], name: &str) -> anyhow::Result<Vec<Embedded>> {
    // The parser handles empty LBF stubs with 0xFFFFFFFF counts.
    // Report any remaining parse errors.
    let mut cursor = std::io::Cursor::new(data);
    let mut out = Vec::new();

    match kind {
        "lf" => {
            let lf = Lf::read_without_data(&mut cursor)?;
            for b in &lf.blocks {
                if b.file_length > 0 {
                    out.push(Embedded {
                        id: format!("{}#block{}", name, b.index),
                        offset: b.file_offset,
                        length: b.file_length,
                    });
                }
            }
        }
        "lbf" => {
            let lbf = Lbf::parse(&mut cursor)?;
            for (bi, block) in lbf.blocks.iter().enumerate() {
                for (oi, obj) in block.objects.iter().enumerate() {
                    if obj.file_length > 0 {
                        out.push(Embedded {
                            id: format!("{}#b{}o{}i{}", name, bi, oi, obj.block_index),
                            offset: obj.file_offset,
                            length: obj.file_length,
                        });
                    }
                }
            }
        }
        "lof" => {
            let lof = Lof::read_without_data(&mut cursor)?;
            for m in &lof.models {
                if m.file_length > 0 {
                    // Prefer model names to indices.
                    let label = if m.file_name.is_empty() {
                        format!("model{}", m.index)
                    } else {
                        m.file_name.replace('\\', "/").to_lowercase()
                    };
                    out.push(Embedded {
                        id: format!("{}#{}", name, label),
                        offset: m.file_offset,
                        length: m.file_length,
                    });
                }
            }
        }
        _ => anyhow::bail!("unknown container kind {}", kind),
    }

    Ok(out)
}

static SPOOKY_KEY: &[u8] = include_bytes!("../resources/agt/spooky_key.bin");

/// Assumed archive identity: filename and byte length.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct AgtKey {
    name: String,
    len: u64,
}

struct AgtGroup {
    key: AgtKey,
    /// Archive paths in discovery order.
    paths: Vec<PathBuf>,
    outcome: Outcome,
}

enum Outcome {
    Pending,
    Parsed {
        /// Entry-table fingerprint of the parsed copy.
        fingerprint: u64,
        entries: Vec<Entry>,
        /// Number of non-UTF-8 paths decoded lossily.
        lossy_paths: usize,
    },
    Failed(String),
}

fn is_agt(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("agt"))
        .unwrap_or(false)
}

fn is_nif(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .map(|e| e.eq_ignore_ascii_case("nif"))
        .unwrap_or(false)
}

/// Normalise internal paths for comparison across archives.
fn norm(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

fn fingerprint(entries: &[Entry]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    entries.len().hash(&mut h);
    for e in entries {
        e.path.hash(&mut h);
        e.chunks_offset.hash(&mut h);
        e.chunk_count.hash(&mut h);
        e.decompressed_length.hash(&mut h);
    }
    h.finish()
}

fn read_entries(path: &Path) -> anyhow::Result<(Vec<Entry>, usize)> {
    let mut file = File::open(path)?;
    let file_count = {
        let mut reader = AgtReader::new(&mut file, SPOOKY_KEY);
        reader.read_header()?.file_count
    };
    if file_count == 0 {
        return Ok((Vec::new(), 0));
    }

    // Try the standard reader with strict UTF-8 decoding first.
    {
        let mut reader = AgtReader::new(&mut file, SPOOKY_KEY);
        if let Ok(entries) = reader.read_entries(file_count) {
            return Ok((entries, 0));
        }
    }

    // Some clients have non-UTF-8 paths, possibly EUC-KR or Big5.
    // Decode lossily so one invalid name does not exclude the whole archive.
    read_entries_lossy(&mut file, file_count)
}

fn read_entries_lossy(file: &mut File, file_count: u32) -> anyhow::Result<(Vec<Entry>, usize)> {
    const ENTRY_FIXED: u64 = 16;
    const MAX_PATH: u64 = 260;
    const TABLE_START: u64 = 32;

    let file_len = file.metadata()?.len();
    let bound = TABLE_START + (file_count as u64).saturating_mul(ENTRY_FIXED + MAX_PATH);
    let end = bound.min(file_len);
    if end <= TABLE_START {
        anyhow::bail!("no entry table");
    }

    let mut buf = vec![0u8; (end - TABLE_START) as usize];
    file.seek(SeekFrom::Start(TABLE_START))?;
    file.read_exact(&mut buf)?;
    for (i, b) in buf.iter_mut().enumerate() {
        *b ^= SPOOKY_KEY[(TABLE_START as usize + i) % SPOOKY_KEY.len()];
    }

    let u32_at = |buf: &[u8], o: usize| -> u32 {
        u32::from_le_bytes([buf[o], buf[o + 1], buf[o + 2], buf[o + 3]])
    };

    let mut entries = Vec::with_capacity(file_count as usize);
    let mut lossy = 0usize;
    let mut off = 0usize;

    for i in 0..file_count {
        if off + 16 > buf.len() {
            anyhow::bail!("entry table truncated at entry {}", i);
        }
        let chunks_offset = u32_at(&buf, off);
        let chunk_count = u32_at(&buf, off + 4);
        let decompressed_length = u32_at(&buf, off + 8);
        let path_len = u32_at(&buf, off + 12) as usize;
        off += 16;

        if path_len > MAX_PATH as usize || off + path_len > buf.len() {
            anyhow::bail!("implausible path length {} at entry {}", path_len, i);
        }
        let raw = &buf[off..off + path_len];
        off += path_len;

        let path = match std::str::from_utf8(raw) {
            Ok(s) => s.to_string(),
            Err(_) => {
                lossy += 1;
                String::from_utf8_lossy(raw).into_owned()
            }
        };

        entries.push(Entry {
            chunks_offset,
            chunk_count,
            decompressed_length,
            path,
        });
    }

    Ok((entries, lossy))
}

fn main() -> anyhow::Result<()> {
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut out_dir = PathBuf::from("nif-gather-out");
    let mut index_containers = true;
    let mut extract_dir: Option<PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-containers" => index_containers = false,
            "--extract" => {
                extract_dir = Some(
                    args.next()
                        .map(PathBuf::from)
                        .ok_or_else(|| anyhow::anyhow!("--extract needs a directory"))?,
                );
            }
            "--out" => {
                out_dir = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("--out needs a directory"))?;
            }
            other => roots.push(PathBuf::from(other)),
        }
    }

    if roots.is_empty() {
        anyhow::bail!("usage: nif_gather <root>... [--out <dir>]");
    }

    // Find archives.

    eprintln!("scanning {} root(s)...", roots.len());
    let mut groups: BTreeMap<AgtKey, AgtGroup> = BTreeMap::new();
    let mut walk_errors = 0usize;
    let mut total_agt_files = 0usize;

    for root in &roots {
        for entry in walkdir::WalkDir::new(root).into_iter() {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => {
                    walk_errors += 1;
                    continue;
                }
            };
            if !entry.file_type().is_file() || !is_agt(entry.path()) {
                continue;
            }
            let len = match entry.metadata() {
                Ok(m) => m.len(),
                Err(_) => {
                    walk_errors += 1;
                    continue;
                }
            };
            let name = entry
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_lowercase();

            total_agt_files += 1;
            let key = AgtKey { name, len };
            groups
                .entry(key.clone())
                .or_insert_with(|| AgtGroup {
                    key,
                    paths: Vec::new(),
                    outcome: Outcome::Pending,
                })
                .paths
                .push(entry.path().to_path_buf());
        }
    }

    eprintln!(
        "found {} agt file(s), {} unique by (name, size)",
        total_agt_files,
        groups.len()
    );
    if walk_errors > 0 {
        eprintln!("  ({} path(s) unreadable during walk)", walk_errors);
    }

    // Parse one copy per archive group.

    let total_groups = groups.len();
    for (i, group) in groups.values_mut().enumerate() {
        if i % 25 == 0 {
            eprintln!("  parsing {}/{}...", i, total_groups);
        }
        // Try another copy if reading one fails.
        let mut last_err = None;
        for path in &group.paths {
            match read_entries(path) {
                Ok((entries, lossy_paths)) => {
                    group.outcome = Outcome::Parsed {
                        fingerprint: fingerprint(&entries),
                        entries,
                        lossy_paths,
                    };
                    last_err = None;
                    break;
                }
                Err(e) => last_err = Some(e.to_string()),
            }
        }
        if let Some(err) = last_err {
            group.outcome = Outcome::Failed(err);
        }
    }

    // Index embedded NIFs in LF/LBF/LOF containers, which outnumber loose NIFs.
    // This requires decompression because container indices are inside the payloads.

    struct ContainerResult {
        agt_name: String,
        agt_len: u64,
        path: String,
        kind: &'static str,
        outcome: Result<Vec<Embedded>, String>,
    }
    let mut containers: Vec<ContainerResult> = Vec::new();

    // SHA-256 -> byte length; counts distinct contents, unlike the (path, len) estimate.
    let mut store_index: HashMap<String, u64> = HashMap::new();
    let mut sources_out = match &extract_dir {
        Some(dir) => {
            std::fs::create_dir_all(dir)?;
            std::fs::create_dir_all(dir.join("store"))?;
            let mut w = BufWriter::new(File::create(dir.join("nif_sources.tsv"))?);
            writeln!(w, "sha256\tbytes\tagt_name\tagt_size\tkind\tsource_id")?;
            Some(w)
        }
        None => None,
    };
    let mut extracted_instances = 0usize;
    let mut extract_errors = 0usize;

    if index_containers || extract_dir.is_some() {
        let agt_jobs: Vec<(AgtKey, PathBuf, Vec<Entry>)> = groups
            .values()
            .filter_map(|g| match &g.outcome {
                Outcome::Parsed { entries, .. } => {
                    Some((g.key.clone(), g.paths.first()?.clone(), entries.clone()))
                }
                _ => None,
            })
            .collect();

        eprintln!(
            "payload pass over {} archive(s){}...",
            agt_jobs.len(),
            if extract_dir.is_some() {
                " (extracting)"
            } else {
                " (indexing only)"
            }
        );

        // Reduce binrw backtraces to one line.
        let brief = |e: anyhow::Error| -> String {
            e.to_string()
                .lines()
                .map(|l| {
                    l.trim_matches(|c: char| !c.is_ascii_graphic() && c != ' ')
                        .trim()
                })
                .find(|l| !l.is_empty() && !l.starts_with('╺') && !l.contains("Backtrace"))
                .unwrap_or("unknown error")
                .chars()
                .take(200)
                .collect()
        };

        for (i, (key, agt_path, entries)) in agt_jobs.iter().enumerate() {
            eprintln!("  [{}/{}] {}", i + 1, agt_jobs.len(), key.name);

            let mut file = match File::open(agt_path) {
                Ok(f) => f,
                Err(_) => {
                    extract_errors += 1;
                    continue;
                }
            };
            // Reuse one reader per archive and hash embedded NIFs from the decompressed buffer.
            let mut reader = AgtReader::new(&mut file, SPOOKY_KEY);

            for entry in entries {
                let is_container = container_kind(&entry.path);
                let want_loose = extract_dir.is_some() && is_nif(&entry.path);
                if is_container.is_none() && !want_loose {
                    continue;
                }

                let data = match reader.read_entry_data(entry) {
                    Ok(d) => d,
                    Err(_) => {
                        extract_errors += 1;
                        continue;
                    }
                };

                if want_loose {
                    if let Err(_) = store_nif(
                        &extract_dir,
                        &mut store_index,
                        &mut sources_out,
                        &data,
                        &key.name,
                        key.len,
                        "loose",
                        &norm(&entry.path),
                    ) {
                        extract_errors += 1;
                    } else {
                        extracted_instances += 1;
                    }
                }

                if let Some(kind) = is_container {
                    // Include the archive name and full internal path: worlds can share block indices.
                    // Omit archive size so client variants of the same archive can deduplicate.
                    let scoped = format!("{}/{}", key.name, norm(&entry.path));
                    let outcome = index_container(kind, &data, &scoped).map_err(brief);

                    if let (Ok(list), true) = (&outcome, extract_dir.is_some()) {
                        for emb in list {
                            let start = emb.offset as usize;
                            let end = start.saturating_add(emb.length as usize);
                            if end > data.len() {
                                extract_errors += 1;
                                continue;
                            }
                            if store_nif(
                                &extract_dir,
                                &mut store_index,
                                &mut sources_out,
                                &data[start..end],
                                &key.name,
                                key.len,
                                kind,
                                &emb.id,
                            )
                            .is_err()
                            {
                                extract_errors += 1;
                            } else {
                                extracted_instances += 1;
                            }
                        }
                    }

                    containers.push(ContainerResult {
                        agt_name: key.name.clone(),
                        agt_len: key.len,
                        path: entry.path.clone(),
                        kind,
                        outcome,
                    });
                }
            }
        }
    }

    if let Some(w) = sources_out.as_mut() {
        w.flush()?;
    }

    // Check for conflicting fingerprints per (name, size).
    // Only one copy per group is parsed, so this cannot detect differences between copies.

    let mut by_name_size: HashMap<(String, u64), HashSet<u64>> = HashMap::new();
    for group in groups.values() {
        if let Outcome::Parsed { fingerprint, .. } = &group.outcome {
            by_name_size
                .entry((group.key.name.clone(), group.key.len))
                .or_default()
                .insert(*fingerprint);
        }
    }
    let dedupe_collisions: Vec<_> = by_name_size
        .iter()
        .filter(|(_, fps)| fps.len() > 1)
        .collect();

    // Write manifests.

    std::fs::create_dir_all(&out_dir)?;

    let mut agts_out = BufWriter::new(File::create(out_dir.join("agts.tsv"))?);
    writeln!(
        agts_out,
        "agt_name\tagt_size\tcopies\tentries\tnif_entries\tnif_bytes\tstatus\texample_path"
    )?;

    let mut nifs_out = BufWriter::new(File::create(out_dir.join("nif_entries.tsv"))?);
    writeln!(
        nifs_out,
        "agt_name\tagt_size\tentry_path\tdecompressed_length\tchunk_count"
    )?;

    // Record source directories to distinguish shipped assets from personal or community edits.
    // Those edits may not reflect what the original game loads.
    let mut paths_out = BufWriter::new(File::create(out_dir.join("agt_paths.tsv"))?);
    writeln!(paths_out, "agt_name\tagt_size\tpath")?;
    for group in groups.values() {
        for path in &group.paths {
            writeln!(
                paths_out,
                "{}\t{}\t{}",
                group.key.name,
                group.key.len,
                path.display()
            )?;
        }
    }
    paths_out.flush()?;

    let mut parsed = 0usize;
    let mut failed = 0usize;
    let mut total_lossy_paths = 0usize;
    let mut total_entries = 0usize;
    let mut total_nif_entries = 0usize;
    let mut total_nif_bytes = 0u64;
    // (normalised path, decompressed length) -> archive count; estimates extraction size only.
    // This can overcount or undercount distinct NIFs: value-only edits keep the same length,
    // and identical contents can have different paths.
    // Across 47 game.agt variants, dedupe_check found 200 keys versus 302 content hashes (66%),
    // with no false splits.
    let mut distinct_nifs: HashMap<(String, u32), usize> = HashMap::new();
    let mut per_archive_name: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    // Track all extensions to identify other NIF container formats.
    let mut by_ext: BTreeMap<String, (usize, u64)> = BTreeMap::new();

    for group in groups.values() {
        let example = group
            .paths
            .first()
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        match &group.outcome {
            Outcome::Parsed {
                entries,
                lossy_paths,
                ..
            } => {
                parsed += 1;
                total_lossy_paths += lossy_paths;
                let nif_entries: Vec<&Entry> = entries.iter().filter(|e| is_nif(&e.path)).collect();
                let nif_bytes: u64 = nif_entries
                    .iter()
                    .map(|e| e.decompressed_length as u64)
                    .sum();

                total_entries += entries.len();
                total_nif_entries += nif_entries.len();
                total_nif_bytes += nif_bytes;

                for e in entries {
                    let ext = e
                        .path
                        .rsplit('.')
                        .next()
                        .filter(|ext| ext.len() <= 8 && !ext.contains('/') && !ext.contains('\\'))
                        .unwrap_or("(none)")
                        .to_lowercase();
                    let slot = by_ext.entry(ext).or_insert((0, 0));
                    slot.0 += 1;
                    slot.1 += e.decompressed_length as u64;
                }

                let stat = per_archive_name
                    .entry(group.key.name.clone())
                    .or_insert((0, 0));
                stat.0 += 1;
                stat.1 += nif_entries.len();

                for e in &nif_entries {
                    *distinct_nifs
                        .entry((norm(&e.path), e.decompressed_length))
                        .or_default() += 1;
                    writeln!(
                        nifs_out,
                        "{}\t{}\t{}\t{}\t{}",
                        group.key.name, group.key.len, e.path, e.decompressed_length, e.chunk_count
                    )?;
                }

                writeln!(
                    agts_out,
                    "{}\t{}\t{}\t{}\t{}\t{}\tok\t{}",
                    group.key.name,
                    group.key.len,
                    group.paths.len(),
                    entries.len(),
                    nif_entries.len(),
                    nif_bytes,
                    example
                )?;
            }
            Outcome::Failed(err) => {
                failed += 1;
                writeln!(
                    agts_out,
                    "{}\t{}\t{}\t\t\t\terr: {}\t{}",
                    group.key.name,
                    group.key.len,
                    group.paths.len(),
                    err.replace('\t', " ").replace('\n', " "),
                    example
                )?;
            }
            Outcome::Pending => {
                failed += 1;
                writeln!(
                    agts_out,
                    "{}\t{}\t{}\t\t\t\tnot attempted\t{}",
                    group.key.name,
                    group.key.len,
                    group.paths.len(),
                    example
                )?;
            }
        }
    }

    agts_out.flush()?;
    nifs_out.flush()?;

    // Print summary.

    println!();
    println!("== archives ==");
    println!("  agt files on disk       {}", total_agt_files);
    println!("  unique by (name, size)  {}", groups.len());
    println!("  parsed ok               {}", parsed);
    println!("  failed                  {}", failed);
    if total_lossy_paths > 0 {
        println!(
            "  non-utf8 entry paths    {} (decoded lossily; strict reader would drop the archive)",
            total_lossy_paths
        );
    }
    if dedupe_collisions.is_empty() {
        println!("  size-dedupe holds       yes (no fingerprint collisions)");
    } else {
        println!(
            "  size-dedupe BROKE       {} (name, size) group(s) had differing entry tables:",
            dedupe_collisions.len()
        );
        for ((name, len), fps) in dedupe_collisions.iter().take(10) {
            println!("      {} ({} bytes): {} distinct", name, len, fps.len());
        }
    }

    println!();
    println!("== nif entries ==");
    println!("  total entries           {}", total_entries);
    println!("  nif entries             {}", total_nif_entries);
    println!(
        "  distinct (path, len)    {}   <- ESTIMATE ONLY (see note)",
        distinct_nifs.len()
    );
    println!(
        "  decompressed nif bytes  {:.2} GiB (before dedupe)",
        total_nif_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    let distinct_bytes: u64 = distinct_nifs.keys().map(|(_, len)| *len as u64).sum();
    println!(
        "  after (path,len) dedupe  {:.2} GiB   <- roughly what stage 2 extracts",
        distinct_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    );

    {
        let mut ext_out = BufWriter::new(File::create(out_dir.join("extensions.tsv"))?);
        writeln!(ext_out, "ext\tentries\tdecompressed_bytes")?;
        for (ext, (n, bytes)) in &by_ext {
            writeln!(ext_out, "{}\t{}\t{}", ext, n, bytes)?;
        }
        ext_out.flush()?;
    }

    // Write container manifests.

    let mut embedded_total = 0usize;
    let mut embedded_bytes = 0u64;
    let mut distinct_embedded: HashSet<(String, u32)> = HashSet::new();
    let mut container_failures: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_kind: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();

    if index_containers {
        let mut cont_out = BufWriter::new(File::create(out_dir.join("containers.tsv"))?);
        writeln!(
            cont_out,
            "agt_name\tagt_size\tcontainer_path\tkind\tembedded\tembedded_bytes\tstatus"
        )?;
        let mut emb_out = BufWriter::new(File::create(out_dir.join("embedded_nifs.tsv"))?);
        writeln!(
            emb_out,
            "agt_name\tagt_size\tcontainer_path\tkind\tembedded_id\tlength"
        )?;

        for c in &containers {
            match &c.outcome {
                Ok(list) => {
                    let bytes: u64 = list.iter().map(|e| e.length as u64).sum();
                    embedded_total += list.len();
                    embedded_bytes += bytes;
                    let slot = per_kind.entry(c.kind).or_insert((0, 0));
                    slot.0 += 1;
                    slot.1 += list.len();
                    for e in list {
                        distinct_embedded.insert((e.id.clone(), e.length));
                        writeln!(
                            emb_out,
                            "{}\t{}\t{}\t{}\t{}\t{}",
                            c.agt_name, c.agt_len, c.path, c.kind, e.id, e.length
                        )?;
                    }
                    writeln!(
                        cont_out,
                        "{}\t{}\t{}\t{}\t{}\t{}\tok",
                        c.agt_name,
                        c.agt_len,
                        c.path,
                        c.kind,
                        list.len(),
                        bytes
                    )?;
                }
                Err(err) => {
                    *container_failures.entry(err.clone()).or_default() += 1;
                    writeln!(
                        cont_out,
                        "{}\t{}\t{}\t{}\t\t\terr: {}",
                        c.agt_name,
                        c.agt_len,
                        c.path,
                        c.kind,
                        err.replace('\t', " ")
                    )?;
                }
            }
        }
        cont_out.flush()?;
        emb_out.flush()?;

        println!();
        println!("== nif containers ==");
        println!("  containers indexed      {}", containers.len());
        for (kind, (n, nifs)) in &per_kind {
            println!("    .{:<4} {:>4} ok, {:>7} embedded nifs", kind, n, nifs);
        }
        println!("  embedded nifs           {}", embedded_total);
        println!(
            "  distinct (id, len)      {}   <- ESTIMATE ONLY",
            distinct_embedded.len()
        );
        println!(
            "  embedded bytes          {:.2} GiB",
            embedded_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
        );
        if !container_failures.is_empty() {
            println!("  failures:");
            for (err, n) in &container_failures {
                println!("    {:>4}x {}", n, err);
            }
        }
        println!();
        println!(
            "  GRAND TOTAL distinct nifs {} (loose {} + embedded {})",
            distinct_nifs.len() + distinct_embedded.len(),
            distinct_nifs.len(),
            distinct_embedded.len()
        );
        println!("  ^ ESTIMATE ONLY, unreliable in BOTH directions. It merges same-length edits");
        println!("    (undercounts: 66% of truth on game.agt across 47 variants) and splits");
        println!("    identical content sitting under two ids (overcounts: this figure came out");
        println!("    45% high against sha256 over the full corpus). Use --extract for the truth.");
    }

    if let Some(dir) = &extract_dir {
        let store_bytes: u64 = store_index.values().sum();
        println!();
        println!("== extraction (content-addressed) ==");
        println!("  nif instances written   {}", extracted_instances);
        println!(
            "  DISTINCT BY SHA-256     {}   <- the real count",
            store_index.len()
        );
        println!(
            "  store size              {:.2} GiB",
            store_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
        );
        if extracted_instances > 0 {
            println!(
                "  dedupe ratio            {:.1}x",
                extracted_instances as f64 / store_index.len().max(1) as f64
            );
        }
        if extract_errors > 0 {
            println!("  errors                  {}", extract_errors);
        }
        println!("  store                   {}", dir.join("store").display());

        let mut nifs_out = BufWriter::new(File::create(dir.join("nifs.tsv"))?);
        writeln!(nifs_out, "sha256\tbytes")?;
        let mut sorted: Vec<_> = store_index.iter().collect();
        sorted.sort();
        for (digest, bytes) in sorted {
            writeln!(nifs_out, "{}\t{}", digest, bytes)?;
        }
        nifs_out.flush()?;
        println!("  wrote {}", dir.join("nifs.tsv").display());
        println!("  wrote {}", dir.join("nif_sources.tsv").display());
    }

    println!();
    println!("== entries by extension (top 20 by count) ==");
    let mut exts: Vec<_> = by_ext.iter().collect();
    exts.sort_by_key(|(_, (n, _))| std::cmp::Reverse(*n));
    for (ext, (n, bytes)) in exts.iter().take(20) {
        let note = match ext.as_str() {
            "lf" | "lbf" | "lof" => "  <- NIF CONTAINER",
            "nif" => "  <- loose nif",
            _ => "",
        };
        println!(
            "  .{:<10} {:>7} entries  {:>9.1} MiB{}",
            ext,
            n,
            *bytes as f64 / (1024.0 * 1024.0),
            note
        );
    }

    println!();
    println!("== archives carrying nifs ==");
    let mut ranked: Vec<_> = per_archive_name
        .iter()
        .filter(|(_, (_, nifs))| *nifs > 0)
        .collect();
    ranked.sort_by_key(|(_, (_, nifs))| std::cmp::Reverse(*nifs));
    for (name, (variants, nifs)) in ranked.iter().take(25) {
        println!(
            "  {:<24} {:>7} nif entries across {} variant(s)",
            name, nifs, variants
        );
    }

    println!();
    println!("wrote {}", out_dir.join("agts.tsv").display());
    println!("wrote {}", out_dir.join("nif_entries.tsv").display());

    Ok(())
}
