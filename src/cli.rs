use crate::{
    config::ConfigFile,
    github_requests::GitHubClient,
    handlers::{
        self, InstallResult, erase_handler, list_handler, list_remote_handler, rollback_handler,
        run_handler, sync_handler, uninstall_handler, update_handler,
    },
    helpers::processes::is_neovim_running,
    version::parse_version_type,
};
use anyhow::Result;
use clap::{ArgAction, Args, CommandFactory, Parser, ValueEnum};
use clap_complete::shells;
use reqwest::Client;
use std::sync::OnceLock;
use tracing::{debug, info};
use tracing_subscriber::Registry;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::reload;
use tracing_subscriber::util::SubscriberInitExt;

/// Global handle used by [`setup_tracing`] to hot-swap the log filter that
/// was originally installed by [`init_tracing`].
static FILTER_RELOAD_HANDLE: OnceLock<reload::Handle<EnvFilter, Registry>> = OnceLock::new();

struct Clients {
    github: GitHubClient,
    download: Client,
}

fn create_clients() -> Result<Clients> {
    let github = GitHubClient::new()?;
    let download = Client::builder()
        .user_agent("bob")
        .build()?;

    Ok(Clients { github, download })
}

/// Top-level CLI options wrapper.
///
/// This struct captures global flags (like verbosity) and flattens the
/// subcommand enum so that `-v`/`--verbose` can appear before any subcommand.
#[derive(Debug, Parser)]
#[command(version, about = "A version manager for neovim", long_about = None)]
pub struct Opts {
    /// Increase verbosity (-v for DEBUG, -vv for TRACE, -vvv for global TRACE)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
    pub verbose: u8,

    /// The subcommand to run (e.g., `use`, `install`, `list`)
    #[command(subcommand)]
    pub command: Cli,
}

// The `Cli` enum represents the different commands that can be used in the command-line interface.
#[derive(Debug, Parser)]
pub(crate) enum Cli {
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

    /// If `Config::version_sync_file_location` is set, the version in that file
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

impl Cli {
    /// Utility method to determine if the current command needs a running neovim check.
    /// Only used at the start of the program.
    fn needs_running_check(&self) -> bool {
        matches!(
            self,
            Cli::Use { .. }
                | Cli::Install { .. }
                | Cli::Sync
                | Cli::Uninstall { .. }
                | Cli::Rollback
                | Cli::Update(_)
        )
    }
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

#[derive(Clone, Copy, Debug, ValueEnum)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Shell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    PowerShell,
    Zsh,
}

impl clap_complete::Generator for Shell {
    fn file_name(&self, name: &str) -> String {
        match self {
            Shell::Bash => shells::Bash.file_name(name),
            Shell::Elvish => shells::Elvish.file_name(name),
            Shell::Fish => shells::Fish.file_name(name),
            Shell::Nushell => clap_complete_nushell::Nushell.file_name(name),
            Shell::PowerShell => shells::PowerShell.file_name(name),
            Shell::Zsh => shells::Zsh.file_name(name),
        }
    }

    fn generate(&self, cmd: &clap::Command, buf: &mut dyn std::io::Write) {
        self.try_generate(cmd, buf)
            .expect("failed to write completion file");
    }

    fn try_generate(
        &self,
        cmd: &clap::Command,
        buf: &mut dyn std::io::Write,
    ) -> Result<(), std::io::Error> {
        match self {
            Shell::Bash => shells::Bash.try_generate(cmd, buf),
            Shell::Elvish => shells::Elvish.try_generate(cmd, buf),
            Shell::Fish => shells::Fish.try_generate(cmd, buf),
            Shell::Nushell => clap_complete_nushell::Nushell.try_generate(cmd, buf),
            Shell::PowerShell => shells::PowerShell.try_generate(cmd, buf),
            Shell::Zsh => shells::Zsh.try_generate(cmd, buf),
        }
    }
}

/// Installs the global tracing subscriber with a reload-capable filter layer.
///
/// The subscriber starts at **INFO** level. Call [`setup_tracing`] later to
/// hot-swap the filter based on CLI verbosity / `RUST_LOG`.
///
/// This must be called exactly once, at the very start of `main`.
///
/// # Errors
///
/// Returns an error if the subscriber or reload handle has already been
/// initialised.
pub fn init_tracing() -> Result<()> {
    let env_filter = EnvFilter::new("info");
    let (filter_layer, reload_handle) = reload::Layer::new(env_filter);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    FILTER_RELOAD_HANDLE
        .set(reload_handle)
        .map_err(|_| anyhow::anyhow!("Tracing reload handle already initialised"))?;

    Ok(())
}

/// Reconfigures the global tracing filter based on CLI verbosity and `RUST_LOG`.
///
/// Precedence:
/// 1. If `verbose > 0`: 1 → DEBUG, 2+ → TRACE.
/// 2. Else if `RUST_LOG` is set and valid, use it via [`EnvFilter::try_from_default_env`].
/// 3. Otherwise keep the existing INFO default (no-op).
///
/// # Errors
///
/// Returns an error if [`init_tracing`] was not called first, or if the
/// filter reload fails.
pub fn setup_tracing(verbose: u8) -> Result<()> {
    let env_filter = if verbose > 0 {
        let crate_name = env!("CARGO_CRATE_NAME");
        match verbose {
            1 => EnvFilter::new(format!("{crate_name}=debug")),
            2 => EnvFilter::new(format!("{crate_name}=trace")),
            _ => EnvFilter::new("trace"),
        }
    } else {
        match EnvFilter::try_from_default_env() {
            Ok(filter) => filter,
            Err(_) => return Ok(()),
        }
    };

    let handle = FILTER_RELOAD_HANDLE
        .get()
        .ok_or_else(|| anyhow::anyhow!("Tracing not initialised — call init_tracing first"))?;

    handle.reload(env_filter)?;

    debug!("Tracing subscriber reconfigured (verbose={verbose})");

    Ok(())
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
    let Clients { github, download } = create_clients()?;
    let opts = Opts::parse();

    setup_tracing(opts.verbose)?;

    let cli = opts.command;

    if cli.needs_running_check()
        && !config.config.ignore_running_instances.unwrap_or(true)
        && is_neovim_running()
    {
        return Err(anyhow::anyhow!(
            "Neovim is currently running. Please close it before switching versions."
        ));
    }

    match cli {
        Cli::Use {
            version,
            no_install,
        } => {
            let version = parse_version_type(&github, &version).await?;

            handlers::use_handler::start(version, !no_install, &github, &download, config).await?;
        }
        Cli::Install { version } => {
            let version = parse_version_type(&github, &version).await?;
            let tag_name: &str = version.tag_name.as_str();

            match handlers::install_handler::start(&version, &github, &download, &config).await? {
                InstallResult::InstallationSuccess(location) => {
                    info!("{tag_name} has been successfully installed in {location}",);
                }
                InstallResult::VersionAlreadyInstalled => {
                    info!("{tag_name} is already installed");
                }
                InstallResult::NightlyIsUpdated => {
                    info!("Nightly up to date!");
                }
                InstallResult::GivenNightlyRollback => (),
            }
        }
        Cli::Sync => {
            info!("Starting sync process");
            sync_handler::start(&github, &download, config).await?;
        }
        Cli::Uninstall { version } => {
            info!("Starting uninstallation process");
            uninstall_handler::start(version.as_deref(), &github, config.config).await?;
        }
        Cli::Rollback => rollback_handler::start(config.config).await?,
        Cli::Erase => erase_handler::start(config.config).await?,
        Cli::List => list_handler::start(config.config).await?,
        Cli::Complete { shell } => {
            clap_complete::generate(shell, &mut Opts::command(), "bob", &mut std::io::stdout());
        }
        Cli::Update(data) => {
            update_handler::start(data, &github, &download, config).await?;
        }
        Cli::ListRemote => list_remote_handler::start(config.config, &github).await?,
        Cli::Run { version, args } => {
            run_handler::start(&version, &args, &github, &download, &config.config).await?;
        }
    }

    Ok(())
}
