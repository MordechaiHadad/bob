use anyhow::Result;
use anyhow::anyhow;
use sha2::{Digest, Sha256};
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
pub fn sha256cmp(a: &Path, b: &Path, filename: &str) -> Result<bool> {
    let checksum_contents = fs::read_to_string(b)?;
    let checksum = checksum_contents
        .lines()
        .find(|line| line.contains(filename))
        .and_then(|line| line.split_whitespace().next())
        .ok_or_else(|| anyhow!("Checksum not found for {}", filename))?;

    let mut hasher = Sha256::new();
    let mut file = fs::File::open(a)?;
    io::copy(&mut file, &mut hasher)?;

    let hash = hasher.finalize();
    let hash = format!("{hash:x}");

    Ok(hash == checksum)
}
