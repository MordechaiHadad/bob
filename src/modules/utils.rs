use crate::models::Version;
use anyhow::{anyhow, Result};
use dirs::data_local_dir;
use regex::Regex;
use reqwest::Client;
use std::path::PathBuf;
use tokio::{fs, process::Command};

pub async fn parse_version(client: &Client, version: &str) -> Result<String> {
    match version {
        "nightly" => Ok(String::from(version)),
        "stable" => {
            let response = client
                .get("https://api.github.com/repos/neovim/neovim/releases/latest")
                .header("user-agent", "bob")
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await?
                .text()
                .await?;

            let latest: Version = serde_json::from_str(&response)?;

            Ok(latest.tag_name)
        }
        _ => {
            let regex = Regex::new(r"^v?[0-9]+\.[0-9]+\.[0-9]+$").unwrap();
            if regex.is_match(version) {
                let mut returned_version = String::from(version);
                if !version.contains('v') {
                    returned_version.insert(0, 'v');
                }
                return Ok(returned_version);
            }
            Err(anyhow!("Please provide a proper version string"))
        }
    }
}

pub async fn get_downloads_folder() -> Result<PathBuf> {
    let data_dir = match data_local_dir() {
        None => return Err(anyhow!("Couldn't get local data folder")),
        Some(value) => value,
    };
    let path_string = &format!("{}/bob", data_dir.to_str().unwrap());
    let does_folder_exist = tokio::fs::metadata(path_string).await.is_ok();

    if !does_folder_exist && tokio::fs::create_dir(path_string).await.is_err() {
        return Err(anyhow!("Couldn't create downloads directory"));
    }
    Ok(PathBuf::from(path_string))
}

pub fn get_installation_folder() -> Result<PathBuf> {
    let data_dir = match data_local_dir() {
        None => return Err(anyhow!("Couldn't get local data folder")),
        Some(value) => value,
    };
    cfg_if::cfg_if! {
        if #[cfg(windows)] {

            let full_path = &format!("{}\\neovim", data_dir.to_str().unwrap());

            Ok(PathBuf::from(full_path))
        } else {
            let full_path = &format!("{}/neovim", data_dir.to_str().unwrap());
            Ok(PathBuf::from(full_path))
        }
    }
}

pub fn get_file_type() -> &'static str {
    if cfg!(target_family = "windows") {
        "zip"
    } else {
        "tar.gz"
    }
}

pub async fn is_version_installed(version: &str) -> bool {
    let downloads_dir = get_downloads_folder().await.unwrap();
    fs::metadata(format!("{}/{version}", downloads_dir.display()))
        .await
        .is_ok()
}

pub async fn is_version_used(version: &str) -> bool {
    let installed_version = match get_current_version().await {
        None => return false,
        Some(value) => value,
    };

    installed_version.contains(version)
}

pub async fn get_current_version() -> Option<String> {
    let output = match Command::new("nvim").arg("--version").output().await {
        Ok(value) => value,
        Err(_) => return None,
    };
    let output = String::from_utf8_lossy(&*output.stdout).to_string();
    if output.contains("dev") {
        return Some(String::from("nightly"));
    }
    let regex = Regex::new(r"v[0-9]\.[0-9]\.[0-9]").unwrap();
    Some(regex.find(output.as_str()).unwrap().as_str().to_owned())
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

pub async fn get_upstream_nightly(client: &Client) -> Version {
    let response = client
        .get("https://api.github.com/repos/neovim/neovim/releases/tags/nightly")
        .header("user-agent", "bob")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    serde_json::from_str(&response).unwrap()
}

pub async fn get_local_nightly() -> Result<Version> {
    let downloads_dir = get_downloads_folder().await.unwrap();
    if let Ok(file) =
        fs::read_to_string(format!("{}/nightly/bob.json", downloads_dir.display())).await
    {
        let file_json: Version = serde_json::from_str(&file).unwrap();
        Ok(file_json)
    } else {
        Err(anyhow!("Couldn't find bob.json"))
    }
}
