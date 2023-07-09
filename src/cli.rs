use crate::{
    config::Config,
    handlers::{
        self, erase_handler, list_handler, rollback_handler, sync_handler, uninstall_handler,
        update_handler, InstallResult,
    },
};
use anyhow::Result;
use clap::{Args, CommandFactory, Parser};
use clap_complete::Shell;
use reqwest::Client;
use tracing::{error, info};

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

    /// If Config::version_sync_file_location is set, the version in that file
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

    /// Generate shell completion
    Complete {
        /// Shell to generate completion for
        shell: Shell,
    },

    /// Update existing version
    Update(Update),
}

#[derive(Args, Debug)]
pub struct Update {
    /// Update specified version |nightly|stable|
    #[arg(conflicts_with = "all")]
    pub version: Option<String>,

    /// Apply the update to all versions
    #[arg(short, long)]
    pub all: bool,
}

pub async fn start(config: Config) -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Use {
            version,
            no_install,
        } => {
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
        Cli::Complete { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "bob", &mut std::io::stdout());
        }
        Cli::Update(data) => {
            if data.version.is_some() || data.all {
                let client = Client::new();
                update_handler::start(data, &client, config).await?;
            } else {
                error!("Please provide a version or use the --all flag");
            }
        }
    }

    Ok(())
}
