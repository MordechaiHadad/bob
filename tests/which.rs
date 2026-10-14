mod common;

use common::TestWorkspace;
use predicates::prelude::*;
use std::fs;

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
        .stdout(predicate::str::contains(
            expected.to_string_lossy().as_ref(),
        ));
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
        .stdout(predicate::str::contains(
            expected.to_string_lossy().as_ref(),
        ));
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
