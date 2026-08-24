#![allow(dead_code)]

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub const NVIM_BIN: &str = if cfg!(windows) { "nvim.exe" } else { "nvim" };

/// Formats a path for use inside a JSON/TOML config value.
///
/// Forward slashes keep the value portable across platforms (Windows accepts
/// them, and backslashes would be interpreted as escapes by TOML/JSON).
pub fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// A self-contained test environment backed by a temporary directory.
///
/// Every test gets its own config file, downloads directory and installation
/// directory, so tests never touch the developer's real bob data. The config
/// always sets `add_neovim_binary_to_path = false` (so nothing edits the real
/// shell rc files) and `ignore_running_instances = true` (so a Neovim running
/// on the host cannot block commands).
pub struct TestWorkspace {
    pub temp_dir: TempDir,
    pub downloads_dir: PathBuf,
    pub installation_dir: PathBuf,
    pub config_path: PathBuf,
}

impl TestWorkspace {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let downloads_dir = temp_dir.path().join("downloads");
        let installation_dir = temp_dir.path().join("nvim-bin");
        let config_path = temp_dir.path().join("config.json");
        fs::create_dir_all(&downloads_dir).expect("failed to create downloads dir");

        let mut workspace = Self {
            temp_dir,
            downloads_dir,
            installation_dir,
            config_path,
        };
        workspace.write_json("");
        workspace
    }

    /// Returns a `bob` command pre-configured with this workspace's config.
    pub fn bob(&self) -> Command {
        let mut cmd = Command::cargo_bin("bob").expect("failed to find bob binary");
        cmd.env("BOB_CONFIG", &self.config_path);
        cmd
    }

    /// Returns a `bob` command with additional environment variables set.
    pub fn bob_with_envs(&self, envs: &[(&str, &str)]) -> Command {
        let mut cmd = self.bob();
        for (key, value) in envs {
            cmd.env(key, value);
        }
        cmd
    }

    /// Returns a `bob` command with `BOB_CONFIG` unset and every home/config/data
    /// directory redirected into the workspace temp dir.
    ///
    /// Use this to exercise a first run without a config file: bob then creates
    /// its default config inside the sandbox instead of touching real user data.
    pub fn bob_sandboxed(&self) -> Command {
        let mut cmd = self.bob();
        cmd.env_remove("BOB_CONFIG");
        cmd.env("HOME", self.temp_dir.path());
        cmd.env("XDG_CONFIG_HOME", self.temp_dir.path().join("config-home"));
        cmd.env("XDG_DATA_HOME", self.temp_dir.path().join("data-home"));
        cmd
    }

    /// Writes an arbitrary config file and points the workspace at it.
    pub fn write_raw(&mut self, filename: &str, contents: &str) {
        let path = self.temp_dir.path().join(filename);
        fs::write(&path, contents).expect("failed to write config file");
        self.config_path = path;
    }

    /// Writes a JSON config with the workspace's base settings.
    ///
    /// `extra` is appended verbatim before the closing brace and must start
    /// with a comma to be valid JSON.
    pub fn write_json(&mut self, extra: &str) {
        let contents = format!(
            "{{\n  \"downloads_location\": \"{}\",\n  \"installation_location\": \"{}\",\n  \"add_neovim_binary_to_path\": false,\n  \"ignore_running_instances\": true{}\n}}",
            path_string(&self.downloads_dir),
            path_string(&self.installation_dir),
            extra
        );
        self.write_raw("config.json", &contents);
    }

    /// Writes a TOML config with the workspace's base settings.
    ///
    /// `extra` is appended verbatim after the base lines (one field per line).
    pub fn write_toml(&mut self, extra: &str) {
        let contents = format!(
            "downloads_location = \"{}\"\ninstallation_location = \"{}\"\nadd_neovim_binary_to_path = false\nignore_running_instances = true\n{}",
            path_string(&self.downloads_dir),
            path_string(&self.installation_dir),
            extra
        );
        self.write_raw("config.toml", &contents);
    }

    /// Creates a fake installed version directory at `<downloads>/<tag>`.
    pub fn fake_version(&self, tag: &str) {
        let bin_dir = self.downloads_dir.join(tag).join("bin");
        fs::create_dir_all(&bin_dir).expect("failed to create fake version directory");
        fs::write(bin_dir.join(NVIM_BIN), "#!/bin/sh\necho fake-nvim\n")
            .expect("failed to write fake nvim binary");
    }
}

impl Default for TestWorkspace {
    fn default() -> Self {
        Self::new()
    }
}
