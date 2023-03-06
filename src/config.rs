use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use tokio::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub enable_nightly_info: Option<bool>,
    pub downloads_location: Option<String>,
    pub installation_location: Option<String>,
    pub version_sync_file_location: Option<String>,
    pub rollback_limit: Option<u8>,
}

pub async fn handle_config() -> Result<Config> {
    let config_file = crate::helpers::directories::get_config_dir()?;
    let config = match fs::read_to_string(config_file).await {
        Ok(config) => {
            let mut config: Config = serde_json::from_str(&config)?;
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
