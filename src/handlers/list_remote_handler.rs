use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::Result;
use yansi::Paint;

use crate::{
    config::Config,
    github_requests::GitHubClient,
    helpers::{self, directories},
};

pub async fn start(config: Config, github: &GitHubClient) -> Result<()> {
    let downloads_dir = directories::get_downloads_directory(&config).await?;
    let versions = github.get_tags().await?;

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

    let filtered_versions: Vec<_> = versions
        .into_iter()
        .filter(|v| v.name.starts_with('v'))
        .collect();

    let stable_version = github.get_latest_release().await?;

    let mut buffer = Vec::with_capacity(1024);

    for version in filtered_versions {
        let version_installed = local_versions.iter().any(|v| {
            v.file_name()
                .and_then(|str| str.to_str())
                .is_some_and(|str| str.contains(&version.name))
        });

        let stable_version_string = if stable_version.tag_name == version.name {
            " (stable)"
        } else {
            ""
        };

        let write_result = if helpers::version::is_version_used(&version.name, &config).await {
            writeln!(
                buffer,
                "{}{}",
                Paint::green(&version.name),
                stable_version_string
            )
        } else if version_installed {
            writeln!(
                buffer,
                "{}{}",
                Paint::yellow(&version.name),
                stable_version_string
            )
        } else {
            writeln!(buffer, "{}{}", version.name, stable_version_string)
        };

        if let Err(e) = write_result {
            if e.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(e.into());
        }

        if version_installed {
            local_versions.retain(|v| {
                v.file_name()
                    .and_then(|str| str.to_str())
                    .is_none_or(|str| !str.contains(&version.name))
            });
        }
    }

    let mut stdout = io::stdout().lock();
    stdout.write_all(&buffer).map_err(|e| {
        if e.kind() == io::ErrorKind::BrokenPipe {
            return anyhow::anyhow!("Failed to write to stdout: Broken pipe");
        }
        e.into()
    })?;

    stdout.flush()?;

    Ok(())
}
