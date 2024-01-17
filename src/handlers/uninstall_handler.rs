use anyhow::{anyhow, Result};
use dialoguer::{
    console::{style, Term},
    theme::ColorfulTheme,
    Confirm, MultiSelect,
};
use reqwest::Client;
use tokio::fs;
use tracing::{info, warn};

use crate::{
    config::Config,
    helpers::{self, directories},
};

pub async fn start(version: Option<&str>, config: Config) -> Result<()> {
    let client = Client::new();

    let version = match version {
        Some(value) => value,
        None => return uninstall_selections(&client, &config).await,
    };

    let version = helpers::version::parse_version_type(&client, version).await?;
    if helpers::version::is_version_used(&version.tag_name, &config).await {
        warn!("Switch to a different version before proceeding");
        return Ok(());
    }

    let downloads_dir = match directories::get_downloads_directory(&config).await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };

    let path = downloads_dir.join(&version.tag_name);

    fs::remove_dir_all(path).await?;
    info!("Successfully uninstalled version: {}", version.tag_name);
    Ok(())
}

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

        if helpers::version::is_version_used(&version.tag_name, config).await {
            continue;
        }
        installed_versions.push(version.tag_name);
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
