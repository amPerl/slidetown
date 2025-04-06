use std::{
    fs::{read_dir, DirEntry},
    path::Path,
};

pub fn visit_files(dir: &Path, mut cb: &mut dyn FnMut(DirEntry)) -> anyhow::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_files(&path, &mut cb)?;
        } else {
            cb(entry);
        }
    }
    Ok(())
}

pub fn visit_dirs(dir: &Path, mut cb: &mut dyn FnMut(DirEntry)) -> anyhow::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            cb(entry);
            visit_dirs(&path, &mut cb)?;
        }
    }
    Ok(())
}
