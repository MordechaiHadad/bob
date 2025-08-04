use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub config: Config,
}

impl ConfigFile {
    pub async fn save_to_file(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let data = match self.format {
            ConfigFormat::Toml => toml::to_string(&self.config)?,
            ConfigFormat::Json => serde_json::to_string_pretty(&self.config)?,
        };

        let tmp_path = self.path.with_extension("tmp");
        let mut file = File::create(&tmp_path).await?;
        file.write_all(data.as_bytes()).await?;
        file.flush().await?;

        // atomic operation i guess
        tokio::fs::rename(tmp_path, &self.path).await?;

        Ok(())
    }
}

impl ConfigFile {
    pub async fn get() -> Result<ConfigFile> {
        let config_file = crate::helpers::directories::get_config_file()?;
        let mut config_format = ConfigFormat::Json;
        let config = match fs::read_to_string(&config_file).await {
            Ok(config) => {
                if config_file.extension().unwrap() == "toml" {
                    let mut config: Config = toml::from_str(&config)?;
                    handle_envars(&mut config)?;
                    config_format = ConfigFormat::Toml;
                    config
                } else {
                    let mut config: Config = serde_json::from_str(&config)?;
                    handle_envars(&mut config)?;
                    config
                }
            }
            Err(_) => Config {
                enable_nightly_info: None,
                enable_release_build: None,
                downloads_location: None,
                installation_location: None,
                version_sync_file_location: None,
                github_mirror: None,
                rollback_limit: None,
                add_neovim_binary_to_path: None,
                ignore_running_instances: None,
            },
        };

        Ok(ConfigFile {
            path: config_file,
            format: config_format,
            config,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ConfigFormat {
    Toml,
    Json,
}

/// Represents the application configuration.
///
/// This struct contains various configuration options for the application, such as whether to enable nightly info, whether to enable release build, the location for downloads, the location for installation, the location for the version sync file, the GitHub mirror to use, and the rollback limit.
///
/// # Fields
///
/// * `enable_nightly_info: Option<bool>` - Whether to enable nightly info. This is optional and may be `None`.
/// * `enable_release_build: Option<bool>` - Whether to enable release build. This is optional and may be `None`.
/// * `downloads_location: Option<String>` - The location for downloads. This is optional and may be `None`.
/// * `installation_location: Option<String>` - The location for installation. This is optional and may be `None`.
/// * `version_sync_file_location: Option<String>` - The location for the version sync file. This is optional and may be `None`.
/// * `github_mirror: Option<String>` - The GitHub mirror to use. This is optional and may be `None`.
/// * `rollback_limit: Option<u8>` - The rollback limit. This is optional and may be `None`.
/// * `add_neovim_binary_to_path: Option<bool>` - Tells bob whenever to add neovim proxy path to $PATH.
///
/// # Example
///
/// ```rust
/// let config = Config {
///     enable_nightly_info: Some(true),
///     enable_release_build: Some(false),
///     downloads_location: Some("/path/to/downloads".to_string()),
///     installation_location: Some("/path/to/installation".to_string()),
///     version_sync_file_location: Some("/path/to/version_sync_file".to_string()),
///     github_mirror: Some("https://github.com".to_string()),
///     rollback_limit: Some(5),
///     rollback_limit: Some(true),
/// };
/// println!("The configuration is {:?}", config);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_nightly_info: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_release_build: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_sync_file_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_mirror: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_limit: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_neovim_binary_to_path: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_running_instances: Option<bool>,
}


/// Private trait for processing environment variables in configuration fields.
/// Allowss creating a list and using polymorphism to handle different types of fields that may
/// contain environment variables.
trait EnvVarProcessor {
    fn process(&mut self) -> Result<()>;
}

impl EnvVarProcessor for Option<String> {
    /// `process` method for `Option<String>`.
    /// This is a method for structs that implement the `EnvVarProcessor` trait.
    ///
    /// It's deigned to process the `Option<String>` type, checking if it contains a value that
    /// matches the `ENVIRONMENT_VAR_REGEX`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - A mutable reference to the `Option<String>` instance.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns `Ok(())` if the processing is successful. Error cases include when the environment variable cannot be found or if the regex fails to match.
    fn process(&mut self) -> Result<()> {
        if let Some(value) = self {
            if ENVIRONMENT_VAR_REGEX.is_match(value) {
                let extract = ENVIRONMENT_VAR_REGEX.find(value).map_or("", |m| m.as_str());

                let var = env::var(extract).expect("Failed to get environment variable");

                *value = value.replace(&format!("${extract}"), &var);
            }
        }
        Ok(())
    }
}

/// Handles environment variables in the configuration.
///
/// This function takes a mutable reference to a `Config` object. It uses a `Regex` to match environment variables in the format `$VARIABLE_NAME`.
/// It then calls the the EnvVarProcessor Trait's `process` method on each field in the `Config`
/// object that may contain an environment variable.
///
///
/// # Arguments
///
/// * `config: &mut Config` - A mutable reference to a `Config` object that may contain environment variables.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the function completes successfully. If an error occurs, it returns `Err`.
///
/// # Example
///
/// ```rust
/// let mut config = Config {
///     downloads_location: Some("DOWNLOADS=${DOWNLOADS}".to_string()),
///     github_mirror: Some("GITHUB=${GITHUB}".to_string()),
///     installation_location: Some("INSTALL=${INSTALL}".to_string()),
///     version_sync_file_location: Some("SYNC=${SYNC}".to_string()),
/// };
/// handle_envars(&mut config).unwrap();
/// assert_eq!(config.downloads_location, Some(format!("DOWNLOADS={}", env::var("DOWNLOADS").unwrap())));
/// assert_eq!(config.github_mirror, Some(format!("GITHUB={}", env::var("GITHUB").unwrap())));
/// assert_eq!(config.installation_location, Some(format!("INSTALL={}", env::var("INSTALL").unwrap())));
/// assert_eq!(config.version_sync_file_location, Some(format!("SYNC={}", env::var("SYNC").unwrap())));
/// ```
fn handle_envars(config: &mut Config) -> Result<()> {
    let mut fields = [
        &mut config.downloads_location,
        &mut config.github_mirror,
        &mut config.installation_location,
        &mut config.version_sync_file_location,
    ];

    fields.iter_mut().try_for_each(|field| field.process())
}
