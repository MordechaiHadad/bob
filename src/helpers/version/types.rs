use chrono::{DateTime, Utc};

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub struct ParsedVersion {
    pub tag_name: String,
    pub version_type: VersionType,
    pub non_parsed_string: String
}

#[derive(PartialEq, Eq, Debug)]
pub enum VersionType {
    Normal,
    Latest,
    Nightly,
    Hash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpstreamVersion {
    pub tag_name: String,
    pub target_commitish: String,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LocalNightly {
    pub data: UpstreamVersion,
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct LocalVersion {
    pub file_name: String,
    pub file_format: String,
    pub path: String,
}
