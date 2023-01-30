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

    let padding = if version_max_len == 16 { 7 } else { 2 };

    println!("┌{}┬{}┐", "─".repeat(version_max_len + 5), "─".repeat(12));
    println!(
        "│{}Version{}│{}Status{}│",
        " ".repeat(padding),
        " ".repeat(padding),
        " ".repeat(3),
        " ".repeat(3)
    );
    println!("├{}┼{}┤", "─".repeat(11), "─".repeat(12));

    for path in paths {
        let path_name = path.file_name().unwrap().to_str().unwrap();
        if path_name == "neovim-git" {
            continue;
        }

        let width = (version_max_len - path_name.len()) + 2;
        if !path.is_dir() {
            continue;
        }

        if utils::is_version_used(path_name, &config).await {
            println!(
                "│  {path_name}{}│  {} {}│",
                " ".repeat(width),
                Paint::green("Used"),
                " ".repeat(5)
            );
        } else {
            println!(
                "│  {path_name}{}│  {} │",
                " ".repeat(width),
                Paint::yellow("Installed")
            );
        }
    }

    println!("└{}┴{}┘", "─".repeat(11), "─".repeat(12));

    Ok(())
}

async fn has_rollbacks(config: &Config) -> Result<bool> {
    let list = rollback_handler::produce_nightly_vec(config).await?;

    Ok(!list.is_empty())
}
