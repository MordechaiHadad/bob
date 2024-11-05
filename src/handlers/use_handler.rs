use anyhow::{anyhow, Result};
use dialoguer::Confirm;
use reqwest::Client;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs::{self};
use tracing::info;

use crate::config::{Config, ConfigFile};
use crate::handlers::{install_handler, InstallResult};
use crate::helpers;
use crate::helpers::directories::get_installation_directory;
use crate::helpers::version::types::{ParsedVersion, VersionType};

/// Starts the process of using a specified version.
///
/// This function checks if the specified version is already used, copies the Neovim proxy to the installation directory, installs the version if it's not already installed and used, switches to the version, and removes the "stable" directory if the version type is "Latest".
///
/// # Arguments
///
/// * `version` - The version to use.
/// * `install` - Whether to install the version if it's not already installed.
/// * `client` - The client to use for HTTP requests.
/// * `config` - The configuration for the operation.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the operation was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The version is not already used and it cannot be installed.
/// * The version cannot be switched to.
/// * The "stable" directory exists and it cannot be removed.
///
/// # Example
///
/// ```rust
/// let version = ParsedVersion::new("1.0.0");
/// let install = true;
/// let client = Client::new();
/// let config = Config::default();
/// start(version, install, &client, config).await.unwrap();
/// ```
pub async fn start(
    mut version: ParsedVersion,
    install: bool,
    client: &Client,
    config: ConfigFile,
) -> Result<()> {
    let is_version_used =
        helpers::version::is_version_used(&version.tag_name, &config.config).await;

    copy_nvim_proxy(&config).await?;
    if is_version_used && version.tag_name != "nightly" {
        info!("{} is already installed and used!", version.tag_name);
        return Ok(());
    }

    if install {
        match install_handler::start(&mut version, client, &config).await {
            Ok(success) => {
                if let InstallResult::NightlyIsUpdated = success {
                    if is_version_used {
                        info!("Nightly is already updated and used!");
                        return Ok(());
                    }
                }
            }
            Err(error) => return Err(error),
        }
    }

    switch(&config.config, &version).await?;

    if let VersionType::Latest = version.version_type {
        if fs::metadata("stable").await.is_ok() {
            fs::remove_dir_all("stable").await?;
        }
    }

    let installation_dir = get_installation_directory(&config.config).await?;

    add_to_path(installation_dir, config).await?;

    info!("You can now use {}!", version.tag_name);

    Ok(())
}

/// Switches to a specified version.
///
/// This function changes the current directory to the downloads directory, writes the version to a file named "used", and if the version is different from the version stored in `version_sync_file_location`, it also writes the version to `version_sync_file_location`.
///
/// # Arguments
///
/// * `config` - The configuration for the operation.
/// * `version` - The version to switch to.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the operation was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The downloads directory cannot be determined.
/// * The current directory cannot be changed to the downloads directory.
/// * The version cannot be written to the "used" file.
/// * The version cannot be read from `version_sync_file_location`.
/// * The version cannot be written to `version_sync_file_location`.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let version = ParsedVersion::new("1.0.0");
/// switch(&config, &version).await.unwrap();
/// ```
pub async fn switch(config: &Config, version: &ParsedVersion) -> Result<()> {
    std::env::set_current_dir(helpers::directories::get_downloads_directory(config).await?)?;

    let file_version: String = if version.version_type == VersionType::Hash {
        if version.non_parsed_string.len() <= 7 {
            let mut current_dir = env::current_dir()?;
            current_dir.push(&version.non_parsed_string);
            current_dir.push("full-hash.txt");
            let hash_result = fs::read_to_string(&current_dir).await;

            if let Ok(hash) = hash_result {
                hash
            } else {
                return Err(anyhow!(
                    "Full hash file doesn't exist, please rebuild this commit"
                ));
            }
        } else {
            version.non_parsed_string.to_string()
        }
    } else {
        version.tag_name.to_string()
    };

    fs::write("used", &file_version).await?;
    if let Some(version_sync_file_location) =
        helpers::version::get_version_sync_file_location(config).await?
    {
        // Write the used version to version_sync_file_location only if it's different
        let stored_version = fs::read_to_string(&version_sync_file_location).await?;
        if stored_version != version.non_parsed_string {
            fs::write(&version_sync_file_location, file_version).await?;
            info!(
                "Written version to {}",
                version_sync_file_location
                    .into_os_string()
                    .into_string()
                    .unwrap()
            );
        }
    }

    Ok(())
}

/// Copies the Neovim proxy to the installation directory.
///
/// This function gets the current executable's path, determines the installation directory, creates it if it doesn't exist, adds it to the system's PATH, and copies the current executable to the installation directory as "nvim" or "nvim.exe" (on Windows).
///
/// If a file named "nvim" or "nvim.exe" already exists in the installation directory, the function checks its version. If the version matches the current version, the function does nothing. Otherwise, it replaces the file with the current executable.
///
/// # Arguments
///
/// * `config` - The configuration for the operation.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the operation was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The current executable's path cannot be determined.
/// * The installation directory cannot be created.
/// * The installation directory cannot be added to the PATH.
/// * The version of the existing file cannot be determined.
/// * The existing file cannot be replaced.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// copy_nvim_proxy(&config).await.unwrap();
/// ```
async fn copy_nvim_proxy(config: &ConfigFile) -> Result<()> {
    let exe_path = env::current_exe().unwrap();
    let mut installation_dir =
        helpers::directories::get_installation_directory(&config.config).await?;

    if fs::metadata(&installation_dir).await.is_err() {
        fs::create_dir_all(&installation_dir).await?;
    }

    if cfg!(windows) {
        installation_dir.push("nvim.exe");
    } else {
        installation_dir.push("nvim");
    }

    if fs::metadata(&installation_dir).await.is_ok() {
        let output = Command::new(&installation_dir)
            .arg("--&bob")
            .output()?
            .stdout;
        let version = String::from_utf8(output)?.trim().to_string();

        if version == env!("CARGO_PKG_VERSION") {
            return Ok(());
        }
    }

    info!("Updating neovim proxy");
    copy_file_with_error_handling(&exe_path, &installation_dir).await?;

    Ok(())
}

/// Asynchronously copies a file from `old_path` to `new_path`, handling specific OS errors.
///
/// This function attempts to copy a file from the specified `old_path` to the specified `new_path`.
/// If the file is being used by another process (OS error 26 or 32), it prints an error message
/// and returns an error indicating that the file is busy. For any other errors, it returns a
/// generic error with additional context.
///
/// # Arguments
///
/// * `old_path` - A reference to the source `Path` of the file to be copied.
/// * `new_path` - A reference to the destination `Path` where the file should be copied.
///
/// # Returns
///
/// This function returns a `Result<()>`. If the file is successfully copied, it returns `Ok(())`.
/// If an error occurs, it returns an `Err` with a detailed error message.
///
/// # Errors
///
/// This function will return an error in the following cases:
/// - If the file is being used by another process (OS error 26 or 32), it returns an error
///   indicating that the file is busy.
/// - For any other errors, it returns a generic error with additional context.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let old_path = Path::new("path/to/source/file");
///     let new_path = Path::new("path/to/destination/file");
///
///     copy_file_with_error_handling(&old_path, &new_path).await?;
///     Ok(())
/// }
/// ```
async fn copy_file_with_error_handling(old_path: &Path, new_path: &Path) -> Result<()> {
    match fs::copy(&old_path, &new_path).await {
        Ok(_) => Ok(()),
        Err(e) => match e.raw_os_error() {
            Some(26) | Some(32) => Err(anyhow::anyhow!(
                "The file {} is busy. Please make sure to close any processes using it.",
                old_path.display()
            )),
            _ => Err(anyhow::anyhow!(e).context("Failed to copy file")),
        },
    }
}

/// Adds the installation directory to the system's PATH.
///
/// This function checks if the installation directory is already in the PATH. If not, it adds the directory to the PATH.
///
/// # Arguments
///
/// * `installation_dir` - The directory to be added to the PATH.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the operation was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The installation directory cannot be converted to a string.
/// * The current user's environment variables cannot be accessed or modified (Windows only).
/// * The PATH environment variable cannot be read (non-Windows only).
///
/// # Example
///
/// ```rust
/// let installation_dir = Path::new("/usr/local/bin");
/// add_to_path(&installation_dir).unwrap();
/// ```
async fn add_to_path(installation_dir: PathBuf, config: ConfigFile) -> Result<()> {
    let installation_dir = installation_dir.to_str().unwrap();

    if what_the_path::shell::exists_in_path("nvim-bin") {
        return Ok(());
    }

    if config.config.add_neovim_binary_to_path == Some(false) {
        info!("Make sure to add {installation_dir} to $PATH");
        return Ok(());
    }

    if config.config.add_neovim_binary_to_path.is_none() {
        let confirmation = Confirm::new()
            .with_prompt("Add bob-managed Neovim binary to your $PATH automatically?")
            .interact()?;
        let mut temp_confg = config.clone();

        temp_confg.config.add_neovim_binary_to_path = Some(confirmation);
        temp_confg.save_to_file().await?;

        if !confirmation {
            return Ok(());
        }

        drop(temp_confg);
    }

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            use winreg::enums::*;
            use winreg::RegKey;

            let current_usr = RegKey::predef(HKEY_CURRENT_USER);
            let env = current_usr.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
            let usr_path: String = env.get_value("Path")?;

            let new_path = if usr_path.ends_with(';') {
                format!("{usr_path}{}", installation_dir)
            } else {
                format!("{usr_path};{}", installation_dir)
            };
            env.set_value("Path", &new_path)?;
        } else {
            use tokio::fs::File;
            use tokio::io::AsyncWriteExt;
            use what_the_path::shell::Shell;

            let shell = Shell::detect_by_shell_var()?;
            let env_paths = copy_env_files_if_not_exist(&config.config, installation_dir).await?;

            match shell {
                Shell::Fish(fish) => {
                    let files = fish.get_rcfiles()?;
                    let fish_file = files[0].join("bob.fish");
                    if fish_file.exists() { return Ok(()) }
                    let mut opened_file = File::create(fish_file).await?;
                    opened_file.write_all(format!("source \"{}\"\n", env_paths[1].to_str().unwrap()).as_bytes()).await?;
                    opened_file.flush().await?;
                },
                shell => {
                    let files = shell.get_rcfiles()?;
                    for file in files {
                        what_the_path::shell::append_to_rcfile(file, &format!(". \"{}\"", env_paths[0].to_str().unwrap().as_bytes()))?;
                    }
                }
            }
        }
    }

    info!("Added {installation_dir} to system PATH. Please start a new terminal session for changes to take effect.");

    Ok(())
}

#[cfg(target_family = "unix")]
async fn copy_env_files_if_not_exist(
    config: &Config,
    installation_dir: &str,
) -> Result<Vec<PathBuf>> {
    use crate::helpers::directories::get_downloads_directory;
    use tokio::io::AsyncWriteExt;

    let fish_env = include_str!("../../env/env.fish").replace("{nvim_bin}", installation_dir);
    let posix_env = include_str!("../../env/env.sh").replace("{nvim_bin}", installation_dir);
    let downloads_dir = get_downloads_directory(config).await?;
    let env_dir = downloads_dir.join("env");

    // Ensure the env directory exists
    fs::create_dir_all(&env_dir).await?;

    // Define the file paths
    let fish_env_path = env_dir.join("env.fish");
    let posix_env_path = env_dir.join("env.sh");

    // Check if the files exist and write the content if they don't
    if !fish_env_path.exists() {
        let mut file = fs::File::create(&fish_env_path).await?;
        file.write_all(fish_env.as_bytes()).await?;
        file.flush().await?;
    }

    if !posix_env_path.exists() {
        let mut file = fs::File::create(&posix_env_path).await?;
        file.write_all(posix_env.as_bytes()).await?;
        file.flush().await?;
    }

    Ok(vec![posix_env_path, fish_env_path])
}
