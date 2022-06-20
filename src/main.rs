mod enums;
mod models;
mod modules;

extern crate core;

use anyhow::{anyhow, Result};
use models::Config;
use std::process::exit;
use tracing::{error, Level};

#[tokio::main]
async fn main() -> Result<()> {
    let collector = tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(collector)?;
    if let Err(error) = run().await {
        error!("Error: {error}");
        exit(1);
    }
    Ok(())
}

async fn run() -> Result<()> {
    let config_dir = dirs::config_dir().unwrap();
    let config_file = format!("{}/bob/config.json", config_dir.to_str().unwrap());
    let config: Config = match tokio::fs::read_to_string(config_file).await {
        Ok(config_file) => serde_json::from_str(&config_file)?,
        Err(_) => Config {
            enable_nightly_info: None,
            downloads_dir: None,
            installation_location: None,
        },
    };
    if let Err(error) = modules::cli::start(config).await {
        return Err(anyhow!(error));
    }
    Ok(())
}
