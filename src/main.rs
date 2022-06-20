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
    let config: Config = match tokio::fs::read_to_string("bob.json").await {
        Ok(config_file) => serde_json::from_str(&config_file)?,
        Err(_) => Config {
            enable_nightly_info: Some(true),
        },
    };
    if let Err(error) = modules::cli::start(config).await {
        return Err(anyhow!(error));
    }
    Ok(())
}
