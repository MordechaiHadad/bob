use crate::{
    config::{Config, ConfigFile},
    helpers::{self, directories},
};
use anyhow::{anyhow, Result};
use dialoguer::{
    console::{style, Term},
    theme::ColorfulTheme,
    Confirm, MultiSelect,
};
use regex::Regex;
use reqwest::Client;
use tokio::fs;
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

    let version = match version {
        Some(value) => value,
        None => return uninstall_selections(&client, &config).await,
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

    let version_regex = Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+$")?;
    let path = if version_regex.is_match(&version.non_parsed_string) {
        let intermediate = format!("v{}", &version.non_parsed_string);
        downloads_dir.join(intermediate)
    } else {
        downloads_dir.join(&version.non_parsed_string)
    };

    fs::remove_dir_all(path).await?;
    info!(
        "Successfully uninstalled version: {}",
        version.non_parsed_string
    );
    Ok(())
}

/// Uninstalls selected versions.
///
/// This function reads the versions from the downloads directory, presents a list of installed versions to the user, allows them to select versions to uninstall, and then uninstalls the selected versions.
///
/// # Arguments
///
/// * `client` - The HTTP client to be used for network requests.
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
/// * The version cannot be parsed from the file name.
/// * The version is currently in use.
/// * The user aborts the uninstall process.
///
/// # Example
///
/// ```rust
/// let client = Client::new();
/// let config = Config::default();
/// uninstall_selections(&client, &config).await.unwrap();
/// ```
async fn uninstall_selections(client: &Client, config: &Config) -> Result<()> {
    let downloads_dir = directories::get_downloads_directory(config).await?;

    let mut paths = fs::read_dir(downloads_dir.clone()).await?;
    let mut installed_versions: Vec<String> = Vec::new();

    while let Some(path) = paths.next_entry().await? {
        let name = path.file_name().to_str().unwrap().to_owned();

        let version = match helpers::version::parse_version_type(client, &name).await {
            Ok(value) => value,
            Err(_) => continue,
        };

        if helpers::version::is_version_used(&version.non_parsed_string, config).await {
            continue;
        }
        installed_versions.push(version.non_parsed_string);
    }

    if installed_versions.is_empty() {
        info!("You only have one neovim instance installed");
        return Ok(());
    }

    let theme = ColorfulTheme {
        checked_item_prefix: style("✓".to_string()).for_stderr().green(),
        unchecked_item_prefix: style("✓".to_string()).for_stderr().black(),
        ..ColorfulTheme::default()
    };

    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Toogle with space the versions you wish to uninstall:")
        .items(&installed_versions)
        .interact_on_opt(&Term::stderr())?;

    match &selections {
        Some(ids) if !ids.is_empty() => {
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you wish to continue?")
                .interact_on_opt(&Term::stderr())?;

            match confirm {
                Some(true) => {}
                None | Some(false) => {
                    info!("Uninstall aborted...");
                    return Ok(());
                }
            }

            for &i in ids {
                let path = downloads_dir.join(&installed_versions[i]);
                fs::remove_dir_all(path).await?;
                info!(
                    "Successfully uninstalled version: {}",
                    &installed_versions[i]
                );
            }
        }
        None | Some(_) => info!("Uninstall aborted..."),
    }
    Ok(())
}
