use std::{fs, path::PathBuf};

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use yansi::Paint;

use crate::{
    config::Config,
    github_requests::{deserialize_response, make_github_request},
    helpers::{self, directories},
};

pub async fn start(config: Config, client: Client) -> Result<()> {
    let downloads_dir = directories::get_downloads_directory(&config).await?;
    let response = make_github_request(
        &client,
        "https://api.github.com/repos/neovim/neovim/tags?per_page=50",
    )
    .await?;

    let mut local_versions: Vec<PathBuf> = fs::read_dir(downloads_dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains('v')
        })
        .map(|entry| entry.path())
        .collect();

    let versions: Vec<RemoteVersion> = deserialize_response(response)?;
    let filtered_versions: Vec<RemoteVersion> = versions
        .into_iter()
        .filter(|v| v.name.starts_with('v'))
        .collect();

    for version in filtered_versions {
        let version_installed = local_versions.iter().any(|v| {
            v.file_name()
                .and_then(|str| str.to_str())
                .map_or(false, |str| str.contains(&version.name))
        });

        if helpers::version::is_version_used(&version.name, &config).await {
            println!("{}", Paint::green(version.name));
        } else if version_installed {
            println!("{}", Paint::yellow(&version.name));

            local_versions.retain(|v| {
                v.file_name()
                    .and_then(|str| str.to_str())
                    .map_or(true, |str| !str.contains(&version.name))
            });
        } else {
            println!("{}", version.name);
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct RemoteVersion {
    pub name: String,
}
