use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

/// Returns the home directory path for the current user.
///
/// This function checks the target operating system using the `cfg!` macro and constructs the home directory path accordingly.
/// For Windows, it uses the "USERPROFILE" environment variable.
/// For macOS, it uses the "/Users/" directory and appends the "SUDO_USER" or "USER" environment variable if they exist and correspond to a valid directory.
/// For other operating systems, it uses the "/home/" directory and appends the "SUDO_USER" or "USER" environment variable if they exist and correspond to a valid directory.
/// If none of the above methods work, it uses the "HOME" environment variable.
///
/// # Returns
///
/// This function returns a `Result` that contains a `PathBuf` representing the home directory path if the operation was successful.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Example
///
/// ```rust
/// let home_dir = get_home_dir()?;
/// ```
pub fn get_home_dir() -> Result<PathBuf> {
    let mut home_str = PathBuf::new();

    if cfg!(windows) {
        home_str.push(std::env::var("USERPROFILE")?);
        return Ok(home_str);
    }

    if cfg!(target_os = "macos") {
        home_str.push("/Users/");
    } else {
        home_str.push("/home/")
    };

    if let Ok(value) = std::env::var("SUDO_USER") {
        home_str.push(&value);
        if fs::metadata(&home_str).is_ok() {
            return Ok(home_str);
        }
    }

    if let Ok(value) = std::env::var("USER") {
        home_str.push(&value);
        if fs::metadata(&home_str).is_ok() {
            return Ok(home_str);
        }
    }

    let home_value = std::env::var("HOME")?;
    home_str = PathBuf::from(home_value);

    Ok(home_str)
}

/// Returns the local data directory path for the current user.
///
/// This function first gets the home directory path by calling the `get_home_dir` function.
/// It then checks the target operating system using the `cfg!` macro and constructs the local data directory path accordingly.
/// For Windows, it appends "AppData/Local" to the home directory path.
/// For other operating systems, it appends ".local/share" to the home directory path.
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
pub fn get_local_data_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_dir()?;
    if cfg!(windows) {
        home_dir.push("AppData/Local");
        return Ok(home_dir);
    }

    home_dir.push(".local/share");
    Ok(home_dir)
}

/// Returns the local data directory path for the current user.
///
/// This function first gets the home directory path by calling the `get_home_dir` function.
/// It then checks the target operating system using the `cfg!` macro and constructs the local data directory path accordingly.
/// For Windows, it appends "AppData/Local" to the home directory path.
/// For other operating systems, it appends ".local/share" to the home directory path.
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
pub fn get_config_file() -> Result<PathBuf> {
    if let Ok(value) = std::env::var("BOB_CONFIG") {
        return Ok(PathBuf::from(value));
    }

    let mut home_dir = get_home_dir()?;

    if cfg!(windows) {
        home_dir.push("AppData/Roaming");
    } else if cfg!(target_os = "macos") {
        home_dir.push("Library/Application Support");
    } else {
        home_dir.push(".config");
    }

    home_dir.push("bob/config.toml");

    if fs::metadata(&home_dir).is_err() {
        home_dir.pop();
        home_dir.push("config.json");
    }

    Ok(home_dir)
}

/// Asynchronously returns the downloads directory path based on the application configuration.
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
    match &config.installation_location {
        Some(path) => Ok(PathBuf::from(path.clone())),
        None => {
            let mut installation_location = get_downloads_directory(config).await?;
            installation_location.push("nvim-bin");

            Ok(installation_location)
        }
    }
}
