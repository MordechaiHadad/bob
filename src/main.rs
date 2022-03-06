mod models;
mod modules;

extern crate core;

use anyhow::{anyhow, Result};
use std::process::exit;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(error) = run().await {
        eprintln!("Error: {error}");
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
