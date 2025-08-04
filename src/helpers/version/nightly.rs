use anyhow::{Result, anyhow};
use tokio::fs;

use super::types::LocalNightly;
use crate::{config::Config, github_requests::UpstreamVersion, helpers::directories};

/// Retrieves the local nightly version.
///
/// This function reads the `bob.json` file in the `nightly` directory of the downloads directory and parses it into an `UpstreamVersion` struct.
///
/// # Arguments
///
/// * `config` - The configuration to retrieve the downloads directory from.
///
/// # Returns
///
/// * `Result<UpstreamVersion>` - Returns a `Result` that contains an `UpstreamVersion` struct with the local nightly version, or an error if the operation failed.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The downloads directory cannot be retrieved.
/// * The `bob.json` file cannot be read.
/// * The `bob.json` file cannot be parsed into an `UpstreamVersion` struct.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let local_nightly = get_local_nightly(&config).await.unwrap();
/// println!("The local nightly version is {:?}", local_nightly);
/// ```
pub async fn get_local_nightly(config: &Config) -> Result<UpstreamVersion> {
    let downloads_dir = directories::get_downloads_directory(config).await?;
    if let Ok(file) =
        fs::read_to_string(format!("{}/nightly/bob.json", downloads_dir.display())).await
    {
        let file_json: UpstreamVersion = serde_json::from_str(&file)?;
        Ok(file_json)
    } else {
        Err(anyhow!("Couldn't find bob.json"))
    }
}

/// Produces a vector of `LocalNightly` structs from the downloads directory.
///
/// This function reads the downloads directory and creates a `LocalNightly` struct for each directory that matches the `nightly-[a-zA-Z0-9]{7,8}` pattern. The `LocalNightly` structs are sorted by the `published_at` field in descending order.
///
/// # Arguments
///
/// * `config` - The configuration to retrieve the downloads directory from.
///
/// # Returns
///
/// * `Result<Vec<LocalNightly>>` - Returns a `Result` that contains a vector of `LocalNightly` structs, or an error if the operation failed.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The downloads directory cannot be retrieved.
/// * The downloads directory cannot be read.
/// * A directory name does not match the `nightly-[a-zA-Z0-9]{7,8}` pattern.
/// * The `bob.json` file in a directory cannot be read.
/// * The `bob.json` file in a directory cannot be parsed into a `UpstreamVersion` struct.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let nightly_vec = produce_nightly_vec(&config).await.unwrap();
/// println!("There are {} nightly versions.", nightly_vec.len());
/// ```
pub async fn produce_nightly_vec(config: &Config) -> Result<Vec<LocalNightly>> {
    let downloads_dir = directories::get_downloads_directory(config).await?;
    let mut paths = fs::read_dir(&downloads_dir).await?;

    let regex = Regex::new(r"nightly-[a-zA-Z0-9]{7,8}")?;

    let mut nightly_vec: Vec<LocalNightly> = Vec::new();

    while let Some(path) = paths.next_entry().await? {
        let name = path.file_name().into_string().unwrap();

        if !regex.is_match(&name) {
            continue;
        }

        let nightly_content = path.path().join("bob.json");
        let nightly_string = fs::read_to_string(nightly_content).await?;

        let nightly_data: UpstreamVersion = serde_json::from_str(&nightly_string)?;

        let mut nightly_entry = LocalNightly {
            data: nightly_data,
            path: path.path(),
        };

        nightly_entry.data.tag_name = name;

        nightly_vec.push(nightly_entry);
    }

    nightly_vec.sort_by(|a, b| b.data.published_at.cmp(&a.data.published_at));

    Ok(nightly_vec)
}
