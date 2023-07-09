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
                if is_version_used && success == InstallResult::NightlyIsUpdated {
                    info!("Nightly is already updated and used!");
                    return Ok(());
                }
            }
            Err(error) => return Err(error),
        }
    }

    switch(&config, &version).await?;

    if version.version_type == VersionType::Latest && fs::metadata("stable").await.is_ok() {
        fs::remove_dir_all("stable").await?;
    }

    info!("You can now use {}!", version.tag_name);

    Ok(())
}

pub async fn switch(config: &Config, version: &ParsedVersion) -> Result<()> {
    std::env::set_current_dir(helpers::directories::get_downloads_directory(config).await?)?;

    copy_nvim_bob(config).await?;
    fs::write("used", &version.tag_name).await?;
    if let Some(version_sync_file_location) =
        helpers::version::get_version_sync_file_location(config).await?
    {
        // Write the used version to version_sync_file_location only if it's different
        let stored_version = fs::read_to_string(&version_sync_file_location).await?;
        if stored_version != version.tag_name {
            fs::write(&version_sync_file_location, &version.tag_name).await?;
            info!(
                "Written version to {}",
                version_sync_file_location
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

    if fs::metadata(&installation_dir).await.is_err() {
        fs::copy(&exe_path, &installation_dir).await?;
    }

    if cfg!(windows) {
        installation_dir = installation_dir.parent().unwrap().to_path_buf();
        installation_dir.push("nvim-qt.exe");

        if fs::metadata(&installation_dir).await.is_ok() {
            return Ok(());
        }

        fs::copy(exe_path, installation_dir).await?;
    }

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
            if !std::env::var("PATH")?.contains("nvim-bin") {
                info!("Make sure to have {installation_dir} in PATH");
            }
        }
    }

    Ok(())
}
