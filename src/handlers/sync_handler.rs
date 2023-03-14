use anyhow::{anyhow, Result};
use reqwest::Client;
use tokio::fs;
use tracing::info;

use crate::{config::Config, helpers::version};

use super::use_handler;

pub async fn start(client: &Client, config: Config) -> Result<()> {
    let sync_version_file_path = version::get_sync_version_file_path(&config)
        .await?
        .ok_or_else(|| anyhow!("sync_version_file_path needs to be set to use bob sync"))?;

    let version = fs::read_to_string(&sync_version_file_path).await?;
    if version.is_empty() {
        return Err(anyhow!("Sync file is empty"));
    }
    let trimmed_version = version.trim();

    if trimmed_version.contains("nightly-") {
        return Err(anyhow!("Cannot sync nightly rollbacks."));
    }

    info!(
        "Using version {version} set in {}",
        sync_version_file_path
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
