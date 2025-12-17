use crate::config::Config;
use anyhow::{Result, anyhow};
use std::path::PathBuf;
use std::time::Duration;
use sysinfo::System;
use tokio::{process::Command, time::sleep};

use crate::helpers::{
    directories, get_platform_name, system,
    version::{self},
};

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

    // Handle system version
    if used_version == "system" {
        let system_nvim = system::find_system_nvim(config)
            .await?
            .ok_or_else(|| anyhow!("System nvim not found"))?;

        return execute_nvim_command(system_nvim, args).await;
    }

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

    execute_nvim_command(location, args).await
}

/// Executes a Neovim command with the given arguments.
///
/// This function spawns a Neovim process and waits for it to complete.
/// On Unix systems, it uses `exec` to replace the current process with Neovim.
/// On Windows, it spawns a new process and monitors its execution.
///
/// # Arguments
///
/// * `nvim_path` - The path to the Neovim binary to execute.
/// * `args` - A slice of `String` arguments to be passed to the Neovim process.
///
/// # Returns
///
/// This function returns a `Result` that indicates whether the operation was successful.
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
/// let nvim_path = PathBuf::from("/usr/bin/nvim");
/// let args = vec!["-v".to_string()];
/// execute_nvim_command(nvim_path, &args).await?;
/// ```
async fn execute_nvim_command(nvim_path: PathBuf, args: &[String]) -> Result<()> {
    let mut child = std::process::Command::new(nvim_path);
    child.args(args);

    // On Unix, replace the current process with nvim
    if cfg!(unix) {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let err = child.exec();
            return Err(anyhow!("Failed to exec neovim: {err}"));
        }
    }

    let mut spawned_child = child.spawn()?;

    loop {
        let child_done = spawned_child.try_wait();
        match child_done {
            Ok(Some(status)) => match status.code() {
                Some(0) => return Ok(()),
                Some(code) => return Err(anyhow!("Process exited with error code {code}")),
                None => return Err(anyhow!("Process terminated by signal")),
            },
            Ok(None) => {
                // short delay to avoid high cpu usage
                sleep(Duration::from_millis(200)).await;
            }
            Err(_) => return Err(anyhow!("Failed to wait on child process")),
        }
    }
}

pub fn is_neovim_running() -> bool {
    let sys = System::new_all();

    for process in sys.processes().values() {
        let name = process.name().to_string_lossy().to_lowercase();
        if name.contains("nvim") {
            return true;
        }
    }
    false
}
