pub mod nightly;
pub mod types;

use self::types::{ParsedVersion, VersionType};
use crate::github_requests::get_upstream_stable;
use crate::helpers::directories;
use crate::{
    config::Config,
    github_requests::{RepoCommit, deserialize_response},
};
use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use semver::Version;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::info;

/// Parses the version type from a version string.
///
/// This function takes a version string and determines the type of the version. It supports the following version types: `Nightly`, `Latest`, `Hash`, `Normal`, and `NightlyRollback`.
///
/// # Arguments
///
/// * `client` - The client to use for fetching the latest version or commit.
/// * `version` - The version string to parse.
///
/// # Returns
///
/// * `Result<ParsedVersion>` - Returns a `Result` that contains a `ParsedVersion` struct with the parsed version information, or an error if the operation failed or the version string is not valid.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The version string is not valid.
/// * The latest version or commit cannot be fetched.
///
/// # Example
///
/// ```rust
/// let client = Client::new();
/// let version = "nightly";
/// let parsed_version = parse_version_type(&client, version).await.unwrap();
/// println!("The parsed version is {:?}", parsed_version);
/// ```
pub async fn parse_version_type(client: &Client, version: &str) -> Result<ParsedVersion> {
    match version {
        "nightly" => Ok(ParsedVersion {
            tag_name: version.to_string(),
            version_type: VersionType::Nightly,
            non_parsed_string: version.to_string(),
            semver: None,
        }),
        "stable" | "latest" => {
            info!("Fetching latest version");
            let stable_version = get_upstream_stable(client).await?;
            let cloned_version = stable_version.tag_name.clone();
            Ok(ParsedVersion {
                tag_name: stable_version.tag_name,
                version_type: VersionType::Latest,
                non_parsed_string: version.to_string(),
                semver: Some(Version::parse(&cloned_version.replace('v', ""))?),
            })
        }
        "head" | "git" | "HEAD" => {
            info!("Fetching latest commit");
            let latest_commit = get_latest_commit(client).await?;
            Ok(ParsedVersion {
                tag_name: latest_commit.chars().take(7).collect(),
                version_type: VersionType::Hash,
                non_parsed_string: latest_commit,
                semver: None,
            })
        }
        _ => {
            if crate::VERSION_REGEX.is_match(version) {
                let mut returned_version = version.to_string();
                if !version.contains('v') {
                    returned_version.insert(0, 'v');
                }
                let cloned_version = returned_version.clone();
                return Ok(ParsedVersion {
                    tag_name: returned_version,
                    version_type: VersionType::Normal,
                    non_parsed_string: version.to_string(),
                    semver: Some(
                        Version::parse(&cloned_version.replace('v', ""))
                            .context("Unable to parse version string in parse_version_type")?,
                    ),
                });
            } else if crate::HASH_REGEX.is_match(version) {
                return Ok(ParsedVersion {
                    tag_name: version.to_string().chars().take(7).collect(),
                    version_type: VersionType::Hash,
                    non_parsed_string: version.to_string(),
                    semver: None,
                });
            }

            if crate::NIGHTLY_REGEX.is_match(version) {
                return Ok(ParsedVersion {
                    tag_name: version.to_string(),
                    version_type: VersionType::NightlyRollback,
                    non_parsed_string: version.to_string(),
                    semver: None,
                });
            }

            Err(anyhow!(
                "Please provide a proper version string. Valid options are:

                    • stable|latest|nightly - Latest stable, most recent, or nightly build
                    • [v]x.x.x              - Specific version (e.g., 0.6.0 or v0.6.0)
                    • <commit-hash>         - Specific commit hash"
            ))
        }
    }
}

/// Retrieves the location of the version sync file.
///
/// This function checks the `version_sync_file_location` field of the provided configuration. If the field is `Some`, it checks if a file exists at the specified path. If the file does not exist, it creates a new file at the path. If the field is `None`, it returns `None`.
///
/// # Arguments
///
/// * `config` - The configuration to retrieve the `version_sync_file_location` field from.
///
/// # Returns
///
/// * `Result<Option<PathBuf>>` - Returns a `Result` that contains an `Option` with the `PathBuf` to the version sync file, or `None` if the `version_sync_file_location` field is `None`, or an error if the operation failed.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The file at the specified path cannot be created.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let version_sync_file_location = get_version_sync_file_location(&config).await.unwrap();
/// println!("The version sync file is located at {:?}", version_sync_file_location);
/// ```
pub async fn get_version_sync_file_location(config: &Config) -> Result<Option<PathBuf>> {
    let path = match &config.version_sync_file_location {
        Some(path) => {
            let path = Path::new(path);
            if tokio::fs::metadata(path).await.is_err() {
                let mut file = File::create(path).await.context(format!("The path provided, \"{}\", does not exist. Please check the path and try again.", path.parent().unwrap().display()))?;
                file.write_all(b"").await?;
            }
            Some(PathBuf::from(path))
        }
        None => return Ok(None),
    };

    Ok(path)
}

/// Checks if a specific version of Neovim is installed.
///
/// This function reads the downloads directory and checks if there is a directory with the name matching the specified version. If such a directory is found, it means that the version is installed.
///
/// # Arguments
///
/// * `version` - The version to check.
/// * `config` - The configuration to retrieve the downloads directory from.
///
/// # Returns
///
/// * `Result<bool>` - Returns a `Result` that contains `true` if the version is installed, `false` otherwise, or an error if the operation failed.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The downloads directory cannot be retrieved.
/// * The downloads directory cannot be read.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let version = "1.0.0";
/// let is_installed = is_version_installed(version, &config).await.unwrap();
/// println!("Is version {} installed? {}", version, is_installed);
/// ```
pub async fn is_version_installed(version: &str, config: &Config) -> Result<bool> {
    let downloads_dir = directories::get_downloads_directory(config).await?;
    let mut dir = tokio::fs::read_dir(&downloads_dir).await?;

    while let Some(directory) = dir.next_entry().await? {
        let name = directory.file_name().to_str().unwrap().to_owned();
        if !version.eq(&name) {
            continue;
        }
        return Ok(true);
    }
    Ok(false)
}

/// Retrieves the current version of Neovim being used.
///
/// This function reads the "used" file from the downloads directory, which contains the current version of Neovim being used. If the "used" file cannot be found, it means that Neovim is not installed through bob.
///
/// # Arguments
///
/// * `config` - The configuration to retrieve the downloads directory from.
///
/// # Returns
///
/// * `Result<String>` - Returns a `Result` that contains the current version of Neovim being used, or an error if the operation failed.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The downloads directory cannot be retrieved.
/// * The "used" file cannot be read.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let current_version = get_current_version(&config).await.unwrap();
/// println!("The current version is {}", current_version);
pub async fn get_current_version(config: &Config) -> Result<String> {
    let mut downloads_dir = directories::get_downloads_directory(config).await?;
    downloads_dir.push("used");
    fs::read_to_string(&downloads_dir).await
        .map_err(|_| anyhow!("The used file required for bob could not be found. This could mean that Neovim is not installed through bob."))
}

/// Checks if a specific version is currently being used.
///
/// This function retrieves the current version from the configuration and checks if it matches the specified version.
///
/// # Arguments
///
/// * `version` - The version to check.
/// * `config` - The configuration to retrieve the current version from.
///
/// # Returns
///
/// * `bool` - Returns `true` if the specified version is currently being used, `false` otherwise.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let version = "1.0.0";
/// let is_used = is_version_used(version, &config).await;
/// println!("Is version {} used? {}", version, is_used);
/// ```
pub async fn is_version_used(version: &str, config: &Config) -> bool {
    match get_current_version(config).await {
        Ok(value) => value.starts_with(version),
        Err(_) => false,
    }
}

/// Fetches the latest commit from the Neovim repository on GitHub.
///
/// This function sends a GET request to the GitHub API to fetch the latest commit from the master branch of the Neovim repository. It then deserializes the response into a `RepoCommit` object and returns the SHA of the commit.
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request.
///
/// # Returns
///
/// * `Result<String>` - Returns a `Result` that contains the SHA of the latest commit, or an error if the operation failed.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The GET request to the GitHub API fails.
/// * The response from the GitHub API cannot be deserialized into a `RepoCommit` object.
///
/// # Example
///
/// ```rust
/// let client = Client::new();
/// let latest_commit = get_latest_commit(&client).await.unwrap();
/// println!("The latest commit is {}", latest_commit);
/// ```
async fn get_latest_commit(client: &Client) -> Result<String> {
    let response = client
        .get("https://api.github.com/repos/neovim/neovim/commits/master")
        .header("user-agent", "bob")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .text()
        .await?;

    let commit: RepoCommit = deserialize_response(&response)?;

    Ok(commit.sha)
}

#[cfg(test)]
mod version_is_hash_tests {

    pub(crate) fn is_hash(version: &str) -> bool {
        crate::HASH_REGEX.is_match(version)
    }

    #[test]
    fn test_is_hash() {
        let version_expected = [
            ("abc123", true),
            ("abc1", false),
            ("", false),
            ("xyz123", false),
            ("abc1", false),
            ("abc123abc123abc123abc123abc123abc123abc123", false),
        ];

        version_expected
            .iter()
            .for_each(|(case, expected)| match expected {
                true => assert!(is_hash(case)),
                false => assert!(!is_hash(case)),
            });

        version_expected.iter().for_each(|(version, expected)| {
            // dbg!(&version);
            assert_eq!(is_hash(version), *expected);
        });
    }

    #[test]
    fn test_is_hash_with_valid_hash() {
        let version = "abc123";
        assert!(is_hash(version));
    }

    #[test]
    fn test_is_hash_with_invalid_hash() {
        let version = "abc1";
        assert!(!is_hash(version));
    }

    #[test]
    fn test_is_hash_with_empty_string() {
        let version = "";
        assert!(!is_hash(version));
    }

    #[test]
    fn test_is_hash_with_non_hexadecimal_characters() {
        let version = "xyz123";
        assert!(!is_hash(version));
    }

    #[test]
    fn test_is_hash_with_short_hash() {
        let version = "abc1";
        assert!(!is_hash(version));
    }

    #[test]
    fn test_is_hash_with_long_hash() {
        let version = "abc123abc123abc123abc123abc123abc123abc123";
        assert!(!is_hash(version));
    }
}
