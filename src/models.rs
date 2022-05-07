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

#[derive(Serialize, Deserialize, Debug)]
pub struct RepoCommit {
    pub commit: Commit
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Commit {
    pub author: CommitAuthor,
    pub message: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitAuthor {
    pub name: String
}
