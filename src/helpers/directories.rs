use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;

/// Returns the home directory path for the current user.
///
/// This function checks the target operating system using the `cfg!` macro and constructs the home directory path accordingly.
/// For Windows, it uses the "USERPROFILE" environment variable.
/// For macOS, it uses the "/Users/" directory and appends the `SUDO_USER` or "USER" environment variable if they exist and correspond to a valid directory.
/// For other operating systems, it uses the "/home/" directory and appends the `SUDO_USER` or "USER" environment variable if they exist and correspond to a valid directory.
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
fn get_home_dir() -> Result<PathBuf> {
    let mut home_str = PathBuf::new();

    if cfg!(windows) {
        home_str.push(std::env::var("USERPROFILE")?);
        return Ok(home_str);
    }

    if cfg!(target_os = "macos") {
        home_str.push("/Users/");
    } else {
        home_str.push("/home/");
    }

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
fn get_local_data_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_dir()?;
    if cfg!(windows) {
        home_dir.push("AppData\\Local");
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
        home_dir.push("AppData\\Roaming");
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

/// Recursively finds all version directories in the downloads directory.
///
/// This function searches through the directory tree starting from `current_dir`,
/// looking for directories that contain a valid Neovim installation (indicated by
/// the presence of a `bin` subdirectory). Build directories (neovim-git/build) are
/// automatically filtered out.
///
/// # Arguments
///
/// * `current_dir` - The current directory being searched
/// * `base_dir` - The base downloads directory used to compute relative paths for filtering
///
/// # Returns
///
/// * `Result<Vec<PathBuf>>` - A vector of paths to version directories, excluding build directories
///
/// # Example
///
/// ```rust
/// let downloads_dir = PathBuf::from("/path/to/downloads");
/// let version_dirs = find_version_dirs(&downloads_dir, &downloads_dir)?;
/// ```
pub fn find_version_dirs(current_dir: &PathBuf, base_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut version_dirs = Vec::new();

    if let Ok(entries) = fs::read_dir(current_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let nvim_exe = path.join("bin").join("nvim");
            if nvim_exe.exists() && nvim_exe.is_file() {
                // Filter out build directories
                if let Ok(relative_path) = path.strip_prefix(base_dir) {
                    if !relative_path.starts_with("neovim-git/build") {
                        version_dirs.push(path);
                    }
                }
            } else if let Ok(mut subdirs) = find_version_dirs(&path, base_dir) {
                version_dirs.append(&mut subdirs);
            }
        }
    }

    Ok(version_dirs)
}

/// Removes empty parent directories up to (but not including) the base directory.
///
/// This function is useful for cleaning up nested directory structures after
/// removing a version. For example, after removing `user/repo@branch`, it will
/// remove empty `user/repo` and `user` directories if they're empty.
///
/// # Arguments
///
/// * `path` - The path that was just removed
/// * `base_dir` - The base directory to stop at (e.g., downloads directory)
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if successful or if no cleanup was needed
///
/// # Example
///
/// ```rust
/// let removed_path = PathBuf::from("/downloads/user/repo@branch");
/// let base = PathBuf::from("/downloads");
/// remove_empty_parents(&removed_path, &base)?;
/// // This will remove /downloads/user/repo and /downloads/user if they're empty
/// ```
pub fn remove_empty_parents(path: &Path, base_dir: &PathBuf) -> Result<()> {
    let mut current = path.parent();

    while let Some(parent) = current {
        // Stop if we've reached the base directory
        if parent == base_dir {
            break;
        }

        // Check if directory is empty
        if let Ok(mut entries) = fs::read_dir(parent) {
            if entries.next().is_none() {
                // Directory is empty, remove it
                fs::remove_dir(parent)?;
                current = parent.parent();
            } else {
                // Directory is not empty, stop cleanup
                break;
            }
        } else {
            // Can't read directory, stop cleanup
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    fn create_test_nvim_install(base: &PathBuf, relative_path: &str) -> PathBuf {
        let install_path = base.join(relative_path);
        let bin_path = install_path.join("bin");

        fs::create_dir_all(&bin_path).unwrap();
        File::create(bin_path.join("nvim")).unwrap();

        install_path
    }

    #[test]
    fn test_find_version_dirs_single_version() {
        let temp_dir = std::env::temp_dir().join("bob_test_single");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a single version
        create_test_nvim_install(&temp_dir, "v0.9.0");

        let results = find_version_dirs(&temp_dir, &temp_dir).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("v0.9.0"));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_find_version_dirs_multiple_versions() {
        let temp_dir = std::env::temp_dir().join("bob_test_multiple");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create multiple versions
        create_test_nvim_install(&temp_dir, "v0.9.0");
        create_test_nvim_install(&temp_dir, "v0.9.1");
        create_test_nvim_install(&temp_dir, "stable");

        let results = find_version_dirs(&temp_dir, &temp_dir).unwrap();

        assert_eq!(results.len(), 3);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_find_version_dirs_nested_fork_versions() {
        let temp_dir = std::env::temp_dir().join("bob_test_forks");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create nested fork versions
        create_test_nvim_install(&temp_dir, "user/repo@main");
        create_test_nvim_install(&temp_dir, "user/repo@dev");
        create_test_nvim_install(&temp_dir, "other/fork@branch");

        let results = find_version_dirs(&temp_dir, &temp_dir).unwrap();

        assert_eq!(results.len(), 3);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_find_version_dirs_filters_build_directories() {
        let temp_dir = std::env::temp_dir().join("bob_test_build_filter");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create regular version
        create_test_nvim_install(&temp_dir, "v0.9.0");

        // Create build directory (should be filtered)
        create_test_nvim_install(&temp_dir, "neovim-git/build");

        let results = find_version_dirs(&temp_dir, &temp_dir).unwrap();

        // Should only find v0.9.0, not the build directory
        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("v0.9.0"));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_find_version_dirs_ignores_non_nvim_directories() {
        let temp_dir = std::env::temp_dir().join("bob_test_non_nvim");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a valid nvim install
        create_test_nvim_install(&temp_dir, "v0.9.0");

        // Create directories without nvim binary
        fs::create_dir_all(temp_dir.join("not_a_version")).unwrap();
        fs::create_dir_all(temp_dir.join("another/nested/dir")).unwrap();

        let results = find_version_dirs(&temp_dir, &temp_dir).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("v0.9.0"));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_remove_empty_parents_single_level() {
        let temp_dir = std::env::temp_dir().join("bob_test_cleanup_single");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let nested = temp_dir.join("user");
        fs::create_dir_all(&nested).unwrap();

        // Remove the nested directory and cleanup
        fs::remove_dir(&nested).unwrap();
        remove_empty_parents(&nested, &temp_dir).unwrap();

        // Parent should still exist (it's the base dir)
        assert!(temp_dir.exists());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_remove_empty_parents_multiple_levels() {
        let temp_dir = std::env::temp_dir().join("bob_test_cleanup_multi");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let nested = temp_dir.join("user").join("repo@branch");
        fs::create_dir_all(&nested).unwrap();

        // Remove the deepest directory and cleanup
        fs::remove_dir(&nested).unwrap();
        remove_empty_parents(&nested, &temp_dir).unwrap();

        // All intermediate directories should be removed
        assert!(!temp_dir.join("user").join("repo@branch").exists());
        assert!(!temp_dir.join("user").exists());
        assert!(temp_dir.exists());

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_remove_empty_parents_stops_at_non_empty() {
        let temp_dir = std::env::temp_dir().join("bob_test_cleanup_stop");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let user_dir = temp_dir.join("user");
        let repo1 = user_dir.join("repo1@main");
        let repo2 = user_dir.join("repo2@main");

        fs::create_dir_all(&repo1).unwrap();
        fs::create_dir_all(&repo2).unwrap();

        // Remove repo1 and cleanup
        fs::remove_dir(&repo1).unwrap();
        remove_empty_parents(&repo1, &temp_dir).unwrap();

        // user_dir should still exist because repo2 is there
        assert!(user_dir.exists());
        assert!(repo2.exists());
        assert!(!repo1.exists());

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
