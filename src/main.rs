mod models;
mod modules;
mod enums;

extern crate core;

use anyhow::{anyhow, Result};
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
    if let Err(error) = modules::cli::start().await {
        return Err(anyhow!(error));
    }
    Ok(())
}
