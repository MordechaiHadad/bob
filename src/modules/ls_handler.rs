use super::utils;
use anyhow::{anyhow, Result};

pub async fn start() -> Result<()> {
    let downloads_dir = match utils::get_downloads_folder().await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };

    let paths = std::fs::read_dir(downloads_dir)?;
    let used_version = match utils::get_current_version().await {
        Some(value) => value,
        None => return Err(anyhow!("Neovim is not installed")),
    };
    const VERSION_MAX_LEN: usize = 7;

    println!("Version | Status");
    println!("{}|{}", "-".repeat(7 + 1) ,"-".repeat(10));

    for path in paths {
        let path = path.unwrap().path();
        let path_name = path.file_name().unwrap().to_str().unwrap();

        let width = (VERSION_MAX_LEN - path_name.len()) + 1;
        if path.is_dir() {
            if path_name.contains(&used_version) {
                println!("{path_name}{}| Used", " ".repeat(width));
            } else {
                println!("{path_name}{}| Installed", " ".repeat(width));
            }
        }
    }
    Ok(())
}
