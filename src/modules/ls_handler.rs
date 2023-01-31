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


    if paths.is_empty() {
        return Err(anyhow!("There are no versions installed"));
    }

    let version_max_len = if has_rollbacks(&config).await? { 16 } else { 7 };
    let status_max_len = 9;
    let padding = 2;

    println!(
        "┌{}┬{}┐",
        "─".repeat(version_max_len + (padding * 2)),
        "─".repeat(status_max_len + (padding * 2))
    );
    println!(
        "│{}Version{}│{}Status{}│",
        " ".repeat(padding),
        " ".repeat(padding + (version_max_len - 7)),
        " ".repeat(padding),
        " ".repeat(padding + (status_max_len - 6))
    );
    println!(
        "├{}┼{}┤",
        "─".repeat(version_max_len + (padding * 2)),
        "─".repeat(status_max_len + (padding * 2))
    );

    for path in paths {
        let path_name = path.file_name().unwrap().to_str().unwrap();
        if path_name == "neovim-git" {
            continue;
        }

        if !path.is_dir() {
            continue;
        }

        let version_pr = (version_max_len - path_name.len()) + padding;
        let status_pr = padding + status_max_len;

        if utils::is_version_used(path_name, &config).await {
            println!(
                "│{}{path_name}{}│{}{}{}│",
                " ".repeat(padding),
                " ".repeat(version_pr),
                " ".repeat(padding),
                Paint::green("Used"),
                " ".repeat(status_pr - 4)
            );
        } else {
            println!(
                "│{}{path_name}{}│{}{}{}│",
                " ".repeat(padding),
                " ".repeat(version_pr),
                " ".repeat(padding),
                Paint::yellow("Installed"),
                " ".repeat(status_pr - 9)
            );
        }
    }

    println!(
        "└{}┴{}┘",
        "─".repeat(version_max_len + (padding * 2)),
        "─".repeat(status_max_len + (padding * 2))
    );

    Ok(())
}

async fn has_rollbacks(config: &Config) -> Result<bool> {
    let list = rollback_handler::produce_nightly_vec(config).await?;

    Ok(!list.is_empty())
}
