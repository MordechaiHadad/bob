mod cli;
mod config;
mod consts;
pub mod github_requests;
mod handlers;
mod helpers;

use std::env;
use std::path::Path;
use std::process::exit;

use anyhow::Result;
use config::ConfigFile;
use helpers::processes::handle_nvim_process;
use helpers::version;
use tracing::{Level, error, warn};

pub(crate) use crate::consts::*;

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

    let exe_name_path = Path::new(&args[0]);
    let exe_name = exe_name_path.file_stem().unwrap().to_str().unwrap();

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
