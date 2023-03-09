use crate::{config::Config, handlers::{self, sync_handler, uninstall_handler, rollback_handler, erase_handler, list_handler, InstallResult}};
use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use tracing::info;

#[derive(Debug, Parser)]
#[command(version)]
enum Cli {
    /// Switch to the specified version, by default will auto-invoke
    /// install command if the version is not installed already
    Use {
        /// Version to switch to |nightly|stable|<version-string>|<commit-hash>|
        version: String,

        /// Whether not to auto-invoke install command
        #[arg(short, long)]
        no_install: bool,
    },

    /// Install the specified version, can also be used to update
    /// out-of-date nightly version
    Install {
        /// Version to be installed |nightly|stable|<version-string>|<commit-hash>|
        version: String,
    },

    /// If Config::sync_version_file_path is set, the version in that file
    /// will be parsed and installed
    Sync,

    /// Uninstall the specified version
    #[clap(visible_alias = "rm")]
    Uninstall {
        /// Version to be uninstalled |nightly|stable|<version-string>|<commit-hash>|
        version: String,
    },

    /// Rollback to an existing nightly rollback
    Rollback,

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
        Cli::Use { version, no_install } => {
            let client = Client::new();
            let version = super::version::parse_version_type(&client, &version).await?;

            handlers::use_handler::start(version, !no_install, &client, config).await?;
        }
        Cli::Install { version } => {
            let client = Client::new();
            let mut version = super::version::parse_version_type(&client, &version).await?;

            match handlers::install_handler::start(&mut version, &client, &config).await? {
                InstallResult::InstallationSuccess(location) => {
                    info!(
                        "{} has been successfully installed in {location}",
                        version.tag_name
                    );
                }
                InstallResult::VersionAlreadyInstalled => {
                    info!("{} is already installed", version.tag_name);
                }
                InstallResult::NightlyIsUpdated => {
                    info!("Nightly up to date!");
                }
            }
        }
        Cli::Sync => {
            let client = Client::new();
            info!("Starting sync process");
            sync_handler::start(&client, config).await?;
        }
        Cli::Uninstall { version } => {
            info!("Starting uninstallation process");
            uninstall_handler::start(&version, config).await?;
        }
        Cli::Rollback => rollback_handler::start(config).await?,
        Cli::Erase => erase_handler::start(config).await?,
        Cli::List => list_handler::start(config).await?,
    }

    Ok(())
}
