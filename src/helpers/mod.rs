pub mod checksum;
pub mod directories;
pub mod filesystem;
pub mod processes;
pub mod sync;
pub mod unarchive;
pub mod version;
use semver::Version;

/// Returns the file type for the Neovim binary download based on the target operating system.
///
/// This function checks the target operating system using the `cfg!` macro and returns a string that corresponds to the appropriate file type for the Neovim binary download.
/// For Windows, it returns "zip".
/// For macOS, it returns "tar.gz".
/// For other operating systems, it returns "appimage".
///
/// # Returns
///
/// This function returns a `&'static str` that corresponds to the file type for the Neovim binary download.
///
/// # Example
///
/// ```rust
/// let file_type = get_file_type();
/// ```
pub fn get_file_type() -> &'static str {
    if cfg!(target_family = "windows") {
        "zip"
    } else if cfg!(target_os = "macos") {
        "tar.gz"
    } else {
        "appimage"
    }
}

/// Returns the platform-specific name for the Neovim binary.
///
/// This function takes an `Option<Version>` as an argument, which represents the version of Neovim.
/// It checks the target operating system and architecture using the `cfg!` macro and returns a string that corresponds to the appropriate Neovim binary for the platform.
/// For Windows, it returns "nvim-win64".
/// For macOS, it checks the version of Neovim. If the version is less than or equal to 0.9.5, it returns "nvim-macos". If the target architecture is "aarch64", it returns "nvim-macos-arm64". Otherwise, it returns "nvim-macos-x86_64".
/// For Linux, it checks the version of Neovim. If the version is less than or equal to 0.10.3, it returns "nvim-linux64". If the target architecture is "aarch64", it returns "nvim-linux-arm64". Otherwise, it returns "nvim-linux-x86_64".
///
/// # Arguments
///
/// * `version` - An `Option<Version>` representing the version of Neovim.
///
/// # Returns
///
/// This function returns a `&'static str` that corresponds to the platform-specific name for the Neovim binary.
///
/// # Example
///
/// ```rust
/// let version = Some(Version::new(0, 9, 5));
/// let platform_name = get_platform_name(&version);
/// ```
pub fn get_platform_name(version: &Option<Version>) -> &'static str {
    if cfg!(target_os = "windows") {
        "nvim-win64"
    } else if cfg!(target_os = "macos") {
        if version
            .as_ref()
            .is_some_and(|x| x <= &Version::new(0, 9, 5))
        {
            "nvim-macos"
        } else if cfg!(target_arch = "aarch64") {
            "nvim-macos-arm64"
        } else {
            "nvim-macos-x86_64"
        }
    } else if version
        .as_ref()
        .is_some_and(|x| x <= &Version::new(0, 10, 3))
    {
        "nvim-linux64"
    } else if cfg!(target_arch = "aarch64") {
        "nvim-linux-arm64"
    } else {
        "nvim-linux-x86_64"
    }
}

/// Returns the platform-specific name for the Neovim download.
///
/// This function takes an `Option<Version>` as an argument, which represents the version of Neovim to be downloaded.
/// It checks the target operating system and architecture using the `cfg!` macro and returns a string that corresponds to the appropriate Neovim download for the platform.
/// For Windows, it returns "nvim-win64".
/// For macOS, it checks the version of Neovim. If the version is less than or equal to 0.9.5, it returns "nvim-macos". If the target architecture is "aarch64", it returns "nvim-macos-arm64". Otherwise, it returns "nvim-macos-x86_64".
/// For Linux, it checks the version of Neovim. If the version is less than or equal to 0.10.3, it returns "nvim". If the target architecture is "aarch64", it returns "nvim-linux-arm64". Otherwise, it returns "nvim-linux-x86_64".
///
/// # Arguments
///
/// * `version` - An `Option<Version>` representing the version of Neovim to be downloaded.
///
/// # Returns
///
/// This function returns a `&'static str` that corresponds to the platform-specific name for the Neovim download.
///
/// # Example
///
/// ```rust
/// let version = Some(Version::new(0, 9, 5));
/// let platform_name = get_platform_name_download(&version);
/// ```
pub fn get_platform_name_download(version: &Option<Version>) -> &'static str {
    if cfg!(target_os = "windows") {
        "nvim-win64"
    } else if cfg!(target_os = "macos") {
        if version
            .as_ref()
            .is_some_and(|x| x <= &Version::new(0, 9, 5))
        {
            "nvim-macos"
        } else if cfg!(target_arch = "aarch64") {
            "nvim-macos-arm64"
        } else {
            "nvim-macos-x86_64"
        }
    } else if version
        .as_ref()
        .is_some_and(|x| x <= &Version::new(0, 10, 3))
    {
        "nvim"
    } else if cfg!(target_arch = "aarch64") {
        "nvim-linux-arm64"
    } else {
        "nvim-linux-x86_64"
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn get_platform_name_none() {
        if cfg!(target_os = "windows") {
            assert_eq!(super::get_platform_name(&None), "nvim-win64");
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            assert_eq!(super::get_platform_name(&None), "nvim-macos-arm64");
            assert_eq!(super::get_platform_name_download(&None), "nvim-macos-arm64");
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            assert_eq!(super::get_platform_name(&None), "nvim-macos-x86_64");
            assert_eq!(
                super::get_platform_name_download(&None),
                "nvim-macos-x86_64"
            );
        } else if cfg!(target_arch = "aarch64") {
            assert_eq!(super::get_platform_name(&None), "nvim-linux-arm64");
            assert_eq!(super::get_platform_name_download(&None), "nvim-linux-arm64");
        } else {
            assert_eq!(super::get_platform_name(&None), "nvim-linux-x86_64");
            assert_eq!(
                super::get_platform_name_download(&None),
                "nvim-linux-x86_64"
            );
        }
    }

    #[test]
    fn get_platform_name_lower() {
        let version = Some(semver::Version::new(0, 9, 5));
        if cfg!(target_os = "windows") {
            assert_eq!(super::get_platform_name(&version), "nvim-win64");
        } else if cfg!(target_os = "macos") {
            assert_eq!(super::get_platform_name(&version), "nvim-macos");
            assert_eq!(super::get_platform_name_download(&version), "nvim-macos");
        } else {
            assert_eq!(super::get_platform_name(&version), "nvim-linux64");
            assert_eq!(super::get_platform_name_download(&version), "nvim");
        }
    }

    #[test]
    fn get_platform_name_higher() {
        let version = Some(semver::Version::new(0, 10, 5));
        if cfg!(target_os = "windows") {
            assert_eq!(super::get_platform_name(&version), "nvim-win64");
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            assert_eq!(super::get_platform_name(&version), "nvim-macos-arm64");
            assert_eq!(
                super::get_platform_name_download(&version),
                "nvim-macos-arm64"
            );
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            assert_eq!(super::get_platform_name(&version), "nvim-macos-x86_64");
            assert_eq!(
                super::get_platform_name_download(&version),
                "nvim-macos-x86_64"
            );
        } else if cfg!(target_arch = "aarch64") {
            assert_eq!(super::get_platform_name(&version), "nvim-linux-arm64");
            assert_eq!(
                super::get_platform_name_download(&version),
                "nvim-linux-arm64"
            );
        } else {
            assert_eq!(super::get_platform_name(&version), "nvim-linux-x86_64");
            assert_eq!(
                super::get_platform_name_download(&version),
                "nvim-linux-x86_64"
            );
        }
    }
}
