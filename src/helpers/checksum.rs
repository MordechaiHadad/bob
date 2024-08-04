use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::{fs, io};

/// Checks whether the checksum of the file at path 'a' matches the checksum saved in the file at path 'b'.
/// # Arguments
///
/// * `a` - A reference to a `PathBuf` object representing the path of the neovim executable.
/// * `b` - A reference to a `PathBuf` object representing the path of the checksum file.
///
/// # Returns
///
/// This function returns a `Result` that contains a `bool` indicating whether the checksum of the file at path 'a' matches the checksum saved in the file at path 'b'.
/// If there is an error opening or reading the files, the function returns `Err(error)`.
pub fn sha256cmp(a: &PathBuf, b: &PathBuf) -> Result<bool> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(a)?;
    io::copy(&mut file, &mut hasher)?;

    let hash: [u8; 32] = hasher
        .finalize()
        .as_slice()
        .try_into()
        .expect("slice with incorrect length");

    let checksum = fs::read_to_string(b)?;
    let checksum = checksum
        .split(' ')
        .next()
        .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");

    let mut decoded = [0; 32];
    hex::decode_to_slice(checksum, &mut decoded)?;

    Ok(hash == decoded)
}
