use crate::helpers::version::nightly::produce_nightly_vec;
use anyhow::Result;
use chrono::{Duration, Utc};
use dialoguer::{Select, console::Term, theme::ColorfulTheme};
use tracing::info;

use crate::{
    config::Config,
    handlers::use_handler,
    helpers::{self, version::types::ParsedVersion},
};
use std::fmt::Write;

/// Starts the rollback process.
///
/// This function presents a list of available versions to the user, allows them to select a version to rollback to, and then performs the rollback.
///
/// # Arguments
///
/// * `config` - The configuration for the rollback process.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the rollback process was successful or not.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// start(config).await.unwrap();
/// ```
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

    if let Some(i) = selection {
        let is_version_used = helpers::version::is_version_used(&name_list[i], &config).await;

        if is_version_used {
            info!("{} is already used.", &name_list[i]);
            return Ok(());
        }

        use_handler::switch(
            &config,
            &ParsedVersion {
                tag_name: name_list[i].clone(),
                version_type: helpers::version::types::VersionType::Normal,
                non_parsed_string: String::default(),
                semver: None,
            },
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
        let humanized = humanize_duration(since);
        info!(
            "Successfully rolled back to version '{}' from {} ago",
            name_list[i], humanized
        );
    } else {
        info!("Rollback aborted...");
    }

    Ok(())
}

/// Converts a `Duration` into a human-readable string.
///
/// This function takes a `Duration` and converts it into a string that represents the duration in weeks, days, and hours.
///
/// # Arguments
///
/// * `duration` - The `Duration` to be converted.
///
/// # Returns
///
/// * `Result<String>` - Returns a `Result` that contains a string representing the duration in a human-readable format, or an error if there is a failure in the conversion process.
///
/// # Example
///
/// ```rust
/// let duration = Duration::hours(25);
/// let humanized_duration = humanize_duration(duration).unwrap();
/// assert_eq!(humanized_duration, "1 day, 1 hour");
/// ```
fn humanize_duration(duration: Duration) -> String {
    let mut humanized_duration = String::new();

    let total_hours = duration.num_hours();

    let weeks = total_hours / 24 / 7;
    let days = (total_hours / 24) % 7;
    let hours = total_hours % 24;

    let mut added_duration = false;

    if weeks != 0 {
        if added_duration {
            let _ = write!(humanized_duration, ",");
        }
        // humanized_duration += &format!("{} week{}", weeks, if weeks > 1 { "s" } else { "" });
        let _ = write!(
            humanized_duration,
            "{} week{}",
            weeks,
            if weeks > 1 { "s" } else { "" }
        );
        added_duration = true;
    }
    if days != 0 {
        if added_duration {
            let _ = write!(humanized_duration, ",");
        }
        if !humanized_duration.is_empty() {
            let _ = write!(humanized_duration, " ");
        }
        // humanized_duration += &format!("{} day{}", days, if days > 1 { "s" } else { "" });
        let _ = write!(
            humanized_duration,
            "{} day{}",
            days,
            if days > 1 { "s" } else { "" }
        );
        added_duration = true;
    }
    if hours != 0 {
        if added_duration {
            let _ = write!(humanized_duration, ",");
        }
        if !humanized_duration.is_empty() {
            let _ = write!(humanized_duration, " ");
        }
        let _ = write!(
            humanized_duration,
            "{} hour{}",
            hours,
            if hours > 1 { "s" } else { "" }
        );
    }

    humanized_duration
}
