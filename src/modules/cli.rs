use super::{ls_handler, use_handler};
use anyhow::{anyhow, Result};
use clap::{arg, Command};

pub async fn start() -> Result<()> {
    let app = Command::new("bob")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("use")
                .arg(arg!([VERSION]).required(true))
                .about("Switch to a different neovim version"),
        )
        .subcommand(Command::new("ls").about("List all downloaded and installed versions"));

    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("use", subcommand)) => {
            if let Err(error) = use_handler::start(subcommand).await {
                return Err(anyhow!(error));
            }
        }
        Some(("ls", _)) => {
            if let Err(error) = ls_handler::start().await {
                return Err(anyhow!(error));
            }
        }
        _ => (),
    }

    Ok(())
}
