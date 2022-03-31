use std::env;
use anyhow::{anyhow, Result};
use reqwest::Client;
use crate::modules::{install_handler, utils};
use tokio::fs;
use tracing::info;
use winreg::RegKey;

pub async fn start(version: &str, client: &Client) -> Result<()> {
    if let Err(error) = install_handler::start(version, client, true).await {
        return Err(anyhow!(error));
    }
    if let Err(error) = link_version(version).await {
        return Err(anyhow!(error));
    }
    info!("Neovim has successfully installed!");
    Ok(())
}

async fn link_version(version: &str) -> Result<()> {
    use dirs::data_dir;

    let installation_dir = match data_dir() {
        None => return Err(anyhow!("Couldn't get data dir")),
        Some(value) => value,
    };

    if fs::metadata(format!("{}/neovim", installation_dir.display())).await.is_ok() {
        fs::remove_dir_all(format!("{}/neovim", installation_dir.display())).await?;
    }

    let current_dir = env::current_dir()?;

    let base_path = &format!("{}/{}", current_dir.display(), version);

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
           use std::os::windows::fs::symlink_dir;
            use winreg::enums::*;

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
        cfg_if::cfg_if!{
            if #[cfg(windows)] {
                use winreg::enums::*;

                let current_usr = RegKey::predef(HKEY_CURRENT_USER);
                let env = current_usr.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
                let usr_path: String = env.get_value("Path")?;
                let new_path = if usr_path.ends_with(';') {
                    format!("{usr_path}{}\\neovim\\bin", installation_dir.display())
                } else {
                    format!("{usr_path};{}\\neovim\\bin", installation_dir.display())
                };
                env.set_value("Path", &new_path)?;
            } else {
                info!(
                    "Add {}/neovim/bin to PATH to complete this installation",
                    installation_dir.display()
                );
            }
        }
    }
    Ok(())
}