//! System-level utilities for finding Neovim installations.
//!
//! This module provides functionality to locate system-installed Neovim binaries
//! that are not managed by bob. It searches through the PATH environment variable
//! while filtering out bob's own installation and download directories.

use crate::config::Config;
use crate::helpers::directories;
use anyhow::Result;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const PATH_ENV: &str = "PATH";

/// Finds the system nvim binary in PATH that is not managed by bob.
///
/// This is a convenience wrapper that fetches bob's directories and calls the implementation.
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
    let path_env = std::env::var(PATH_ENV).unwrap_or_default();
    let installation_dir = directories::get_installation_directory(config).await?;
    let downloads_dir = directories::get_downloads_directory(config).await?;

    Ok(find_system_nvim_impl(
        &path_env,
        &installation_dir,
        &downloads_dir,
    ))
}

/// Implementation of system nvim finder that does the actual work.
///
/// This function searches through all directories in the PATH environment variable
/// to find an nvim executable that is neither bob's shim nor managed by bob.
///
/// # Arguments
///
/// * `path_env` - The PATH environment variable value to search through.
/// * `installation_dir` - Bob's installation directory (where the shim is located).
/// * `downloads_dir` - Bob's downloads directory (where managed versions are stored).
///
/// # Returns
///
/// * `Option<PathBuf>` - Returns `Some(PathBuf)` if a system nvim is found,
///   `None` if no system nvim is found.
fn find_system_nvim_impl(
    path_env: &str,
    installation_dir: &PathBuf,
    downloads_dir: &PathBuf,
) -> Option<PathBuf> {
    let nvim_name = if cfg!(windows) { "nvim.exe" } else { "nvim" };

    // Split PATH and search for nvim
    for path_dir in std::env::split_paths(path_env) {
        // Skip path in bob's installation directory
        if path_dir.starts_with(installation_dir) {
            continue;
        }

        // Skip path in bob's downloads directory
        if path_dir.starts_with(downloads_dir) {
            continue;
        }

        let nvim_path = path_dir.join(nvim_name);

        // Skip path if nvim binary doesn't exist
        if !nvim_path.exists() {
            continue;
        }

        // On Unix, also check if the nvim binary is executable
        #[cfg(unix)]
        {
            if let Ok(metadata) = std::fs::metadata(&nvim_path) {
                let permissions = metadata.permissions();
                // Check if the nvim binary has any execute permission (user, group, or other)
                if permissions.mode() & 0o111 == 0 {
                    continue;
                }
            } else {
                continue;
            }
        }

        return Some(nvim_path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    /// Helper function to create a mock nvim executable in a directory
    fn create_mock_nvim(dir: &std::path::Path) -> PathBuf {
        let nvim_name = if cfg!(windows) { "nvim.exe" } else { "nvim" };
        let nvim_path = dir.join(nvim_name);

        #[cfg(unix)]
        {
            // Create a shell script that acts as nvim
            fs::write(&nvim_path, "#!/bin/sh\necho 'mock nvim'\n").unwrap();
            // Make it executable
            let metadata = fs::metadata(&nvim_path).unwrap();
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&nvim_path, permissions).unwrap();
        }

        #[cfg(windows)]
        {
            // Create a batch file for Windows
            fs::write(&nvim_path, "@echo off\r\necho mock nvim\r\n").unwrap();
        }

        nvim_path
    }

    /// Helper function to create a non-executable file
    #[cfg(unix)]
    fn create_non_executable_nvim(dir: &std::path::Path) -> PathBuf {
        let nvim_path = dir.join("nvim");
        fs::write(&nvim_path, "#!/bin/sh\necho 'mock nvim'\n").unwrap();
        // Make it non-executable
        let metadata = fs::metadata(&nvim_path).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&nvim_path, permissions).unwrap();
        nvim_path
    }

    #[test]
    fn test_empty_path() {
        let installation_dir = PathBuf::from("/fake/installation");
        let downloads_dir = PathBuf::from("/fake/downloads");

        let result = find_system_nvim_impl("", &installation_dir, &downloads_dir);

        assert!(result.is_none());
    }

    #[test]
    fn test_finds_nvim_in_path() {
        let _dir = TempDir::new().unwrap();
        let nvim_path = create_mock_nvim(_dir.path());

        let installation_dir = PathBuf::from("/fake/installation");
        let downloads_dir = PathBuf::from("/fake/downloads");
        let path_env = _dir.path().to_string_lossy().to_string();

        let result = find_system_nvim_impl(&path_env, &installation_dir, &downloads_dir);

        assert_eq!(result, Some(nvim_path));
    }

    #[test]
    fn test_filters_out_installation_dir() {
        let _installation_dir = TempDir::new().unwrap();
        let _other_dir = TempDir::new().unwrap();

        create_mock_nvim(_installation_dir.path());
        let other_nvim_path = create_mock_nvim(_other_dir.path());

        let downloads_dir = PathBuf::from("/fake/downloads");
        let path_env = format!(
            "{}{}{}",
            _installation_dir.path().to_string_lossy(),
            if cfg!(windows) { ";" } else { ":" },
            _other_dir.path().to_string_lossy()
        );

        let result = find_system_nvim_impl(
            &path_env,
            &_installation_dir.path().to_path_buf(),
            &downloads_dir,
        );

        assert_eq!(result, Some(other_nvim_path));
    }

    #[test]
    fn test_filters_out_downloads_dir() {
        let _downloads_dir = TempDir::new().unwrap();
        let _other_dir = TempDir::new().unwrap();

        create_mock_nvim(_downloads_dir.path());
        let other_nvim_path = create_mock_nvim(_other_dir.path());

        let installation_dir = PathBuf::from("/fake/installation");
        let path_env = format!(
            "{}{}{}",
            _downloads_dir.path().to_string_lossy(),
            if cfg!(windows) { ";" } else { ":" },
            _other_dir.path().to_string_lossy()
        );

        let result = find_system_nvim_impl(
            &path_env,
            &installation_dir,
            &_downloads_dir.path().to_path_buf(),
        );

        assert_eq!(result, Some(other_nvim_path));
    }

    #[test]
    fn test_returns_first_valid_nvim() {
        let _dir1 = TempDir::new().unwrap();
        let _dir2 = TempDir::new().unwrap();
        let _dir3 = TempDir::new().unwrap();

        // Only create nvim in dir2 and dir3
        let nvim_path2 = create_mock_nvim(_dir2.path());
        create_mock_nvim(_dir3.path());

        let installation_dir = PathBuf::from("/fake/installation");
        let downloads_dir = PathBuf::from("/fake/downloads");
        let path_env = format!(
            "{}{}{}{}{}",
            _dir1.path().to_string_lossy(),
            if cfg!(windows) { ";" } else { ":" },
            _dir2.path().to_string_lossy(),
            if cfg!(windows) { ";" } else { ":" },
            _dir3.path().to_string_lossy()
        );

        let result = find_system_nvim_impl(&path_env, &installation_dir, &downloads_dir);

        assert_eq!(result, Some(nvim_path2));
    }

    #[test]
    #[cfg(unix)]
    fn test_skips_non_executable_file() {
        let _dir1 = TempDir::new().unwrap();
        let _dir2 = TempDir::new().unwrap();

        create_non_executable_nvim(_dir1.path());
        let nvim_path2 = create_mock_nvim(_dir2.path());

        let installation_dir = PathBuf::from("/fake/installation");
        let downloads_dir = PathBuf::from("/fake/downloads");
        let path_env = format!(
            "{}:{}",
            _dir1.path().to_string_lossy(),
            _dir2.path().to_string_lossy()
        );

        let result = find_system_nvim_impl(&path_env, &installation_dir, &downloads_dir);

        assert_eq!(result, Some(nvim_path2));
    }

    #[test]
    fn test_filters_out_both_installation_and_downloads_dirs() {
        let _installation_dir = TempDir::new().unwrap();
        let _downloads_dir = TempDir::new().unwrap();
        let _other_dir = TempDir::new().unwrap();

        create_mock_nvim(_installation_dir.path());
        create_mock_nvim(_downloads_dir.path());
        let other_nvim_path = create_mock_nvim(_other_dir.path());

        let path_env = format!(
            "{}{}{}{}{}",
            _installation_dir.path().to_string_lossy(),
            if cfg!(windows) { ";" } else { ":" },
            _downloads_dir.path().to_string_lossy(),
            if cfg!(windows) { ";" } else { ":" },
            _other_dir.path().to_string_lossy()
        );

        let result = find_system_nvim_impl(
            &path_env,
            &_installation_dir.path().to_path_buf(),
            &_downloads_dir.path().to_path_buf(),
        );

        assert_eq!(result, Some(other_nvim_path));
    }

    #[test]
    fn test_returns_none_when_only_filtered_dirs_in_path() {
        let _installation_dir = TempDir::new().unwrap();
        let _downloads_dir = TempDir::new().unwrap();

        create_mock_nvim(_installation_dir.path());
        create_mock_nvim(_downloads_dir.path());

        let path_env = format!(
            "{}{}{}",
            _installation_dir.path().to_string_lossy(),
            if cfg!(windows) { ";" } else { ":" },
            _downloads_dir.path().to_string_lossy()
        );

        let result = find_system_nvim_impl(
            &path_env,
            &_installation_dir.path().to_path_buf(),
            &_downloads_dir.path().to_path_buf(),
        );

        assert_eq!(result, None);
    }
}
