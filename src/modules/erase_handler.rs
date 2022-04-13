use super::utils;
use anyhow::Result;
use tokio::fs;

pub async fn start() -> Result<()> {
    let downloads = utils::get_downloads_folder().await?;
    let installation_dir = utils::get_installation_folder()?;

    fs::remove_dir_all(downloads).await?;
    fs::remove_dir_all(installation_dir).await?;

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            use winreg::RegKey;
            let current_usr = RegKey::predef(HKEY_CURRENT_USER);
            let env = current_usr.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
            let usr_path: String = env.get_value("Path")?;
        }
    }

    Ok(())
}
