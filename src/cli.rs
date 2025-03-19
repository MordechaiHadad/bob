use crate::{
    config::ConfigFile,
    handlers::{
        self, erase_handler, list_handler, list_remote_handler, rollback_handler, run_handler,
        sync_handler, uninstall_handler, update_handler, InstallResult,
    },
};
use anyhow::Result;
use clap::{Args, CommandFactory, Parser};
use clap_complete::Shell;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client, Error};
use tracing::info;

/// Creates a new `reqwest::Client` with default headers.
///
/// This function fetches the `GITHUB_TOKEN` environment variable and uses it to set the `Authorization` header for the client.
///
/// # Returns
///
/// This function returns a `Result` that contains a `reqwest::Client` if the client was successfully created, or an `Error` if the client could not be created.
///
/// # Example
///
/// ```rust
/// let client = create_reqwest_client();
/// ```
///
/// # Errors
///
/// This function will return an error if the `reqwest::Client` could not be built.
fn create_reqwest_client() -> Result<Client, Error> {
    // fetch env variable
    let github_token = std::env::var("GITHUB_TOKEN");

    let mut headers = HeaderMap::new();

    if let Ok(github_token) = github_token {
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", github_token)).unwrap(),
        );
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    Ok(client)
}

/// The `Cli` enum represents the different commands that can be used in the command-line interface.
#[derive(Debug, Parser)]
#[command(version)]
enum Cli {
    /// Switch to the specified version, by default will auto-invoke
    /// install command if the version is not installed already
    Use {
        /// Version to switch to |nightly|stable|<version-string>|<commit-hash>|
        ///
        /// A version-string can either be `vx.x.x` or `x.x.x` examples: `v0.6.1` and `0.6.0`
        version: String,

        /// Whether not to auto-invoke install command
        #[arg(short, long)]
        no_install: bool,
    },

    /// Install the specified version, can also be used to update
    /// out-of-date nightly version
    Install {
        /// Version to be installed |nightly|stable|<version-string>|<commit-hash>|
        ///
        /// A version-string can either be `vx.x.x` or `x.x.x` examples: `v0.6.1` and `0.6.0`
        version: String,
    },

    /// If Config::version_sync_file_location is set, the version in that file
    /// will be parsed and installed
    Sync,

    /// Uninstall the specified version
    #[clap(alias = "remove", visible_alias = "rm")]
    Uninstall {
        /// Optional Version to be uninstalled |nightly|stable|<version-string>|<commit-hash>|
        ///
        /// A version-string can either be `vx.x.x` or `x.x.x` examples: `v0.6.1` and `0.6.0`
        ///
        /// If no Version is provided a prompt is used to select the versions to be uninstalled
        version: Option<String>,
    },

    /// Rollback to an existing nightly rollback
    Rollback,

    /// Erase any change bob ever made, including neovim installation,
    /// neovim version downloads and registry changes
    Erase,

    /// List all installed and used versions
    #[clap(visible_alias = "ls")]
    List,

    #[clap(visible_alias = "ls-remote")]
    ListRemote,

    /// Generate shell completion
    Complete {
        /// Shell to generate completion for
        shell: Shell,
    },

    /// Update existing version |nightly|stable|--all|
    Update(Update),

    #[clap(trailing_var_arg = true)]
    Run {
        /// Optional version to run |nightly|stable|<version-string>|<commit-hash>|
        version: String,

        /// Arguments to pass to Neovim (flags, files, commands, etc.)
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// Represents an update command in the CLI.
///
/// This struct contains options for the update command, such as the version to update and whether to update all versions.
///
/// # Fields
///
/// * `version: Option<String>` - The version to update. This can be either "nightly" or "stable". This field conflicts with the `all` field, meaning you can't specify a version and use `all` at the same time.
/// * `all: bool` - Whether to apply the update to all versions. If this is `true`, the `version` field must be `None`.
///
/// # Example
///
/// ```rust
/// let update = Update {
///     version: Some("nightly".to_string()),
///     all: false,
/// };
/// ```
#[derive(Args, Debug)]
pub struct Update {
    /// Update specified version |nightly|stable|
    #[arg(conflicts_with = "all")]
    pub version: Option<String>,

    /// Apply the update to all versions
    #[arg(short, long)]
    pub all: bool,
}

/// Starts the CLI application.
///
/// This function takes a `Config` object as input and returns a `Result`. It creates a Reqwest client, parses the CLI arguments, and then handles the arguments based on their type.
///
/// # Arguments
///
/// * `config: Config` - The configuration for the application.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result`. If the function completes successfully, it returns `Ok(())`. If an error occurs, it returns `Err`.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// start(config).await.unwrap();
/// ```
pub async fn start(config: ConfigFile) -> Result<()> {
    let client = create_reqwest_client()?;
    let cli = Cli::parse();

    match cli {
        Cli::Use {
            version,
            no_install,
        } => {
            let version = super::version::parse_version_type(&client, &version).await?;

            handlers::use_handler::start(version, !no_install, &client, config).await?;
        }
        Cli::Install { version } => {
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
                InstallResult::GivenNightlyRollback => (),
            }
        }
        Cli::Sync => {
            info!("Starting sync process");
            sync_handler::start(&client, config).await?;
        }
        Cli::Uninstall { version } => {
            info!("Starting uninstallation process");
            uninstall_handler::start(version.as_deref(), config.config).await?;
        }
        Cli::Rollback => rollback_handler::start(config.config).await?,
        Cli::Erase => erase_handler::start(config.config).await?,
        Cli::List => list_handler::start(config.config).await?,
        Cli::Complete { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "bob", &mut std::io::stdout())
        }
        Cli::Update(data) => {
            update_handler::start(data, &client, config).await?;
        }
        Cli::ListRemote => list_remote_handler::start(config, client).await?,
        Cli::Run { version, args } => run_handler::start(&version, &args, &client, &config).await?,
    }

    Ok(())
}
