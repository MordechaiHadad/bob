use super::utils;
use crate::models::{DownloadedVersion, Version};
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
use tracing::info;

pub async fn start(version: &str, client: &Client, via_use: bool) -> Result<()> {
    let root = match utils::get_downloads_folder().await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };
    env::set_current_dir(&root)?;
    let root = root.as_path();

    let nightly_version = if version == "nightly" {
        let response = client
            .get("https://api.github.com/repos/neovim/neovim/releases/tags/nightly")
            .header("user-agent", "bob")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?
            .text()
            .await?;
        let nightly: Version = serde_json::from_str(&response)?;
        if let Ok(file) = fs::read_to_string("nightly/bob.json").await {
            let file_json: Version = serde_json::from_str(&file)?;
            if nightly.published_at != file_json.published_at {
                fs::remove_dir_all(format!("{}/nightly", root.display())).await?;
            }
        }
        Some(nightly)
    } else {
        None
    };

    let is_version_installed = utils::is_version_installed(version, root).await;
    if !via_use && is_version_installed {
        info!("{version} is already installed");
        return Ok(());
    }
    if !is_version_installed {
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
        if !via_use {
            info!("Successfully installed version: {version}");
        }
    }

    Ok(())
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
