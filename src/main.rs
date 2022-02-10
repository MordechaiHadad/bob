mod models;
mod modules;

extern crate core;

use anyhow::{anyhow, Result};
use clap::{arg, App};
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
    let app = App::new("bob")
        .subcommand(App::new("use").arg(arg!([VERSION]).required(true)))
        .get_matches();

    if let Some(subcommand) = app.subcommand_matches("use") {
        if let Err(error) = modules::use_handler::start(subcommand).await {
            return Err(anyhow!(error));
        }
    }
    Ok(())
}
