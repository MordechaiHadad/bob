pub enum InstallResult {
    InstallationSuccess(String),
    VersionAlreadyInstalled,
    NightlyIsUpdated,
}

pub enum VersionType {
    Standard,
    Hash,
}
