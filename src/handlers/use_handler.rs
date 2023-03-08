use anyhow::Result;
use reqwest::Client;
use std::env;
use std::path::Path;
use tokio::fs;
use tracing::info;

use crate::config::Config;
use crate::handlers::{install_handler, InstallResult};
use crate::helpers;
use crate::helpers::version::types::{ParsedVersion, VersionType};

pub async fn start(
    mut version: ParsedVersion,
    install: bool,
    client: &Client,
    config: Config,
) -> Result<()> {
    let is_version_used = helpers::version::is_version_used(&version.tag_name, &config).await;

    copy_nvim_bob(&config).await?;
    if is_version_used && version.tag_name != "nightly" {
        info!("{} is already installed and used!", version.tag_name);
        return Ok(());
    }

    if install {
        match install_handler::start(&mut version, client, &config).await {
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
    }

    switch(&config, &version).await?;

    if let VersionType::Latest = version.version_type {
        if fs::metadata("stable").await.is_ok() {
            fs::remove_dir_all("stable").await?;
        }
    }

    info!("You can now use {}!", version.tag_name);

    Ok(())
}

pub async fn switch(config: &Config, version: &ParsedVersion) -> Result<()> {
    std::env::set_current_dir(helpers::directories::get_downloads_directory(config).await?)?;

    copy_nvim_bob(config).await?;
    fs::write("used", &version.tag_name).await?;
    if let Some(sync_version_file_path) =
        helpers::version::get_sync_version_file_path(config).await?
    {
        // Write the used version to sync_version_file_path only if it's different
        let stored_version = fs::read_to_string(&sync_version_file_path).await?;
        if stored_version != version.tag_name {
            fs::write(&sync_version_file_path, &version.tag_name).await?;
            info!(
                "Written version to {}",
                sync_version_file_path
                    .into_os_string()
                    .into_string()
                    .unwrap()
            );
        }
    }

    Ok(())
}

async fn copy_nvim_bob(config: &Config) -> Result<()> {
    let exe_path = env::current_exe().unwrap();
    let mut installation_dir = helpers::directories::get_installation_directory(config).await?;

    if fs::metadata(&installation_dir).await.is_err() {
        fs::create_dir_all(&installation_dir).await?;
    }

    add_to_path(&installation_dir)?;

    if cfg!(windows) {
        installation_dir.push("nvim.exe");
    } else {
        installation_dir.push("nvim");
    }

    if fs::metadata(&installation_dir).await.is_ok() {
        return Ok(());
    }

    fs::copy(exe_path, &installation_dir).await?;

    Ok(())
}

fn add_to_path(installation_dir: &Path) -> Result<()> {
    let path_env = std::env::var_os("PATH")
        .unwrap()
        .to_string_lossy()
        .to_string();
    let installation_dir = installation_dir.to_str().unwrap();

    if path_env.contains(installation_dir) {
        return Ok(());
    }

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            use winreg::enums::*;
            use winreg::RegKey;

            let current_usr = RegKey::predef(HKEY_CURRENT_USER);
            let env = current_usr.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
            let usr_path: String = env.get_value("Path")?;
            let new_path = if usr_path.ends_with(';') {
                format!("{usr_path}{}", installation_dir)
            } else {
                format!("{usr_path};{}", installation_dir)
            };
            env.set_value("Path", &new_path)?;
        } else {
            info!("Make sure to have {installation_dir} in PATH");
        }
    }

    Ok(())
}
