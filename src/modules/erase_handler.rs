use super::utils;
use anyhow::{anyhow, Result};
use tokio::fs;
use tracing::info;

pub async fn start() -> Result<()> {
    let downloads = utils::get_downloads_folder().await?;
    let installation_dir = utils::get_installation_folder()?;

    if fs::remove_dir_all(&installation_dir).await.is_ok() {
        info!("Successfully removed neovim's installation folder");
    }
    if fs::remove_dir_all(&downloads).await.is_ok() {
        // this for some reason always throws true for this directory no matter what, even if I run fs::metadate(path).is_ok() it still wont work... makes no sense
        info!("Successfully removed neovim downloads folder",);
    } else {
        return Err(anyhow!("There's nothing to erase"));
    }

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            use winreg::RegKey;
            use winreg::enums::*;

            let current_usr = RegKey::predef(HKEY_CURRENT_USER);
            let env = current_usr.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
            let usr_path: String = env.get_value("Path")?;
            let usr_path = usr_path.replace(&format!("{}\\bin", installation_dir.display()), "");
            env.set_value("Path", &usr_path)?;

            info!("Successfully removed neovim's installation PATH from registry");
        }
    }

    Ok(())
}
