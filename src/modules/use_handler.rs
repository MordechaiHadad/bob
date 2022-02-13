use crate::models::{DownloadedVersion, StableVersion};
use crate::modules::expand_archive;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use futures_util::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::Client;
use std::cmp::min;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::fs;
use tokio::fs::{File, symlink_dir};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

pub async fn start(command: &ArgMatches) -> Result<()> {
    let client = Client::new();
    let version = if let Some(value) = command.value_of("VERSION") {
        match parse_version(&client, value).await {
            Ok(version) => version,
            Err(error) => return Err(anyhow!(error)),
        }
    } else {
        return Err(anyhow!("Todo.."));
    };

    let root = match get_downloads_folder().await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };
    env::set_current_dir(&root)?;
    let root = root.as_path();

    if !is_version_available(&version).await {
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
    Ok(())
}

async fn parse_version(client: &Client, version: &str) -> Result<String> {
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

async fn get_downloads_folder() -> Result<PathBuf> {
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

async fn is_version_available(version: &str) -> bool {
    let paths = tokio::task::spawn_blocking(move || std::fs::read_dir("./").unwrap())
        .await
        .unwrap();
    for path in paths {
        if path
            .unwrap()
            .file_name()
            .to_str()
            .unwrap()
            .contains(version)
        {
            return true;
        }
    }
    false
}

async fn link_version(version: &str) -> Result<()> {
    use dirs::data_dir;

    let installation_dir = match data_dir() {
        None => return Err(anyhow!("Couldn't get data dir")),
        Some(value) => value
    };

   if cfg!(target_family = "windows") {
       use std::os::windows::fs::symlink_dir;

       // TODO: check if version already linked

       if let Err(_) = symlink_dir(format!("{}/{}/Neovim", env::current_dir().unwrap().display(),version), format!("{}/Neovim", installation_dir.display())) {
           return Err(anyhow!("Couldn't install version, please run application as administrator to complete installation"))
       }
    } // TODO: Unix

    println!("Add {}/Neovim to PATH to complete this installation", installation_dir.display()); // TODO: do this automatically
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

                let file_type = get_file_type();
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

async fn send_request(
    client: &Client,
    version: &String,
) -> Result<reqwest::Response, reqwest::Error> {
    let os = if cfg!(target_os = "linux") {
        "linux64"
    } else if cfg!(target_os = "windows") {
        "win64"
    } else {
        "macos"
    };
    let request_url = format!(
        "https://github.com/neovim/neovim/releases/download/{version}/nvim-{os}.{}",
        get_file_type()
    );

    client
        .get(request_url)
        .header("user-agent", "bob")
        .send()
        .await
}

fn get_file_type() -> String {
    if cfg!(target_family = "windows") {
        String::from("zip")
    } else {
        String::from("tar.gz")
    }
}