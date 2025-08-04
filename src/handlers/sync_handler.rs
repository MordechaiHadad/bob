use anyhow::{Result, anyhow};
use reqwest::Client;
use tokio::fs;
use tracing::info;

use crate::{config::ConfigFile, helpers::version};

use super::use_handler;

/// Starts the synchronization process.
///
/// This function reads the version from a sync file and starts the use handler with the read version.
///
/// # Arguments
///
/// * `client` - The HTTP client to be used for network requests.
/// * `config` - The configuration for the synchronization process.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the synchronization process was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The `version_sync_file_location` is not set in the configuration.
/// * The sync file is empty.
/// * The version read from the sync file contains "nightly-".
///
/// # Example
///
/// ```rust
/// let client = Client::new();
/// let config = Config::default();
/// start(&client, config).await.unwrap();
/// ```
pub async fn start(client: &Client, config: ConfigFile) -> Result<()> {
    let version_sync_file_location = version::get_version_sync_file_location(&config.config)
        .await?
        .ok_or_else(|| anyhow!("version_sync_file_location needs to be set to use bob sync"))?;

    let version = fs::read_to_string(&version_sync_file_location).await?;
    if version.is_empty() {
        return Err(anyhow!("Sync file is empty"));
    }
    let trimmed_version = version.trim();

    if trimmed_version.contains("nightly-") {
        return Err(anyhow!("Cannot sync nightly rollbacks."));
    }

    info!(
        "Using version {version} set in {}",
        version_sync_file_location
            .into_os_string()
            .into_string()
            .unwrap()
    );

    use_handler::start(
        version::parse_version_type(client, trimmed_version).await?,
        true,
        client,
        config,
    )
    .await?;

    Ok(())
}
