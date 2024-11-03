use crate::config::ConfigFile;
use crate::helpers::version::is_version_installed;
use crate::{cli::Update, config::Config};
use anyhow::Result;
use reqwest::Client;
use tracing::{info, warn};

use super::{install_handler, InstallResult};

/// Starts the update process based on the provided `Update` data, `Client`, and `Config`.
///
/// # Arguments
///
/// * `data: Update` - Contains the version information to be updated. If `data.version` is `None` or `data.all` is `true`, it will attempt to update all installed versions.
/// * `client: &Client` - A reference to the `Client` used for making requests.
/// * `config: Config` - Contains the configuration settings.
///
/// # Behavior
///
/// If `data.version` is `None` or `data.all` is `true`, the function will attempt to update both the "stable" and "nightly" versions if they are installed. If an update is successful, `did_update` is set to `true`.
///
/// If neither version is updated, a warning message "There was nothing to update." is logged.
///
/// If `data.version` is not `None` and `data.all` is not `true`, the function will attempt to update the specified version if it is installed. If the version is not installed, a warning message is logged.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the function executes successfully, otherwise it returns an error.
///
/// # Errors
///
/// This function will return an error if there's a problem with parsing the version type, checking if a version is installed, or starting the installation process.
///
/// # Example
///
/// ```rust
/// let data = Update { version: Some("0.2.2"), all: false };
/// let client = Client::new();
/// let config = Config::default();
/// start(data, &client, config).await?;
/// ```
///
/// # Note
///
/// This function is asynchronous and must be awaited.
///
/// # See Also
///
/// * [`crate::version::parse_version_type`](src/version.rs)
/// * [`is_version_installed`](src/helpers/version.rs)
/// * [`install_handler::start`](src/handlers/install_handler.rs)
pub async fn start(data: Update, client: &Client, config: ConfigFile) -> Result<()> {
    if data.version.is_none() || data.all {
        let mut did_update = false;

        let mut stable = crate::version::parse_version_type(client, "stable").await?;
        if is_version_installed(&stable.tag_name, &config.config).await? {
            match install_handler::start(&mut stable, client, &config).await? {
                InstallResult::InstallationSuccess(_) => did_update = true,
                InstallResult::VersionAlreadyInstalled
                | InstallResult::NightlyIsUpdated
                | InstallResult::GivenNightlyRollback => (),
            }
        }

        if is_version_installed("nightly", &config.config).await? {
            let mut nightly = crate::version::parse_version_type(client, "nightly").await?;
            match install_handler::start(&mut nightly, client, &config).await? {
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

    let mut version = crate::version::parse_version_type(client, &data.version.unwrap()).await?;

    if !is_version_installed(&version.tag_name, &config.config).await? {
        warn!("{} is not installed.", version.non_parsed_string);
        return Ok(());
    }
    match install_handler::start(&mut version, client, &config).await? {
        InstallResult::NightlyIsUpdated => info!("Nightly is already updated!"),
        InstallResult::VersionAlreadyInstalled => info!("Stable is already updated!"),
        InstallResult::InstallationSuccess(_) | InstallResult::GivenNightlyRollback => (),
    }
    Ok(())
}
