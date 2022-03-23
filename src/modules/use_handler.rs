use std::env;
use anyhow::{anyhow, Result};
use reqwest::Client;
use crate::modules::{install_handler, utils};
use tokio::fs;
use tracing::info;

pub async fn start(version: &str, client: &Client) -> Result<()> {
    if let Err(error) = install_handler::start(version, client, true).await {
        return Err(anyhow!(error));
    }
    if let Err(error) = link_version(version).await {
        return Err(anyhow!(error));
    }
    info!("You can now open neovim");
    Ok(())
}

async fn link_version(version: &str) -> Result<()> {
    use dirs::data_dir;

    let installation_dir = match data_dir() {
        None => return Err(anyhow!("Couldn't get data dir")),
        Some(value) => value,
    };

    if fs::metadata(format!("{}/neovim", installation_dir.display())).await.is_ok() {
        info!("Deleting existing neovim link");
        fs::remove_dir_all(format!("{}/neovim", installation_dir.display())).await?;
    }

    let current_dir = match env::current_dir() {
        Ok(value) => value,
        Err(_) => return Err(anyhow!("Couldn't get current environment directory"))
    };

    let base_path = &format!("{}/{}", current_dir.display(), version);
    info!("Preparing to install from {base_path}");

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
           use std::os::windows::fs::symlink_dir;
            let base_dir = if fs::metadata(&format!("{base_path}/Neovim")).await.is_ok() {
                "Neovim"
            } else {
                "nvim-win64"
            };

           if symlink_dir(format!("{base_path}/{base_dir}"),
               format!("{}/neovim", installation_dir.display())).is_err() {
                   return Err(anyhow!("Please restart this application as admin to complete the installation."));
               }
        } else {
            use std::os::unix::fs::symlink;
            let folder_name = if cfg!(target_os = "macos") {
                info!("Starting linking process: {base_path}/nvim-osx64");
                "nvim-osx64"
            } else {
                "nvim-linux64"
            };
            if let Err(error) = symlink(format!("{base_path}/{folder_name}"), format!("{}/neovim", installation_dir.display())) {
                return Err(anyhow!(format!("Couldn't find {base_path}/{folder_name}")))
            }
        }
    }

    info!("Linked {version} to {}/neovim", installation_dir.display());

    if !utils::is_version_used(version).await {
        info!(
            "Add {}/neovim/bin to PATH to complete this installation",
            installation_dir.display()
        ); // TODO: do this automatically
    }
    Ok(())
}