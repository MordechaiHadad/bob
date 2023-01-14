use super::utils;
use crate::models::Config;
use anyhow::{anyhow, Result};
use reqwest::Client;
use tokio::fs;
use tracing::info;

use super::use_handler;

pub async fn start(client: &Client, config: Config) -> Result<()> {
    if let Some(sync_version_file_path) = utils::get_sync_version_file_path(&config).await? {
        let version = fs::read_to_string(&sync_version_file_path).await?;
        info!(
            "Using version {version} set in {}",
            sync_version_file_path
                .into_os_string()
                .into_string()
                .unwrap()
        );
        use_handler::start(
            utils::parse_version_type(client, &version).await?,
            client,
            config,
        )
        .await?;
    } else {
        return Err(anyhow!(
            "sync_version_file_path needs to be set to use `bob sync`"
        ));
    }

    Ok(())
}
