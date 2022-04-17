use super::{install_handler, ls_handler, uninstall_handler, use_handler, utils, erase_handler};
use crate::enums::InstallResult;
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
        .subcommand(Command::new("erase")
                    .about("Erase any change bob ever made including: neovim installation, neovim installs and registry changes"))
        .subcommand(Command::new("ls").about("List all downloaded and installed versions"));

    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("use", subcommand)) | Some(("install", subcommand)) => {
            let client = Client::new();
            if let Some(value) = subcommand.value_of("VERSION") {
                let version = match utils::parse_version(&client, value).await {
                    Ok(version) => version,
                    Err(error) => return Err(error),
                };

                match matches.subcommand_name().unwrap() {
                    "use" => {
                        if let Err(error) = use_handler::start(&version, &client).await {
                            return Err(anyhow!(error));
                        }
                    }
                    "install" => match install_handler::start(&version, &client).await {
                        Ok(result) => match result {
                            InstallResult::InstallationSuccess(location) => {
                                info!("{version} has been successfully installed in {location}")
                            }
                            InstallResult::VersionAlreadyInstalled => {
                                info!("{version} is already installed!")
                            }
                            InstallResult::NightlyIsUpdated => info!("Nightly is up to date!"),
                        },
                        Err(error) => return Err(error),
                    },
                    _ => (),
                }
            }
        }
        Some(("uninstall", subcommand)) => {
            info!("Starting uninstallation process");
            if let Err(error) = uninstall_handler::start(subcommand).await {
                return Err(error);
            }
        }
        Some(("ls", _)) => {
            if let Err(error) = ls_handler::start().await {
                return Err(error);
            }
        }
        Some(("erase", _)) => {
            if let Err(error) = erase_handler::start().await {
                return Err(error);
            }
        },
        _ => (),
    }

    Ok(())
}
