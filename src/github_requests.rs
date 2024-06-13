use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Represents the version of the upstream software in the GitHub API.
///
/// This struct contains the tag name of the version, the target commitish of the version, and the date and time the version was published.
///
/// # Fields
///
/// * `tag_name: String` - The tag name of the version.
/// * `target_commitish: Option<String>` - The target commitish of the version. This is optional and may be `None`.
/// * `published_at: DateTime<Utc>` - The date and time the version was published, represented as a `DateTime<Utc>` object.
///
/// # Example
///
/// ```rust
/// let upstream_version = UpstreamVersion {
///     tag_name: "v1.0.0".to_string(),
///     target_commitish: Some("abc123".to_string()),
///     published_at: Utc::now(),
/// };
/// println!("The tag name is {}", upstream_version.tag_name);
/// println!("The target commitish is {}", upstream_version.target_commitish.unwrap_or_default());
/// println!("The published date and time is {}", upstream_version.published_at);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpstreamVersion {
    pub tag_name: String,
    pub target_commitish: Option<String>,
    pub published_at: DateTime<Utc>,
}

/// Represents a repository commit in the GitHub API.
///
/// This struct contains the SHA of a commit and the commit details, as returned by the GitHub API.
///
/// # Fields
///
/// * `sha: String` - The SHA of the commit.
/// * `commit: Commit` - The details of the commit, represented as a `Commit` object.
///
/// # Example
///
/// ```rust
/// let commit_author = CommitAuthor {
///     name: "Alice".to_string(),
/// };
/// let commit = Commit {
///     author: commit_author,
///     message: "Initial commit".to_string(),
/// };
/// let repo_commit = RepoCommit {
///     sha: "abc123".to_string(),
///     commit: commit,
/// };
/// println!("The commit SHA is {}", repo_commit.sha);
/// println!("The commit author is {}", repo_commit.commit.author.name);
/// println!("The commit message is {}", repo_commit.commit.message);
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct RepoCommit {
    pub sha: String,
    pub commit: Commit,
}

/// Represents a commit in the GitHub API.
///
/// This struct contains the author of a commit and the commit message, as returned by the GitHub API.
///
/// # Fields
///
/// * `author: CommitAuthor` - The author of the commit, represented as a `CommitAuthor` object.
/// * `message: String` - The commit message.
///
/// # Example
///
/// ```rust
/// let commit_author = CommitAuthor {
///     name: "Alice".to_string(),
/// };
/// let commit = Commit {
///     author: commit_author,
///     message: "Initial commit".to_string(),
/// };
/// println!("The commit author is {}", commit.author.name);
/// println!("The commit message is {}", commit.message);
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct Commit {
    pub author: CommitAuthor,
    pub message: String,
}

/// Represents the author of a commit in the GitHub API.
///
/// This struct contains the name of the author of a commit, as returned by the GitHub API.
///
/// # Fields
///
/// * `name: String` - The name of the author of the commit.
///
/// # Example
///
/// ```rust
/// let commit_author = CommitAuthor {
///     name: "Alice".to_string(),
/// };
/// println!("The commit author is {}", commit_author.name);
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct CommitAuthor {
    pub name: String,
}

/// Represents an error response from the GitHub API.
///
/// This struct contains information about an error response from the GitHub API, including the error message and the URL of the documentation related to the error.
///
/// # Fields
///
/// * `message: String` - The error message from the GitHub API.
/// * `documentation_url: String` - The URL of the documentation related to the error.
///
/// # Example
///
/// ```rust
/// let error_response = ErrorResponse {
///     message: "Not Found".to_string(),
///     documentation_url: "https://docs.github.com/rest".to_string(),
/// };
/// println!("The error message is {}", error_response.message);
/// println!("The documentation URL is {}", error_response.documentation_url);
/// ```
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

/// Fetches the commits for the nightly version from the GitHub API.
///
/// This function sends a GET request to the GitHub API to fetch the commits for the nightly version of the software. The commits are fetched for a specified time range, from `since` to `until`.
///
/// # Parameters
///
/// * `client: &Client` - The HTTP client used to send the request.
/// * `since: &DateTime<Utc>` - The start of the time range for which to fetch the commits.
/// * `until: &DateTime<Utc>` - The end of the time range for which to fetch the commits.
///
/// # Returns
///
/// * `Result<Vec<RepoCommit>>` - A vector of `RepoCommit` objects representing the commits for the nightly version, or an error if the request failed.
///
/// # Example
///
/// ```rust
/// let client = Client::new();
/// let since = Utc::now() - Duration::days(1);
/// let until = Utc::now();
/// let result = get_commits_for_nightly(&client, &since, &until).await;
/// match result {
///     Ok(commits) => println!("Received {} commits", commits.len()),
///     Err(e) => println!("An error occurred: {:?}", e),
/// }
/// ```
pub async fn get_commits_for_nightly(
    client: &Client,
    since: &DateTime<Utc>,
    until: &DateTime<Utc>,
) -> Result<Vec<RepoCommit>> {
    let response = make_github_request(client, format!(
            "https://api.github.com/repos/neovim/neovim/commits?since={since}&until={until}&per_page=100")).await?;

    deserialize_response(response)
}

/// Deserializes a JSON response from the GitHub API.
///
/// This function takes a JSON response as a string and attempts to deserialize it into a specified type `T`. If the response contains a "message" field, it is treated as an error response and the function will return an error with the message from the response. If the error is related to rate limiting, a specific error message is returned.
///
/// # Parameters
///
/// * `response: String` - The JSON response from the GitHub API as a string.
///
/// # Returns
///
/// * `Result<T>` - The deserialized response as the specified type `T`, or an error if the response could not be deserialized or contains an error message.
///
/// # Errors
///
/// This function will return an error if the response contains a "message" field (indicating an error from the GitHub API), or if the response could not be deserialized into the specified type `T`.
///
/// # Example
///
/// ```rust
/// let response = "{\"data\": \"some data\"}";
/// let result: Result<MyType> = deserialize_response(response);
/// match result {
///     Ok(data) => println!("Received data: {:?}", data),
///     Err(e) => println!("An error occurred: {:?}", e),
/// }
/// ```
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
