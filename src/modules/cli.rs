use super::{erase_handler, install_handler, ls_handler, uninstall_handler, use_handler, utils};
use crate::{enums::InstallResult, models::Config};
use anyhow::Result;
use clap::{AppSettings, Parser};
use reqwest::Client;
use tracing::info;

#[derive(Debug, Parser)]
#[clap(global_setting = AppSettings::DeriveDisplayOrder)]
enum Cli {
    /// Switch to the specified version, will auto-invoke install command
    /// if the version is not installed already
    Use {
        /// Version to switch to
        version: String,
    },

    /// Install the specified version, can also be used to update
    /// out-of-date nightly version
    Install {
        /// Version to be installed
        version: String,
    },

    /// Uninstall the specified version
    Uninstall {
        /// Version to be uninstalled
        version: String,
    },

    /// Erase any change bob ever made, including neovim installation,
    /// neovim version downloads and registry changes
    Erase,

    /// List all installed and used versions
    #[clap(visible_alias = "ls")]
    List,
}

pub async fn start(config: Config) -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Use { version } => {
            let client = Client::new();
            let version = utils::parse_version(&client, &version).await?;

            use_handler::start(&version, &client, config).await?;
        }
        Cli::Install { version } => {
            let client = Client::new();
            let version = utils::parse_version(&client, &version).await?;

            match install_handler::start(&version, &client, &config).await? {
                InstallResult::InstallationSuccess(location) => {
                    info!("{version} has been successfully installed in {location}");
                }
                InstallResult::VersionAlreadyInstalled => {
                    info!("{version} is already installed");
                }
                InstallResult::NightlyIsUpdated => {
                    info!("Nightly up to date!");
                }
            }
        }
        Cli::Uninstall { version } => {
            info!("Starting uninstallation process");
            uninstall_handler::start(&version, config).await?;
        }
        Cli::Erase => {
            erase_handler::start(config).await?;
        }
        Cli::List => {
            ls_handler::start(config).await?;
        }
    }

    Ok(())
}
