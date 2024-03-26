pub mod directories;
pub mod filesystem;
pub mod sync;
pub mod unarchive;
pub mod version;

use anyhow::{anyhow, Result};
use semver::Version;
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

pub fn get_platform_name(version: &Option<Version>) -> &'static str {
    if cfg!(target_os = "windows") {
        "nvim-win64"
    } else if cfg!(target_os = "macos") {
        if version.is_none() || version.as_ref().unwrap() >= &Version::new(0, 9, 5) {
            "nvim-macos"
        } else if cfg!(target_arch = "aarch64") {
            "nvim-macos-arm64"
        } else {
            "nvim-macos-x86_64"
        }
    } else {
        "nvim-linux64"
    }
}

pub fn get_platform_name_download(version: &Option<Version>) -> &'static str {
    if cfg!(target_os = "windows") {
        "nvim-win64"
    } else if cfg!(target_os = "macos") {
        if version.is_none() || version.as_ref().unwrap() >= &Version::new(0, 9, 5) {
            "nvim-macos"
        } else if cfg!(target_arch = "aarch64") {
            "nvim-macos-arm64"
        } else {
            "nvim-macos-x86_64"
        }
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
