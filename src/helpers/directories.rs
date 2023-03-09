use std::path::PathBuf;
use anyhow::{anyhow, Result};

use crate::config::Config;

pub fn get_home_dir() -> Result<PathBuf> {
    if cfg!(windows) {
        let home_str = std::env::var("USERPROFILE")?;
        return Ok(PathBuf::from(home_str));
    }

    let mut home_str = "/home/".to_string();
    if let Ok(value) = std::env::var("SUDO_USER") {
        home_str.push_str(&value);

        return Ok(PathBuf::from(home_str));
    }

    let env_value = std::env::var("USER")?;
    home_str.push_str(&env_value);

    Ok(PathBuf::from(home_str))
}

pub fn get_local_data_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_dir()?;
    if cfg!(windows) {
        home_dir.push("AppData/Local");
        return Ok(home_dir);
    }

    home_dir.push(".local/share");
    Ok(home_dir)
}

pub fn get_config_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_dir()?;

    if cfg!(linux) {
        home_dir.push(".config");
    } else if cfg!(macos) {
        home_dir.push("Library/Application Support");
    } else {
        home_dir.push("AppData/Roaming");
    }

    home_dir.push("bob/config.json");

    Ok(home_dir)
}

pub async fn get_downloads_directory(config: &Config) -> Result<PathBuf> {
    let path = match &config.downloads_location {
        Some(path) => {
            if tokio::fs::metadata(path).await.is_err() {
                return Err(anyhow!("Custom directory {path} doesn't exist!"));
            }

            PathBuf::from(path)
        }
        None => {
            let mut data_dir = get_local_data_dir()?;

            data_dir.push("bob");
            println!("{}", data_dir.display());
            let does_folder_exist = tokio::fs::metadata(&data_dir).await.is_ok();

            if !does_folder_exist && tokio::fs::create_dir(&data_dir).await.is_err() {
                return Err(anyhow!("Couldn't create downloads directory"));
            }
            data_dir
        }
    };

    Ok(path)
}

pub async fn get_installation_directory(config: &Config) -> Result<PathBuf> {
    match &config.installation_location {
        Some(path) => Ok(PathBuf::from(path.clone())),
        None => {
  
            let mut installation_location = get_downloads_directory(config).await?;
            installation_location.push("nvim-bin");

            Ok(installation_location)
        }
    }
}
