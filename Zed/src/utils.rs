use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn get_absolute_path<P: AsRef<Path>>(rel_path: P) -> std::io::Result<PathBuf> {
    let root = std::env::current_dir()?;
    Ok(root.join(rel_path))
}

pub(crate) fn is_file<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).map_or(false, |stat| stat.is_file())
}
