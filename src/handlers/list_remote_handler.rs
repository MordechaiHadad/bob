use std::{fs, path::PathBuf};

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use yansi::Paint;

use crate::{
    config::Config,
    github_requests::{deserialize_response, make_github_request},
    helpers::{self, directories, version::search_stable_version},
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

    let length = filtered_versions.len();
    let mut counter = 0;
    let stable_version = search_stable_version(&client).await?;

    for version in filtered_versions {
        counter += 1;
        let version_installed = local_versions.iter().any(|v| {
            v.file_name()
                .and_then(|str| str.to_str())
                .map_or(false, |str| str.contains(&version.name))
        });

        let stable_version_string = if stable_version  == version.name {
            " (stable)"
        } else {
            ""
        };

        if helpers::version::is_version_used(&version.name, &config).await {
            println!("{}{}", Paint::green(version.name), stable_version_string);
        } else if version_installed {
            println!("{}{}", Paint::yellow(&version.name), stable_version_string);

            local_versions.retain(|v| {
                v.file_name()
                    .and_then(|str| str.to_str())
                    .map_or(true, |str| !str.contains(&version.name))
            });
        } else {
            println!("{}{}", version.name, stable_version_string);
        }

        if length != counter {
            println!("");
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct RemoteVersion {
    pub name: String,
}
