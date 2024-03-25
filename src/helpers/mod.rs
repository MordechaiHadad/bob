pub mod directories;
pub mod filesystem;
pub mod sync;
pub mod unarchive;
pub mod version;

use anyhow::{anyhow, Result};
use tokio::process::Command;

pub fn get_file_type() -> &'static str {
    if cfg!(target_family = "windows") {
        "zip"
    } else if cfg!(target_os = "macos") {
        "tar.gz"
    } else {
        "appimage"
    }
}

pub fn get_platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "nvim-win64"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "arm64") {
        "nvim-macos-arm64"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        "nvim-macos-x86_64"
    } else {
        "nvim-linux64"
    }
}

pub fn get_platform_name_download() -> &'static str {
    if cfg!(target_os = "windows") {
        "nvim-win64"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "arm64") {
        "nvim-macos-arm64"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        "nvim-macos-x86_64"
    } else {
        "nvim"
    }
}

pub async fn handle_subprocess(process: &mut Command) -> Result<()> {
    match process.status().await?.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!(code)),
        None => Err(anyhow!("process terminated by signal")),
    }
}
