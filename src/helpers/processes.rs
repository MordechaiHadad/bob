use crate::config::Config;
use anyhow::{anyhow, Result};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::process::Command;

use super::{directories, get_platform_name, version};

pub async fn handle_subprocess(process: &mut Command) -> Result<()> {
    match process.status().await?.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!(code)),
        None => Err(anyhow!("process terminated by signal")),
    }
}

#[derive(Debug, PartialEq)]
pub enum NvimProcessType {
    Nvim,
    NvimQt,
}

pub async fn handle_nvim_process(
    config: &Config,
    args: &[String],
    process_type: NvimProcessType,
) -> Result<()> {
    let term = Arc::new(AtomicBool::new(false));
    #[cfg(unix)]
    {
        signal_hook::flag::register(signal_hook::consts::SIGUSR1, Arc::clone(&term))?;
    }

    let downloads_dir = directories::get_downloads_directory(&config).await?;
    let used_version = version::get_current_version(&config).await?;
    let version = semver::Version::parse(&used_version.replace('v', "")).ok();
    let platform = get_platform_name(&version);

    let location = downloads_dir
        .join(used_version)
        .join(platform)
        .join("bin")
        .join("nvim");

    let mut child = std::process::Command::new(&location);
    child.args(args);

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            if process_type == NvimProcessType::NvimQt {
                use std::os::windows::process::CommandExt;
                child.creation_flags(0x00000008);
            }
        }
    }

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
                    if term.load(Ordering::Relaxed) {
                        let pid = spawned_child.id() as i32;
                        signal::kill(Pid::from_raw(pid), Signal::SIGTERM)?;
                        return Err(anyhow!("Terminated due to SIGUSR1"));
                    }
                }
            }
            Err(_) => return Err(anyhow!("Failed to wait on child process")),
        }
    }
}
