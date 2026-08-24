use eyre::{Result, bail, eyre};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

#[cfg(unix)]
fn get_sudo_user_home() -> Option<PathBuf> {
    let uid: libc::uid_t = std::env::var("SUDO_UID").ok()?.parse().ok()?;

    let buf_size = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    let buf_size = if buf_size <= 0 {
        4096
    } else {
        buf_size as usize
    };
    let mut buf = vec![0u8; buf_size];
    let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
    let mut result: *mut libc::passwd = std::ptr::null_mut();

    let ret = unsafe {
        libc::getpwuid_r(
            uid,
            pwd.as_mut_ptr(),
            buf.as_mut_ptr() as *mut libc::c_char,
            buf_size,
            &mut result,
        )
    };

    if ret == 0 && !result.is_null() {
        let pwd = unsafe { pwd.assume_init() };
        if !pwd.pw_dir.is_null() {
            let home = unsafe { std::ffi::CStr::from_ptr(pwd.pw_dir) };
            return Some(PathBuf::from(home.to_str().ok()?));
        }
    }
    None
}

/// Returns the user's home directory path.
///
/// On Unix systems running under sudo, this resolves the real invoking user's home
/// directory instead of root's. Otherwise it delegates to the `dirs` crate.
///
/// # Returns
///
/// The home directory as a `PathBuf`, or `None` if it cannot be determined.
#[cfg(target_os = "linux")]
pub fn get_user_home() -> Option<PathBuf> {
    get_sudo_user_home().or_else(dirs::home_dir)
}

fn get_sudo_data_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    if std::env::var("SUDO_USER").is_ok() {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return Some(PathBuf::from(xdg));
        }
        if let Some(home) = get_sudo_user_home() {
            return Some(if cfg!(target_os = "macos") {
                home.join("Library/Application Support")
            } else {
                home.join(".local/share")
            });
        }
    }
    None
}

fn get_sudo_config_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    if std::env::var("SUDO_USER").is_ok() {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg));
        }
        if let Some(home) = get_sudo_user_home() {
            return Some(if cfg!(target_os = "macos") {
                home.join("Library/Application Support")
            } else {
                home.join(".config")
            });
        }
    }
    None
}

/// Returns the local data directory path for the current user.
///
/// On Unix systems, if running under sudo, it checks `XDG_DATA_HOME` first, then falls back
/// to the real user's home directory with `.local/share`. Otherwise, relies on the `dirs` crate.
fn get_local_data_dir() -> Result<PathBuf> {
    get_sudo_data_dir()
        .or_else(dirs::data_local_dir)
        .ok_or_else(|| eyre!("Could not determine local data directory"))
}

/// Returns the configuration file path for the current user.
///
/// This function prioritizes the `BOB_CONFIG` environment variable.
/// On Unix systems, if running under sudo, it appends ".config" (or "Library/Application Support" on macOS) to the real user's home directory.
/// Otherwise, it relies on the `dirs` crate which respects `XDG_CONFIG_HOME` on Linux.
///
/// When neither `config.toml` nor `config.json` exists yet, the path of
/// `config.toml` is returned so that fresh setups get a TOML config created
/// for them on first run. Existing JSON setups keep resolving to `config.json`.
///
/// # Returns
///
/// This function returns a `Result` that contains a `PathBuf` representing the config file path if the operation was successful.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Example
///
/// ```ignore
/// let config_file = get_config_file().unwrap();
/// ```
pub fn get_config_file() -> Result<PathBuf> {
    if let Ok(value) = std::env::var("BOB_CONFIG") {
        return Ok(PathBuf::from(value));
    }

    let config_dir = get_sudo_config_dir()
        .or_else(dirs::config_dir)
        .ok_or_else(|| eyre!("Could not determine config directory"))?;

    let mut config_dir = config_dir;

    config_dir.push("bob/config.toml");

    if fs::metadata(&config_dir).is_ok() {
        return Ok(config_dir);
    }

    let mut json_config_dir = config_dir.clone();
    json_config_dir.pop();
    json_config_dir.push("config.json");

    if fs::metadata(&json_config_dir).is_ok() {
        return Ok(json_config_dir);
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
            bail!("Custom directory {path} doesn't exist!");
        }

        PathBuf::from(path)
    } else {
        let mut data_dir = get_local_data_dir()?;

        data_dir.push("bob");
        let does_folder_exist = tokio::fs::metadata(&data_dir).await.is_ok();
        let is_folder_created = tokio::fs::create_dir_all(&data_dir).await.is_ok();

        if !does_folder_exist && !is_folder_created {
            bail!("Couldn't create downloads directory");
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
