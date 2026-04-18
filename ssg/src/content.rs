use anyhow::Result;
use memmap2::Mmap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn walk_dir(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(walk_dir(&path)?);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }

    Ok(files)
}

pub fn copy_assets(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_assets(&path, &dest_path)?;
        } else {
            let file = fs::File::open(&path)?;
            let metadata = file.metadata()?;
            if metadata.len() > 0 {
                let mmap = unsafe { Mmap::map(&file)? };
                fs::write(dest_path, &mmap[..])?;
            } else {
                fs::copy(path, dest_path)?;
            }
        }
    }
    Ok(())
}
