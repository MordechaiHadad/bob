use anyhow::Result;
use std::{fs, path::PathBuf};
use tracing::info;
use yansi::Paint;

use crate::{
    config::Config,
    helpers::{self, directories, system::find_system_nvim, version::nightly::produce_nightly_vec},
};

/// Starts the list handler.
///
/// This function reads the downloads directory and lists all the installed versions in a formatted table. It also checks if a version is currently in use.
///
/// # Arguments
///
/// * `config` - The configuration object.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the operation is successful, or an error if there are no versions installed or if there is a failure in reading the directory or checking if a version is in use.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let result = start(config).await;
/// assert!(result.is_ok());
/// ```
pub async fn start(config: Config) -> Result<()> {
    let versions = collect_versions(&config).await?;

    if versions.is_empty() {
        info!("There are no versions installed");
        return Ok(());
    }

    render_versions_table(&versions, &config).await?;
    Ok(())
}

/// Represents the status of a version.
#[derive(Debug, Clone, PartialEq)]
enum VersionStatus {
    Used,
    Missing,   // System version that doesn't exist
    Available, // System version not in use
    Installed, // Downloaded version not in use
}

impl VersionStatus {
    fn as_str(&self) -> &str {
        match self {
            VersionStatus::Used => "Used",
            VersionStatus::Missing => "Missing",
            VersionStatus::Available => "Available",
            VersionStatus::Installed => "Installed",
        }
    }

    fn display(&self) -> Paint<&str> {
        match self {
            VersionStatus::Used => Paint::green(self.as_str()),
            VersionStatus::Missing => Paint::red(self.as_str()),
            VersionStatus::Available => Paint::cyan(self.as_str()),
            VersionStatus::Installed => Paint::yellow(self.as_str()),
        }
    }

    fn len(&self) -> usize {
        self.as_str().len()
    }
}

/// Represents a version entry with its name and status.
#[derive(Debug)]
struct VersionEntry {
    name: String,
    status: VersionStatus,
}

/// Collects all version entries with their statuses.
async fn collect_versions(config: &Config) -> Result<Vec<VersionEntry>> {
    let mut entries = Vec::new();

    // Check for system version
    let has_system = find_system_nvim(config).await?.is_some();
    let is_system_used = helpers::version::is_version_used("system", config).await;

    if has_system || is_system_used {
        let status = if is_system_used {
            if has_system {
                VersionStatus::Used
            } else {
                VersionStatus::Missing
            }
        } else {
            VersionStatus::Available
        };

        entries.push(VersionEntry {
            name: "system".to_string(),
            status,
        });
    }

    // Collect downloaded versions
    let downloads_dir = directories::get_downloads_directory(config).await?;
    let paths: Vec<PathBuf> = fs::read_dir(downloads_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();

    for path in paths {
        if !path.is_dir() {
            continue;
        }

        let path_name = path.file_name().unwrap().to_str().unwrap();

        if !is_version(path_name) {
            continue;
        }

        let status = if helpers::version::is_version_used(path_name, config).await {
            VersionStatus::Used
        } else {
            VersionStatus::Installed
        };

        entries.push(VersionEntry {
            name: path_name.to_string(),
            status,
        });
    }

    Ok(entries)
}

/// Table formatter for rendering version entries.
struct TableFormatter {
    version_col_width: usize,
    status_col_width: usize,
    padding: usize,
}

impl TableFormatter {
    const VERSION_HEADER: &'static str = "Version";
    const STATUS_HEADER: &'static str = "Status";
    const PADDING: usize = 2;
    fn new(entries: &[VersionEntry], _has_rollbacks: bool) -> Self {
        let (max_version_len, max_status_len) =
            entries.iter().fold((0, 0), |(max_v, max_s), entry| {
                (max_v.max(entry.name.len()), max_s.max(entry.status.len()))
            });

        // Ensure columns are at least as wide as their headers
        let version_col_width = max_version_len.max(Self::VERSION_HEADER.len());
        let status_col_width = max_status_len.max(Self::STATUS_HEADER.len());

        Self {
            version_col_width,
            status_col_width,
            padding: Self::PADDING,
        }
    }

    fn print_border<W: std::io::Write>(
        &self,
        writer: &mut W,
        left: &str,
        mid: &str,
        right: &str,
    ) -> std::io::Result<()> {
        writeln!(
            writer,
            "{}{}{}{}{}",
            left,
            "─".repeat(self.version_col_width + (self.padding * 2)),
            mid,
            "─".repeat(self.status_col_width + (self.padding * 2)),
            right
        )
    }

    fn print_header<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let version_padding = self.version_col_width - Self::VERSION_HEADER.len();
        let status_padding = self.status_col_width - Self::STATUS_HEADER.len();

        writeln!(
            writer,
            "│{}{}{}│{}{}{}│",
            " ".repeat(self.padding),
            Self::VERSION_HEADER,
            " ".repeat(version_padding + self.padding),
            " ".repeat(self.padding),
            Self::STATUS_HEADER,
            " ".repeat(status_padding + self.padding)
        )
    }

    fn print_row<W: std::io::Write>(
        &self,
        writer: &mut W,
        entry: &VersionEntry,
    ) -> std::io::Result<()> {
        let version_padding = self.version_col_width - entry.name.len();
        let status_padding = self.status_col_width - entry.status.len();

        writeln!(
            writer,
            "│{}{}{}│{}{}{}│",
            " ".repeat(self.padding),
            entry.name,
            " ".repeat(version_padding + self.padding),
            " ".repeat(self.padding),
            entry.status.display(),
            " ".repeat(status_padding + self.padding)
        )
    }

    fn render<W: std::io::Write>(
        &self,
        writer: &mut W,
        entries: &[VersionEntry],
    ) -> std::io::Result<()> {
        self.print_border(writer, "┌", "┬", "┐")?;
        self.print_header(writer)?;
        self.print_border(writer, "├", "┼", "┤")?;

        for entry in entries {
            self.print_row(writer, entry)?;
        }

        self.print_border(writer, "└", "┴", "┘")?;
        Ok(())
    }
}

/// Renders a table of version entries.
async fn render_versions_table(entries: &[VersionEntry], config: &Config) -> Result<()> {
    let has_rollbacks = has_rollbacks(config).await?;
    let formatter = TableFormatter::new(entries, has_rollbacks);
    formatter.render(&mut std::io::stdout(), entries)?;
    Ok(())
}

/// Checks if there are any rollbacks available.
///
/// This function produces a vector of nightly versions and checks if it is empty.
///
/// # Arguments
///
/// * `config` - A reference to the configuration object.
///
/// # Returns
///
/// * `Result<bool>` - Returns a `Result` that contains `true` if there are rollbacks available, or `false` otherwise. Returns an error if there is a failure in producing the vector of nightly versions.
///
/// # Example
///
/// ```rust
/// let config = Config::default();
/// let has_rollbacks = has_rollbacks(&config).await?;
/// assert_eq!(has_rollbacks, true);
/// ```
async fn has_rollbacks(config: &Config) -> Result<bool> {
    let list = produce_nightly_vec(config).await?;

    Ok(!list.is_empty())
}

/// Checks if a given string is a valid version.
///
/// This function checks if the given string is "stable", contains "nightly", or matches the version or hash regex.
///
/// # Arguments
///
/// * `name` - A reference to a string that could be a version.
///
/// # Returns
///
/// * `bool` - Returns `true` if the string is a valid version, `false` otherwise.
///
/// # Example
///
/// ```rust
/// let version = "v1.0.0";
/// let is_version = is_version(version);
/// assert_eq!(is_version, true);
/// ```
fn is_version(name: &str) -> bool {
    match name {
        "stable" => true,
        nightly_name if nightly_name.contains("nightly") => true,
        name => {
            if crate::VERSION_REGEX.is_match(name) {
                return true;
            }
            crate::HASH_REGEX.is_match(name)
        }
    }
}

#[cfg(test)]
mod list_handler_is_version_tests {
    use super::*;

    #[test]
    fn test_table_formatter_basic() {
        yansi::Paint::disable(); // Suppress colors for this test

        let entries = vec![
            super::VersionEntry {
                name: "system".to_string(),
                status: super::VersionStatus::Available,
            },
            super::VersionEntry {
                name: "nightly".to_string(),
                status: super::VersionStatus::Used,
            },
            super::VersionEntry {
                name: "v0.11.5".to_string(),
                status: super::VersionStatus::Installed,
            },
            super::VersionEntry {
                name: "nightly-0197f13".to_string(),
                status: super::VersionStatus::Installed,
            },
        ];
        let formatter = super::TableFormatter::new(&entries, false);
        let mut buf = Vec::new();
        formatter.render(&mut buf, &entries).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let expected = "\
┌───────────────────┬─────────────┐
│  Version          │  Status     │
├───────────────────┼─────────────┤
│  system           │  Available  │
│  nightly          │  Used       │
│  v0.11.5          │  Installed  │
│  nightly-0197f13  │  Installed  │
└───────────────────┴─────────────┘
";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_table_formatter_with_rollbacks_width() {
        let entries = vec![super::VersionEntry {
            name: "short".to_string(),
            status: super::VersionStatus::Used,
        }];
        let formatter = super::TableFormatter::new(&entries, true);
        // Should use the max of the header or the entry length
        assert_eq!(
            formatter.version_col_width,
            super::TableFormatter::VERSION_HEADER
                .len()
                .max("short".len())
        );
    }

    #[test]
    fn test_is_version() {
        let cases_expected = [
            ("v1.0.0", true),
            ("stable", true),
            ("nightly-2023-10-01", true),
            ("invalid-version", false),
            ("", false),
        ];

        cases_expected
            .iter()
            .for_each(|(case, expected)| match expected {
                true => assert!(is_version(case)),
                false => assert!(!is_version(case)),
            });

        cases_expected.iter().for_each(|(case, expected)| {
            assert_eq!(is_version(case), *expected);
        });
    }

    #[test]
    fn test_with_v_semvar() {
        let version = "v1.2.3";
        assert!(
            is_version(version),
            "Expected '{}' to be a valid version",
            version
        );
    }

    #[test]
    fn test_as_stable() {
        let version = "stable";
        assert!(
            is_version(version),
            "Expected '{}' to be a valid version",
            version
        );
    }

    #[test]
    fn test_with_nightly_and_date() {
        let version = "nightly-2023-10-01";
        assert!(
            is_version(version),
            "Expected '{}' to be a valid version",
            version
        );
    }

    #[test]
    fn test_with_invalid_version() {
        let version = "invalid-version";
        // let res = is_version(version);
        assert!(
            !is_version(version),
            "Expected '{}' to not be a valid version",
            version
        );
    }

    #[test]
    #[should_panic]
    fn test_with_invalid_version_panic() {
        let version = "invalid-version-wow";
        assert!(
            is_version(version),
            "Expected '{}' to not be a valid version",
            version
        );
    }

    #[test]
    fn test_with_empty_string() {
        let version = "";
        assert!(
            !is_version(version),
            "Expected '{}' to not be a valid version",
            version
        );
    }
}
