pub enum InstallResult {
    InstallationSuccess(String),
    VersionAlreadyInstalled,
    NightlyIsUpdated,
}

pub enum VersionType {
    Version(String),
    Hash(String),
}
