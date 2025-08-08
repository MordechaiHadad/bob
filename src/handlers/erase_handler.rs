use anyhow::{Result, anyhow};
use tokio::fs;
use tracing::info;

use crate::{
    config::Config,
    helpers::directories::{self},
};

/// Starts the erase process based on the provided `Config`.
///
/// # Arguments
///
/// * `config: Config` - Contains the configuration settings.
///
/// # Behavior
///
/// The function attempts to remove the installation and downloads directories. If successful, it logs a success message. If the directories do not exist, it returns an error.
///
/// On Windows, it also attempts to remove the Neovim installation path from the registry. If successful, it logs a success message.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the function executes successfully, otherwise it returns an error.
///
/// # Errors
///
/// This function will return an error if there's a problem with removing the directories or modifying the registry.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// start(config).await?;
/// ```
///
/// # Note
///
/// This function is asynchronous and must be awaited.
///
/// # See Also
///
/// * [`directories::get_downloads_directory`](src/helpers/directories.rs)
/// * [`directories::get_installation_directory`](src/helpers/directories.rs)
pub async fn start(config: Config) -> Result<()> {
    let downloads = directories::get_downloads_directory(&config).await?;
    let mut installation_dir = directories::get_installation_directory(&config).await?;

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
        } else {
            use what_the_path::shell::Shell;

            let shell = Shell::detect_by_shell_var()?;

            match shell {
                Shell::Fish(fish) => {
                   if let Ok(files) = fish.get_rcfiles() {
                       let fish_file = files[0].join("bob.fish");
                       if !fish_file.exists() { return Ok(()) }
                       fs::remove_file(fish_file).await?;
                   }
                },
                shell => {
                    if let Ok(files) = shell.get_rcfiles() {
                        let env_path = downloads.join("env/env.sh");
                        let source_string = format!(". \"{}\"", env_path.display());
                        for file in files {
                            what_the_path::shell::remove_from_rcfile(file, &source_string)?;
                        }

                    }
                }
            }
        }
    }

    if config.installation_location.is_some() {
        installation_dir.push("nvim");
        if fs::remove_file(&installation_dir).await.is_ok() {
            info!("Successfully removed neovim executable");
        }
    } else if fs::remove_dir_all(&installation_dir).await.is_ok() {
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

    Ok(())
}
