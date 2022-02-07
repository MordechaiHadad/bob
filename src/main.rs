mod modules;

extern crate core;

use anyhow::{anyhow, Result};
use clap::{arg, App, Arg, ArgMatches};
use regex::Regex;
use std::process::exit;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(error) = real_main().await {
        eprintln!("Error: {error}");
        exit(1);
    }
    Ok(())
}

async fn real_main() -> Result<()> {
    let app = App::new("bob")
        .subcommand(App::new("use").arg(arg!([VERSION]).required(true)))
        .get_matches();

    if let Some(subcommand) = app.subcommand_matches("use") {
        if let Some(value) = subcommand.value_of("VERSION") {
            let version = String::from(value);
            let version = match parse_version(version).await {
                Ok(value) => value,
                Err(error) => return Err(anyhow!(error)),
            };
            if let Err(error) = modules::use_handler::start(&version).await {
                return Err(anyhow!(error));
            }
        }
    }
    Ok(())
}
async fn parse_version(mut version: String) -> Result<String> {
    match version.as_str() {
        "nightly" | "stable" => Ok(version),
        _ => {
            let regex = Regex::new(r"^v?[0-9]+\.[0-9]+\.[0-9]+$").unwrap();
            if regex.is_match(version.as_str()) {
                if !version.contains("v") {
                    version = format!("v{}", version);
                }
                return Ok(version);
            }
            Err(anyhow!("Please provide a proper version string"))
        }
    }
}
