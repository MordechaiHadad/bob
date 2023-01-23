use super::{use_handler, utils};
use crate::models::{Config, LocalNightly, Nightly};
use anyhow::Result;
use chrono::{Utc, Duration};
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use regex::Regex;
use tokio::fs;
use tracing::info;

pub async fn start(config: Config) -> Result<()> {
    let nightly_vec = produce_nightly_vec(&config).await?;

    let mut name_list: Vec<String> = Vec::new();

    for entry in &nightly_vec {
        name_list.push(
            entry
                .path
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap(),
        );
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose which rollback to use (Newest to Oldest):\n")
        .items(&name_list)
        .default(0)
        .interact_on_opt(&Term::stderr())?;

    match selection {
        Some(i) => {
            use_handler::switch(
                &config,
                &crate::models::InputVersion {
                    tag_name: name_list[i].clone(),
                    version_type: crate::enums::VersionType::Hash,
                },
                true,
            )
            .await?;

            let find = nightly_vec
                .iter()
                .find(|item| {
                    item.path
                        .file_name()
                        .unwrap()
                        .to_os_string()
                        .into_string()
                        .unwrap()
                        .contains(&name_list[i])
                })
                .unwrap();

            let now = Utc::now();
            let since = now.signed_duration_since(find.data.published_at);
            info!(
                "Rolled back to version {} {}",
                name_list[i], since
            );
        }
        None => info!("Rollback aborted..."),
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
