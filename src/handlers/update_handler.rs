use crate::helpers::version::is_version_installed;
use crate::{cli::Update, config::Config};
use anyhow::Result;
use reqwest::Client;
use tracing::{info, warn};

use super::{install_handler, InstallResult};

pub async fn start(data: Update, client: &Client, config: Config) -> Result<()> {
    if data.all {
        let mut did_update = false;

        let mut stable = crate::version::parse_version_type(client, "stable").await?;
        if is_version_installed(&stable.tag_name, &config).await? {
            match install_handler::start(&mut stable, client, &config).await? {
                InstallResult::VersionAlreadyInstalled => info!("Stable is already updated!"),
                InstallResult::InstallationSuccess(_) | InstallResult::NightlyIsUpdated => (),
            }
            did_update = true;
        }

        if is_version_installed("nightly", &config).await? {
            let mut nightly = crate::version::parse_version_type(client, "nightly").await?;
            match install_handler::start(&mut nightly, client, &config).await? {
                InstallResult::NightlyIsUpdated => info!("Nightly is already updated!"),
                InstallResult::InstallationSuccess(_) | InstallResult::VersionAlreadyInstalled => {
                    ()
                }
            }

            did_update = true;
        }

        if !did_update {
            warn!("There was nothing to update.");
        }

        return Ok(());
    }

    let mut version = crate::version::parse_version_type(client, &data.version.unwrap()).await?;

    if !is_version_installed(&version.tag_name, &config).await? {
        warn!("{} is not installed.", version.non_parsed_string);
        return Ok(());
    } 
    match install_handler::start(&mut version, client, &config).await? {
        InstallResult::NightlyIsUpdated => info!("Nightly is already updated!"),
        InstallResult::VersionAlreadyInstalled => info!("Stable is already updated!"),
        InstallResult::InstallationSuccess(_) => (),
    }
    Ok(())
}
