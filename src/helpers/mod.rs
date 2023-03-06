pub mod filesystem;
pub mod directories;
pub mod version;
pub mod unarchive;

use anyhow::{anyhow, Result};
use tokio::process::Command;


pub fn get_file_type() -> &'static str {
    if cfg!(target_family = "windows") {
        "zip"
    } else {
        "tar.gz"
    }
}

pub fn get_platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "nvim-win64"
    } else if cfg!(target_os = "macos") {
        "nvim-macos"
    } else {
        "nvim-linux64"
    }
}

pub async fn handle_subprocess(process: &mut Command) -> Result<()> {
    match process.status().await?.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!(code)),
        None => Err(anyhow!("process terminated by signal")),
    }
}
