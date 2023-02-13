use super::enums::VersionType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Nightly {
    pub tag_name: String,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LocalNightly {
    pub data: Nightly,
    pub path: PathBuf,
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
    pub downloads_location: Option<String>,
    pub installation_location: Option<String>,
    pub version_sync_file_location: Option<String>,
    pub rollback_limit: Option<u8>,
}

pub struct InputVersion {
    pub tag_name: String,
    pub version_type: VersionType,
}
