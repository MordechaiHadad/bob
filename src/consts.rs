use regex::Regex;
use std::sync::LazyLock;

/// Version regex to match semantic versioning format.
///
/// # Example
///
/// ```rust
/// let var = "1.2.3";
/// assert!(VERSION_REGEX.is_match(var));
/// ```
pub static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^v?([0-9]+(\.)?){1,3}").expect("Failed to compile static VERSION_REGEX")
});

// pub static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
//     Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+$").expect("Failed to compile static VERSION_REGEX")
// });

/// Hash regex to match SHA-1 or SHA-256 hashes.
///
/// # Example
/// ```rust
/// let var = "abcdef1234567890abcdef1234567890abcdef12";
/// assert_eq!(HASH_REGEX.is_match(var), true);
///
/// ```
pub static HASH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[0-9a-f]{5,40}\b").expect("Failed to compile static HASH_REGEX")
});

/// Rollback regex to match nightly versions with a specific format.
///
/// # Example
/// ```rust
/// let var = "nightly-abcdefg";
/// assert!(ROLLBACK_REGEX.is_match(var));
/// ```
pub static ROLLBACK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"nightly-[a-zA-Z0-9]{7,8}").expect("Failed to compile static ROLLBACK_REGEX")
});

/// Nightly regex to match nightly versions with a specific format.
///
/// # Example
///
/// ```rust
/// assert!(NIGHTLY_REGEX.is_match("nightly-abcdefg"));
/// ```
pub static NIGHTLY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"nightly-[a-zA-Z0-9]{7,8}").expect("Failed to compile static NIGHTLY_REGEX")
});

/// Environment variable regex to match environment variables in the format `$VAR_NAME`.
/// Used to match user configuration variables and substitute them with their actual values
/// from the host environment.
///
/// # Example
///
/// ```rust
/// let var = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
/// assert!(ENVIRONMENT_VAR_REGEX.is_match(&format!("$HOME={}", var)));
/// ```
pub static ENVIRONMENT_VAR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$([A-Z_]+)").expect("Failed to compile static ENVIRONMENT_VAR_REGEX")
});

/// # Unix platform-specific compile time constant for the filetype extension of the Neovim binary extension.
///
/// For Windows, it returns "zip".
/// For unix, it returns "tar.gz".
///
/// # Example
///
/// ```rust
/// #[cfg(target_family = "unix")]
/// {
///   let filetype_ext = FILETYPE_EXT;
///   assert_eq!(filetype_ext, "tar.gz");
/// }
///
/// #[cfg(target_family = "windows")]
/// {
///   let filetype_ext = FILETYPE_EXT;
///   assert_eq!(filetype_ext, "zip");
/// }
///
/// ```
#[cfg(target_family = "unix")]
pub const FILETYPE_EXT: &str = "tar.gz";

/// # Windows platform-specific compile time constant for the filetype extension of the Neovim binary extension.
///
/// For Windows, it returns "zip".
/// For unix, it returns "tar.gz".
///
/// # Example
///
/// ```rust
/// #[cfg(target_family = "unix")]
/// {
///   let filetype_ext = FILETYPE_EXT;
///   assert_eq!(filetype_ext, "tar.gz");
/// }
///
/// #[cfg(target_family = "windows")]
/// {
///   let filetype_ext = FILETYPE_EXT;
///   assert_eq!(filetype_ext, "zip");
/// }
///
/// ```
#[cfg(target_family = "windows")]
pub const FILETYPE_EXT: &str = "zip";
