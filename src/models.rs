use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Serialize, Deserialize)]
pub struct StableVersion {
    pub tag_name: String,
}

#[derive(Clone)]
pub struct DownloadedVersion {
    pub file_name: String,
    pub file_format: String,
    pub path: String,
}
