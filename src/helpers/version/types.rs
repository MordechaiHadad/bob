use semver::Version;

use crate::github_requests::UpstreamVersion;
use std::path::PathBuf;

/// Represents a parsed version of the software.
///
/// This struct contains information about a parsed version of the software, including the tag name, version type, non-parsed string, and semantic version.
///
/// # Fields
///
/// * `tag_name: String` - The tag name of the parsed version.
/// * `version_type: VersionType` - The type of the parsed version.
/// * `non_parsed_string: String` - The non-parsed string of the parsed version.
/// * `semver: Option<Version>` - The semantic version of the parsed version, or `None` if the version is not a semantic version.
///
/// # Example
///
/// ```rust
/// let parsed_version = ParsedVersion {
///     tag_name: "v1.0.0".to_string(),
///     version_type: VersionType::Normal,
///     non_parsed_string: "version-1.0.0".to_string(),
///     semver: Some(Version::parse("1.0.0").unwrap()),
/// };
/// println!("The parsed version is {:?}", parsed_version);
/// ```
pub struct ParsedVersion {
    pub tag_name: String,
    pub version_type: VersionType,
    pub non_parsed_string: String,
    pub semver: Option<Version>,
}

/// Represents the type of a software version.
///
/// This enum is used to distinguish between different types of software versions, such as normal versions, the latest version, nightly versions, versions identified by a hash, and nightly versions that have been rolled back.
///
/// # Variants
///
/// * `Normal` - Represents a normal version.
/// * `Latest` - Represents the latest version.
/// * `Nightly` - Represents a nightly version.
/// * `Hash` - Represents a version identified by a hash.
/// * `NightlyRollback` - Represents a nightly version that has been rolled back.
///
/// # Example
///
/// ```rust
/// let version_type = VersionType::Nightly;
/// match version_type {
///     VersionType::Normal => println!("This is a normal version."),
///     VersionType::Latest => println!("This is the latest version."),
///     VersionType::Nightly => println!("This is a nightly version."),
///     VersionType::Hash => println!("This is a version identified by a hash."),
///     VersionType::NightlyRollback => println!("This is a nightly version that has been rolled back."),
/// }
/// ```
#[derive(PartialEq, Eq, Debug)]
pub enum VersionType {
    Normal,
    Latest,
    Nightly,
    Hash,
    NightlyRollback,
}

/// Represents a local nightly version of the software.
///
/// This struct contains information about a local nightly version of the software, including the upstream version data and the path to the version file.
///
/// # Fields
///
/// * `data: UpstreamVersion` - The upstream version data for the local nightly version.
/// * `path: PathBuf` - The path to the file that contains the local nightly version.
///
/// # Example
///
/// ```rust
/// let upstream_version = UpstreamVersion {
///     // initialize fields
/// };
/// let local_nightly = LocalNightly {
///     data: upstream_version,
///     path: PathBuf::from("/path/to/nightly/version"),
/// };
/// println!("The local nightly version is {:?}", local_nightly);
/// ```
#[derive(Debug, Clone)]
pub struct LocalNightly {
    pub data: UpstreamVersion,
    pub path: PathBuf,
}

/// Represents a local version of the software.
///
/// This struct contains information about a local version of the software, including the file name, file format, path, and semantic version.
///
/// # Fields
///
/// * `file_name: String` - The name of the file that contains the local version.
/// * `file_format: String` - The format of the file that contains the local version.
/// * `path: String` - The path to the file that contains the local version.
/// * `semver: Option<Version>` - The semantic version of the local version, or `None` if the version is not a semantic version.
///
/// # Example
///
/// ```rust
/// let local_version = LocalVersion {
///     file_name: "version-1.0.0.tar.gz".to_string(),
///     file_format: "tar.gz".to_string(),
///     path: "/path/to/version-1.0.0.tar.gz".to_string(),
///     semver: Some(Version::parse("1.0.0").unwrap()),
/// };
/// println!("The local version is {:?}", local_version);
/// ```
#[derive(Clone, PartialEq)]
pub struct LocalVersion {
    pub file_name: String,
    pub file_format: String,
    pub path: String,
    pub semver: Option<Version>,
}
