use chrono::{DateTime, Datelike, Utc};
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let root_dir_str = std::env::args().nth(1).unwrap();
    let root_dir = Path::new(&root_dir_str);

    for client_dir in std::fs::read_dir(root_dir)?.filter_map(Result::ok) {
        let client_dir_path = client_dir.path();
        let client_name = client_dir_path.file_name().unwrap().to_str().unwrap();
        println!("Client: {}", client_name);

        rename_client(&client_dir_path)?;
    }

    Ok(())
}

fn rename_client(client_dir_path: &Path) -> anyhow::Result<()> {
    let client_name = client_dir_path.file_name().unwrap();

    let mut latest_file: Option<(String, DateTime<Utc>)> = None;
    let mut locales = Vec::new();

    for dir in std::fs::read_dir(&client_dir_path)?.filter_map(Result::ok) {
        let dir_path = dir.path();
        let dir_path_str = dir_path.file_name().unwrap().to_string_lossy();
        let dir_path_str_lower = dir_path_str.to_lowercase();

        if dir_path.is_dir() && dir_path_str_lower.contains("data_") {
            let name_split = dir_path_str.split('_');
            for thing in name_split.skip(1) {
                locales.push(thing.to_string());
            }
        }
    }

    let client_files = collect_local_paths(&client_dir_path);
    for client_file_path in client_files {
        let file_name_lower = client_file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase();

        if file_name_lower.contains(".agt")
            || file_name_lower.contains("patch.0")
            || file_name_lower.contains("game.nui")
        {
            let meta = client_file_path.metadata().unwrap();
            let modified = meta.modified().unwrap();
            let modified_chrono: DateTime<Utc> = modified.into();

            if let Some(latest_file_inner) = &latest_file {
                if latest_file_inner.1 < modified_chrono {
                    latest_file = Some((file_name_lower.clone(), modified_chrono));
                }
            } else {
                latest_file = Some((file_name_lower.clone(), modified_chrono));
            }

            // dbg!((file_name_lower, modified_chrono));
        }
    }

    let (latest_file_name, latest_file_time) = latest_file.unwrap();

    println!(
        "{} - {}, {}",
        client_name.to_string_lossy(),
        latest_file_name,
        latest_file_time
    );

    let mut old_file_name = client_dir_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    if old_file_name.contains("__") {
        old_file_name = old_file_name.split("__").last().unwrap().to_string();
    }

    let mut new_file_name_prefix = format!(
        "{}_{:02}_{:02}",
        latest_file_time.year(),
        latest_file_time.month(),
        latest_file_time.day(),
    );

    for locale in locales.iter() {
        new_file_name_prefix = format!("{}_{}", new_file_name_prefix, &locale);
    }

    if locales.is_empty() {
        new_file_name_prefix = format!("{}_{}", new_file_name_prefix, "KR");
    }

    let new_file_name = format!("{}__{}", new_file_name_prefix, old_file_name);

    if old_file_name != new_file_name {
        println!("Renaming:\n\t{}\nto\t{}", old_file_name, new_file_name);
        let new_file_path = client_dir_path.with_file_name(new_file_name);
        std::fs::rename(&client_dir_path, new_file_path)?;
    }

    Ok(())
}

fn is_junk_file(path: &str) -> bool {
    if path.contains("_Log") {
        return true;
    }
    if path.contains("GameLog_") {
        return true;
    }
    if path.contains("ScreenShot") {
        return true;
    }
    if path.contains("desktop.ini") {
        return true;
    }
    if path.ends_with(".dmp") {
        return true;
    }
    if path.ends_with("play_record.r0") {
        return true;
    }
    if path.ends_with("goption.ini") {
        return true;
    }
    if path.ends_with("pc_spec.txt") {
        return true;
    }
    if path.ends_with("memmsg.log") {
        return true;
    }
    false
}

fn collect_local_paths(local_dir: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(&local_dir)
        .contents_first(true)
        .sort_by(|a, b| {
            let a_file = a.file_type().is_file();
            let b_file = b.file_type().is_file();

            if a_file == b_file {
                return std::cmp::Ordering::Equal;
            }

            if a_file {
                return std::cmp::Ordering::Less;
            }

            std::cmp::Ordering::Greater
        })
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|p| p.into_path())
        .filter(|p| !is_junk_file(&p.display().to_string()))
        .collect()
}
