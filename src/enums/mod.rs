use crate::models::LocalVersion;

pub enum InstallResult {
    InstallationSuccess(String),
    VersionAlreadyInstalled,
    NightlyIsUpdated,
}

pub enum VersionType {
    Standard,
    Hash,
}

pub enum PostDownloadVersionType {
    Standard(LocalVersion),
    Hash,
}
