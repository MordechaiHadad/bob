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

pub enum InstallResult {
    InstallationSuccess(String),
    VersionAlreadyInstalled,
    NightlyIsUpdated,
    GivenNightlyRollback,
}

#[derive(PartialEq)]
pub enum PostDownloadVersionType {
    None,
    Standard(LocalVersion),
    Hash,
}
