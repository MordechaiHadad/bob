use crate::config::Config;
use anyhow::{anyhow, Result};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use tokio::{process::Command, time::sleep};

use super::{
    directories, get_platform_name,
    version::{self, is_hash},
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
/// It then constructs the path to the Neovim binary and spawns a new process with the given arguments.
/// The function then enters a loop where it continuously checks the status of the spawned process.
/// If the process exits with a status code of `0`, the function returns `Ok(())`.
/// If the process exits with a non-zero status code, the function returns an error with the status code as the error message.
/// If the process is terminated by a signal, the function returns an error with the message "Process terminated by signal".
/// If the function fails to wait on the child process, it returns an error with the message "Failed to wait on child process".
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
    let platform = get_platform_name(&version);

    let new_version: String = if is_hash(&used_version) {
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

    let _term = Arc::new(AtomicBool::new(false));
    #[cfg(unix)]
    {
        signal_hook::flag::register(signal_hook::consts::SIGUSR1, Arc::clone(&_term))?;
    }

    let mut child = std::process::Command::new(location);
    child.args(args);

    let mut spawned_child = child.spawn()?;

    loop {
        let child_done = spawned_child.try_wait();
        match child_done {
            Ok(Some(status)) => match status.code() {
                Some(0) => return Ok(()),
                Some(code) => return Err(anyhow!("Process exited with error code {}", code)),
                None => return Err(anyhow!("Process terminated by signal")),
            },
            Ok(None) => {
                #[cfg(unix)]
                {
                    use nix::sys::signal::{self, Signal};
                    use nix::unistd::Pid;
                    use std::sync::atomic::Ordering;
                    if _term.load(Ordering::Relaxed) {
                        let pid = spawned_child.id() as i32;
                        signal::kill(Pid::from_raw(pid), Signal::SIGUSR1)?;
                        _term.store(false, Ordering::Relaxed);
                    }
                }
                // short delay to a void high cpu usage
                sleep(Duration::from_millis(200)).await;
            }
            Err(_) => return Err(anyhow!("Failed to wait on child process")),
        }
    }
}
