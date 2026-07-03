use anyhow::Result;
use anyhow::anyhow;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::{fs, io};

pub fn hash_file_hex(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    io::copy(&mut reader, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{hash:x}"))
}

/// Checks whether the checksum of the file at path 'a' matches the checksum saved in the file at path 'b'.
pub fn sha256cmp(a: &Path, b: &Path, filename: &str) -> Result<bool> {
    let checksum_contents = fs::read_to_string(b)?;
    let expected = checksum_contents
        .lines()
        .find(|line| line.contains(filename))
        .and_then(|line| line.split_whitespace().next())
        .ok_or_else(|| anyhow!("Checksum not found for {filename}"))?;

    let hash = hash_file_hex(a)?;
    Ok(hash == expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::io;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn hash_file_hex_same_content() {
        let mut f1 = NamedTempFile::new().unwrap();
        let mut f2 = NamedTempFile::new().unwrap();
        write!(f1, "hello world").unwrap();
        write!(f2, "hello world").unwrap();
        f1.flush().unwrap();
        f2.flush().unwrap();

        assert_eq!(
            hash_file_hex(f1.path()).unwrap(),
            hash_file_hex(f2.path()).unwrap()
        );
    }

    #[test]
    fn hash_file_hex_different_content() {
        let mut f1 = NamedTempFile::new().unwrap();
        let mut f2 = NamedTempFile::new().unwrap();
        write!(f1, "a").unwrap();
        write!(f2, "b").unwrap();
        f1.flush().unwrap();
        f2.flush().unwrap();

        assert_ne!(
            hash_file_hex(f1.path()).unwrap(),
            hash_file_hex(f2.path()).unwrap()
        );
    }

    #[test]
    fn sha256cmp_with_checksum_file() {
        // create a data file and write content
        let mut data = NamedTempFile::new().unwrap();
        write!(data, "payload").unwrap();
        data.flush().unwrap();

        // compute hex sha256 of the data file (so we can place it in the checksum file)
        let mut hasher = Sha256::new();
        let mut file = fs::File::open(data.path()).unwrap();
        io::copy(&mut file, &mut hasher).unwrap();
        let hex = format!("{:x}", hasher.finalize());

        // prepare a checksum file in the format: "<hex>  <filename>"
        // Use the real filename component so sha256cmp's search-by-filename will match.
        let filename = data.path().file_name().unwrap().to_str().unwrap();
        let mut checksum = NamedTempFile::new().unwrap();
        write!(checksum, "{}  {}", hex, filename).unwrap();
        checksum.flush().unwrap();

        // sha256cmp reads the checksum file and compares the computed digest of the data file
        assert!(sha256cmp(data.path(), checksum.path(), filename).unwrap());
    }
}
