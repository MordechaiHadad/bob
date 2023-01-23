use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
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

pub async fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let path = from.as_ref().to_owned();
    let destination = to.as_ref().to_owned();

    fs::create_dir(&destination).await?;

    let mut entries = fs::read_dir(path).await?;


    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_dir() {
            let new_dest = destination.join(path.strip_prefix("nightly")?);
            fs::create_dir(&new_dest).await?;
        } else {
            let new_dest = destination.join(path.strip_prefix("nightly")?);
            fs::copy(path, new_dest).await?;
        }

    }

    Ok(())
}
