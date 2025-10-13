use anyhow::{Result, anyhow};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::version::types::LocalVersion;

/// Starts the process of expanding a downloaded file.
///
/// This function is asynchronous and uses `tokio::task::spawn_blocking` to run the `expand` function in a separate thread.
/// It takes a `LocalVersion` struct which contains information about the downloaded file, such as its name, format, and path.
/// The function first clones the `LocalVersion` struct and passes it to the `expand` function.
/// If the `expand` function returns an error, the `start` function also returns an error.
/// If the `expand` function is successful, the `start` function removes the original downloaded file.
///
/// # Arguments
///
/// * `file` - A `LocalVersion` struct representing the downloaded file.
///
/// # Returns
///
/// This function returns a `Result` that indicates whether the operation was successful.
/// If the operation was successful, the function returns `Ok(())`.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The `expand` function returns an error.
/// * The original downloaded file could not be removed.
///
/// # Example
///
/// ```rust
/// let downloaded_file = LocalVersion {
///     file_name: "nvim-linux",
///     file_format: "tar.gz",
///     semver: semver::Version::parse("0.5.0").unwrap(),
///     path: "/path/to/downloaded/file",
/// };
/// start(downloaded_file).await;
/// ```
pub async fn start(file: LocalVersion) -> Result<()> {
    let temp_file = file.clone();
    match tokio::task::spawn_blocking(move || match expand(temp_file) {
        Ok(_) => Ok(()),
        Err(error) => Err(anyhow!(error)),
    })
    .await
    {
        Ok(_) => (),
        Err(error) => return Err(anyhow!(error)),
    }
    tokio::fs::remove_file(format!(
        "{}/{}.{}",
        file.path, file.file_name, file.file_format
    ))
    .await?;
    Ok(())
}
/// Expands a downloaded file on Windows.
///
/// This function is specific to Windows due to the use of certain features like `zip::ZipArchive`.
/// It takes a `LocalVersion` struct which contains information about the downloaded file, such as its name and format.
/// The function then opens the file and extracts its contents using `zip::ZipArchive`.
/// During the extraction process, a progress bar is displayed to the user.
/// After extraction, the function removes the original zip file.
///
/// # Arguments
///
/// * `downloaded_file` - A `LocalVersion` struct representing the downloaded file.
///
/// # Returns
///
/// This function returns a `Result` that indicates whether the operation was successful.
/// If the operation was successful, the function returns `Ok(())`.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The downloaded file could not be opened.
/// * The file could not be extracted.
/// * The original zip file could not be removed.
///
/// # Example
///
/// ```rust
/// let downloaded_file = LocalVersion {
///     file_name: "nvim-windows",
///     file_format: "zip",
///     semver: semver::Version::parse("0.5.0").unwrap(),
///     path: "/path/to/downloaded/file",
/// };
/// expand(downloaded_file);
/// ```
#[cfg(target_family = "windows")]
fn expand(downloaded_file: LocalVersion) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    use std::cmp::min;
    use std::fs::File;
    use std::io;
    use std::path::Path;
    use zip::ZipArchive;

    if fs::metadata(&downloaded_file.file_name).is_ok() {
        fs::remove_dir_all(&downloaded_file.file_name)?;
    }

    let file = File::open(format!(
        "{}.{}",
        downloaded_file.file_name, downloaded_file.file_format
    ))?;

    let mut archive = ZipArchive::new(file)?;
    let totalsize: u64 = archive.len() as u64;

    let pb = ProgressBar::new(totalsize);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}",
        )
        .unwrap()
        .progress_chars("█  "),
    );
    pb.set_message("Expanding archive");

    std::fs::create_dir(downloaded_file.file_name.clone())?;

    let mut downloaded: u64 = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = remove_base_parent(&file.mangled_name()).unwrap();
        let outpath = Path::new(&downloaded_file.file_name).join(file_path);

        if file.is_dir() {
            fs::create_dir_all(outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = fs::File::create(outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
        let new = min(downloaded + 1, totalsize);
        downloaded = new;
        pb.set_position(new);
    }
    pb.finish_with_message(format!(
        "Finished unzipping to {}/{}",
        downloaded_file.path, downloaded_file.file_name
    ));

    Ok(())
}

/// Expands a downloaded file on Unix systems (macOS and Linux).
///
/// This function is specific to Unix systems due to the use of certain features like `os::unix::fs::PermissionsExt`.
/// It takes a `LocalVersion` struct which contains information about the downloaded file, such as its name and format.
/// The function then opens the file, decompresses it using `GzDecoder`, and extracts its contents using `tar::Archive`.
/// During the extraction process, a progress bar is displayed to the user.
/// After extraction, the function renames the `nvim-osx64` directory to `nvim-macos` if it exists.
/// Finally, it sets the permissions of the `nvim` binary to `0o551`.
///
/// # Arguments
///
/// * `downloaded_file` - A `LocalVersion` struct representing the downloaded file.
///
/// # Returns
///
/// This function returns a `Result` that indicates whether the operation was successful.
/// If the operation was successful, the function returns `Ok(())`.
/// If the operation failed, the function returns `Err` with a description of the error.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The downloaded file could not be opened.
/// * The file could not be decompressed or extracted.
/// * The `nvim-osx64` directory could not be renamed.
/// * The permissions of the `nvim` binary could not be set.
///
/// # Example
///
/// ```rust
/// let downloaded_file = LocalVersion {
///     file_name: "nvim-linux",
///     file_format: "tar.gz",
///     semver: semver::Version::parse("0.5.0").unwrap(),
///     path: "/path/to/downloaded/file",
/// };
/// expand(downloaded_file);
/// ```
#[cfg(unix)]
fn expand(downloaded_file: LocalVersion) -> Result<()> {
    use flate2::read::GzDecoder;
    use indicatif::{ProgressBar, ProgressStyle};
    use std::cmp::min;
    use std::fs::File;
    use std::io;
    use std::{os::unix::fs::PermissionsExt, path::PathBuf};
    use tar::Archive;

    if fs::metadata(&downloaded_file.file_name).is_ok() {
        fs::remove_dir_all(&downloaded_file.file_name)?;
    }

    let file = match File::open(format!(
        "{}.{}",
        downloaded_file.file_name, downloaded_file.file_format
    )) {
        Ok(value) => value,
        Err(error) => {
            return Err(anyhow!(
                "Failed to open file {}.{}, file doesn't exist. additional info: {error}",
                downloaded_file.file_name,
                downloaded_file.file_format
            ));
        }
    };
    let decompress_stream = GzDecoder::new(file);
    let mut archive = Archive::new(decompress_stream);

    let totalsize = 1692; // hard coding this is pretty unwise, but you cant get the length of an archive in tar-rs unlike zip-rs
    let pb = ProgressBar::new(totalsize);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}",
        )
        .unwrap()
        .progress_chars("█  "),
    );
    pb.set_message("Expanding archive");

    let mut downloaded: u64 = 0;
    for file in archive.entries()? {
        match file {
            Ok(mut file) => {
                let mut outpath = PathBuf::new();
                outpath.push(&downloaded_file.file_name);
                let no_parent_file = remove_base_parent(&file.path().unwrap()).unwrap();
                outpath.push(no_parent_file);

                let file_name = format!("{}", file.path()?.display()); // file.path()?.is_dir() always returns false... weird
                if file_name.ends_with('/') {
                    fs::create_dir_all(outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }
                    let mut outfile = fs::File::create(outpath)?;
                    io::copy(&mut file, &mut outfile)?;
                }
                let new = min(downloaded + 1, totalsize);
                downloaded = new;
                pb.set_position(new);
            }
            Err(error) => println!("{error}"),
        }
    }
    pb.finish_with_message(format!(
        "Finished expanding to {}/{}",
        downloaded_file.path, downloaded_file.file_name
    ));

    let file = &format!("{}/bin/nvim", downloaded_file.file_name);
    let mut perms = fs::metadata(file)?.permissions();
    perms.set_mode(0o551);
    fs::set_permissions(file, perms)?;
    Ok(())
}

/// Removes the base parent from a given path.
///
/// This function takes a path and removes its base parent component. For example, on Windows,
/// if the path is "D:\\test.txt", this function will return "test.txt", effectively removing
/// the drive letter and the root directory.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` from which the base parent will be removed.
///
/// # Returns
///
/// This function returns an `Option<PathBuf>`. If the path has a base parent that can be
/// removed, it returns `Some(PathBuf)` with the modified path. If the path does not have
/// a base parent or cannot be modified, it may return `None`, although in the current
/// implementation, it always returns `Some(PathBuf)` even if the path is unchanged.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use std::path::{Path, PathBuf};
/// use your_crate_name::remove_base_parent; // Adjust the use path according to your crate's structure
///
/// let path = Path::new("D:\\test.txt");
/// let new_path = remove_base_parent(path).unwrap();
/// assert_eq!(new_path, PathBuf::from("test.txt"));
/// ```
#[allow(dead_code)]
fn remove_base_parent(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();

    components.next();

    Some(components.as_path().to_path_buf())
}
