use anyhow::{Result, anyhow};
use std::{fs, path::PathBuf};
use yansi::Paint;

use crate::{
    config::Config,
    helpers::{self, directories, version::nightly::produce_nightly_vec},
};

/// Starts the list handler.
///
/// This function reads the downloads directory and lists all the installed versions in a formatted table. It also checks if a version is currently in use.
///
/// # Arguments
///
/// * `config` - The configuration object.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the operation is successful, or an error if there are no versions installed or if there is a failure in reading the directory or checking if a version is in use.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let result = start(config).await;
/// assert!(result.is_ok());
/// ```
pub async fn start(config: Config) -> Result<()> {
    let downloads_dir = directories::get_downloads_directory(&config).await?;

    let paths: Vec<PathBuf> = fs::read_dir(downloads_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();

    if paths.is_empty() {
        return Err(anyhow!("There are no versions installed"));
    }

    let version_max_len = if has_rollbacks(&config).await? { 16 } else { 7 };
    let status_max_len = 9;
    let padding = 2;

    println!(
        "┌{}┬{}┐",
        "─".repeat(version_max_len + (padding * 2)),
        "─".repeat(status_max_len + (padding * 2))
    );
    println!(
        "│{}Version{}│{}Status{}│",
        " ".repeat(padding),
        " ".repeat(padding + (version_max_len - 7)),
        " ".repeat(padding),
        " ".repeat(padding + (status_max_len - 6))
    );
    println!(
        "├{}┼{}┤",
        "─".repeat(version_max_len + (padding * 2)),
        "─".repeat(status_max_len + (padding * 2))
    );

    for path in paths {
        if !path.is_dir() {
            continue;
        }

        let path_name = path.file_name().unwrap().to_str().unwrap();

        if !is_version(path_name) {
            continue;
        }

        let version_pr = (version_max_len - path_name.len()) + padding;
        let status_pr = padding + status_max_len;

        if helpers::version::is_version_used(path_name, &config).await {
            println!(
                "│{}{path_name}{}│{}{}{}│",
                " ".repeat(padding),
                " ".repeat(version_pr),
                " ".repeat(padding),
                Paint::green("Used"),
                " ".repeat(status_pr - 4)
            );
        } else {
            println!(
                "│{}{path_name}{}│{}{}{}│",
                " ".repeat(padding),
                " ".repeat(version_pr),
                " ".repeat(padding),
                Paint::yellow("Installed"),
                " ".repeat(status_pr - 9)
            );
        }
    }

    println!(
        "└{}┴{}┘",
        "─".repeat(version_max_len + (padding * 2)),
        "─".repeat(status_max_len + (padding * 2))
    );

    Ok(())
}

/// Checks if there are any rollbacks available.
///
/// This function produces a vector of nightly versions and checks if it is empty.
///
/// # Arguments
///
/// * `config` - A reference to the configuration object.
///
/// # Returns
///
/// * `Result<bool>` - Returns a `Result` that contains `true` if there are rollbacks available, or `false` otherwise. Returns an error if there is a failure in producing the vector of nightly versions.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let has_rollbacks = has_rollbacks(&config).await?;
/// assert_eq!(has_rollbacks, true);
/// ```
async fn has_rollbacks(config: &Config) -> Result<bool> {
    let list = produce_nightly_vec(config).await?;

    Ok(!list.is_empty())
}

/// Checks if a given string is a valid version.
///
/// This function checks if the given string is "stable", contains "nightly", or matches the version or hash regex.
///
/// # Arguments
///
/// * `name` - A reference to a string that could be a version.
///
/// # Returns
///
/// * `bool` - Returns `true` if the string is a valid version, `false` otherwise.
///
/// # Example
///
/// ```rust
/// let version = "v1.0.0";
/// let is_version = is_version(version);
/// assert_eq!(is_version, true);
/// ```
fn is_version(name: &str) -> bool {
    match name {
        "stable" => true,
        nightly_name if nightly_name.contains("nightly") => true,
        name => {
            if crate::VERSION_REGEX.is_match(name) {
                return true;
            }
            crate::HASH_REGEX.is_match(name)
        }
    }
}
