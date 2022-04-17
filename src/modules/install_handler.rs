use super::utils;
use crate::enums::InstallResult;
use crate::models::DownloadedVersion;
use crate::modules::expand_archive;
use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::cmp::min;
use std::env;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn start(version: &str, client: &Client) -> Result<InstallResult> {
    let root = match utils::get_downloads_folder().await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };
    env::set_current_dir(&root)?;
    let root = root.as_path();

    let is_version_installed = utils::is_version_installed(version).await;

    let nightly_version = if version == "nightly" {
        let upstream_nightly = utils::get_upstream_nightly(client).await;
        if is_version_installed {
            let local_nightly = utils::get_local_nightly().await?;

            if local_nightly.published_at == upstream_nightly.published_at {
                return Ok(InstallResult::NightlyIsUpdated);
            }
        }
        Some(upstream_nightly)
    } else {
        if is_version_installed {
            return Ok(InstallResult::VersionAlreadyInstalled);
        }
        None
    };

    let downloaded_file = match download_version(client, version, root).await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };
    if let Err(error) = expand_archive::start(downloaded_file).await {
        return Err(anyhow!(error));
    }

    if let Some(nightly_version) = nightly_version {
        let nightly_string = serde_json::to_string(&nightly_version)?;
        let mut file = fs::File::create("nightly/bob.json").await?;
        file.write(nightly_string.as_bytes()).await?;
    }
    Ok(InstallResult::InstallationSuccess(
        root.display().to_string(),
    ))
}

async fn download_version(
    client: &Client,
    version: &str,
    root: &Path,
) -> Result<DownloadedVersion> {
    let response = send_request(client, version).await;

    match response {
        Ok(response) => {
            if response.status() == 200 {
                let total_size = response.content_length().unwrap();
                let mut response_bytes = response.bytes_stream();

                // Progress Bar Setup
                let pb = ProgressBar::new(total_size);
                pb.set_style(ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                    .progress_chars("█  "));
                pb.set_message(format!("Downloading version: {version}"));

                let file_type = utils::get_file_type();
                let mut file = tokio::fs::File::create(format!("{version}.{file_type}")).await?;

                let mut downloaded: u64 = 0;

                while let Some(item) = response_bytes.next().await {
                    let chunk = item.or(anyhow::private::Err(anyhow::Error::msg("hello")))?;
                    file.write(&chunk).await?;
                    let new = min(downloaded + (chunk.len() as u64), total_size);
                    downloaded = new;
                    pb.set_position(new);
                }

                pb.finish_with_message(format!(
                    "Downloaded version {version} to {}/{version}.{file_type}",
                    root.display()
                ));

                Ok(DownloadedVersion {
                    file_name: String::from(version),
                    file_format: file_type.to_string(),
                    path: root.display().to_string(),
                })
            } else {
                Err(anyhow!("Please provide an existing neovim version"))
            }
        }
        Err(error) => Err(anyhow!(error)),
    }
}

async fn send_request(client: &Client, version: &str) -> Result<reqwest::Response, reqwest::Error> {
    let platform = utils::get_platform_name();
    let file_type = utils::get_file_type();
    let request_url = format!(
        "https://github.com/neovim/neovim/releases/download/{version}/{platform}.{file_type}",
    );

    client
        .get(request_url)
        .header("user-agent", "bob")
        .send()
        .await
}
