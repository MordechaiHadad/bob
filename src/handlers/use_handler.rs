use dialoguer::Confirm;
use eyre::{Result, bail, eyre};
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, trace};

use crate::config::{Config, ConfigFile};
use crate::github_requests::GitHubClient;
use crate::handlers::{InstallResult, install_handler};
use crate::helpers;
use crate::helpers::checksum::hash_file_hex;
use crate::helpers::directories::get_installation_directory;
use crate::helpers::version::types::{ParsedVersion, VersionType};

/// Starts the process of using a specified version.
///
/// Checks if the version is already used, copies the Neovim proxy,
/// installs the version if needed, switches to it, and cleans up.
///
/// # Arguments
///
/// * `version` - The version to use.
/// * `install` - Whether to install the version if not already installed.
/// * `github` - The GitHub API client.
/// * `config` - The configuration for the operation.
///
/// # Errors
///
/// Returns an error if installation, switch, or PATH modification fails.
pub async fn start(
    version: ParsedVersion,
    install: bool,
    github: &GitHubClient,
    config: ConfigFile,
) -> Result<()> {
    let is_version_used =
        helpers::version::is_version_used(&version.tag_name, &config.config).await;

    copy_nvim_proxy(&config).await?;
    if is_version_used && version.tag_name != "nightly" {
        info!("{} is already installed and used!", version.tag_name);
        return Ok(());
    }

    if install {
        match install_handler::start(&version, github, &config).await {
            Ok(success) => {
                if let InstallResult::NightlyIsUpdated = success {
                    if is_version_used {
                        info!("Nightly is already updated and used!");
                        return Ok(());
                    }
                }
            }
            Err(error) => return Err(error),
        }
    }

    switch(&config.config, &version).await?;

    if let VersionType::Latest = version.version_type {
        if fs::metadata("stable").await.is_ok() {
            fs::remove_dir_all("stable").await?;
        }
    }

    let installation_dir = get_installation_directory(&config.config).await?;

    add_to_path(installation_dir, config).await?;

    info!("You can now use {}!", version.tag_name);

    Ok(())
}

/// Switches to a specified version.
///
/// This function changes the current directory to the downloads directory, writes the version to a file named "used", and if the version is different from the version stored in `version_sync_file_location`, it also writes the version to `version_sync_file_location`.
///
/// # Arguments
///
/// * `config` - The configuration for the operation.
/// * `version` - The version to switch to.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the operation was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The downloads directory cannot be determined.
/// * The current directory cannot be changed to the downloads directory.
/// * The version cannot be written to the "used" file.
/// * The version cannot be read from `version_sync_file_location`.
/// * The version cannot be written to `version_sync_file_location`.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let version = ParsedVersion::new("1.0.0");
/// switch(&config, &version).await.unwrap();
/// ```
pub async fn switch(config: &Config, version: &ParsedVersion) -> Result<()> {
    std::env::set_current_dir(helpers::directories::get_downloads_directory(config).await?)?;

    let file_version: String = if version.version_type == VersionType::Hash {
        if version.non_parsed_string.len() <= 7 {
            let mut current_dir = env::current_dir()?;
            current_dir.push(&version.non_parsed_string);
            current_dir.push("full-hash.txt");
            let hash_result = fs::read_to_string(&current_dir).await;

            if let Ok(hash) = hash_result {
                hash
            } else {
                bail!("Full hash file doesn't exist, please rebuild this commit");
            }
        } else {
            version.non_parsed_string.clone()
        }
    } else {
        version.tag_name.clone()
    };

    fs::write("used", &file_version).await?;
    if let Some(version_sync_file_location) =
        helpers::version::get_version_sync_file_location(config).await?
    {
        // Write the used version to version_sync_file_location only if it's different
        let stored_version = fs::read_to_string(&version_sync_file_location).await?;
        if stored_version != version.non_parsed_string {
            fs::write(&version_sync_file_location, file_version).await?;
            info!(
                "Written version to {}",
                version_sync_file_location
                    .into_os_string()
                    .into_string()
                    .unwrap()
            );
        }
    }

    Ok(())
}

/// Copies the Neovim proxy to the installation directory.
///
/// This function gets the current executable's path, determines the installation directory, creates it if it doesn't exist, adds it to the system's PATH, and copies the current executable to the installation directory as "nvim" or "nvim.exe" (on Windows).
///
/// If a file named "nvim" or "nvim.exe" already exists in the installation directory, the function compare the checksum. If the checksum matches, the function does nothing. Otherwise, it replaces the file with the current executable.
///
/// # Arguments
///
/// * `config` - The configuration for the operation.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the operation was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The current executable's path cannot be determined.
/// * The installation directory cannot be created.
/// * The installation directory cannot be added to the PATH.
/// * The version of the existing file cannot be determined.
/// * The existing file cannot be replaced.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// copy_nvim_proxy(&config).await.unwrap();
/// ```
#[tracing::instrument(skip(config))]
async fn copy_nvim_proxy(config: &ConfigFile) -> Result<()> {
    let exe_path = env::current_exe().unwrap();
    trace!("copy_nvim_proxy: current_exe = {}", exe_path.display());
    let mut installation_dir =
        helpers::directories::get_installation_directory(&config.config).await?;

    trace!(
        "copy_nvim_proxy: installation_dir (pre-filename) = {}",
        installation_dir.display()
    );

    if fs::metadata(&installation_dir).await.is_err() {
        fs::create_dir_all(&installation_dir).await?;
    }

    if cfg!(windows) {
        installation_dir.push("nvim.exe");
    } else {
        installation_dir.push("nvim");
    }

    if fs::metadata(&installation_dir).await.is_ok()
        && hash_file_hex(&exe_path)? == hash_file_hex(&installation_dir)?
    {
        return Ok(());
    }

    info!("Updating neovim proxy");
    trace!(
        "copy_nvim_proxy: copying {} -> {}",
        exe_path.display(),
        installation_dir.display()
    );
    copy_file_with_error_handling(&exe_path, &installation_dir).await?;
    trace!("copy_nvim_proxy: copy completed successfully");

    Ok(())
}

/// Asynchronously copies a file from `old_path` to `new_path`, handling specific OS errors.
///
/// This function attempts to copy a file from the specified `old_path` to the specified `new_path`.
/// If the file is being used by another process (OS error 26 or 32), it prints an error message
/// and returns an error indicating that the file is busy. For any other errors, it returns a
/// generic error with additional context.
///
/// # Arguments
///
/// * `old_path` - A reference to the source `Path` of the file to be copied.
/// * `new_path` - A reference to the destination `Path` where the file should be copied.
///
/// # Returns
///
/// This function returns a `Result<()>`. If the file is successfully copied, it returns `Ok(())`.
/// If an error occurs, it returns an `Err` with a detailed error message.
///
/// # Errors
///
/// This function will return an error in the following cases:
/// - If the file is being used by another process (OS error 26 or 32), it returns an error
///   indicating that the file is busy.
/// - For any other errors, it returns a generic error with additional context.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use eyre::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let old_path = Path::new("path/to/source/file");
///     let new_path = Path::new("path/to/destination/file");
///
///     copy_file_with_error_handling(&old_path, &new_path).await?;
///     Ok(())
/// }
/// ```
#[tracing::instrument(skip(old_path, new_path))]
async fn copy_file_with_error_handling(old_path: &Path, new_path: &Path) -> Result<()> {
    trace!(
        "copy_file_with_error_handling: attempting copy {} -> {}",
        old_path.display(),
        new_path.display()
    );
    match fs::copy(&old_path, &new_path).await {
        Ok(_) => {
            trace!(
                "copy_file_with_error_handling: copy succeeded {} -> {}",
                old_path.display(),
                new_path.display()
            );
            Ok(())
        }
        Err(e) => {
            if let Some(code @ (26 | 32)) = e.raw_os_error() {
                debug!(
                    "copy_file_with_error_handling: file busy copying {} -> {} (OS error: {})",
                    old_path.display(),
                    new_path.display(),
                    code
                );
                bail!(
                    "The file {} is busy. Please make sure to close any processes using it.",
                    old_path.display()
                )
            } else {
                debug!(
                    "copy_file_with_error_handling: copy failed {} -> {}: {:?}",
                    old_path.display(),
                    new_path.display(),
                    e
                );
                bail!(eyre!(e).wrap_err("Failed to copy file"))
            }
        }
    }
}

/// Adds the installation directory to the system's PATH.
///
/// This function checks if the installation directory is already in the PATH. If not, it adds the directory to the PATH.
///
/// # Arguments
///
/// * `installation_dir` - The directory to be added to the PATH.
///
/// # Returns
///
/// * `Result<()>` - Returns a `Result` that indicates whether the operation was successful or not.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The installation directory cannot be converted to a string.
/// * The current user's environment variables cannot be accessed or modified (Windows only).
/// * The PATH environment variable cannot be read (non-Windows only).
///
/// # Example
///
/// ```rust
/// let installation_dir = Path::new("/usr/local/bin");
/// add_to_path(&installation_dir).unwrap();
/// ```
async fn add_to_path(installation_dir: PathBuf, config: ConfigFile) -> Result<()> {
    let installation_dir = installation_dir.to_str().unwrap();

    // On Linux this guard must not short-circuit the migration: stale rc-file
    // setups keep `nvim-bin` in PATH forever, so the symlink path below has to
    // run instead.
    #[cfg(not(target_os = "linux"))]
    if what_the_path::shell::exists_in_path("nvim-bin") {
        return Ok(());
    }

    if config.config.add_neovim_binary_to_path == Some(false) {
        info!("Make sure to add {installation_dir} to $PATH");
        return Ok(());
    }

    let temp_config = std::cell::RefCell::new(&config);
    let temp_path = std::cell::RefCell::new(temp_config.borrow().config.add_neovim_binary_to_path);

    if !(dialoguer::console::user_attended() && dialoguer::console::user_attended_stderr())
        && config.config.add_neovim_binary_to_path.is_none()
    {
        info!(
            "You're running in a non-interactive shell. Automatically adding {installation_dir} to system PATH"
        );
        let _ = temp_path.replace(Some(true));
        let tc = temp_config.into_inner(); // use into_inner to gain ownerhsip + original for saving
        tc.save_to_file().await?;
        return Ok(());
    }

    if config.config.add_neovim_binary_to_path.is_none() {
        let timeout = tokio::time::timeout(tokio::time::Duration::from_secs(120), async {
            Confirm::new()
                .with_prompt("Add bob-managed Neovim binary to your $PATH automatically?")
                .interact()
        })
        .await
        .ok();

        match timeout {
            Some(Ok(confirmation)) => {
                // valid confirmation + within time
                let _ = temp_path.replace(Some(confirmation));
                let tc = temp_config.into_inner(); // use into_inner to gain ownerhsip + original for saving
                tc.save_to_file().await?;

                if !confirmation {
                    return Ok(());
                }
            }
            Some(Err(e)) => {
                // non valid due to some error
                bail!(eyre!(e).wrap_err("Failed to read user input"));
            }
            None => {
                // none due to timeout elapsing
                info!("No input received within 120 seconds. Skipping adding to PATH.");
                return Ok(());
            }
        }
    }

    #[cfg(target_family = "windows")]
    return modify_path(installation_dir).await;

    #[cfg(not(target_family = "windows"))]
    return modify_path(&config, installation_dir).await;
}

#[cfg(target_family = "windows")]
async fn modify_path(installation_dir: &str) -> Result<()> {
    use winreg::RegKey;
    use winreg::enums::*;

    let current_usr = RegKey::predef(HKEY_CURRENT_USER);
    let env = current_usr.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
    let usr_path: String = env.get_value("Path")?;
    let usr_path_lower = usr_path.replace('/', "\\").to_lowercase();
    let installation_dir = installation_dir.replace('/', "\\").to_lowercase();

    if usr_path_lower.contains(&installation_dir) {
        return Ok(());
    }

    let new_path = if usr_path_lower.ends_with(';') {
        format!("{usr_path_lower}{installation_dir}")
    } else {
        format!("{usr_path_lower};{installation_dir}")
    };

    env.set_value("Path", &new_path)?;

    info!(
        "Added {installation_dir} to system PATH. Please start a new terminal session for changes to take effect."
    );

    Ok(())
}

#[cfg(target_os = "linux")]
fn path_contains_entry(path_var: &str, directory: &Path) -> bool {
    path_var
        .split(':')
        .any(|entry| Path::new(entry) == directory)
}

/// Attempts to make the bob-managed `nvim` shim reachable through `~/.local/bin`.
///
/// A symlink is created at `~/.local/bin/nvim` pointing to the shim inside the
/// installation directory, but only if `~/.local/bin` exists and is already an
/// entry of `$PATH`. This avoids modifying any shell configuration files.
///
/// # Returns
///
/// `Ok(true)` when the symlink is present and up to date, `Ok(false)` when
/// `~/.local/bin` cannot be used and the caller should fall back to rc file
/// modification.
///
/// # Errors
///
/// This function will return an error if inspecting, removing, or creating the
/// symlink fails.
#[cfg(target_os = "linux")]
async fn try_symlink_shim_to_local_bin(installation_dir: &str) -> Result<bool> {
    use crate::helpers::directories::get_user_home;
    use tracing::warn;

    let Some(home_dir) = get_user_home() else {
        warn!("Could not determine home directory, falling back to rc file modification");
        return Ok(false);
    };

    let local_bin_dir = home_dir.join(".local").join("bin");
    let path_var = env::var("PATH").unwrap_or_default();

    if !local_bin_dir.is_dir() || !path_contains_entry(&path_var, &local_bin_dir) {
        return Ok(false);
    }

    let shim_source = PathBuf::from(installation_dir).join("nvim");
    let shim_link = local_bin_dir.join("nvim");

    let existing_link_target = match fs::symlink_metadata(&shim_link).await {
        Ok(_) => match fs::read_link(&shim_link).await {
            Ok(target) => Some(target),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };

    if existing_link_target.as_deref() != Some(shim_source.as_path()) {
        if existing_link_target.is_some() {
            fs::remove_file(&shim_link).await?;
        }
        fs::symlink(&shim_source, &shim_link).await?;
    }

    info!("Linked nvim shim into {}", local_bin_dir.display());
    Ok(true)
}

/// Removes PATH setup leftovers written by older bob versions.
///
/// Deletes `<fish config dir>/bob.fish` when resolvable and removes the
/// `. "<downloads>/env/env.sh"` source line from the rc files of the currently
/// detected POSIX shell. All failures are reported as warnings and never abort
/// the surrounding operation.
#[cfg(target_os = "linux")]
async fn cleanup_stale_rc_entries(config: &ConfigFile) {
    use crate::helpers::directories::get_downloads_directory;
    use tracing::warn;
    use what_the_path::error::ShellError;
    use what_the_path::shell::{Fish, Shell};

    if let Ok(fish_files) = Shell::Fish(Fish).get_rcfiles() {
        if let Some(fish_conf_dir) = fish_files.first() {
            let bob_fish_file = fish_conf_dir.join("bob.fish");
            if bob_fish_file.exists()
                && let Err(error) = fs::remove_file(&bob_fish_file).await
            {
                warn!(
                    "Failed to remove stale fish config {}: {error}",
                    bob_fish_file.display()
                );
            }
        }
    }

    let shell = match Shell::detect_by_shell_var() {
        Ok(shell) => shell,
        Err(error) => {
            warn!("Failed to detect shell for stale rc entry cleanup: {error}");
            return;
        }
    };

    if matches!(shell, Shell::Fish(_)) {
        return;
    }

    let downloads_dir = match get_downloads_directory(&config.config).await {
        Ok(downloads_dir) => downloads_dir,
        Err(error) => {
            warn!("Failed to resolve downloads directory for stale rc entry cleanup: {error}");
            return;
        }
    };
    let stale_line = format!(
        ". \"{}\"\n",
        downloads_dir.join("env").join("env.sh").display()
    );

    let Ok(rc_files) = shell.get_rcfiles() else {
        warn!("Failed to get rc files for stale entry cleanup");
        return;
    };

    for rc_file in rc_files {
        match what_the_path::shell::remove_from_rcfile(rc_file.clone(), &stale_line) {
            Ok(()) | Err(ShellError::RCFileNotFound(_)) => {}
            Err(error) => warn!(
                "Failed to clean stale PATH entry in {}: {error}",
                rc_file.display()
            ),
        }
    }
}

#[cfg(not(target_family = "windows"))]
async fn modify_path(config: &ConfigFile, installation_dir: &str) -> Result<()> {
    use tracing::warn;
    use what_the_path::shell::Shell;

    #[cfg(target_os = "linux")]
    {
        match try_symlink_shim_to_local_bin(installation_dir).await {
            Ok(true) => {
                cleanup_stale_rc_entries(config).await;
                info!("Added {installation_dir} to system PATH via ~/.local/bin symlink");
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => {
                warn!("Failed to set up ~/.local/bin symlink: {error}");
                return Ok(());
            }
        }
    }

    let shell = match Shell::detect_by_shell_var() {
        Ok(shell) => shell,
        Err(error) => {
            warn!("Failed to detect shell: {error}");
            return Ok(());
        }
    };
    let env_paths = copy_env_files_if_not_exist(&config.config, installation_dir).await?;

    let msg = format!(
        "Added {installation_dir} to system PATH. Please start a new terminal session for changes to take effect."
    );

    let files = match get_rc_files_from_shell(&shell) {
        Ok(files) => std::rc::Rc::new(files),
        Err(error) => {
            let kind_str = format!("{shell:?}");
            warn!("Failed to get {kind_str} rc files: {error}");
            return Ok(());
        }
    };

    match shell {
        Shell::Fish(_fish) => {
            let fish_file = files
                .first()
                .ok_or_else(|| {
                    warn!("No fish rc files found");
                    eyre!("No fish rc files found")
                })?
                .as_ref()
                .join("bob.fish");

            let env_path = env_paths.fish_script.to_str().unwrap();

            create_if_not_exist(&fish_file, env_path).await.map_or_else(
                |error| {
                    warn!("Failed to create fish config file: {error}");
                    Ok(())
                },
                |()| {
                    info!(msg);
                    Ok(())
                },
            )
        }
        _shell => {
            let env_path: &str = env_paths.sh_script.to_str().unwrap();

            let line = format!(". \"{env_path}\"");
            for file in files.iter() {
                let file = file.as_ref().to_path_buf();
                if let Err(error) = what_the_path::shell::append_to_rcfile(file, &line) {
                    warn!("Failed to append to rc file: {error}");
                    return Ok(());
                }
            }
            info!(msg);
            Ok(())
        }
    }
}

// Developer note:
// The `+ use<>` here (without anything in it)
// indicates we want to opt-out of the
// RPIT (return-position `impl Trait` (RPIT) types)
// lifetime capturing.
//
// This is a change in the 2024 edition and up-
// Read more in the `use` docs under `precise capturing`.
//
#[cfg(not(target_family = "windows"))]
fn get_rc_files_from_shell(
    shell: &what_the_path::shell::Shell,
) -> Result<Vec<impl AsRef<Path> + use<>>> {
    Ok(match shell.get_rcfiles() {
        Ok(files) => files,
        Err(error) => {
            bail!(eyre!(error).wrap_err("Failed to get rc files"));
        }
    })
}

#[cfg(not(target_family = "windows"))]
async fn create_if_not_exist<P>(file_path: P, env_path: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    if file_path.as_ref().exists() {
        tracing::warn!(
            "Fish rc file already exists: {}",
            file_path.as_ref().display()
        );
        return Ok(());
    }

    let mut opened_file = File::create(file_path).await?;

    opened_file
        .write_all(format!("source \"{}\"\n", env_path).as_bytes())
        .await?;
    opened_file.flush().await?;

    Ok(())
}

#[cfg(not(target_family = "windows"))]
#[derive(Debug)]
struct FishScriptPath<F>(F);

#[cfg(not(target_family = "windows"))]
#[derive(Debug)]
struct ShScriptPath<S>(S);

#[cfg(not(target_family = "windows"))]
impl<F> std::ops::Deref for FishScriptPath<F> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(not(target_family = "windows"))]
impl<S> std::ops::Deref for ShScriptPath<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(not(target_family = "windows"))]
#[derive(Debug)]
struct EnvPaths<F, S> {
    fish_script: F,
    sh_script: S,
}

#[cfg(not(target_family = "windows"))]
impl<F, S> From<(F, S)> for EnvPaths<F, S> {
    fn from(paths: (F, S)) -> Self {
        EnvPaths {
            fish_script: paths.0,
            sh_script: paths.1,
        }
    }
}

#[cfg(not(target_family = "windows"))]
type EnvPathsBufs = EnvPaths<FishScriptPath<PathBuf>, ShScriptPath<PathBuf>>;

#[cfg(not(target_family = "windows"))]
async fn copy_env_files_if_not_exist(
    config: &Config,
    installation_dir: &str,
) -> Result<EnvPathsBufs> {
    use crate::helpers::directories::get_downloads_directory;
    use tokio::io::AsyncWriteExt;

    let fish_env = include_str!("../../env/env.fish").replace("{nvim_bin}", installation_dir);
    let posix_env = include_str!("../../env/env.sh").replace("{nvim_bin}", installation_dir);
    let downloads_dir = get_downloads_directory(config).await?;
    let env_dir = downloads_dir.join("env");

    // Ensure the env directory exists
    fs::create_dir_all(&env_dir).await?;

    // Define the file paths
    let fish_env_path = env_dir.join("env.fish");
    let posix_env_path = env_dir.join("env.sh");

    // Check if the files exist and write the content if they don't
    if !fish_env_path.exists() {
        let mut file = fs::File::create(&fish_env_path).await?;
        file.write_all(fish_env.as_bytes()).await?;
        file.flush().await?;
    }

    if !posix_env_path.exists() {
        let mut file = fs::File::create(&posix_env_path).await?;
        file.write_all(posix_env.as_bytes()).await?;
        file.flush().await?;
    }

    Ok(EnvPaths::from((
        FishScriptPath(fish_env_path),
        ShScriptPath(posix_env_path),
    )))
}

#[cfg(not(target_family = "windows"))]
#[cfg(test)]
mod use_handler_tests {
    use super::*;
    // Debug using the `dbg!()` macros via:
    //                                         V- to binary
    // `cargo test --bin bob use_handler_tests -- --no-capture`

    #[cfg(target_os = "linux")]
    #[test]
    fn path_contains_entry_test() {
        assert!(path_contains_entry(
            "/usr/local/bin:/home/tester/.local/bin",
            Path::new("/home/tester/.local/bin")
        ));

        assert!(path_contains_entry(
            "/opt/tools/:/home/tester/.local/bin/",
            Path::new("/home/tester/.local/bin")
        ));

        assert!(!path_contains_entry(
            "/home/tester/.local/bin-extra",
            Path::new("/home/tester/.local/bin")
        ));

        assert!(!path_contains_entry(
            "/usr/local/bin:/opt/bin",
            Path::new("/home/tester/.local/bin")
        ));

        assert!(!path_contains_entry("", Path::new("/usr/bin")));
    }

    #[tokio::test]
    async fn copy_env_files_test() {
        let config = ConfigFile::get().await.unwrap();
        let installation_dir = get_installation_directory(&config.config).await.unwrap();
        let env_paths =
            copy_env_files_if_not_exist(&config.config, installation_dir.to_str().unwrap())
                .await
                .unwrap();

        dbg!(&env_paths.fish_script);
        dbg!(&env_paths.sh_script);

        assert!(env_paths.fish_script.exists());
        assert!(env_paths.sh_script.exists());
    }

    #[test]
    fn fish_get_rc_files_test() {
        use what_the_path::shell::Shell;

        let fish_shell = what_the_path::shell::Fish;
        let fish_type = Shell::Fish(fish_shell);

        let fish_files = get_rc_files_from_shell(&fish_type).unwrap();

        let printable = fish_files
            .iter()
            .map(|p| p.as_ref().to_string_lossy())
            .collect::<Vec<_>>()
            .join(", ");

        dbg!(&printable);

        let fish_file = fish_files
            .first()
            .ok_or_else(|| eyre::eyre!("No fish rc files found"))
            .unwrap()
            .as_ref()
            .join("bob.fish");

        dbg!(&fish_file);

        assert!(fish_file.ends_with("bob.fish"));

        assert_ne!(fish_files.len(), 0);
    }

    #[test]
    fn sh_get_rc_files_test() {
        use what_the_path::shell::Shell;

        let bash_shell = what_the_path::shell::Bash;
        let bash_type = Shell::Bash(bash_shell);

        let bash_files = get_rc_files_from_shell(&bash_type).unwrap();

        let printable = bash_files
            .iter()
            .map(|p| p.as_ref().to_string_lossy())
            .collect::<Vec<_>>()
            .join(", ");

        dbg!(&printable);

        assert_ne!(bash_files.len(), 0);
    }

    #[tokio::test]
    async fn sh_get_rc_with_env_test() {
        let config = ConfigFile::get().await.unwrap();
        let installation_dir = get_installation_directory(&config.config).await.unwrap();
        let env_paths =
            copy_env_files_if_not_exist(&config.config, installation_dir.to_str().unwrap())
                .await
                .unwrap();

        let env_path: &str = env_paths.sh_script.to_str().unwrap();

        let inner_shell = what_the_path::shell::Bash;
        let shell = what_the_path::shell::Shell::Bash(inner_shell);

        let files = match get_rc_files_from_shell(&shell) {
            Ok(files) => std::rc::Rc::new(files),
            Err(error) => {
                panic!("Failed to get POSIX rc files: {error}");
            }
        };

        // Inside the match arm for _shell (aka: non-Fish)
        let line = format!(". \"{}\"", env_path);
        for file in files.iter() {
            let file = file.as_ref().to_path_buf();
            if let Err(error) = what_the_path::shell::append_to_rcfile(file.clone(), &line) {
                dbg!(&file);
                dbg!(&line);
                dbg!(&env_path);
                eprintln!("Failed to append to rc file: {error}");
                return;
            }
            // otherwise we should be calling the error branch above
            // Can be dubugged by running:
            // `cargo test --bin bob sh_get_rc_with_env_test -- --no-capture`
            assert!(file.exists());
        }
    }
}
