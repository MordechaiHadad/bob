use crate::enums::InstallResult;
use crate::models::Config;
use crate::modules::{install_handler, utils};
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::env;
use tokio::fs;
use tracing::info;

pub async fn start(version: &str, client: &Client, config: Config) -> Result<()> {
    let is_version_used = utils::is_version_used(version).await;
    if is_version_used && version != "nightly" {
        info!("{version} is already installed and used!");
        return Ok(());
    }

    match install_handler::start(version, client, &config).await {
        Ok(success) => {
            if let InstallResult::NightlyIsUpdated = success {
                if is_version_used {
                    info!("Nightly is already updated and used!");
                    return Ok(());
                }
            }
        }
        Err(error) => return Err(error),
    }

    if let Err(error) = link_version(version, &config).await {
        return Err(error);
    }
    info!("You can now use {version}!");

    Ok(())
}

async fn link_version(version: &str, config: &Config) -> Result<()> {
    let installation_dir = match utils::get_installation_folder(&config) {
        Err(_) => return Err(anyhow!("Couldn't get data dir")),
        Ok(value) => value,
    };
    let current_dir = env::current_dir()?;

    let base_path = &format!("{}/{}", current_dir.display(), version);

    if fs::metadata(&installation_dir).await.is_ok() {
        fs::remove_dir_all(&installation_dir).await?;
    }

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
           use std::os::windows::fs::symlink_dir;
            use winreg::RegKey;


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
            if fs::metadata(format!("{base_path}/nvim-osx64")).await.is_ok() {
                fs::rename(format!("{base_path}/nvim-osx64"), "nvim-macos").await?;
            }
            let folder_name = utils::get_platform_name();
            if let Err(error) = symlink(format!("{base_path}/{folder_name}"), &installation_dir) {
                return Err(anyhow!(error))
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
            } else {
                info!("Please add {}/bin to PATH", installation_dir.display());
            }
        }
    }
    Ok(())
}
