use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpstreamVersion {
    pub tag_name: String,
    pub target_commitish: Option<String>,
    pub published_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RepoCommit {
    pub sha: String,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub message: String,
    pub documentation_url: String,
}

pub async fn make_github_request<T: AsRef<str> + reqwest::IntoUrl>(
    client: &Client,
    url: T,
) -> Result<String> {
    let response = client
        .get(url)
        .header("user-agent", "bob")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}

pub async fn get_upstream_nightly(client: &Client) -> Result<UpstreamVersion> {
    let response = make_github_request(
        client,
        "https://api.github.com/repos/neovim/neovim/releases/tags/nightly",
    )
    .await?;

    deserialize_response(response)
}

pub async fn get_commits_for_nightly(
    client: &Client,
    since: &DateTime<Utc>,
    until: &DateTime<Utc>,
) -> Result<Vec<RepoCommit>> {
    let response = make_github_request(client, format!(
            "https://api.github.com/repos/neovim/neovim/commits?since={since}&until={until}&per_page=100")).await?;

    deserialize_response(response)
}

pub fn deserialize_response<T: DeserializeOwned>(response: String) -> Result<T> {
    let value: serde_json::Value = serde_json::from_str(&response)?;

    if value.get("message").is_some() {
        let result: ErrorResponse = serde_json::from_value(value)?;

        if result.documentation_url.contains("rate-limiting") {
            return Err(anyhow!("Github API rate limit has been reach, either wait an hour or checkout https://github.com/MordechaiHadad/bob#increasing-github-rate-limit"));
        }

        return Err(anyhow!(result.message));
    }

    Ok(serde_json::from_value(value)?)
}
