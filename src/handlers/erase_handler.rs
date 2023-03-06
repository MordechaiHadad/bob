use anyhow::{anyhow, Result};
use tokio::fs;
use tracing::info;

use crate::{config::Config, helpers::directories};

pub async fn start(config: Config) -> Result<()> {
    let downloads = directories::get_downloads_directory(&config).await?;
    let installation_dir = directories::get_installation_directory(&config).await?;

    if fs::remove_dir_all(&installation_dir).await.is_ok() {
        info!("Successfully removed neovim's installation folder");
    }
    if fs::remove_dir_all(downloads).await.is_ok() {
        // For some weird reason this check doesn't really work for downloads folder
        // as it keeps thinking the folder exists and it runs with no issues even tho the folder
        // doesn't exist damn...
        info!("Successfully removed neovim downloads folder");
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
            if usr_path.contains("neovim") {
                let usr_path = usr_path.replace(&format!("{}", installation_dir.display()), "");
                env.set_value("Path", &usr_path)?;

                info!("Successfully removed neovim's installation PATH from registry");
            }

        }
    }

    Ok(())
}
