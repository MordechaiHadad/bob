use crate::config::Config;
use crate::helpers::directories;
use anyhow::Result;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Finds the system nvim binary in PATH that is not managed by bob.
///
/// This function searches through all directories in the PATH environment variable
/// to find an nvim executable that is neither bob's shim nor managed by bob.
///
/// # Arguments
///
/// * `config` - A reference to the Config struct to get bob's directories.
///
/// # Returns
///
/// * `Result<Option<PathBuf>>` - Returns `Ok(Some(PathBuf))` if a system nvim is found,
///   `Ok(None)` if no system nvim is found, or an error if the operation failed.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let system_nvim = find_system_nvim(&config).await?;
/// ```
pub async fn find_system_nvim(config: &Config) -> Result<Option<PathBuf>> {
    let path_env = std::env::var("PATH").unwrap_or_default();
    let installation_dir = directories::get_installation_directory(config).await?;
    let downloads_dir = directories::get_downloads_directory(config).await?;

    let nvim_name = if cfg!(windows) { "nvim.exe" } else { "nvim" };

    // Split PATH and search for nvim
    for path_dir in std::env::split_paths(&path_env) {
        // Skip if it's in bob's installation directory (the shim)
        if path_dir.starts_with(&installation_dir) {
            continue;
        }

        // Skip if it's in bob's downloads directory (managed versions)
        if path_dir.starts_with(&downloads_dir) {
            continue;
        }

        let nvim_path = path_dir.join(nvim_name);
        if !nvim_path.exists() {
            continue;
        }

        // On Unix, also check if the file is executable
        #[cfg(unix)]
        {
            if let Ok(metadata) = std::fs::metadata(&nvim_path) {
                let permissions = metadata.permissions();
                // Check if the file has any execute permission (user, group, or other)
                if permissions.mode() & 0o111 == 0 {
                    continue;
                }
            } else {
                continue;
            }
        }

        return Ok(Some(nvim_path));
    }

    Ok(None)
}
