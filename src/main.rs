mod enums;
mod models;
mod modules;

extern crate core;

use anyhow::{anyhow, Result};
use models::Config;
use modules::utils;
use regex::Regex;
use std::{
    env,
    process::{exit, Command}, path::Path,
};
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

    let args: Vec<String> = env::args().collect();

    let exe_name_path = Path::new(&args[0]);
    let exe_name = exe_name_path.file_name().unwrap().to_str().unwrap();


    if !exe_name.contains("bob") {
        let rest_args = &args[1..];

        let downloads_dir = utils::get_downloads_folder(&config).await?;
        let platform = utils::get_platform_name();
        let used_version = utils::get_current_version(&config).await?;

        let location = downloads_dir
            .join(used_version)
            .join(platform)
            .join("bin")
            .join("nvim");

        let mut child = Command::new(location)
            .args(rest_args)
            .spawn()
            .expect("Failed to spawn child process");

        let exit_status = child
            .wait()
            .expect("Failed to wait on child process")
            .code();

        match exit_status {
            Some(0) => return Ok(()),
            Some(code) => return Err(anyhow!("Process exited with error code {}", code)),
            None => return Err(anyhow!("Process terminated by signal")),
        }
    }

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
            downloads_location: None,
            installation_location: None,
            version_sync_file_location: None,
            rollback_limit: None,
        },
    };

    Ok(config)
}

fn handle_envars(config: &mut Config) -> Result<()> {
    let re = Regex::new(r"\$([A-Z_]+)").unwrap();

    handle_envar(&mut config.downloads_location, &re)?;

    handle_envar(&mut config.installation_location, &re)?;

    handle_envar(&mut config.version_sync_file_location, &re)?;

    Ok(())
}

fn handle_envar(item: &mut Option<String>, re: &Regex) -> Result<()> {
    let value = if let Some(value) = item.as_ref() {
        value
    } else {
        return Ok(());
    };

    if re.is_match(value) {
        let extract = re.captures(value).unwrap().get(1).unwrap().as_str();
        let var =
            env::var(extract).unwrap_or(format!("Couldn't find {extract} environment variable"));

        *item = Some(value.replace(&format!("${extract}"), &var))
    }

    Ok(())
}
