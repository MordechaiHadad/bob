use anyhow::{Result, anyhow};
use tokio::fs;
use tracing::info;

use crate::github_requests::GitHubClient;
use crate::{config::ConfigFile, helpers::version};

use crate::handlers::use_handler;

/// Starts the synchronization process.
///
/// Reads the version from a sync file and starts the use handler with the
/// read version.
///
/// # Arguments
///
/// * `github` - The GitHub API client.
/// * `config` - The configuration for the synchronization process.
///
/// # Errors
///
/// Returns an error if `version_sync_file_location` is not set, the sync
/// file is empty, or it contains "nightly-".
pub async fn start(github: &GitHubClient, config: ConfigFile) -> Result<()> {
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
        version::parse_version_type(github, trimmed_version).await?,
        true,
        github,
        config,
    )
    .await?;

    Ok(())
}
