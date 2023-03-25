use anyhow::{anyhow, Result};
use std::path::PathBuf;

use crate::config::Config;

pub fn get_home_dir() -> Result<PathBuf> {
    let mut home_str = PathBuf::new();

    if cfg!(windows) {
        home_str.push(std::env::var("USERPROFILE")?);
        return Ok(home_str);
    }

    if let Ok(value) = std::env::var("HOME") {
        home_str = PathBuf::from(value);
        return Ok(home_str)
    }
    
    if cfg!(target_os = "macos") {
        home_str.push("/Users/");
    } else {
        home_str.push("/home/")
    };

    if let Ok(value) = std::env::var("SUDO_USER") {
        home_str.push(&value);
        return Ok(home_str);
    }

    let env_value = std::env::var("USER")?;
    home_str.push(&env_value);

    Ok(home_str)
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

    if cfg!(windows) {
        home_dir.push("AppData/Roaming");
    } else if cfg!(target_os = "macos") {
        home_dir.push("Library/Application Support");
    } else {
        home_dir.push(".config");
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
            let does_folder_exist = tokio::fs::metadata(&data_dir).await.is_ok();
            let is_folder_created = tokio::fs::create_dir_all(&data_dir).await.is_ok();

            if !does_folder_exist && !is_folder_created {
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
