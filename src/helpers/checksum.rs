use anyhow::Result;
use sha2::{Digest, Sha256};
use tracing::info;
use std::path::Path;
use std::{fs, io};

/// Checks whether the checksum of the file at path 'a' matches the checksum saved in the file at path 'b'.
/// # Arguments
///
/// * `a` - A reference to a `&Path` object representing the path of the neovim archive.
/// * `b` - A reference to a `&Path` object representing the path of the checksum file.
///
/// # Returns
///
/// This function returns a `Result` that contains a `bool` indicating whether the checksum of the file at path 'a' matches the checksum saved in the file at path 'b'.
/// If there is an error opening or reading the files, the function returns `Err(error)`.
pub fn sha256cmp(a: &Path, b: &Path) -> Result<bool> {
    info!("Checking checksum of file at path: {:?}", a);
    let checksum = fs::read_to_string(b)?;
    let checksum = checksum.split(' ').next().unwrap();

    let mut hasher = Sha256::new();
    let mut file = fs::File::open(a)?;
    io::copy(&mut file, &mut hasher)?;
    info!("Checksum of file at path {:?} is: {:?}", a, hasher);

    let hash = hasher.finalize();
    let hash = format!("{:x}", hash);

    info!("Checksum of file at path {:?} is: {:?}", a, hash);

    Ok(hash == checksum)
}
