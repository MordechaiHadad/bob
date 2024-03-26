use semver::Version;

use crate::github_requests::UpstreamVersion;
use std::path::PathBuf;

pub struct ParsedVersion {
    pub tag_name: String,
    pub version_type: VersionType,
    pub non_parsed_string: String,
    pub semver: Option<Version>
}

#[derive(PartialEq, Eq, Debug)]
pub enum VersionType {
    Normal,
    Latest,
    Nightly,
    Hash,
    NightlyRollback,
}

#[derive(Debug, Clone)]
pub struct LocalNightly {
    pub data: UpstreamVersion,
    pub path: PathBuf,
}

#[derive(Clone, PartialEq)]
pub struct LocalVersion {
    pub file_name: String,
    pub file_format: String,
    pub path: String,
}
