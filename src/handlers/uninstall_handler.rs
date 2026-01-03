use crate::{
    config::Config,
    helpers::{self, directories},
};
use anyhow::{Result, anyhow};
use dialoguer::{
    Confirm, MultiSelect,
    console::{Term, style},
    theme::ColorfulTheme,
};
use futures_util::stream::{self, StreamExt};
use reqwest::Client;
use tokio::fs as async_fs;
use tracing::{info, warn};

/// Starts the uninstall process.
///
/// This function creates a new HTTP client, determines the version to uninstall, checks if the version is currently in use, and if not, removes the version's directory.
///
/// # Arguments
///
/// * `version` - An optional string that represents the version to uninstall. If `None`, the function will call `uninstall_selections` to allow the user to select versions to uninstall.
/// * `config` - The configuration for the uninstall process.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the uninstall process was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The version cannot be parsed.
/// * The version is currently in use.
/// * The downloads directory cannot be determined.
/// * The version's directory cannot be removed.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// start(Some("1.0.0"), config).await.unwrap();
/// ```
pub async fn start(version: Option<&str>, config: Config) -> Result<()> {
    let client = Client::new();

    let Some(version) = version else {
        return uninstall_selections(&config).await;
    };

    let version = helpers::version::parse_version_type(&client, version).await?;
    if helpers::version::is_version_used(&version.non_parsed_string, &config).await {
        warn!("Switch to a different version before proceeding");
        return Ok(());
    }

    let downloads_dir = match directories::get_downloads_directory(&config).await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };

    let path = downloads_dir.join(&version.tag_name);

    async_fs::remove_dir_all(&path).await?;

    // Clean up empty parent directories
    directories::remove_empty_parents(&path, &downloads_dir)?;

    info!(
        "Successfully uninstalled version: {}",
        version.non_parsed_string
    );
    Ok(())
}

/// Uninstalls selected versions.
///
/// This function recursively searches the downloads directory for all installed versions,
/// presents a list of available versions to the user, allows them to select versions to
/// uninstall, and then uninstalls the selected versions.
///
/// # Arguments
///
/// * `config` - The configuration for the uninstall process.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the uninstall process was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The downloads directory cannot be read.
/// * The version directories cannot be found recursively.
/// * The user aborts the uninstall process.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// uninstall_selections(&config).await.unwrap();
/// ```
async fn uninstall_selections(config: &Config) -> Result<()> {
    let downloads_dir = directories::get_downloads_directory(config).await?;

    // Recursively find all version directories (build directories are filtered out)
    let paths = directories::find_version_dirs(&downloads_dir, &downloads_dir)?;

    if paths.is_empty() {
        info!("There are no versions installed");
        return Ok(());
    }

    // Filter out currently used versions and collect version names with their paths
    let installed_versions = stream::iter(paths)
        .filter_map(|path| {
            let downloads_dir = downloads_dir.clone();
            async move {
                let version_name = path.strip_prefix(&downloads_dir).unwrap().to_str().unwrap();

                if !helpers::version::is_version_used(version_name, config).await {
                    Some((version_name.to_owned(), path))
                } else {
                    None
                }
            }
        })
        .collect::<Vec<_>>()
        .await;

    if installed_versions.is_empty() {
        info!("You only have one neovim instance installed");
        return Ok(());
    }

    let theme = ColorfulTheme {
        checked_item_prefix: style("✓".to_string()).for_stderr().green(),
        unchecked_item_prefix: style("✓".to_string()).for_stderr().black(),
        ..ColorfulTheme::default()
    };

    let version_names: Vec<&str> = installed_versions
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();

    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Toogle with space the versions you wish to uninstall:")
        .items(&version_names)
        .interact_on_opt(&Term::stderr())?;

    match &selections {
        Some(ids) if !ids.is_empty() => {
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you wish to continue?")
                .interact_on_opt(&Term::stderr())?;

            if !matches!(confirm, Some(true)) {
                info!("Uninstall aborted...");
                return Ok(());
            }

            for &i in ids {
                let (version_name, path) = &installed_versions[i];
                async_fs::remove_dir_all(path).await?;

                // Clean up empty parent directories
                directories::remove_empty_parents(path, &downloads_dir)?;

                info!("Successfully uninstalled version: {}", version_name);
            }
        }
        None | Some(_) => info!("Uninstall aborted..."),
    }
    Ok(())
}
