use crate::models::Config;

use super::utils;
use anyhow::{anyhow, Result};
use std::fs;
use yansi::Paint;

pub async fn start(config: Config) -> Result<()> {
    let downloads_dir = match utils::get_downloads_folder(&config).await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };
    if cfg!(target_os = "macos") {
        println!("Downloads dir is: {}", downloads_dir.display());
    }

    let paths = fs::read_dir(downloads_dir)?
        .filter_map(|e| e.ok())
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    const VERSION_MAX_LEN: usize = 7;

    if paths.len() == 0 {
        return Err(anyhow!("There are no versions installed"));
    }

    println!("Version | Status");
    println!("{}+{}", "-".repeat(7 + 1), "-".repeat(10));

    for path in paths {
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
