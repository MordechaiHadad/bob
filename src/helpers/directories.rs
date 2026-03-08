use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

/// Returns the local data directory path for the current user.
///
/// On Unix systems, if running under sudo, it appends ".local/share" (or "Library/Application Support" on macOS) to the real user's home directory.
/// Otherwise, it relies on the `dirs` crate which respects `XDG_DATA_HOME` on Linux.
///
/// # Returns
///
/// This function returns a `Result` that contains a `PathBuf` representing the local data directory path if the operation was successful.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Example
///
/// ```rust
/// let local_data_dir = get_local_data_dir()?;
/// ```
fn get_local_data_dir() -> Result<PathBuf> {
    #[cfg(unix)]
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if let Some(user_info) = users::get_user_by_name(&sudo_user) {
            let mut home: PathBuf = user_info.home_dir().into();
            if cfg!(target_os = "macos") {
                home.push("Library/Application Support");
            } else {
                home.push(".local/share");
            }
            return Ok(home);
        }
    }

    dirs::data_local_dir().ok_or_else(|| anyhow!("Could not determine local data directory"))
}

/// Returns the configuration file path for the current user.
///
/// This function prioritizes the `BOB_CONFIG` environment variable.
/// On Unix systems, if running under sudo, it appends ".config" (or "Library/Application Support" on macOS) to the real user's home directory.
/// Otherwise, it relies on the `dirs` crate which respects `XDG_CONFIG_HOME` on Linux.
///
/// # Returns
///
/// This function returns a `Result` that contains a `PathBuf` representing the config file path if the operation was successful.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Example
///
/// ```rust
/// let config_file = get_config_file()?;
/// ```
pub fn get_config_file() -> Result<PathBuf> {
    if let Ok(value) = std::env::var("BOB_CONFIG") {
        return Ok(PathBuf::from(value));
    }

    let mut config_dir = {
        #[allow(unused_mut)]
        let mut dir = None;
        #[cfg(unix)]
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            if let Some(user_info) = users::get_user_by_name(&sudo_user) {
                let mut home: PathBuf = user_info.home_dir().into();
                if cfg!(target_os = "macos") {
                    home.push("Library/Application Support");
                } else {
                    home.push(".config");
                }
                dir = Some(home);
            }
        }
        dir
    };

    if config_dir.is_none() {
        config_dir = Some(
            dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?,
        );
    }

    let mut config_dir = config_dir.unwrap();

    config_dir.push("bob/config.toml");

    if fs::metadata(&config_dir).is_err() {
        config_dir.pop();
        config_dir.push("config.json");
    }

    Ok(config_dir)
}

/// Asynchronously returns the 'downloads' directory path based on the application configuration.
///
/// This function takes a reference to a `Config` as an argument, which contains the application configuration.
/// It first checks if the `downloads_location` field in the `Config` is set. If it is, it checks if the directory exists. If the directory does not exist, it returns an error.
/// If the `downloads_location` field in the `Config` is not set, it gets the local data directory path by calling the `get_local_data_dir` function and appends "bob" to it.
/// It then checks if the "bob" directory exists. If the directory does not exist, it attempts to create it. If the creation fails, it returns an error.
///
/// # Arguments
///
/// * `config` - A reference to a `Config` containing the application configuration.
///
/// # Returns
///
/// This function returns a `Result` that contains a `PathBuf` representing the downloads directory path if the operation was successful.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let downloads_directory = get_downloads_directory(&config).await?;
/// ```
pub async fn get_downloads_directory(config: &Config) -> Result<PathBuf> {
    let path = if let Some(path) = &config.downloads_location {
        if tokio::fs::metadata(path).await.is_err() {
            return Err(anyhow!("Custom directory {path} doesn't exist!"));
        }

        PathBuf::from(path)
    } else {
        let mut data_dir = get_local_data_dir()?;

        data_dir.push("bob");
        let does_folder_exist = tokio::fs::metadata(&data_dir).await.is_ok();
        let is_folder_created = tokio::fs::create_dir_all(&data_dir).await.is_ok();

        if !does_folder_exist && !is_folder_created {
            return Err(anyhow!("Couldn't create downloads directory"));
        }
        data_dir
    };

    Ok(path)
}

/// Asynchronously returns the installation directory path based on the application configuration.
///
/// This function takes a reference to a `Config` as an argument, which contains the application configuration.
/// It first checks if the `installation_location` field in the `Config` is set. If it is, it returns the value of this field as a `PathBuf`.
/// If the `installation_location` field in the `Config` is not set, it gets the downloads directory path by calling the `get_downloads_directory` function and appends "nvim-bin" to it.
///
/// # Arguments
///
/// * `config` - A reference to a `Config` containing the application configuration.
///
/// # Returns
///
/// This function returns a `Result` that contains a `PathBuf` representing the installation directory path if the operation was successful.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let installation_directory = get_installation_directory(&config).await?;
/// ```
pub async fn get_installation_directory(config: &Config) -> Result<PathBuf> {
    if let Some(path) = &config.installation_location {
        Ok(PathBuf::from(path.clone()))
    } else {
        let mut installation_location = get_downloads_directory(config).await?;
        installation_location.push("nvim-bin");

        Ok(installation_location)
    }
}
