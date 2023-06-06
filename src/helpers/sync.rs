use anyhow::{anyhow, Result};
use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
pub fn handle_subprocess(process: &mut Command) -> Result<()> {
    match process.stdout(Stdio::null()).status()?.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!(code)),
        None => Err(anyhow!("process terminated by signal")),
    }
}
