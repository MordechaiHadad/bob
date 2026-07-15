use crate::cli::Update;
use crate::config::ConfigFile;
use crate::github_requests::GitHubClient;
use crate::helpers::version::is_version_installed;
use anyhow::Result;
use tracing::{info, warn};

use crate::handlers::{InstallResult, install_handler};

/// Starts the update process.
///
/// If `data.version` is `None` or `data.all` is `true`, attempts to update
/// both "stable" and "nightly" if installed. Otherwise updates the
/// specified version.
///
/// # Arguments
///
/// * `data` - Contains the version information to be updated.
/// * `github` - The GitHub API client.
/// * `config` - The configuration settings.
pub async fn start(data: Update, github: &GitHubClient, config: ConfigFile) -> Result<()> {
    if data.version.is_none() || data.all {
        let mut did_update = false;

        let stable = crate::version::parse_version_type(github, "stable").await?;
        if is_version_installed(&stable.tag_name, &config.config).await? {
            match install_handler::start(&stable, github, &config).await? {
                InstallResult::InstallationSuccess(_) => did_update = true,
                InstallResult::VersionAlreadyInstalled
                | InstallResult::NightlyIsUpdated
                | InstallResult::GivenNightlyRollback => (),
            }
        }

        if is_version_installed("nightly", &config.config).await? {
            let nightly = crate::version::parse_version_type(github, "nightly").await?;
            match install_handler::start(&nightly, github, &config).await? {
                InstallResult::InstallationSuccess(_) => did_update = true,
                InstallResult::NightlyIsUpdated
                | InstallResult::VersionAlreadyInstalled
                | InstallResult::GivenNightlyRollback => (),
            }
        }

        if !did_update {
            warn!("There was nothing to update.");
        }

        return Ok(());
    }

    let version = crate::version::parse_version_type(github, &data.version.unwrap()).await?;

    if !is_version_installed(&version.tag_name, &config.config).await? {
        warn!("{} is not installed.", version.non_parsed_string);
        return Ok(());
    }
    match install_handler::start(&version, github, &config).await? {
        InstallResult::NightlyIsUpdated => info!("Nightly is already updated!"),
        InstallResult::VersionAlreadyInstalled => info!("Stable is already updated!"),
        InstallResult::InstallationSuccess(_) | InstallResult::GivenNightlyRollback => (),
    }
    Ok(())
}
