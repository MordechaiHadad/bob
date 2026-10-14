use crate::{
    config::Config,
    github_requests::GitHubClient,
    helpers::{directories, version::get_current_version},
    version::parse_version_type,
};
use eyre::{Result, bail};

/// Prints the path of the installed Neovim executable for a version.
///
/// When no version is given, Bob's currently selected version is used.
pub async fn start(version: Option<&str>, github: &GitHubClient, config: &Config) -> Result<()> {
    let version = match version {
        Some(version) => version.to_owned(),
        None => get_current_version(config).await?,
    };
    let version = parse_version_type(github, version.trim()).await?;
    let downloads_dir = directories::get_downloads_directory(config).await?;

    let mut executable = downloads_dir
        .join(&version.tag_name)
        .join("bin")
        .join("nvim");
    if cfg!(target_family = "windows") {
        executable.set_extension("exe");
    }

    if !executable.is_file() {
        bail!(
            "Neovim executable for version {} was not found at {}",
            version.tag_name,
            executable.display()
        );
    }

    println!("{}", executable.display());
    Ok(())
}
