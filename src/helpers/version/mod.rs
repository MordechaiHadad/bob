pub mod nightly;
pub mod types;

use self::types::{ParsedVersion, VersionType};
use super::directories;
use crate::{
    config::Config,
    github_requests::{deserialize_response, RepoCommit, UpstreamVersion},
};
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use reqwest::Client;
use semver::Version;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::info;

pub async fn parse_version_type(client: &Client, version: &str) -> Result<ParsedVersion> {
    match version {
        "nightly" => Ok(ParsedVersion {
            tag_name: version.to_string(),
            version_type: VersionType::Nightly,
            non_parsed_string: version.to_string(),
            semver: None,
        }),
        "stable" | "latest" => {
            info!("Fetching latest version");
            let stable_version = search_stable_version(client).await?;
            let cloned_version = stable_version.clone();
            Ok(ParsedVersion {
                tag_name: stable_version,
                version_type: VersionType::Latest,
                non_parsed_string: version.to_string(),
                semver: Some(Version::parse(&cloned_version.replace('v', ""))?),
            })
        }
        "head" | "git" => {
            info!("Fetching latest commit");
            let latest_commit = get_latest_commit(client).await?;
            Ok(ParsedVersion {
                tag_name: latest_commit.chars().take(7).collect(),
                version_type: VersionType::Hash,
                non_parsed_string: version.to_string(),
                semver: None,
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
                let cloned_version = returned_version.clone();
                return Ok(ParsedVersion {
                    tag_name: returned_version,
                    version_type: VersionType::Normal,
                    non_parsed_string: version.to_string(),
                    semver: Some(Version::parse(&cloned_version.replace('v', ""))?),
                });
            } else if hash_regex.is_match(version) {
                return Ok(ParsedVersion {
                    tag_name: version.to_string().chars().take(7).collect(),
                    version_type: VersionType::Hash,
                    non_parsed_string: version.to_string(),
                    semver: None,
                });
            }

            let rollback_regex = Regex::new(r"nightly-[a-zA-Z0-9]{7,8}")?;

            if rollback_regex.is_match(version) {
                return Ok(ParsedVersion {
                    tag_name: version.to_string(),
                    version_type: VersionType::NightlyRollback,
                    non_parsed_string: version.to_string(),
                    semver: None,
                });
            }

            Err(anyhow!("Please provide a proper version string"))
        }
    }
}

pub async fn get_version_sync_file_location(config: &Config) -> Result<Option<PathBuf>> {
    let path = match &config.version_sync_file_location {
        Some(path) => {
            let path = Path::new(path);
            if tokio::fs::metadata(path).await.is_err() {
                let mut file = File::create(path).await.context(format!("The path provided, \"{}\", does not exist. Please check the path and try again.", path.parent().unwrap().display()))?;
                file.write_all(b"").await?;
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
        if !version.eq(&name) {
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

async fn search_stable_version(client: &Client) -> Result<String> {
    let response = client
        .get("https://api.github.com/repos/neovim/neovim/releases?per_page=10")
        .header("user-agent", "bob")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .text()
        .await?;

    let versions: Vec<UpstreamVersion> = deserialize_response(response)?;
    let stable_release = versions
        .iter()
        .find(|v| v.tag_name == "stable")
        .ok_or(anyhow!("Cannot find stable release"))?;
    let stable_pin_release = versions
        .iter()
        .find(|v| v.tag_name != "stable" && v.target_commitish == stable_release.target_commitish)
        .ok_or(anyhow!("Cannot find version of stable release"))?;
    Ok(stable_pin_release.tag_name.clone())
}

async fn get_latest_commit(client: &Client) -> Result<String> {
    let response = client
        .get("https://api.github.com/repos/neovim/neovim/commits/master")
        .header("user-agent", "bob")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .text()
        .await?;

    let commit: RepoCommit = deserialize_response(response)?;

    Ok(commit.sha)
}
