use crate::config::{Config, ConfigFile};
use crate::github_requests::{get_commits_for_nightly, get_upstream_nightly, UpstreamVersion};
use crate::helpers::checksum::sha256cmp;
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
use tracing::{info, warn};
use yansi::Paint;

use super::{InstallResult, PostDownloadVersionType};

/// Starts the installation process for a given version.
///
/// # Arguments
///
/// * `version` - A mutable reference to a `ParsedVersion` object representing the version to be installed.
/// * `client` - A reference to a `Client` object used for making HTTP requests.
/// * `config` - A reference to a `Config` object containing the configuration settings.
///
/// # Returns
///
/// * `Result<InstallResult>` - Returns a `Result` that contains an `InstallResult` enum on success, or an error on failure.
///
/// # Errors
///
/// This function will return an error if:
/// * The version is below 0.2.2.
/// * There is a problem setting the current directory.
/// * There is a problem checking if the version is already installed.
/// * There is a problem getting the upstream nightly version.
/// * There is a problem getting the local nightly version.
/// * There is a problem handling a rollback.
/// * There is a problem printing commits.
/// * There is a problem downloading the version.
/// * There is a problem handling building from source.
/// * There is a problem unarchiving the downloaded file.
/// * There is a problem creating the file `nightly/bob.json`.
///
/// # Panics
///
/// This function does not panic.
///
/// # Examples
///
/// ```rust
/// let mut version = ParsedVersion::new(VersionType::Normal, "1.0.0");
/// let client = Client::new();
/// let config = Config::default();
/// let result = start(&mut version, &client, &config).await;
/// ```
pub async fn start(
    version: &mut ParsedVersion,
    client: &Client,
    config: &ConfigFile,
) -> Result<InstallResult> {
    if version.version_type == VersionType::NightlyRollback {
        return Ok(InstallResult::GivenNightlyRollback);
    }

    if let Some(version) = &version.semver {
        if version <= &Version::new(0, 2, 2) {
            return Err(anyhow!("Versions below 0.2.2 are not supported"));
        }
    }

    let root = directories::get_downloads_directory(&config.config).await?;

    env::set_current_dir(&root)?;
    let root = root.as_path();

    let is_version_installed =
        helpers::version::is_version_installed(&version.tag_name, &config.config).await?;

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
        let local_nightly = helpers::version::nightly::get_local_nightly(&config.config).await?;

        if upstream_nightly.published_at == local_nightly.published_at {
            return Ok(InstallResult::NightlyIsUpdated);
        }

        handle_rollback(&config.config).await?;

        match config.config.enable_nightly_info {
            Some(boolean) if boolean => {
                print_commits(client, &local_nightly, upstream_nightly).await?
            }
            None => print_commits(client, &local_nightly, upstream_nightly).await?,
            _ => (),
        }
    }

    let downloaded_archive = match version.version_type {
        VersionType::Normal | VersionType::Latest => {
            download_version(client, version, root, &config.config, false).await
        }
        VersionType::Nightly => {
            if config.config.enable_release_build == Some(true) {
                handle_building_from_source(version, &config.config).await
            } else {
                download_version(client, version, root, &config.config, false).await
            }
        }
        VersionType::Hash => handle_building_from_source(version, &config.config).await,
        VersionType::NightlyRollback => Ok(PostDownloadVersionType::None),
    }?;

    if let PostDownloadVersionType::Standard(downloaded_archive) = downloaded_archive {
        if version.semver.is_some() && version.semver.as_ref().unwrap() <= &Version::new(0, 4, 4) {
            unarchive::start(downloaded_archive).await?
        } else {
            let downloaded_checksum =
                download_version(client, version, root, &config.config, true).await?;
            let archive_path = root.join(format!(
                "{}.{}",
                downloaded_archive.file_name, downloaded_archive.file_format
            ));

            if let PostDownloadVersionType::Standard(downloaded_checksum) = downloaded_checksum {
                let checksum_path = root.join(format!(
                    "{}.{}",
                    downloaded_checksum.file_name, downloaded_checksum.file_format
                ));

                let platform = helpers::get_platform_name_download(&version.semver);

                if !sha256cmp(
                    &archive_path,
                    &checksum_path,
                    &format!("{}.{}", platform, downloaded_archive.file_format),
                )? {
                    tokio::fs::remove_file(archive_path).await?;
                    tokio::fs::remove_file(checksum_path).await?;
                    return Err(anyhow!("Checksum mismatch!"));
                }

                info!("Checksum matched!");
                tokio::fs::remove_file(checksum_path).await?;
                unarchive::start(downloaded_archive).await?
            } else if let PostDownloadVersionType::None = downloaded_checksum {
                warn!("No checksum provided, skipping checksum verification");
                unarchive::start(downloaded_archive).await?
            }
        }
    }

    if let VersionType::Nightly = version.version_type {
        if let Some(nightly_version) = nightly_version {
            let nightly_string = serde_json::to_string(&nightly_version)?;

            let downloads_dir = root.join("nightly").join("bob.json");
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

/// Asynchronously handles the rollback of the nightly version of Neovim.
///
/// This function checks if the nightly version is used and if the rollback limit is not zero.
/// If these conditions are met, it produces a vector of nightly versions and removes the oldest version if the vector's length is greater than or equal to the rollback limit.
/// It also handles permissions for older installations of nightly on Unix platforms.
/// Finally, it creates a rollback by copying the nightly directory to a new directory with the ID of the target commitish and updates the JSON file in the new directory.
///
/// # Arguments
///
/// * `config` - A reference to the configuration object.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that contains `()` on success, or an error on failure.
///
/// # Errors
///
/// This function will return an error if:
/// * There is a failure in producing the vector of nightly versions.
/// * There is a failure in removing the oldest version.
/// * There is a failure in reading the nightly JSON file.
/// * There is a failure in parsing the JSON file.
/// * There is a failure in copying the nightly directory.
/// * There is a failure in writing the updated JSON file.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// handle_rollback(&config).await?;
/// `
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
    filesystem::copy_dir_async("nightly", format!("nightly-{id}")).await?;

    json_struct.tag_name += &format!("-{id}");

    let json_file = serde_json::to_string(&json_struct)?;
    fs::write(format!("nightly-{id}/bob.json"), json_file).await?;

    Ok(())
}

/// Asynchronously prints the commits between two versions of Neovim.
///
/// This function fetches the commits between the published dates of the local and upstream versions of Neovim.
/// It then prints each commit with the author's name in blue and the commit message.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `local` - A reference to the local version of Neovim.
/// * `upstream` - A reference to the upstream version of Neovim.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that contains `()` on success, or an error on failure.
///
/// # Errors
///
/// This function will return an error if:
/// * There is a failure in fetching the commits between the published dates of the local and upstream versions.
///
/// # Example
///
/// ```rust
/// let client = Client::new();
/// let local = UpstreamVersion::get_local_version();
/// let upstream = UpstreamVersion::get_upstream_version(&client).await?;
/// print_commits(&client, &local, &upstream).await?;
/// ```
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

/// Asynchronously downloads a specified version of Neovim.
///
/// This function sends a request to download the specified version of Neovim based on the version type.
/// If the version type is Normal, Nightly, or Latest, it sends a request to download the version.
/// If the version type is Hash, it handles building from source.
/// If the version type is NightlyRollback, it does nothing.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `version` - A reference to the parsed version of Neovim to be downloaded.
/// * `root` - A reference to the path where the downloaded file will be saved.
/// * `config` - A reference to the configuration object.
/// * `sha256sum` - A boolean indicating whether to get the sha256sum
///
/// # Returns
///
/// * `Result<PostDownloadVersionType>` - Returns a `Result` that contains a `PostDownloadVersionType` on success, or an error on failure.
///
/// # Errors
///
/// This function will return an error if:
/// * There is a failure in sending the request to download the version.
/// * The response status is not 200.
/// * There is a failure in creating the file where the downloaded version will be saved.
/// * There is a failure in writing the downloaded bytes to the file.
///
/// # Example
///
/// ```rust
/// let client = Client::new();
/// let version = ParsedVersion::parse("0.5.0");
/// let root = Path::new("/path/to/save");
/// let config = Config::default();
/// let result = download_version(&client, &version, &root, &config).await;
/// ```
async fn download_version(
    client: &Client,
    version: &ParsedVersion,
    root: &Path,
    config: &Config,
    get_sha256sum: bool,
) -> Result<PostDownloadVersionType> {
    match version.version_type {
        VersionType::Normal | VersionType::Nightly | VersionType::Latest => {
            let response = send_request(client, config, version, get_sha256sum).await;

            match response {
                Ok(response) => {
                    if response.status() == 200 {
                        let total_size = response.content_length().unwrap_or(0);
                        let mut response_bytes = response.bytes_stream();

                        // Progress Bar Setup
                        let pb = ProgressBar::new(total_size);
                        pb.set_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                    .unwrap()
                    .progress_chars("█  "));
                        let dl = if get_sha256sum { "checksum" } else { "version" };
                        pb.set_message(format!("Downloading {dl}: {}", version.tag_name));

                        let file_type = helpers::get_file_type();
                        let file_type = if get_sha256sum {
                            if version.version_type == VersionType::Nightly
                                || version.semver.as_ref().unwrap() > &Version::new(0, 10, 4)
                            {
                                "shasum.txt".to_string()
                            } else {
                                format!("{file_type}.sha256sum")
                            }
                        } else {
                            file_type.to_owned()
                        };

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

                        file.flush().await?;
                        file.sync_all().await?;

                        pb.finish_with_message(format!(
                            "Downloaded {dl} {} to {}/{}.{file_type}",
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
                        if get_sha256sum {
                            return Ok(PostDownloadVersionType::None);
                        }
                        let error_text = response.text().await?;
                        if error_text.contains("Not Found") {
                            Err(anyhow!(
                                "Version does not exist in Neovim releases. Please check available versions with 'bob list-remote'"
                            ))
                        } else {
                            Err(anyhow!(
                                "Please provide an existing neovim version, {}",
                                error_text
                            ))
                        }
                    }
                }
                Err(error) => Err(anyhow!(error)),
            }
        }
        VersionType::Hash => handle_building_from_source(version, config).await,
        VersionType::NightlyRollback => Ok(PostDownloadVersionType::None),
    }
}

/// Asynchronously handles the building of a specified version from source.
///
/// This function checks for the presence of necessary tools (like Clang, GCC, Cmake, and Git) in the system.
/// It then proceeds to create a directory named "neovim-git" if it doesn't exist, and sets the current directory to it.
/// It initializes a Git repository if one doesn't exist, and sets the remote to Neovim's GitHub repository.
/// It fetches the specified version from the remote repository and checks out the fetched files.
/// It then builds the fetched files and installs them to a specified location.
///
/// # Arguments
///
/// * `version` - A reference to the parsed version of Neovim to be built.
/// * `config` - A reference to the configuration object.
///
/// # Returns
///
/// * `Result<PostDownloadVersionType>` - Returns a `Result` that contains a `PostDownloadVersionType` on success, or an error on failure.
///
/// # Errors
///
/// This function will return an error if:
/// * The necessary tools are not installed in the system.
/// * There is a failure in creating the "neovim-git" directory.
/// * There is a failure in initializing the Git repository.
/// * There is a failure in setting the remote repository.
/// * There is a failure in fetching the specified version from the remote repository.
/// * There is a failure in checking out the fetched files.
/// * There is a failure in building and installing the fetched files.
///
/// # Example
///
/// ```rust
/// let version = ParsedVersion::parse("0.5.0");
/// let config = Config::default();
/// let result = handle_building_from_source(&version, &config).await;
/// ```
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

    let build_type = match config.enable_release_build {
        Some(true) => "Release",
        _ => "RelWithDebInfo",
    };

    let build_arg = format!("CMAKE_BUILD_TYPE={build_type}");

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            if fs::metadata(".deps").await.is_ok() {
                helpers::filesystem::remove_dir(".deps").await?;
            }
            handle_subprocess(Command::new("cmake").arg("-S").arg("cmake.deps").arg("-B").arg(".deps").arg("-D").arg(&build_arg)).await?;
            handle_subprocess(Command::new("cmake").arg("--build").arg(".deps").arg("--config").arg(build_type)).await?;
            handle_subprocess(Command::new("cmake").arg("-B").arg("build").arg("-D").arg(&build_arg)).await?;
            handle_subprocess(Command::new("cmake").arg("--build").arg("build").arg("--config").arg(build_type)).await?;
            handle_subprocess(Command::new("cmake").arg("--install").arg("build").arg("--prefix").arg(&folder_name)).await?;
        } else {
            let location_arg = format!(
                "CMAKE_INSTALL_PREFIX={}",
                folder_name.to_string_lossy()
            );

            handle_subprocess(Command::new("make").arg(&location_arg).arg(&build_arg)).await?;
            handle_subprocess(Command::new("make").arg("install")).await?;
        }
    }

    let mut file = File::create(folder_name.join("full-hash.txt")).await?;
    file.write_all(version.non_parsed_string.as_bytes()).await?;

    Ok(PostDownloadVersionType::Hash)
}

/// Sends a GET request to the specified URL to download a specific version of Neovim.
///
/// # Arguments
///
/// * `client: &Client` - A reference to the `Client` used for making requests.
/// * `config: &Config` - Contains the configuration settings.
/// * `version: &ParsedVersion` - Contains the version information to be downloaded.
/// * `get_sha256sum: bool` - A boolean indicating whether to get the sha256sum.
///
/// # Behavior
///
/// The function constructs the download URL based on the provided `version` and `config.github_mirror`. If `config.github_mirror` is `None`, it defaults to "https://github.com".
///
/// It then sends a GET request to the constructed URL with the header "user-agent" set to "bob".
///
/// # Returns
///
/// * `Result<reqwest::Response, reqwest::Error>` - Returns a `Result` containing the server's response to the GET request. If the request fails, it returns an error.
///
/// # Example
///
/// ```rust
/// let client = Client::new();
/// let config = Config::default();
/// let version = ParsedVersion { tag_name: "v0.2.2", semver: Version::parse("0.2.2").unwrap() };
/// let response = send_request(&client, &config, &version, false).await?;
/// ```
///
/// # Note
///
/// This function is asynchronous and must be awaited.
///
/// # See Also
///
/// * [`helpers::get_platform_name_download`](src/helpers/platform.rs)
/// * [`helpers::get_file_type`](src/helpers/file.rs)
async fn send_request(
    client: &Client,
    config: &Config,
    version: &ParsedVersion,
    get_sha256sum: bool,
) -> Result<reqwest::Response, reqwest::Error> {
    let platform = helpers::get_platform_name_download(&version.semver);
    let file_type = helpers::get_file_type();

    let url = match &config.github_mirror {
        Some(val) => val.to_string(),
        None => "https://github.com".to_string(),
    };
    let version_tag = &version.tag_name;
    let request_url = if get_sha256sum {
        if version.version_type == VersionType::Nightly
            || version.semver.as_ref().unwrap() > &Version::new(0, 10, 4)
        {
            format!("{url}/neovim/neovim/releases/download/{version_tag}/shasum.txt")
        } else {
            format!(
                "{url}/neovim/neovim/releases/download/{version_tag}/{platform}.{file_type}.sha256sum"
            )
        }
    } else {
        format!("{url}/neovim/neovim/releases/download/{version_tag}/{platform}.{file_type}")
    };

    client
        .get(request_url)
        .header("user-agent", "bob")
        .send()
        .await
}
