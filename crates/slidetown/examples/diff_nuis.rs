use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

fn collect_local_paths(local_dir: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(local_dir)
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
        .collect()
}

fn main() -> anyhow::Result<()> {
    let root_dir_str = std::env::args().nth(1).unwrap();
    let root_dir = Path::new(&root_dir_str);

    let mut dialog_controls_seen: HashMap<usize, HashSet<usize>> = HashMap::new();

    for client_dir in std::fs::read_dir(root_dir)?.filter_map(Result::ok) {
        let client_dir_path = client_dir.path();
        let client_name = client_dir_path.file_name().unwrap().to_str().unwrap();
        println!("\n** Client: {} **\n", client_name);

        let client_files = collect_local_paths(&client_dir_path);
        for client_file_path in client_files {
            let file_name_lower = client_file_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_lowercase();

            if file_name_lower.contains("game.nui") {
                let nui_file = std::fs::File::open(client_file_path)?;
                let nui_doc = slidetown::parsers::nui::Document::from_read(nui_file)?;

                if nui_doc.component_list.is_some() {
                    continue;
                }

                for dialog in nui_doc.dialog_list.dialogs.iter() {
                    let mut controls_by_id = HashMap::new();
                    for control in dialog.controls.iter() {
                        controls_by_id.insert(control.id, control);
                    }
                    let controls_by_id_set: HashSet<usize> =
                        controls_by_id.keys().cloned().collect::<HashSet<_>>();

                    if let Some(existing_controls) = dialog_controls_seen.get_mut(&dialog.id) {
                        let old_controls = existing_controls.clone();
                        let new_controls = controls_by_id_set
                            .difference(&old_controls)
                            .cloned()
                            .collect::<Vec<usize>>();

                        if !new_controls.is_empty() {
                            let mut new_controls_summary = Vec::new();

                            for &new_control in new_controls.iter() {
                                existing_controls.insert(new_control);

                                if let Some(text_id) =
                                    &controls_by_id.get(&new_control).unwrap().text_id
                                {
                                    new_controls_summary.push(text_id);
                                }
                            }
                            println!(
                                "Dialog {} new named controls ({}/{}): {:?}",
                                dialog.id,
                                new_controls_summary.len(),
                                new_controls.len(),
                                new_controls_summary
                            );
                        }
                    } else {
                        let named_controls = dialog
                            .controls
                            .iter()
                            .filter_map(|c| c.text_id.clone())
                            .collect::<Vec<String>>();

                        println!(
                            "NEW! Dialog {} with named controls ({}/{}): {:?}",
                            dialog.id,
                            named_controls.len(),
                            controls_by_id_set.len(),
                            named_controls
                        );
                        dialog_controls_seen.insert(dialog.id, controls_by_id_set);
                    }
                }
            }
        }

        // break;
    }

    Ok(())
}
