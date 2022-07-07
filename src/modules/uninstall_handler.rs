use crate::{modules::utils, models::Config};
use anyhow::{anyhow, Result};
use reqwest::Client;
use tokio::fs;
use tracing::{info, warn};

pub async fn start(version: &str, config: Config) -> Result<()> {
    let client = Client::new();
    let version = utils::parse_version_type(&client, version).await?;

    if utils::is_version_used(&version).await {
        warn!("Switch to a different version before proceeding");
        return Ok(());
    }

    let downloads_dir = match utils::get_downloads_folder(&config).await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };

    fs::remove_dir_all(&format!("{}/{version}", downloads_dir.display())).await?;
    info!("Successfully uninstalled version: {version}");
    Ok(())
}
