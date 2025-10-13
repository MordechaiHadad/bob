use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use yansi::Paint;

use crate::config::Config;
use crate::github_requests::{deserialize_response, make_github_request};
use crate::helpers::version::search_stable_version;
use crate::helpers::{self, directories};

/// Asynchronously starts the process of listing remote versions of Neovim.
///
/// This function takes a `Config` and a `Client` as arguments. It first gets the downloads directory path by calling the `get_downloads_directory` function.
/// It then makes a GitHub API request to get the tags of the Neovim repository, which represent the versions of Neovim.
/// The function then reads the downloads directory and filters the entries that contain 'v' in their names, which represent the local versions of Neovim.
/// It deserializes the response from the GitHub API request into a vector of `RemoteVersion`.
/// It filters the versions that start with 'v' and then iterates over the filtered versions.
/// For each version, it checks if it is installed locally and if it is the stable version.
/// It then prints the version name in green if it is being used, in yellow if it is installed but not being used, and in default color if it is not installed.
/// It also appends ' (stable)' to the version name if it is the stable version.
///
/// # Arguments
///
/// * `config` - A `Config` containing the application configuration.
/// * `client` - A `Client` used to make the GitHub API request.
///
/// # Returns
///
/// This function returns a `Result` that contains `()` if the operation was successful.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let client = Client::new();
/// start(config, client).await?;
/// ```
pub async fn start(config: Config, client: Client) -> Result<()> {
    let downloads_dir = directories::get_downloads_directory(&config).await?;
    let response =
        make_github_request(&client, "https://api.github.com/repos/neovim/neovim/tags?per_page=50").await?;

    let mut local_versions: Vec<PathBuf> = fs::read_dir(downloads_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().file_name().unwrap().to_str().unwrap().contains('v'))
        .map(|entry| entry.path())
        .collect();

    let versions: Vec<RemoteVersion> = deserialize_response(response)?;
    let filtered_versions: Vec<RemoteVersion> =
        versions.into_iter().filter(|v| v.name.starts_with('v')).collect();

    let stable_version = search_stable_version(&client).await?;

    let mut buffer = Vec::with_capacity(1024);

    for version in filtered_versions {
        let version_installed = local_versions.iter().any(|v| {
            v.file_name()
                .and_then(|str| str.to_str())
                .is_some_and(|str| str.contains(&version.name))
        });

        let stable_version_string = if stable_version == version.name {
            " (stable)"
        } else {
            ""
        };

        let write_result = if helpers::version::is_version_used(&version.name, &config).await {
            writeln!(buffer, "{}{}", Paint::green(&version.name), stable_version_string)
        } else if version_installed {
            writeln!(buffer, "{}{}", Paint::yellow(&version.name), stable_version_string)
        } else {
            writeln!(buffer, "{}{}", &version.name, stable_version_string)
        };

        if let Err(e) = write_result {
            if e.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            } else {
                return Err(e.into());
            }
        }

        if version_installed {
            local_versions.retain(|v| {
                v.file_name()
                    .and_then(|str| str.to_str())
                    .is_none_or(|str| !str.contains(&version.name))
            });
        }
    }

    let mut stdout = io::stdout().lock();
    stdout.write_all(&buffer).map_err(|e| {
        if e.kind() == io::ErrorKind::BrokenPipe {
            return anyhow::anyhow!("Failed to write to stdout: Broken pipe");
        }
        e.into()
    })?;

    stdout.flush()?;

    Ok(())
}

/// Represents a remote version of Neovim.
///
/// This struct is used to deserialize the response from the GitHub API request that gets the tags of the Neovim repository.
/// Each tag represents a version of Neovim, and the `name` field of the `RemoteVersion` struct represents the name of the version.
///
/// # Fields
///
/// * `name` - A `String` that represents the name of the version.
///
/// # Example
///
/// ```rust
/// let remote_version = RemoteVersion {
///     name: "v0.5.0".to_string(),
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct RemoteVersion {
    pub name: String,
}
