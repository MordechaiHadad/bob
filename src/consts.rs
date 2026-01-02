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
    Regex::new(r"^v?([0-9]+(\.)+){1,3}").expect("Failed to compile static VERSION_REGEX")
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

/// Fork regex to match fork installations in the format `owner/repo@ref`.
/// Where `owner` is the GitHub username, `repo` is the repository name, and `ref` is a branch or commit hash.
///
/// # Example
///
/// ```rust
/// assert!(FORK_REGEX.is_match("username/neovim@feature-branch"));
/// assert!(FORK_REGEX.is_match("username/neovim@abc1234"));
/// ```
pub static FORK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([a-zA-Z0-9_-]+)/([a-zA-Z0-9_-]+)@([^\s]+)$")
        .expect("Failed to compile static FORK_REGEX")
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

#[cfg(test)]
mod fork_regex_tests {
    use super::*;

    #[test]
    fn test_fork_regex_with_valid_formats() {
        let valid_cases = [
            "username/neovim@feature-branch",
            "user-name/repo-name@branch-name",
            "user_name/repo_name@branch_name",
            "user123/repo456@branch789",
            "username/neovim@abc1234",
            "username/neovim@v1.0.0",
            "username/neovim@refs/heads/main",
            "octo-cat/my-nvim@feature/new-thing",
        ];

        for case in valid_cases {
            assert!(
                FORK_REGEX.is_match(case),
                "Expected '{}' to match fork regex",
                case
            );
        }
    }

    #[test]
    fn test_fork_regex_with_invalid_formats() {
        let invalid_cases = [
            "username/neovim",            // Missing @ref
            "username@neovim",            // Wrong separator
            "neovim@branch",              // Missing repo
            "@branch",                    // Missing owner and repo
            "username/",                  // Missing repo and ref
            "/neovim@branch",             // Missing owner
            "username/@branch",           // Missing repo
            "user name/neovim@branch",    // Space in owner
            "username/neo vim@branch",    // Space in repo
            "username/neovim@branch asd", // Trailing @
            "",                           // Empty string
            "justtext",                   // No format
        ];

        for case in invalid_cases {
            assert!(
                !FORK_REGEX.is_match(case),
                "Expected '{}' to not match fork regex",
                case
            );
        }
    }

    #[test]
    fn test_fork_regex_captures_components() {
        let fork_string = "myuser/myrepo@mybranch";
        let captures = FORK_REGEX.captures(fork_string).unwrap();

        assert_eq!(captures.get(1).unwrap().as_str(), "myuser");
        assert_eq!(captures.get(2).unwrap().as_str(), "myrepo");
        assert_eq!(captures.get(3).unwrap().as_str(), "mybranch");
    }

    #[test]
    fn test_fork_regex_with_complex_ref() {
        let fork_string = "user/repo@refs/heads/feature/new-branch";
        assert!(FORK_REGEX.is_match(fork_string));

        let captures = FORK_REGEX.captures(fork_string).unwrap();
        assert_eq!(
            captures.get(3).unwrap().as_str(),
            "refs/heads/feature/new-branch"
        );
    }

    #[test]
    fn test_fork_regex_with_commit_hash_ref() {
        let fork_string = "user/repo@abc123def456";
        assert!(FORK_REGEX.is_match(fork_string));

        let captures = FORK_REGEX.captures(fork_string).unwrap();
        assert_eq!(captures.get(3).unwrap().as_str(), "abc123def456");
    }
}
