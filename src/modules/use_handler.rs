use crate::modules::{install_handler, utils};
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::env;
use tokio::fs;
use tracing::info;

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
    let installation_dir = match utils::get_install_folder() {
        Err(_) => return Err(anyhow!("Couldn't get data dir")),
        Ok(value) => value,
    };
    let current_dir = env::current_dir()?;

    let base_path = &format!("{}/{}", current_dir.display(), version);

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
           use std::os::windows::fs::symlink_dir;
            use winreg::RegKey;

            if fs::metadata(&installation_dir).await.is_ok() {
                fs::remove_dir_all(&installation_dir).await?;
            }

            let base_dir = if fs::metadata(&format!("{base_path}/Neovim")).await.is_ok() {
                "Neovim"
            } else {
                "nvim-win64"
            };

            if symlink_dir(format!("{base_path}/{base_dir}"),
               &installation_dir).is_err() {
                   return Err(anyhow!("Please restart this application as admin to complete the installation."));
            }
        } else {
            use std::os::unix::fs::symlink;
            let folder_name = if cfg!(target_os = "macos") {
                "nvim-osx64"
            } else {
                "nvim-linux64"
            };
            if let Err(error) = symlink(format!("{base_path}/{folder_name}/nvim"), format!("{}/nvim", installation_dir.display())) {
                return Err(anyhow!(format!("Couldn't find {base_path}/{folder_name}/nvim")))
            }
        }
    }

    if !utils::is_version_used(version).await {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                use winreg::enums::*;

                let current_usr = RegKey::predef(HKEY_CURRENT_USER);
                let env = current_usr.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
                let usr_path: String = env.get_value("Path")?;
                let new_path = if usr_path.ends_with(';') {
                    format!("{usr_path}{}\\bin", installation_dir.display())
                } else {
                    format!("{usr_path};{}\\bin", installation_dir.display())
                };
                env.set_value("Path", &new_path)?;
            }
        }
    }
    Ok(())
}
