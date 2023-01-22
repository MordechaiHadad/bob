mod enums;
mod models;
mod modules;

extern crate core;

use anyhow::{anyhow, Result};
use models::Config;
use regex::Regex;
use std::{env, process::exit};
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
    let config_dir = dirs::config_dir().ok_or_else(|| anyhow!("config directory not found"))?;
    let config_file = config_dir.join("bob").join("config.json");
    let config: Config = handle_config(tokio::fs::read_to_string(config_file).await)?;
    modules::cli::start(config).await?;
    Ok(())
}

fn handle_config(config_file: Result<String, std::io::Error>) -> Result<Config> {
    let config = match config_file {
        Ok(config_file) => {
            let mut config: Config = serde_json::from_str(&config_file)?;
            handle_envars(&mut config)?;
            config
        }
        Err(_) => Config {
            enable_nightly_info: None,
            downloads_dir: None,
            installation_location: None,
            sync_version_file_path: None,
            rollback_limit: None,
        },
    };

    Ok(config)
}

fn handle_envars(config: &mut Config) -> Result<()> {
    let re = Regex::new(r"\$([A-Z_]+)").unwrap();

    if let Some(value) = &config.downloads_dir {
        if re.is_match(value) {
            let new_value = handle_envar(value, &re)?;
            config.downloads_dir = Some(new_value);
        }
    }

    if let Some(value) = &config.installation_location {
        if re.is_match(value) {
            let new_value = handle_envar(value, &re)?;
            config.installation_location = Some(new_value);
        }
    }

    if let Some(value) = &config.sync_version_file_path {
        if re.is_match(value) {
            let new_value = handle_envar(value, &re)?;
            config.sync_version_file_path = Some(new_value);
        }
    }

    Ok(())
}

fn handle_envar(value: &str, re: &Regex) -> Result<String> {
    let extract = re.captures(value).unwrap().get(1).unwrap().as_str();

    let var = env::var(extract)?;

    Ok(value.replace(&format!("${extract}"), &var))
}
