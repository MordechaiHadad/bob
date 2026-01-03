use anyhow::Result;
use async_recursion::async_recursion;
use std::path::Path;
use tokio::fs;

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
