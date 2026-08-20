use chrono::{DateTime, Utc};
use eyre::{Result, eyre};
use octocrab::Octocrab;
use octocrab::models::repos::{Release, Tag};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::debug;

trait OctocrabResultExt<T> {
    fn map_rate_limit(self) -> Result<T>;
}

impl<T> OctocrabResultExt<T> for octocrab::Result<T> {
    fn map_rate_limit(self) -> Result<T> {
        match self {
            Err(octocrab::Error::GitHub { source, .. })
                if source.status_code == StatusCode::FORBIDDEN
                    || source.status_code == StatusCode::TOO_MANY_REQUESTS =>
            {
                Err(eyre!(
                    "GitHub API rate limit reached. Either wait an hour or \
                     see https://github.com/MordechaiHadad/bob#increasing-github-rate-limit",
                ))
            }
            result => Ok(result?),
        }
    }
}

pub struct GitHubClient {
    octocrab: Octocrab,
    download: reqwest::Client,
}

impl GitHubClient {
    pub fn new() -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.is_empty());

        let mut builder = Octocrab::builder();
        if let Some(token) = token {
            builder = builder.personal_token(token);
        } else {
            debug!(
                "GITHUB_TOKEN not set -- unauthenticated requests are rate-limited to 60/hour, set GITHUB_TOKEN in your environment to increase to 5,000/hour"
            );
        }

        let octocrab = builder.build()?;
        let download = reqwest::Client::builder().user_agent("bob").build()?;

        Ok(Self { octocrab, download })
    }

    pub fn download(&self) -> &reqwest::Client {
        &self.download
    }

    pub async fn get_release_by_tag(&self, tag: &str) -> Result<Release> {
        self.octocrab
            .repos("neovim", "neovim")
            .releases()
            .get_by_tag(tag)
            .await
            .map_rate_limit()
    }

    pub async fn get_nightly_release(&self) -> Result<NightlyInfo> {
        self.get_release_by_tag("nightly").await?.try_into()
    }

    pub async fn get_latest_release(&self) -> Result<Release> {
        self.octocrab
            .repos("neovim", "neovim")
            .releases()
            .get_latest()
            .await
            .map_rate_limit()
    }

    pub async fn get_commits_between(
        &self,
        since: &DateTime<Utc>,
        until: &DateTime<Utc>,
    ) -> Result<Vec<octocrab::models::repos::RepoCommit>> {
        let page = self
            .octocrab
            .repos("neovim", "neovim")
            .list_commits()
            .since(*since)
            .until(*until)
            .per_page(100)
            .send()
            .await
            .map_rate_limit()?;

        Ok(page.items)
    }

    pub async fn get_latest_commit_sha(&self) -> Result<String> {
        let page = self
            .octocrab
            .repos("neovim", "neovim")
            .list_commits()
            .per_page(1)
            .send()
            .await
            .map_rate_limit()?;

        let commit = page
            .items
            .into_iter()
            .next()
            .ok_or_else(|| eyre!("No commits found"))?;

        Ok(commit.sha)
    }

    pub async fn get_tags(&self) -> Result<Vec<Tag>> {
        let page = self
            .octocrab
            .repos("neovim", "neovim")
            .list_tags()
            .per_page(100)
            .send()
            .await
            .map_rate_limit()?;

        Ok(page.items)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NightlyInfo {
    pub tag_name: String,
    pub target_commitish: Option<String>,
    pub published_at: DateTime<Utc>,
}

impl TryFrom<Release> for NightlyInfo {
    type Error = eyre::Report;

    fn try_from(release: Release) -> Result<Self> {
        let published_at = release
            .published_at
            .ok_or_else(|| eyre!("Release {} has no published_at", release.tag_name))?;
        Ok(Self {
            tag_name: release.tag_name,
            target_commitish: Some(release.target_commitish),
            published_at,
        })
    }
}
