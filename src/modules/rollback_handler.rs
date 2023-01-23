use regex::Regex;
use tokio::fs;
use anyhow::Result;
use crate::models::{Config, LocalNightly, Nightly};
use super::utils;

pub async fn start(config: &Config) -> Result<()> {
    println!("Choose which rollback to use(Ordered from newest to oldest):\n");
    let nightly_vec = produce_nightly_vec(config).await?;
    for entry in nightly_vec {
        println!("{}", entry.path.file_name().unwrap().to_os_string().into_string().unwrap());
    }

    Ok(())
}

pub async fn produce_nightly_vec(config: &Config) -> Result<Vec<LocalNightly>> {
    let downloads_dir = utils::get_downloads_folder(config).await?;
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

        let nightly_data: Nightly = serde_json::from_str(&nightly_string)?;

        let nightly_entry = LocalNightly {
            data: nightly_data,
            path: path.path(),
        };

        nightly_vec.push(nightly_entry);
    }

    nightly_vec.sort_by(|a, b| a.data.published_at.cmp(&b.data.published_at));

    Ok(nightly_vec)
}
