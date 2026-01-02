use anyhow::Result;
use std::{fs, path::PathBuf};
use tracing::info;
use yansi::Paint;

use crate::{
    config::Config,
    helpers::{self, directories},
};

/// Recursively finds all version directories in the downloads directory.
///
/// This function searches through the directory tree starting from `current_dir`,
/// looking for directories that contain a valid Neovim installation (indicated by
/// the presence of a `bin` subdirectory).
///
/// # Arguments
///
/// * `current_dir` - The current directory being searched
///
/// # Returns
///
/// * `Result<Vec<PathBuf>>` - A vector of paths to version directories with their
///   relative paths from the base directory
fn find_version_dirs(current_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut version_dirs = Vec::new();

    if let Ok(entries) = fs::read_dir(current_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let nvim_exe = path.join("bin").join("nvim");
            if nvim_exe.exists() && nvim_exe.is_file() {
                version_dirs.push(path);
            } else if let Ok(mut subdirs) = find_version_dirs(&path) {
                version_dirs.append(&mut subdirs);
            }
        }
    }

    Ok(version_dirs)
}

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

    // Recursively find all version directories
    let paths = find_version_dirs(&downloads_dir)?;

    if paths.is_empty() {
        info!("There are no versions installed");
        return Ok(());
    }

    let relative_paths = paths
        .iter()
        .map(|path| path.strip_prefix(&downloads_dir).unwrap())
        .filter(|path| !path.starts_with("neovim-git/build"))
        .map(|path| path.to_str().unwrap())
        .collect::<Vec<&str>>();

    // Calculate the maximum version name length dynamically
    let version_max_len = relative_paths
        .iter()
        .filter_map(|path| {
            if !is_version(path) {
                return None;
            }
            Some(path.len())
        })
        .max()
        .unwrap_or(7)
        .max(7); // Ensure at least 7 for the "Version" header

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

    for path_name in relative_paths {
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

/// Checks if a given string is a valid version.
///
/// This function checks if the given string is "stable", contains "nightly", matches the version or hash regex,
/// or is a fork version (contains forward slashes).
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
            crate::FORK_REGEX.is_match(name)
                || crate::VERSION_REGEX.is_match(name)
                || crate::HASH_REGEX.is_match(name)
        }
    }
}

#[cfg(test)]
mod list_handler_is_version_tests {
    use super::*;

    #[test]
    fn test_is_version() {
        let cases_expected = [
            ("v1.0.0", true),
            ("stable", true),
            ("nightly-2023-10-01", true),
            ("invalid-version", false),
            ("", false),
            ("user/repo@branch", true),  // fork version
            ("owner/fork@main", true),   // fork version
            ("user/repo/branch", false), // invalid fork format (missing @)
        ];

        cases_expected
            .iter()
            .for_each(|(case, expected)| match expected {
                true => assert!(is_version(case)),
                false => assert!(!is_version(case)),
            });

        cases_expected.iter().for_each(|(case, expected)| {
            assert_eq!(is_version(case), *expected);
        });
    }

    #[test]
    fn test_with_v_semvar() {
        let version = "v1.2.3";
        assert!(
            is_version(version),
            "Expected '{}' to be a valid version",
            version
        );
    }

    #[test]
    fn test_as_stable() {
        let version = "stable";
        assert!(
            is_version(version),
            "Expected '{}' to be a valid version",
            version
        );
    }

    #[test]
    fn test_with_nightly_and_date() {
        let version = "nightly-2023-10-01";
        assert!(
            is_version(version),
            "Expected '{}' to be a valid version",
            version
        );
    }

    #[test]
    fn test_with_invalid_version() {
        let version = "invalid-version";
        // let res = is_version(version);
        assert!(
            !is_version(version),
            "Expected '{}' to not be a valid version",
            version
        );
    }

    #[test]
    #[should_panic]
    fn test_with_invalid_version_panic() {
        let version = "invalid-version-wow";
        assert!(
            is_version(version),
            "Expected '{}' to not be a valid version",
            version
        );
    }

    #[test]
    fn test_with_empty_string() {
        let version = "";
        assert!(
            !is_version(version),
            "Expected '{}' to not be a valid version",
            version
        );
    }
}
