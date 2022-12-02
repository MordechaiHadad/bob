use super::utils;
use crate::enums::{InstallResult, PostDownloadVersionType, VersionType};
use crate::models::{Config, InputVersion, LocalVersion, UpstreamVersion};
use crate::modules::expand_archive;
use crate::modules::utils::handle_subprocess;
use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::cmp::min;
use std::env;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::{fs, process::Command};
use tracing::info;
use yansi::Paint;

pub async fn start(
    version: &InputVersion,
    client: &Client,
    config: &Config,
) -> Result<InstallResult> {
    let root = match utils::get_downloads_folder(config).await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };
    env::set_current_dir(&root)?;
    let root = root.as_path();

    let is_version_installed = utils::is_version_installed(&version.tag_name, config).await?;

    let nightly_version = if version.tag_name == "nightly" {
        let upstream_nightly = match utils::get_upstream_nightly(client).await {
            Ok(value) => value,
            Err(error) => return Err(error),
        };
        if is_version_installed {
            info!("Looking for nightly updates...");
            let local_nightly = utils::get_local_nightly(config).await?;

            match config.enable_nightly_info {
                Some(boolean) if boolean => {
                    print_commits(client, &local_nightly, &upstream_nightly).await?
                }
                None => print_commits(client, &local_nightly, &upstream_nightly).await?,
                _ => (),
            }

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

    let downloaded_file = match download_version(client, version, root, config).await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };

    if let PostDownloadVersionType::Standard(downloaded_file) = downloaded_file {
        if let Err(error) = expand_archive::start(downloaded_file).await {
            return Err(anyhow!(error));
        }
    }

    if let Some(nightly_version) = nightly_version {
        let nightly_string = serde_json::to_string(&nightly_version)?;
        let mut file = match fs::File::create("nightly/bob.json").await {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow!(
                    "Failed to create file nightly/bob.json, reason: {error}"
                ))
            }
        };
        file.write_all(nightly_string.as_bytes()).await?;
    }
    Ok(InstallResult::InstallationSuccess(
        root.display().to_string(),
    ))
}

async fn print_commits(
    client: &Client,
    local: &UpstreamVersion,
    upstream: &UpstreamVersion,
) -> Result<()> {
    let commits =
        utils::get_commits_for_nightly(client, &local.published_at, &upstream.published_at).await?;

    for commit in commits {
        println!(
            "| {} {}\n",
            Paint::blue(commit.commit.author.name).bold(),
            commit.commit.message.replace('\n', "\n| ")
        );
    }

    Ok(())
}

async fn download_version(
    client: &Client,
    version: &InputVersion,
    root: &Path,
    config: &Config,
) -> Result<PostDownloadVersionType> {
    match version.version_type {
        VersionType::Standard => {
            let response = send_request(client, &version.tag_name).await;

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
                        pb.set_message(format!("Downloading version: {}", version.tag_name));

                        let file_type = utils::get_file_type();
                        let mut file =
                            tokio::fs::File::create(format!("{}.{file_type}", version.tag_name))
                                .await?;

                        let mut downloaded: u64 = 0;

                        while let Some(item) = response_bytes.next().await {
                            let chunk = item.or(Err(anyhow!("hello")))?;
                            file.write(&chunk).await?;
                            let new = min(downloaded + (chunk.len() as u64), total_size);
                            downloaded = new;
                            pb.set_position(new);
                        }

                        pb.finish_with_message(format!(
                            "Downloaded version {} to {}/{}.{file_type}",
                            version.tag_name,
                            root.display(),
                            version.tag_name
                        ));

                        Ok(PostDownloadVersionType::Standard(LocalVersion {
                            file_name: version.tag_name.to_owned(),
                            file_format: file_type.to_string(),
                            path: root.display().to_string(),
                        }))
                    } else {
                        Err(anyhow!("Please provide an existing neovim version"))
                    }
                }
                Err(error) => Err(anyhow!(error)),
            }
        }
        VersionType::Hash => handle_building_from_source(version, config).await,
    }
}

async fn handle_building_from_source(
    version: &InputVersion,
    config: &Config,
) -> Result<PostDownloadVersionType> {
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            if let Err(_) = env::var("VisualStudioVersion") {
                return Err(anyhow!("Please make sure you are using Developer PowerShell/Command Prompt for VS"));
            }

        } else {
            let is_clang_present = match Command::new("clang").output().await {
                Ok(_) => true,
                Err(error) => match error.kind() {
                    std::io::ErrorKind::NotFound => false,
                    _ => true,
                },
            };
            let is_gcc_present = match Command::new("gcc").output().await {
                Ok(_) => true,
                Err(error) => match error.kind() {
                    std::io::ErrorKind::NotFound => false,
                    _ => true,
                },
            };
            if !is_gcc_present && !is_clang_present {
                return Err(anyhow!(
                    "Clang or GCC have to be installed in order to build neovim from source"
                ));
            }

        }

    }
    match Command::new("cmake").output().await {
        Ok(_) => (),
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => {
                return Err(anyhow!(
                    "Cmake has to be installed in order to build neovim from source"
                ))
            }
            _ => (),
        },
    }
    let (mut child, is_installed) = if fs::metadata("neovim-git").await.is_err() {
        // check if neovim-git
        // directory exists
        // to clone repo, else
        // git pull changes
        let child = match Command::new("git")
            .arg("clone")
            .arg("https://github.com/neovim/neovim")
            .arg("neovim-git")
            .spawn()
        {
            Ok(value) => value,
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {
                    return Err(anyhow!(
                        "Git has to be installed in order to build neovim from source"
                    ))
                }
                _ => return Err(anyhow!("Failed to clone neovim's repository")),
            },
        };
        (child, false)
    } else {
        env::set_current_dir("neovim-git")?; // cd into neovim-git
        let child = match Command::new("git").arg("pull").spawn() {
            Ok(value) => value,
            Err(_) => return Err(anyhow!("Failed to pull upstream updates")),
        };
        (child, true)
    };
    child.wait().await?;
    if !is_installed {
        env::set_current_dir("neovim-git")?; // cd into neovim-git
    }
    Command::new("git")
        .arg("checkout")
        .arg(&version.tag_name)
        .spawn()?
        .wait()
        .await?;

    if fs::metadata("build").await.is_ok() {
        utils::remove_dir("build").await?;
    }
    fs::create_dir("build").await?;

    let mut downloads_location = utils::get_downloads_folder(config).await?;
    downloads_location.push(&version.tag_name[0..7]);
    downloads_location.push(utils::get_platform_name());

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            if fs::metadata(".deps").await.is_ok() {
                utils::remove_dir(".deps").await?;
            }
            fs::create_dir(".deps").await?;
            env::set_current_dir(".deps")?;
            handle_subprocess(Command::new("cmake").arg("../cmake.deps")).await?;
            handle_subprocess(Command::new("cmake").arg("--build").arg(".")).await?;

            let current_dir = env::current_dir()?;
            let parent = current_dir.parent().unwrap();
            env::set_current_dir(parent.join("build"))?;

            handle_subprocess(Command::new("cmake").arg("..")).await?;
            handle_subprocess(Command::new("cmake").arg("--build").arg(".")).await?;
            handle_subprocess(Command::new("cmake").arg("--install").arg(".").arg("--prefix").arg(downloads_location)).await?;
        } else {
            let location_arg = format!(
                "CMAKE_INSTALL_PREFIX={}",
                downloads_location.to_string_lossy()
            );
            handle_subprocess(Command::new("make").arg(&location_arg).arg("CMAKE_BUILD_TYPE=RelWithDebInfo")).await?;
            handle_subprocess(Command::new("make").arg("install")).await?;
        }
    }
    Ok(PostDownloadVersionType::Hash)
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
