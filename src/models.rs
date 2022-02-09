use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct DownloadedFile {
    pub name: String,
    pub extension: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct StableVersion {
    pub tag_name: String,
}
