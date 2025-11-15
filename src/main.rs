mod cli;
mod config;
mod consts;
pub mod github_requests;
mod handlers;
mod helpers;

use anyhow::Result;
use config::ConfigFile;
use helpers::{processes::handle_nvim_process, version};
use std::{env, process::exit};
use tracing::{Level, error, warn};

pub(crate) use crate::consts::{
    ENVIRONMENT_VAR_REGEX,
    FILETYPE_EXT,
    HASH_REGEX,
    //
    NIGHTLY_REGEX,
    VERSION_REGEX,
};

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
    let config = ConfigFile::get().await?;

    let args: Vec<String> = env::args().collect();

    let exe_path = std::env::current_exe()?;
    let exe_name = exe_path
        .is_symlink()
        .then(|| exe_path.read_link().ok())
        .flatten()
        .and_then(|p| p.file_stem().map(|s| s.to_owned()))
        .unwrap_or_else(|| exe_path.file_stem().unwrap().to_owned());

    let rest_args = &args[1..];

    if exe_name.eq("nvim") {
        if !rest_args.is_empty() && rest_args[0].eq("--&bob") {
            print!("{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }

        handle_nvim_process(&config.config, rest_args).await?;

        return Ok(());
    }

    if cfg!(target_os = "freebsd") {
        let fbsd_linux_url = "https://docs.freebsd.org/en/books/handbook/linuxemu/";
        warn!("Using Bob on FreeBSD requires the Linux compat; see {fbsd_linux_url}");
    }

    cli::start(config).await?;
    Ok(())
}
