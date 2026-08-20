use eyre::Result;
use tokio::process::Command;

use crate::config::Config;
use crate::github_requests::GitHubClient;
use crate::helpers;

/// Starts the process of running a specific version of Neovim with the provided arguments.
///
/// # Arguments
///
/// * `version` - The version to run (nightly|stable|<version-string>|<commit-hash>)
/// * `args` - Arguments to pass to Neovim
/// * `github` - The GitHub API client.
/// * `config` - The configuration for the operation
pub async fn start(
    version: &str,
    args: &[String],
    github: &GitHubClient,
    config: &Config,
) -> Result<()> {
    // Parse the specified version
    let version = crate::version::parse_version_type(github, version).await?;
    let downloads_dir = helpers::directories::get_downloads_directory(config).await?;
    let version_path = downloads_dir.join(&version.tag_name);

    // If not installed, suggest installing it first
    if !version_path.exists() {
        eyre::bail!(
            "Version {} is not installed. Install it first with: bob install {}",
            version.tag_name,
            version.tag_name
        );
    }

    // Use the specific version's binary (With OS specific extension)
    let bin_path = if cfg!(target_family = "windows") {
        version_path.join("bin").join("nvim").with_extension("exe")
    } else {
        version_path.join("bin").join("nvim")
    };

    if !bin_path.exists() {
        eyre::bail!(
            "Neovim binary not found at expected path: {}",
            bin_path.display()
        );
    }

    // Run the specific version with the provided args
    let mut cmd = Command::new(bin_path);
    cmd.args(args);
    helpers::processes::handle_subprocess(&mut cmd).await
}
