pub mod types;
pub mod nightly;

use self::types::{ParsedVersion, UpstreamVersion, VersionType};
use super::directories;
use crate::config::Config;
use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs;

pub async fn parse_version_type(client: &Client, version: &str) -> Result<ParsedVersion> {
    match version {
        "nightly" => Ok(ParsedVersion {
            tag_name: version.to_string(),
            version_type: VersionType::Standard,
        }),
        "stable" | "latest" => {
            let response = client
                .get("https://api.github.com/repos/neovim/neovim/releases?per_page=2")
                .header("user-agent", "bob")
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await?
                .text()
                .await?;

            let versions: Vec<UpstreamVersion> = serde_json::from_str(&response)?;

            Ok(ParsedVersion {
                tag_name: versions[1].tag_name.clone(),
                version_type: VersionType::Standard,
            })
        }
        _ => {
            let version_regex = Regex::new(r"^v?[0-9]+\.[0-9]+\.[0-9]+$")?;
            let hash_regex = Regex::new(r"\b[0-9a-f]{5,40}\b")?;
            if version_regex.is_match(version) {
                let mut returned_version = version.to_string();
                if !version.contains('v') {
                    returned_version.insert(0, 'v');
                }
                return Ok(ParsedVersion {
                    tag_name: returned_version,
                    version_type: VersionType::Standard,
                });
            } else if hash_regex.is_match(version) {
                return Ok(ParsedVersion {
                    tag_name: version.to_string(),
                    version_type: VersionType::Hash,
                });
            }
            Err(anyhow!("Please provide a proper version string"))
        }
    }
}

pub async fn get_sync_version_file_path(config: &Config) -> Result<Option<PathBuf>> {
    let path = match &config.version_sync_file_location {
        Some(path) => {
            if let Err(e) = tokio::fs::metadata(path).await {
                return Err(anyhow!(
                    "Error when trying to retrieve sync_version_file_path {path}: {e}"
                ));
            }
            Some(PathBuf::from(path))
        }
        None => return Ok(None),
    };

    Ok(path)
}

pub async fn is_version_installed(version: &str, config: &Config) -> Result<bool> {
    let downloads_dir = directories::get_downloads_directory(config).await?;
    let mut dir = tokio::fs::read_dir(&downloads_dir).await?;

    while let Some(directory) = dir.next_entry().await? {
        let name = directory.file_name().to_str().unwrap().to_owned();
        if !version.contains(&name) {
            continue;
        } else {
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn get_current_version(config: &Config) -> Result<String> {
    let mut downloads_dir = directories::get_downloads_directory(config).await?;
    downloads_dir.push("used");
    fs::read_to_string(&downloads_dir).await
        .map_err(|_| anyhow!("The used file required for bob could not be found. This could mean that Neovim is not installed through bob."))
}

pub async fn is_version_used(version: &str, config: &Config) -> bool {
    match get_current_version(config).await {
        Ok(value) => value.eq(version),
        Err(_) => false,
    }
}
