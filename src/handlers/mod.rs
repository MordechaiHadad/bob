pub mod erase_handler;
pub mod install_handler;
pub mod list_handler;
pub mod list_remote_handler;
pub mod rollback_handler;
pub mod sync_handler;
pub mod uninstall_handler;
pub mod update_handler;
pub mod use_handler;

use super::version::types::LocalVersion;

/// Represents the result of an installation attempt.
///
/// This enum has four variants:
///
/// * `InstallationSuccess(String)` - The installation was successful.
/// * `ChecksumMismatch` - The given checksum does not match the checksum of the downloaded file.
/// * `VersionAlreadyInstalled` - The version that was attempted to be installed is already installed.
/// * `NightlyIsUpdated` - The nightly version is updated.
/// * `GivenNightlyRollback` - The given nightly version is a rollback.
pub enum InstallResult {
    InstallationSuccess(String),
    VersionAlreadyInstalled,
    NightlyIsUpdated,
    GivenNightlyRollback,
}

/// Represents the type of a version after it has been downloaded.
///
/// This enum has three variants:
///
/// * `None` - No specific version type is assigned.
/// * `Standard(LocalVersion)` - The version is a standard version. The `LocalVersion` contains the details of the version.
/// * `Hash` - The version is identified by a hash.
#[derive(PartialEq)]
pub enum PostDownloadVersionType {
    None,
    Standard(LocalVersion),
    Hash,
}
