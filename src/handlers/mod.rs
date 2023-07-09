pub mod erase_handler;
pub mod install_handler;
pub mod list_handler;
pub mod rollback_handler;
pub mod sync_handler;
pub mod uninstall_handler;
pub mod update_handler;
pub mod use_handler;

use super::version::types::LocalVersion;

#[derive(PartialEq, Eq)]
pub enum InstallResult {
    InstallationSuccess(String),
    VersionAlreadyInstalled,
    NightlyIsUpdated,
}

pub enum PostDownloadVersionType {
    Standard(LocalVersion),
    Hash,
}
