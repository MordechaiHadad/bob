use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use tokio::fs;

pub async fn remove_dir(directory: &str) -> Result<()> {
    let path = Path::new(directory);
    let size = path.read_dir()?.count();
    let read_dir = path.read_dir()?;

    let pb = ProgressBar::new(size.try_into()?);
    pb.set_style(ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({per_sec}, {eta})")
                    .progress_chars("█  "));
    pb.set_message(format!("Deleting {}", path.display()));

    let mut removed = 0;

    for entry in read_dir.flatten() {
        let path = entry.path();

        if path.is_dir() {
            fs::remove_dir_all(&path).await?;
        } else {
            fs::remove_file(&path).await?;
        }
        removed += 1;
        pb.set_position(removed);
    }

    if let Err(e) = fs::remove_dir(directory).await {
        return Err(anyhow!("Failed to remove {directory}: {}", e));
    }

    pb.finish_with_message(format!("Finished removing {}", path.display()));

    Ok(())
}

#[async_recursion(?Send)]
pub async fn copy_dir(
    from: impl AsRef<Path> + 'static,
    to: impl AsRef<Path> + 'static,
) -> Result<()> {
    let original_path = from.as_ref().to_owned();
    let destination = to.as_ref().to_owned();

    fs::create_dir(&destination).await?;

    let mut entries = fs::read_dir(original_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_dir() {
            let new_dest = destination.join(path.file_name().unwrap());
            copy_dir(path, new_dest).await?;
        } else {
            let new_dest = destination.join(path.file_name().unwrap());
            fs::copy(path, new_dest).await?;
        }
    }

    Ok(())
}

pub fn get_home_dir() -> Result<PathBuf> {
    if cfg!(windows) {
        let home_str = std::env::var("USERPROFILE")?;
        return Ok(PathBuf::from(home_str));
    }

    let mut home_str = "/home/".to_string();
    if let Ok(value) = std::env::var("SUDO_USER") {
        home_str.push_str(&value);

        return Ok(PathBuf::from(home_str));
    }

    let env_value = std::env::var("USER")?;
    home_str.push_str(&env_value);

    Ok(PathBuf::from(home_str))
}

pub fn get_local_data_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_dir()?;
    if cfg!(windows) {
        home_dir.push("AppData/Local");
        return Ok(home_dir);
    }

    home_dir.push(".local/share");
    Ok(home_dir)
}

pub fn get_config_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_dir()?;

    if cfg!(linux) {
        home_dir.push(".config");
    } else if cfg!(macos) {
        home_dir.push("Library/Application Support");
    } else {
        home_dir.push("AppData/Roaming");
    }

    home_dir.push("bob/config.json");

    Ok(home_dir)
}
