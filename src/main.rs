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
    if let Err(error) = modules::cli::start(config).await {
        return Err(anyhow!(error));
    }
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
        },
    };

    Ok(config)
}

fn handle_envars(config: &mut Config) -> Result<()> {
    let re = Regex::new(r"\$([A-Z_]+)").unwrap();

    if let Some(value) = &config.downloads_dir {
        let extract = re.captures(value).unwrap().get(1).unwrap().as_str();

        let var = env::var(extract)?;

        let new_value = value.replace(&format!("${extract}"), &var);
        config.downloads_dir = Some(new_value);
    }

    if let Some(value) = &config.installation_location {
        let extract = re.captures(value).unwrap().get(1).unwrap().as_str();

        let var = env::var(extract)?;

        let new_value = value.replace(&format!("${extract}"), &var);
        config.installation_location = Some(new_value);
    }

    Ok(())
}
