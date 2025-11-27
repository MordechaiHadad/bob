use anyhow::{Result, anyhow};
use sysinfo::System;
use tokio::process::Command;

use crate::config::Config;
use crate::helpers::{directories, get_platform_name, version};

/// Handles the execution of a subprocess.
///
/// This function takes a mutable reference to a `Command` struct, which represents the subprocess to be executed.
/// It then awaits the status of the subprocess.
/// If the subprocess exits with a status code of `0`, the function returns `Ok(())`.
/// If the subprocess exits with a non-zero status code, the function returns an error with the status code as the error message.
/// If the subprocess is terminated by a signal, the function returns an error with the message "process terminated by signal".
///
/// # Arguments
///
/// * `process` - A mutable reference to a `Command` struct representing the subprocess to be executed.
///
/// # Returns
///
/// This function returns a `Result` that indicates whether the operation was successful.
/// If the operation was successful, the function returns `Ok(())`.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The subprocess exits with a non-zero status code.
/// * The subprocess is terminated by a signal.
///
/// # Example
///
/// ```rust
/// let mut process = Command::new("ls");
/// handle_subprocess(&mut process).await;
/// ```
pub async fn handle_subprocess(process: &mut Command) -> Result<()> {
    match process.status().await?.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!(code)),
        None => Err(anyhow!("process terminated by signal")),
    }
}

/// Handles the execution of the Neovim process.
///
/// This function takes a reference to a `Config` struct and a slice of `String` arguments.
/// It retrieves the downloads directory and the currently used version of Neovim from the configuration.
/// It then constructs the path to the Neovim binary and executes it with the given arguments.
///
/// On Unix systems, this function uses `exec` to replace the current process with Neovim.
/// On Windows, it spawns a new process and monitors its execution.
///
/// If running on Windows and the process exits with a non-zero status code, returns an error with the status code.
/// If the process is terminated by a signal on Windows, returns an error with "Process terminated by signal".
///
/// # Arguments
///
/// * `config` - A reference to a `Config` struct containing the configuration for the Neovim process.
/// * `args` - A slice of `String` arguments to be passed to the Neovim process.
///
/// # Returns
///
/// This function returns a `Result` that indicates whether the operation was successful.
/// If the operation was successful, the function returns `Ok(())`.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The Neovim process exits with a non-zero status code.
/// * The Neovim process is terminated by a signal.
/// * The function fails to wait on the child process.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let args = vec!["-v".to_string()];
/// handle_nvim_process(&config, &args).await;
/// ```
pub async fn handle_nvim_process(config: &Config, args: &[String]) -> Result<()> {
    let downloads_dir = directories::get_downloads_directory(config).await?;
    let used_version = version::get_current_version(config).await?;
    let version = semver::Version::parse(&used_version.replace('v', "")).ok();
    let platform = get_platform_name(version.as_ref());

    let new_version: String = if crate::HASH_REGEX.is_match(&used_version) {
        used_version.chars().take(7).collect()
    } else {
        used_version
    };

    let mut location = downloads_dir.join(&new_version).join("bin").join("nvim");

    if cfg!(windows) {
        location = location.with_extension("exe");
    }

    if !location.exists() {
        location = downloads_dir
            .join(new_version)
            .join(platform)
            .join("bin")
            .join("nvim");

        if cfg!(windows) {
            location = location.with_extension("exe");
        }
    }

    // let mut child = std::process::Command::new(location);
    // child.args(args);

    // On Unix, replace the current process with nvim
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = tokio::process::Command::new(&location)
            .args(args)
            .as_std_mut()
            .exec();
        #[allow(clippy::needless_return)]
        return Err(anyhow!("Failed to exec neovim: {err}"));
    }

    #[cfg(windows)]
    {
        let status = tokio::process::Command::new(&location)
            .args(args)
            .spawn()?
            .wait()
            .await?;

        if let Some(code) = status.code() {
            std::process::exit(code);
        } else {
            std::process::exit(1);
        }
    }
}

pub fn is_neovim_running() -> bool {
    System::new_all()
        .processes()
        .values()
        .any(|process| process.name().to_string_lossy().to_lowercase().contains("nvim"))
}

#[cfg(test)]
mod processes_tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_subprocess_success() {
        let mut cmd = Command::new("true");
        let result = handle_subprocess(&mut cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_subprocess_failure() {
        let mut cmd = Command::new("false");
        let result = handle_subprocess(&mut cmd).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_neovim_running() {
        // attempt to find nvim on the system PATH,
        // if it's not available, we can skip this test

        if !t_ensure_nvim_available() {
            eprintln!("Neovim not found in PATH, skipping test");
            return;
        }

        let cmd = tokio::process::Command::new("nvim")
            .args(["--headless", "-c", "sleep 1"])
            .spawn()
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let checking_handle = tokio::spawn(async {
            for _pid in 0..10 {
                if is_neovim_running() {
                    return true;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                continue;
            }
            false
        });

        let result = checking_handle.await.unwrap();

        tokio::spawn(async move {
            t_cleanup_nvim(cmd.id().unwrap()).await;
        });

        if !result {
            panic!("Neovim process was not detected as running");
        } else {
            assert!(result);
        }
    }

    /// Test function to ensure Neovim is available in the system PATH.
    /// We call this to ensure the testing environment has Neovim installed.
    fn t_ensure_nvim_available() -> bool {
        let system_paths = std::env::var_os("PATH").unwrap_or_default();
        std::env::split_paths(&system_paths)
            .any(|path| path.join("nvim").exists() || path.join("nvim.exe").exists())
    }

    /// Test function to clean up the Neovim process after testing.
    /// Checks for the process by its PID and kills it if found.
    async fn t_cleanup_nvim(pid: u32) {
        let system = System::new_all();
        if let Some(process) = system
            .processes()
            .values()
            .find(|process| process.pid().as_u32() == pid)
        {
            let _ = process.kill();
        }
    }
}
