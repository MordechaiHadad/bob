use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, anyhow};
use dialoguer::Confirm;
use reqwest::Client;
use tokio::fs::{self};
use tracing::info;

use crate::config::{Config, ConfigFile};
use crate::handlers::{InstallResult, install_handler};
use crate::helpers;
use crate::helpers::directories::get_installation_directory;
use crate::helpers::version::types::{ParsedVersion, VersionType};

/// Starts the process of using a specified version.
///
/// This function checks if the specified version is already used, copies the Neovim proxy to the installation directory, installs the version if it's not already installed and used, switches to the version, and removes the "stable" directory if the version type is "Latest".
///
/// # Arguments
///
/// * `version` - The version to use.
/// * `install` - Whether to install the version if it's not already installed.
/// * `client` - The client to use for HTTP requests.
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
/// * The version is not already used and it cannot be installed.
/// * The version cannot be switched to.
/// * The "stable" directory exists and it cannot be removed.
///
/// # Example
///
/// ```rust
/// let version = ParsedVersion::new("1.0.0");
/// let install = true;
/// let client = Client::new();
/// let config = Config::default();
/// start(version, install, &client, config).await.unwrap();
/// ```
pub async fn start(version: ParsedVersion, install: bool, client: &Client, config: ConfigFile) -> Result<()> {
    let is_version_used = helpers::version::is_version_used(&version.tag_name, &config.config).await;

    copy_nvim_proxy(&config).await?;
    if is_version_used && version.tag_name != "nightly" {
        info!("{} is already installed and used!", version.tag_name);
        return Ok(());
    }

    if install {
        match install_handler::start(&version, client, &config).await {
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
                return Err(anyhow!("Full hash file doesn't exist, please rebuild this commit"));
            }
        } else {
            version.non_parsed_string.clone()
        }
    } else {
        version.tag_name.clone()
    };

    fs::write("used", &file_version).await?;
    if let Some(version_sync_file_location) = helpers::version::get_version_sync_file_location(config).await?
    {
        // Write the used version to version_sync_file_location only if it's different
        let stored_version = fs::read_to_string(&version_sync_file_location).await?;
        if stored_version != version.non_parsed_string {
            fs::write(&version_sync_file_location, file_version).await?;
            info!(
                "Written version to {}",
                version_sync_file_location.into_os_string().into_string().unwrap()
            );
        }
    }

    Ok(())
}

/// Copies the Neovim proxy to the installation directory.
///
/// This function gets the current executable's path, determines the installation directory, creates it if it doesn't exist, adds it to the system's PATH, and copies the current executable to the installation directory as "nvim" or "nvim.exe" (on Windows).
///
/// If a file named "nvim" or "nvim.exe" already exists in the installation directory, the function checks its version. If the version matches the current version, the function does nothing. Otherwise, it replaces the file with the current executable.
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
async fn copy_nvim_proxy(config: &ConfigFile) -> Result<()> {
    let exe_path = env::current_exe().unwrap();
    let mut installation_dir = helpers::directories::get_installation_directory(&config.config).await?;

    if fs::metadata(&installation_dir).await.is_err() {
        fs::create_dir_all(&installation_dir).await?;
    }

    if cfg!(windows) {
        installation_dir.push("nvim.exe");
    } else {
        installation_dir.push("nvim");
    }

    if fs::metadata(&installation_dir).await.is_ok() {
        let output = Command::new(&installation_dir).arg("--&bob").output()?.stdout;
        let version = String::from_utf8(output)?.trim().to_string();

        if version == env!("CARGO_PKG_VERSION") {
            return Ok(());
        }
    }

    info!("Updating neovim proxy");
    copy_file_with_error_handling(&exe_path, &installation_dir).await?;

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
/// use anyhow::Result;
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
async fn copy_file_with_error_handling(old_path: &Path, new_path: &Path) -> Result<()> {
    match fs::copy(&old_path, &new_path).await {
        Ok(_) => Ok(()),
        Err(e) => {
            match e.raw_os_error() {
                Some(26 | 32) => {
                    Err(anyhow::anyhow!(
                        "The file {} is busy. Please make sure to close any processes using it.",
                        old_path.display()
                    ))
                }
                _ => Err(anyhow::anyhow!(e).context("Failed to copy file")),
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
                return Err(anyhow::anyhow!(e).context("Failed to read user input"));
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

#[cfg(not(target_family = "windows"))]
async fn modify_path(config: &ConfigFile, installation_dir: &str) -> Result<()> {
    use tracing::warn;
    use what_the_path::shell::Shell;

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
                    anyhow!("No fish rc files found")
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
fn get_rc_files_from_shell(shell: &what_the_path::shell::Shell) -> Result<Vec<impl AsRef<Path> + use<>>> {
    Ok(match shell.get_rcfiles() {
        Ok(files) => files,
        Err(error) => {
            return Err(anyhow::anyhow!(error).context("Failed to get rc files"));
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
        tracing::warn!("Fish rc file already exists: {}", file_path.as_ref().display());
        return Ok(());
    }

    let mut opened_file = File::create(file_path).await?;

    opened_file
        .write_all(format!("source \"{}\"\n", &env_path).as_bytes())
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
    sh_script:   S,
}

#[cfg(not(target_family = "windows"))]
impl<F, S> From<(F, S)> for EnvPaths<F, S> {
    fn from(paths: (F, S)) -> Self {
        EnvPaths {
            fish_script: paths.0,
            sh_script:   paths.1,
        }
    }
}

#[cfg(not(target_family = "windows"))]
type EnvPathsBufs = EnvPaths<FishScriptPath<PathBuf>, ShScriptPath<PathBuf>>;

#[cfg(not(target_family = "windows"))]
async fn copy_env_files_if_not_exist(config: &Config, installation_dir: &str) -> Result<EnvPathsBufs> {
    use tokio::io::AsyncWriteExt;

    use crate::helpers::directories::get_downloads_directory;

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

    Ok(EnvPaths::from((FishScriptPath(fish_env_path), ShScriptPath(posix_env_path))))
}

#[cfg(not(target_family = "windows"))]
#[cfg(test)]
mod use_handler_tests {
    use super::*;
    // Debug using the `dbg!()` macros via:
    //                                         V- to binary
    // `cargo test --bin bob use_handler_tests -- --no-capture`

    #[tokio::test]
    async fn copy_env_files_test() {
        let config = ConfigFile::get().await.unwrap();
        let installation_dir = get_installation_directory(&config.config).await.unwrap();
        let env_paths = copy_env_files_if_not_exist(&config.config, installation_dir.to_str().unwrap())
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
            .ok_or_else(|| anyhow::anyhow!("No fish rc files found"))
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
        let env_paths = copy_env_files_if_not_exist(&config.config, installation_dir.to_str().unwrap())
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
