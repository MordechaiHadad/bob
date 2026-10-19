mod common;

use common::TestWorkspace;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Matches stdout against a path, ignoring separator style.
///
/// Bob builds the printed path from the `downloads_location` config value
/// (which tests write with forward slashes for portability) and then joins the
/// version components, which use the platform separator. On Windows that makes
/// the output mix `/` and `\`, so compare paths as paths instead of strings.
fn stdout_is_path(expected: PathBuf) -> impl Predicate<str> {
    predicate::function(move |stdout: &str| Path::new(stdout.trim()) == expected)
}

#[test]
fn which_prints_the_current_version_executable() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("nightly");
    fs::write(workspace.downloads_dir.join("used"), "nightly\n")
        .expect("failed to write current version");

    let expected = workspace
        .downloads_dir
        .join("nightly")
        .join("bin")
        .join(common::NVIM_BIN);
    workspace
        .bob()
        .arg("which")
        .assert()
        .success()
        .stdout(stdout_is_path(expected));
}

#[test]
fn which_prints_the_requested_version_executable() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");

    let expected = workspace
        .downloads_dir
        .join("v0.9.5")
        .join("bin")
        .join(common::NVIM_BIN);
    workspace
        .bob()
        .args(["which", "0.9.5"])
        .assert()
        .success()
        .stdout(stdout_is_path(expected));
}

#[test]
fn which_fails_when_the_executable_is_not_installed() {
    let workspace = TestWorkspace::new();
    fs::create_dir_all(workspace.downloads_dir.join("nightly").join("bin"))
        .expect("failed to create fake version directory");

    workspace
        .bob()
        .args(["which", "nightly"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("was not found"));
}
