mod cli;
mod config;
pub mod github_requests;
mod handlers;
mod helpers;

extern crate core;

use anyhow::{anyhow, Result};
use config::{handle_config, Config};
use helpers::{
    processes::{handle_nvim_process, NvimProcessType},
    version,
};
use std::{env, path::Path, process::exit};
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

    let rest_args = &args[1..];

    if exe_name.contains("nvim") {
        let is_qt = exe_name.contains("qt");
        if cfg!(unix) && is_qt {
            return Err(anyhow!("This is only usable on windows"));
        }
        if !rest_args.is_empty() && rest_args[0].eq("--&bob") {
            print!("{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }

        let process_type = if is_qt {
            NvimProcessType::NvimQt
        } else {
            NvimProcessType::Nvim
        };

        handle_nvim_process(&config, rest_args, process_type).await?;
    }

    cli::start(config).await?;
    Ok(())
}
