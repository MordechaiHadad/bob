use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub struct DownloadedFile {
    pub name: String,
    pub extension: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct StableVersion {
    pub tag_name: String,
}
