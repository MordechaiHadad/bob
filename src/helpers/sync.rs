use std::process::Command;
use anyhow::{anyhow, Result};

pub fn handle_subprocess(process: &mut Command) -> Result<()> {
    match process.status()?.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!(code)),
        None => Err(anyhow!("process terminated by signal")),
    }
}