use crate::models::Config;

use super::{rollback_handler, utils};
use anyhow::{anyhow, Result};
use std::fs;
use yansi::Paint;

pub async fn start(config: Config) -> Result<()> {
    let downloads_dir = utils::get_downloads_folder(&config).await?;

    let paths = fs::read_dir(downloads_dir)?
        .filter_map(|e| e.ok())
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    let version_max_len = if has_rollbacks(&config).await? { 16 } else { 7 };

    if paths.is_empty() {
        return Err(anyhow!("There are no versions installed"));
    }

    println!("Version | Status");
    println!("{}+{}", "-".repeat(7 + 1), "-".repeat(10));

    for path in paths {
        let path_name = path.file_name().unwrap().to_str().unwrap();
        if path_name == "neovim-git" {
            continue;
        }

        let width = (version_max_len - path_name.len()) + 1;
        if !path.is_dir() {
            continue;
        }

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
    Ok(())
}

async fn has_rollbacks(config: &Config) -> Result<bool> {
    let list = rollback_handler::produce_nightly_vec(config).await?;

    Ok(!list.is_empty())
}
