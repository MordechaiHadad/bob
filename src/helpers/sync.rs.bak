#[cfg(target_os = "linux")]
use anyhow::{anyhow, Result};
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};

/// Handles the execution of a subprocess.
///
/// This function takes a mutable reference to a `Command` struct, which represents the subprocess to be executed.
/// It sets the standard output of the subprocess to `null` and then executes the subprocess.
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
/// handle_subprocess(&mut process);
/// ```
#[cfg(target_os = "linux")]
pub fn handle_subprocess(process: &mut Command) -> Result<()> {
    match process.stdout(Stdio::null()).status()?.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!(code)),
        None => Err(anyhow!("process terminated by signal")),
    }
}
