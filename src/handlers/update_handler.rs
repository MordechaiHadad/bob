use crate::{cli::Update, config::Config};
use anyhow::Result;
use reqwest::Client;
use tracing::info;

use super::{install_handler, InstallResult};

pub async fn start(data: Update, client: &Client, config: Config) -> Result<()> {
    if data.all {
        let mut stable = crate::version::parse_version_type(client, "stable").await?;
        match install_handler::start(&mut stable, client, &config).await? {
            InstallResult::VersionAlreadyInstalled => info!("Stable is already updated!"),
            InstallResult::InstallationSuccess(_) | InstallResult::NightlyIsUpdated => (),
        }

        let mut nightly = crate::version::parse_version_type(client, "nightly").await?;
        match install_handler::start(&mut nightly, client, &config).await? {
            InstallResult::NightlyIsUpdated => info!("Nightly is already updated!"),
            InstallResult::InstallationSuccess(_) | InstallResult::VersionAlreadyInstalled => (),
        }

        return Ok(());
    }

    let mut version = crate::version::parse_version_type(client, &data.version.unwrap()).await?;

    match install_handler::start(&mut version, client, &config).await? {
        InstallResult::NightlyIsUpdated => info!("Nightly is already updated!"),
        InstallResult::VersionAlreadyInstalled => info!("Stable is already updated!"),
        InstallResult::InstallationSuccess(_) => (),
    }
    Ok(())
}
