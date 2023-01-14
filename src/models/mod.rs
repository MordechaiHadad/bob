use super::enums::VersionType;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct UpstreamVersion {
    pub tag_name: String,
    pub published_at: String,
}

#[derive(Clone)]
pub struct LocalVersion {
    pub file_name: String,
    pub file_format: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RepoCommit {
    pub commit: Commit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Commit {
    pub author: CommitAuthor,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitAuthor {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub enable_nightly_info: Option<bool>,
    pub downloads_dir: Option<String>,
    pub installation_location: Option<String>,
    pub sync_version_file_path: Option<String>,
}

pub struct InputVersion {
    pub tag_name: String,
    pub version_type: VersionType,
}
