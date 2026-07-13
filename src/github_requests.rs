use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use octocrab::Octocrab;
use octocrab::models::repos::{Release, Tag};

use serde::{Deserialize, Serialize};

pub struct GitHubClient {
    octocrab: Octocrab,
    download: reqwest::Client,
}

impl GitHubClient {
    pub fn new() -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").ok();

        let mut builder = Octocrab::builder();
        if let Some(token) = token {
            builder = builder.personal_token(token);
        }

        let octocrab = builder.build()?;
        let download = reqwest::Client::builder().user_agent("bob").build()?;

        Ok(Self { octocrab, download })
    }

    pub fn download(&self) -> &reqwest::Client {
        &self.download
    }

    pub async fn get_release_by_tag(&self, tag: &str) -> Result<Release> {
        Ok(self
            .octocrab
            .repos("neovim", "neovim")
            .releases()
            .get_by_tag(tag)
            .await?)
    }

    pub async fn get_nightly_release(&self) -> Result<NightlyInfo> {
        self.get_release_by_tag("nightly").await?.try_into()
    }

    pub async fn get_latest_release(&self) -> Result<Release> {
        Ok(self
            .octocrab
            .repos("neovim", "neovim")
            .releases()
            .get_latest()
            .await?)
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
            .await?;

        Ok(page.items)
    }

    pub async fn get_latest_commit_sha(&self) -> Result<String> {
        let page = self
            .octocrab
            .repos("neovim", "neovim")
            .list_commits()
            .per_page(1)
            .send()
            .await?;

        let commit = page
            .items
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No commits found"))?;

        Ok(commit.sha)
    }

    pub async fn get_tags(&self) -> Result<Vec<Tag>> {
        let page = self
            .octocrab
            .repos("neovim", "neovim")
            .list_tags()
            .per_page(100)
            .send()
            .await?;

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
    type Error = anyhow::Error;

    fn try_from(release: Release) -> Result<Self> {
        let tag_name = release.tag_name.clone();
        Ok(Self {
            tag_name: release.tag_name,
            target_commitish: Some(release.target_commitish),
            published_at: release
                .published_at
                .ok_or_else(|| anyhow!("Release {tag_name} has no published_at"))?,
        })
    }
}
