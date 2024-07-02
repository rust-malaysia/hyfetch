use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

pub fn get_cache_path() -> Result<PathBuf> {
    let path = ProjectDirs::from("", "", "hyfetch")
        .context("failed to get base dirs")?
        .cache_dir()
        .to_owned();
    Ok(path)
}
