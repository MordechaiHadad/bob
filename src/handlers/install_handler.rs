use crate::config::Config;
use crate::github_requests::{get_commits_for_nightly, get_upstream_nightly, UpstreamVersion};
use crate::helpers::directories::get_downloads_directory;
use crate::helpers::processes::handle_subprocess;
use crate::helpers::version::nightly::produce_nightly_vec;
use crate::helpers::version::types::{LocalVersion, ParsedVersion, VersionType};
use crate::helpers::{self, directories, filesystem, unarchive};
use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use semver::Version;
use std::cmp::min;
use std::env;
use std::path::Path;
use std::process::Stdio;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::{fs, process::Command};
use tracing::info;
use yansi::Paint;

use super::{InstallResult, PostDownloadVersionType};

pub async fn start(
    version: &mut ParsedVersion,
    client: &Client,
    config: &Config,
) -> Result<InstallResult> {
    if version.version_type == VersionType::NightlyRollback {
        return Ok(InstallResult::GivenNightlyRollback);
    }

    if let Some(version) = &version.semver {
        if version <= &Version::new(0, 2, 2) {
            return Err(anyhow!("Versions below 0.2.2 are not supported"));
        }
    }

    let root = directories::get_downloads_directory(config).await?;

    env::set_current_dir(&root)?;
    let root = root.as_path();

    let is_version_installed =
        helpers::version::is_version_installed(&version.tag_name, config).await?;

    if is_version_installed && version.version_type != VersionType::Nightly {
        return Ok(InstallResult::VersionAlreadyInstalled);
    }

    let nightly_version = if version.version_type == VersionType::Nightly {
        Some(get_upstream_nightly(client).await?)
    } else {
        None
    };

    if is_version_installed && version.version_type == VersionType::Nightly {
        info!("Looking for nightly updates");

        let upstream_nightly = nightly_version.as_ref().unwrap();
        let local_nightly = helpers::version::nightly::get_local_nightly(config).await?;

        if upstream_nightly.published_at == local_nightly.published_at {
            return Ok(InstallResult::NightlyIsUpdated);
        }

        handle_rollback(config).await?;

        match config.enable_nightly_info {
            Some(boolean) if boolean => {
                print_commits(client, &local_nightly, upstream_nightly).await?
            }
            None => print_commits(client, &local_nightly, upstream_nightly).await?,
            _ => (),
        }
    }

    let downloaded_file = match version.version_type {
        VersionType::Normal | VersionType::Latest => {
            download_version(client, version, root, config).await
        }
        VersionType::Nightly => {
            if config.enable_release_build == Some(true) {
                handle_building_from_source(version, config).await
            } else {
                download_version(client, version, root, config).await
            }
        }
        VersionType::Hash => handle_building_from_source(version, config).await,
        VersionType::NightlyRollback => Ok(PostDownloadVersionType::None),
    }?;

    if let PostDownloadVersionType::Standard(downloaded_file) = downloaded_file {
        unarchive::start(downloaded_file).await?
    }

    if let VersionType::Nightly = version.version_type {
        if let Some(nightly_version) = nightly_version {
            let nightly_string = serde_json::to_string(&nightly_version)?;

            let mut downloads_dir = get_downloads_directory(config).await?;
            downloads_dir.push("nightly");
            downloads_dir.push("bob.json");
            let mut json_file = File::create(downloads_dir).await?;

            if let Err(error) = json_file.write_all(nightly_string.as_bytes()).await {
                return Err(anyhow!(
                    "Failed to create file nightly/bob.json, reason: {error}"
                ));
            }
        }
    }

    Ok(InstallResult::InstallationSuccess(
        root.display().to_string(),
    ))
}

async fn handle_rollback(config: &Config) -> Result<()> {
    if !helpers::version::is_version_used("nightly", config).await {
        return Ok(());
    }

    let rollback_limit = config.rollback_limit.unwrap_or(3);

    if rollback_limit == 0 {
        return Ok(());
    }

    let mut nightly_vec = produce_nightly_vec(config).await?;

    if nightly_vec.len() >= rollback_limit.into() {
        let oldest_path = nightly_vec.pop().unwrap().path;
        fs::remove_dir_all(oldest_path).await?;
    }

    // handle this for older installations of nightly instead of introducing breaking changes
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            use std::os::unix::prelude::PermissionsExt;

            let platform = helpers::get_platform_name(&None);
            let file = &format!("nightly/{platform}/bin/nvim");
            let mut perms = fs::metadata(file).await?.permissions();
            let octal_perms = format!("{:o}", perms.mode());

            if octal_perms == "100111" {
            perms.set_mode(0o551);
            fs::set_permissions(file, perms).await?;
            }

        }
    }

    let nightly_file = fs::read_to_string("nightly/bob.json").await?;
    let mut json_struct: UpstreamVersion = serde_json::from_str(&nightly_file)?;
    let id: String = json_struct
        .target_commitish
        .as_ref()
        .unwrap()
        .chars()
        .take(7)
        .collect();

    info!("Creating rollback: nightly-{id}");
    filesystem::copy_dir("nightly", format!("nightly-{id}")).await?;

    json_struct.tag_name += &format!("-{id}");

    let json_file = serde_json::to_string(&json_struct)?;
    fs::write(format!("nightly-{id}/bob.json"), json_file).await?;

    Ok(())
}

async fn print_commits(
    client: &Client,
    local: &UpstreamVersion,
    upstream: &UpstreamVersion,
) -> Result<()> {
    let commits =
        get_commits_for_nightly(client, &local.published_at, &upstream.published_at).await?;

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
    version: &ParsedVersion,
    root: &Path,
    config: &Config,
) -> Result<PostDownloadVersionType> {
    match version.version_type {
        VersionType::Normal | VersionType::Nightly | VersionType::Latest => {
            let response = send_request(client, config, version).await;

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

                        let file_type = helpers::get_file_type();
                        let mut file =
                            tokio::fs::File::create(format!("{}.{file_type}", version.tag_name))
                                .await?;

                        let mut downloaded: u64 = 0;

                        while let Some(item) = response_bytes.next().await {
                            let chunk = item.map_err(|_| anyhow!("hello"))?;
                            file.write_all(&chunk).await?;
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
                            semver: version.semver.clone(),
                        }))
                    } else {
                        Err(anyhow!(
                            "Please provide an existing neovim version, {}",
                            response.text().await?
                        ))
                    }
                }
                Err(error) => Err(anyhow!(error)),
            }
        }
        VersionType::Hash => handle_building_from_source(version, config).await,
        VersionType::NightlyRollback => Ok(PostDownloadVersionType::None),
    }
}

async fn handle_building_from_source(
    version: &ParsedVersion,
    config: &Config,
) -> Result<PostDownloadVersionType> {
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            if env::var("VisualStudioVersion").is_err() {
                return Err(anyhow!("Please make sure you are using Developer PowerShell/Command Prompt for VS"));
            }

        } else {
            let is_clang_present = match Command::new("clang").output().await {
                Ok(_) => true,
                Err(error) => !matches!(error.kind(), std::io::ErrorKind::NotFound)
            };
            let is_gcc_present = match Command::new("gcc").output().await {
                Ok(_) => true,
                Err(error) => !matches!(error.kind(), std::io::ErrorKind::NotFound)
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
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                return Err(anyhow!(
                    "Cmake has to be installed in order to build neovim from source"
                ));
            }
        }
    }

    if let Err(error) = Command::new("git").output().await {
        if error.kind() == std::io::ErrorKind::NotFound {
            return Err(anyhow!(
                "Git has to be installed in order to build neovim from source"
            ));
        }
    }

    // create neovim-git if it does not exist
    let dirname = "neovim-git";
    if let Err(error) = fs::metadata(dirname).await {
        match error.kind() {
            std::io::ErrorKind::NotFound => {
                fs::create_dir(dirname).await?;
            }
            _ => return Err(anyhow!("unknown error: {}", error)),
        }
    }

    env::set_current_dir(dirname)?; // cd into neovim-git

    // check if repo is initialized
    if let Err(error) = fs::metadata(".git").await {
        match error.kind() {
            std::io::ErrorKind::NotFound => {
                Command::new("git")
                    .arg("init")
                    .stdout(Stdio::null())
                    .spawn()?
                    .wait()
                    .await?;
            }

            _ => return Err(anyhow!("unknown error: {}", error)),
        }
    };

    // check if repo has a remote
    let remote = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .stdout(Stdio::null())
        .spawn()?
        .wait()
        .await?;

    if !remote.success() {
        // add neovim's remote
        Command::new("git")
            .arg("remote")
            .arg("add")
            .arg("origin")
            .arg("https://github.com/neovim/neovim.git")
            .spawn()?
            .wait()
            .await?;
    } else {
        // set neovim's remote otherwise
        Command::new("git")
            .arg("remote")
            .arg("set-url")
            .arg("origin")
            .arg("https://github.com/neovim/neovim.git")
            .spawn()?
            .wait()
            .await?;
    };
    // fetch version from origin
    let fetch_successful = Command::new("git")
        .arg("fetch")
        .arg("--depth")
        .arg("1")
        .arg("origin")
        .arg(&version.non_parsed_string)
        .spawn()?
        .wait()
        .await?
        .success();

    if !fetch_successful {
        return Err(anyhow!(
            "fetching remote failed, try providing the full commit hash"
        ));
    }

    // checkout fetched files
    Command::new("git")
        .arg("checkout")
        .arg("FETCH_HEAD")
        .stdout(Stdio::null())
        .spawn()?
        .wait()
        .await?;

    if fs::metadata("build").await.is_ok() {
        filesystem::remove_dir("build").await?;
    }
    fs::create_dir("build").await?;

    let downloads_location = directories::get_downloads_directory(config).await?;
    let folder_name = downloads_location.join(&version.tag_name[0..7]);
    let build_location = folder_name.join(helpers::get_platform_name(&version.semver));

    let build_type = match config.enable_release_build {
        Some(true) => "Release",
        _ => "RelWithDebInfo",
    };

    let build_arg = format!("CMAKE_BUILD_TYPE={}", build_type);

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            if fs::metadata(".deps").await.is_ok() {
                helpers::filesystem::remove_dir(".deps").await?;
            }
            handle_subprocess(Command::new("cmake").arg("-S").arg("cmake.deps").arg("-B").arg(".deps").arg("-D").arg(&build_arg)).await?;
            handle_subprocess(Command::new("cmake").arg("--build").arg(".deps").arg("--config").arg(build_type)).await?;
            handle_subprocess(Command::new("cmake").arg("-B").arg("build").arg("-D").arg(&build_arg)).await?;
            handle_subprocess(Command::new("cmake").arg("--build").arg("build").arg("--config").arg(build_type)).await?;
            handle_subprocess(Command::new("cmake").arg("--install").arg("build").arg("--prefix").arg(build_location)).await?;
        } else {
            let location_arg = format!(
                "CMAKE_INSTALL_PREFIX={}",
                downloads_location.to_string_lossy()
            );

            handle_subprocess(Command::new("make").arg(&location_arg).arg(&build_arg)).await?;
            handle_subprocess(Command::new("make").arg("install")).await?;
        }
    }

    let mut file = File::create(folder_name.join("full-hash.txt")).await?;
    file.write_all(version.non_parsed_string.as_bytes()).await?;

    Ok(PostDownloadVersionType::Hash)
}

async fn send_request(
    client: &Client,
    config: &Config,
    version: &ParsedVersion,
) -> Result<reqwest::Response, reqwest::Error> {
    let platform = helpers::get_platform_name_download(&version.semver);
    let file_type = helpers::get_file_type();
    let url = match &config.github_mirror {
        Some(val) => val.to_string(),
        None => "https://github.com".to_string(),
    };
    let version = &version.tag_name;
    let request_url =
        format!("{url}/neovim/neovim/releases/download/{version}/{platform}.{file_type}",);

    client
        .get(request_url)
        .header("user-agent", "bob")
        .send()
        .await
}
