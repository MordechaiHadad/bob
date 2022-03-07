use super::utils;
use anyhow::{anyhow, Result};

pub async fn start() -> Result<()> {
    let downloads_dir = match utils::get_downloads_folder().await {
        Ok(value) => value,
        Err(error) => return Err(anyhow!(error)),
    };

    let paths = std::fs::read_dir(downloads_dir)?;
    let mut versions = String::new();
    let installed_version = match utils::get_current_version().await {
        Some(value) => value,
        None => return Err(anyhow!("Neovim is not installed")),
    };
    let mut first = true;
    for path in paths {
        let path = path.unwrap().path();
        let path_name = path.file_name().unwrap();
        if path.is_dir() {
            if !first {
                if path_name.to_str().unwrap().contains(&installed_version) {
                    versions += &format!(", {}(installed)", path_name.to_str().unwrap());
                } else {
                    versions += &format!(", {}", path_name.to_str().unwrap());
                }
            } else {
                if path_name.to_str().unwrap().contains(&installed_version) {
                    versions += &format!("{}(installed)", path_name.to_str().unwrap());
                } else {
                    versions += path_name.to_str().unwrap()
                }
                first = false;
            }
        }
    }
    println!("{versions}");
    Ok(())
}
