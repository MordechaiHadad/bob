#![windows_subsystem = "windows"]

mod cli;
mod config;
mod handlers;
mod helpers;

extern crate core;

use crate::helpers::directories;
use anyhow::{anyhow, Result};
use config::{handle_config, Config};
use helpers::version;
use std::{
    env,
    path::Path,
    process::{exit, Command},
};
use tracing::{error, Level};

#[tokio::main]
async fn main() -> Result<()> {
    let collector = tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(collector)?;
    if let Err(error) = run().await {
        error!("Error: {error}");
        exit(1);
    }
    Ok(())
}

async fn run() -> Result<()> {
    let config: Config = handle_config().await?;

    let args: Vec<String> = env::args().collect();

    let exe_name_path = Path::new(&args[0]);
    let exe_name = exe_name_path.file_name().unwrap().to_str().unwrap();

    if exe_name.contains("nvim-qt") {
        let rest_args = &args[1..];

        let downloads_dir = directories::get_downloads_directory(&config).await?;
        let platform = helpers::get_platform_name();
        let used_version = version::get_current_version(&config).await?;

        let location = downloads_dir
            .join(used_version)
            .join(platform)
            .join("bin")
            .join("nvim-qt");

        let mut child = Command::new(location);
        child.args(rest_args);

        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                use std::os::windows::process::CommandExt;
                child.creation_flags(0x08000000);
            }
        }


        let mut child = child.spawn().expect("Failed to spawn child process");

        let exit_status = child
            .wait()
            .expect("Failed to wait on child process")
            .code();

        match exit_status {
            Some(0) => return Ok(()),
            Some(code) => return Err(anyhow!("Process exited with error code {}", code)),
            None => return Err(anyhow!("Process terminated by signal")),
        }
    } else if exe_name.contains("nvim") {
        let rest_args = &args[1..];

        let downloads_dir = directories::get_downloads_directory(&config).await?;
        let platform = helpers::get_platform_name();
        let used_version = version::get_current_version(&config).await?;

        let location = downloads_dir
            .join(used_version)
            .join(platform)
            .join("bin")
            .join("nvim");

        let mut child = Command::new(location)
            .args(rest_args)
            .spawn()
            .expect("Failed to spawn child process");

        let exit_status = child
            .wait()
            .expect("Failed to wait on child process")
            .code();

        match exit_status {
            Some(0) => return Ok(()),
            Some(code) => return Err(anyhow!("Process exited with error code {}", code)),
            None => return Err(anyhow!("Process terminated by signal")),
        }
    }

    cli::start(config).await?;
    Ok(())
}
