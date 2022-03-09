use super::{install_handler, ls_handler};
use crate::modules::{uninstall_handler, use_handler, utils};
use anyhow::{anyhow, Result};
use clap::{arg, Command};
use reqwest::Client;
use tracing::info;

pub async fn start() -> Result<()> {
    let app = Command::new("bob")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("use")
                .arg(arg!([VERSION]).required(true))
                .about("Switch to the specified neovim version"),
        )
        .subcommand(
            Command::new("install")
                .arg(arg!([VERSION]).required(true))
                .about("Install the specified version"),
        )
        .subcommand(
            Command::new("uninstall")
                .arg(arg!([VERSION]).required(true))
                .about("Uninstall the specified version"),
        )
        .subcommand(Command::new("ls").about("List all downloaded and installed versions"));

    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("use", subcommand))
        | Some(("install", subcommand)) => {
            let client = Client::new();
            if let Some(value) = subcommand.value_of("VERSION") {
                let version = match utils::parse_version(&client, value).await {
                    Ok(version) => version,
                    Err(error) => return Err(anyhow!(error)),
                };

                if utils::is_version_used(&version).await {
                    info!("{version} is already installed and used");
                    return Ok(());
                }

                match matches.subcommand_name().unwrap() {
                    "use" => {
                        if let Err(error) = use_handler::start(&version, &client).await {
                            return Err(anyhow!(error));
                        }

                    }
                    "install" => {
                        if let Err(error) = install_handler::start(&version, &client, false).await {
                            return Err(anyhow!(error));
                        }
                    }
                    _ => (),
                }


            }
        }
        Some(("uninstall", subcommand)) => {
            info!("Starting uninstallation process");
            if let Err(error) = uninstall_handler::start(subcommand).await {
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
