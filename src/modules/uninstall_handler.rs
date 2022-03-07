use clap::ArgMatches;
use anyhow::{anyhow, Result};
use reqwest::Client;
use crate::modules::utils;
use tokio::fs;

pub async fn start(subcommand: &ArgMatches) -> Result<()> {
    let client = Client::new();
    let version = if let Some(value) = subcommand.value_of("VERSION") {
        match utils::parse_version(&client, value).await {
            Ok(version) => version,
            Err(error) => return Err(anyhow!(error)),
        }
    } else {
        return Err(anyhow!("Todo.."));
    };


    if utils::is_version_used(&version).await {
        println!("Switch to a different version before proceeding");
        return Ok(());
    }

    let downloads_dir = match utils::get_downloads_folder().await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };

    fs::remove_dir_all(&format!("{}/{version}", downloads_dir.display())).await?;
    println!("Successfully uninstalled version: {version}");
   Ok(())
}