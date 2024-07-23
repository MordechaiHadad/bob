use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use tokio::fs;

/// Asynchronously removes a directory and all its contents.
///
/// This function takes a string reference as an argument, which represents the directory to be removed.
/// It first reads the directory and counts the number of entries. Then, it creates a progress bar with the total number of entries.
/// It iterates over each entry in the directory. If the entry is a directory, it removes the directory and all its contents. If the entry is a file, it removes the file.
/// After removing each entry, it updates the progress bar.
/// Finally, it attempts to remove the directory itself. If this operation fails, it returns an error.
///
/// # Arguments
///
/// * `directory` - A string reference representing the directory to be removed.
///
/// # Returns
///
/// This function returns a `Result` that indicates whether the operation was successful.
/// If the operation was successful, the function returns `Ok(())`.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Example
///
/// ```rust
/// let directory = "/path/to/directory";
/// remove_dir(directory).await;
/// ```
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

/// Asynchronously copies a directory from one location to another.
///
/// This function takes two arguments: the source directory and the destination directory. Both arguments are implemented as references to `Path` and are static.
/// It first creates the destination directory, then reads the entries of the source directory.
/// For each entry in the source directory, it checks if the entry is a directory or a file.
/// If the entry is a directory, it recursively calls `copy_dir` to copy the directory to the destination.
/// If the entry is a file, it copies the file to the destination.
///
/// # Arguments
///
/// * `from` - A reference to a `Path` representing the source directory.
/// * `to` - A reference to a `Path` representing the destination directory.
///
/// # Returns
///
/// This function returns a `Result` that indicates whether the operation was successful.
/// If the operation was successful, the function returns `Ok(())`.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Example
///
/// ```rust
/// let from = Path::new("/path/to/source");
/// let to = Path::new("/path/to/destination");
/// copy_dir(from, to).await;
/// ```
#[async_recursion(?Send)]
pub async fn copy_dir_async(
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
            copy_dir_async(path, new_dest).await?;
        } else {
            let new_dest = destination.join(path.file_name().unwrap());
            fs::copy(path, new_dest).await?;
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let original_path = from.as_ref().to_owned();
    let destination = to.as_ref().to_owned();

    std::fs::create_dir(&destination)?;

    let entries = std::fs::read_dir(original_path)?;

    for entry in entries {
        let path = entry?.path();

        if path.is_dir() {
            let new_dest = destination.join(path.file_name().unwrap());
            copy_dir(path, new_dest)?;
        } else {
            let new_dest = destination.join(path.file_name().unwrap());
            std::fs::copy(path, new_dest)?;
        }
    }

    Ok(())
}
