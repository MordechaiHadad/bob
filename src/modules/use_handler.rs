use super::utils;
use crate::models::DownloadedVersion;
use crate::modules::expand_archive;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use futures_util::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::cmp::min;
use std::env;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn start(command: &ArgMatches) -> Result<()> {
    let client = Client::new();
    let version = if let Some(value) = command.value_of("VERSION") {
        match utils::parse_version(&client, value).await {
            Ok(version) => version,
            Err(error) => return Err(anyhow!(error)),
        }
    } else {
        return Err(anyhow!("Todo.."));
    };

    let root = match utils::get_downloads_folder().await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };
    env::set_current_dir(&root)?;
    let root = root.as_path();

    if utils::is_version_installed(&version).await {
        println!("{version} is already installed");
        return Ok(());
    }

    let mut does_folder_exist = utils::does_folder_exist(&version, root).await;
    if version == "nightly" && does_folder_exist {
        fs::remove_dir_all(format!("{}/nightly", root.display())).await?;
        does_folder_exist = false;
    }

    if !does_folder_exist {
        let downloaded_file = match download_version(&client, &version, root).await {
            Ok(value) => value,
            Err(error) => return Err(anyhow!(error)),
        };

        if let Err(error) = expand_archive::start(downloaded_file).await {
            return Err(anyhow!(error));
        }
    }

    if let Err(error) = link_version(&version).await {
        return Err(anyhow!(error));
    }
    println!("You can now open neovim");
    Ok(())
}

async fn link_version(version: &str) -> Result<()> {
    use dirs::data_dir;

    let installation_dir = match data_dir() {
        None => return Err(anyhow!("Couldn't get data dir")),
        Some(value) => value,
    };

    if utils::does_folder_exist("Neovim", installation_dir.as_path()).await {
        fs::remove_dir_all(format!("{}/Neovim", installation_dir.display())).await?;
    }

    if cfg!(target_family = "windows") {
        use std::os::windows::fs::symlink_dir;

        if let Err(error) = symlink_dir(
            format!(
                "{}/{}/Neovim",
                env::current_dir().unwrap().display(),
                version
            ),
            format!("{}/Neovim", installation_dir.display()),
        ) {
            return Err(anyhow!(error));
        }
    } // TODO: Unix

    println!("Linked {version} to {}/Neovim", installation_dir.display());

    if !utils::is_version_installed(version).await {
        println!(
            "Add {}/Neovim/bin to PATH to complete this installation",
            installation_dir.display()
        ); // TODO: do this automatically
    }
    Ok(())
}

async fn download_version(
    client: &Client,
    version: &String,
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
                    file_name: version.clone(),
                    file_format: file_type,
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
    let os = if cfg!(target_os = "linux") {
        "linux64"
    } else if cfg!(target_os = "windows") {
        "win64"
    } else {
        "macos"
    };
    let request_url = format!(
        "https://github.com/neovim/neovim/releases/download/{version}/nvim-{os}.{}",
        utils::get_file_type()
    );

    client
        .get(request_url)
        .header("user-agent", "bob")
        .send()
        .await
}
