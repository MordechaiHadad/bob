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
        if version
            .as_ref()
            .map_or(false, |x| x <= &Version::new(0, 9, 5))
        {
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
        if version
            .as_ref()
            .map_or(false, |x| x <= &Version::new(0, 9, 5))
        {
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

#[cfg(test)]
mod tests {

    #[test]
    fn get_platform_name_none() {
        if cfg!(target_os = "windows") {
            assert_eq!(super::get_platform_name(&None), "nvim-win64");
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            assert_eq!(super::get_platform_name(&None), "nvim-macos-arm64");
            assert_eq!(super::get_platform_name_download(&None), "nvim-macos-arm64");
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            assert_eq!(super::get_platform_name(&None), "nvim-macos-x86_64");
            assert_eq!(
                super::get_platform_name_download(&None),
                "nvim-macos-x86_64"
            );
        } else {
            assert_eq!(super::get_platform_name(&None), "nvim-linux64");
            assert_eq!(super::get_platform_name_download(&None), "nvim");
        }
    }

    #[test]
    fn get_platform_name_lower() {
        let version = Some(semver::Version::new(0, 9, 5));
        if cfg!(target_os = "windows") {
            assert_eq!(super::get_platform_name(&version), "nvim-win64");
        } else if cfg!(target_os = "macos") {
            assert_eq!(super::get_platform_name(&version), "nvim-macos");
            assert_eq!(super::get_platform_name_download(&version), "nvim-macos");
        } else {
            assert_eq!(super::get_platform_name(&None), "nvim-linux64");
            assert_eq!(super::get_platform_name_download(&None), "nvim");
        }
    }

    #[test]
    fn get_platform_name_higher() {
        let version = Some(semver::Version::new(0, 10, 5));
        if cfg!(target_os = "windows") {
            assert_eq!(super::get_platform_name(&version), "nvim-win64");
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            assert_eq!(super::get_platform_name(&version), "nvim-macos-arm64");
            assert_eq!(
                super::get_platform_name_download(&version),
                "nvim-macos-arm64"
            );
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            assert_eq!(super::get_platform_name(&version), "nvim-macos-x86_64");
            assert_eq!(
                super::get_platform_name_download(&version),
                "nvim-macos-x86_64"
            );
        } else {
            assert_eq!(super::get_platform_name(&version), "nvim-linux64");
            assert_eq!(super::get_platform_name_download(&version), "nvim");
        }
    }
}
