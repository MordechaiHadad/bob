use crate::models::Config;

use super::utils;
use anyhow::{anyhow, Result};
use yansi::Paint;

pub async fn start(config: Config) -> Result<()> {
    let downloads_dir = match utils::get_downloads_folder(&config).await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };

    let paths = std::fs::read_dir(downloads_dir)?;
    const VERSION_MAX_LEN: usize = 7;

    println!("Version | Status");
    println!("{}+{}", "-".repeat(7 + 1), "-".repeat(10));

    for path in paths {
        let path = path.unwrap().path();
        let path_name = path.file_name().unwrap().to_str().unwrap();
        if path_name == "neovim-git" {
            continue;
        }

        let width = (VERSION_MAX_LEN - path_name.len()) + 1;
        if path.is_dir() {
            if utils::is_version_used(path_name, &config).await {
                println!("{path_name}{}| {}", " ".repeat(width), Paint::green("Used"));
            } else {
                println!(
                    "{path_name}{}| {}",
                    " ".repeat(width),
                    Paint::yellow("Installed")
                );
            }
        }
    }
    Ok(())
}
