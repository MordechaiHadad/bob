use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub tag_name: String,
    pub published_at: String,
}

#[derive(Clone)]
pub struct DownloadedVersion {
    pub file_name: String,
    pub file_format: String,
    pub path: String,
}
