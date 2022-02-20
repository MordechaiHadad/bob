use crate::models::StableVersion;
use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::Client;
use std::future::Future;
use std::path::{Path, PathBuf};
use tokio::process::Command;

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

            let latest: StableVersion = serde_json::from_str(response.as_str())?;

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
    use dirs::data_local_dir;
    let data_dir = match data_local_dir() {
        None => return Err(anyhow!("Couldn't get local data folder")),
        Some(value) => value,
    };
    let path_string = format!("{}/bob", data_dir.to_str().unwrap());
    let does_folder_exist = tokio::fs::metadata(path_string.clone()).await.is_ok();

    if !does_folder_exist {
        if let Err(error) = tokio::fs::create_dir(path_string.clone()).await {
            return Err(anyhow!(error));
        }
    }
    Ok(PathBuf::from(path_string))
}

pub async fn does_folder_exist(directory: &str, path: &Path) -> bool {
    let path = path.to_owned();
    let paths = tokio::task::spawn_blocking(move || std::fs::read_dir(path).unwrap())
        .await
        .unwrap();
    for path in paths {
        if path
            .unwrap()
            .file_name()
            .to_str()
            .unwrap()
            .contains(directory)
        {
            return true;
        }
    }
    false
}

pub fn get_file_type() -> String {
    if cfg!(target_family = "windows") {
        String::from("zip")
    } else {
        String::from("tar.gz")
    }
}

pub async fn is_version_installed(version: &str) -> bool {
    let installed_version = match get_current_version().await {
        None => return false,
        Some(value) => value,
    };

    if installed_version.contains(version) {
        return true;
    }
    false
}

pub async fn get_current_version() -> Option<String> {
    let output = match Command::new("nvim").arg("--version").output().await {
        Ok(value) => value,
        Err(_) => return None,
    };
    let regex = Regex::new(r"v[0-9]\.[0-9]\.[0-9]").unwrap();
    let output = String::from_utf8_lossy(&*output.stdout).to_string();
    Some(regex.find(output.as_str()).unwrap().as_str().to_owned())
}
