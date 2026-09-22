use eyre::{Result, bail};
use tokio::fs;

use crate::helpers::version::types::LocalNightly;
use crate::{config::Config, github_requests::NightlyInfo, helpers::directories};

pub async fn get_local_nightly(config: &Config) -> Result<NightlyInfo> {
    let downloads_dir = directories::get_downloads_directory(config).await?;
    if let Ok(file) =
        fs::read_to_string(format!("{}/nightly/bob.json", downloads_dir.display())).await
    {
        let file_json: NightlyInfo = serde_json::from_str(&file)?;
        Ok(file_json)
    } else {
        bail!("Couldn't find bob.json")
    }
}

pub async fn produce_nightly_vec(config: &Config) -> Result<Vec<LocalNightly>> {
    let downloads_dir = directories::get_downloads_directory(config).await?;
    let mut paths = fs::read_dir(&downloads_dir).await?;

    let mut nightly_vec: Vec<LocalNightly> = Vec::new();

    while let Some(path) = paths.next_entry().await? {
        let name = path.file_name().into_string().unwrap();

        if !crate::NIGHTLY_REGEX.is_match(&name) {
            continue;
        }

        let nightly_content = path.path().join("bob.json");
        let nightly_string = fs::read_to_string(nightly_content).await?;

        let mut nightly_data: NightlyInfo = serde_json::from_str(&nightly_string)?;

        nightly_data.tag_name = name;

        nightly_vec.push(LocalNightly {
            data: nightly_data,
            path: path.path(),
        });
    }

    nightly_vec.sort_by_key(|b| std::cmp::Reverse(b.data.published_at));

    Ok(nightly_vec)
}
