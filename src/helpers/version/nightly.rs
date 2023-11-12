use anyhow::{anyhow, Result};
use regex::Regex;
use tokio::fs;

use super::types::LocalNightly;
use crate::{config::Config, helpers::directories, github_requests::UpstreamVersion};


pub async fn get_local_nightly(config: &Config) -> Result<UpstreamVersion> {
    let downloads_dir = directories::get_downloads_directory(config).await?;
    if let Ok(file) =
        fs::read_to_string(format!("{}/nightly/bob.json", downloads_dir.display())).await
    {
        let file_json: UpstreamVersion = serde_json::from_str(&file)?;
        Ok(file_json)
    } else {
        Err(anyhow!("Couldn't find bob.json"))
    }
}

pub async fn produce_nightly_vec(config: &Config) -> Result<Vec<LocalNightly>> {
    let downloads_dir = directories::get_downloads_directory(config).await?;
    let mut paths = fs::read_dir(&downloads_dir).await?;

    let regex = Regex::new(r"nightly-[a-zA-Z0-9]{8}")?;

    let mut nightly_vec: Vec<LocalNightly> = Vec::new();

    while let Some(path) = paths.next_entry().await? {
        let name = path.file_name().into_string().unwrap();

        if !regex.is_match(&name) {
            continue;
        }

        let nightly_content = path.path().join("bob.json");
        let nightly_string = fs::read_to_string(nightly_content).await?;

        let nightly_data: UpstreamVersion = serde_json::from_str(&nightly_string)?;

        let mut nightly_entry = LocalNightly {
            data: nightly_data,
            path: path.path(),
        };

        nightly_entry.data.tag_name = name;

        nightly_vec.push(nightly_entry);
    }

    nightly_vec.sort_by(|a, b| b.data.published_at.cmp(&a.data.published_at));

    Ok(nightly_vec)
}
