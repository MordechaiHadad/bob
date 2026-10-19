mod common;

use common::TestWorkspace;
use predicates::prelude::*;
use std::fs;

#[test]
fn install_released_version() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("install")
        .arg("v0.9.5")
        .assert()
        .success();
    let nvim = workspace
        .downloads_dir
        .join("v0.9.5")
        .join("bin")
        .join(common::NVIM_BIN);
    assert!(
        nvim.exists(),
        "installed nvim should exist at {}",
        nvim.display()
    );
}

#[test]
fn install_version_with_release_digest() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("install")
        .arg("v0.11.3")
        .assert()
        .success();
    let nvim = workspace
        .downloads_dir
        .join("v0.11.3")
        .join("bin")
        .join(common::NVIM_BIN);
    assert!(
        nvim.exists(),
        "installed nvim should exist at {}",
        nvim.display()
    );
}

#[test]
fn install_nightly() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("install")
        .arg("nightly")
        .assert()
        .success();
    let nvim = workspace
        .downloads_dir
        .join("nightly")
        .join("bin")
        .join(common::NVIM_BIN);
    assert!(
        nvim.exists(),
        "installed nvim should exist at {}",
        nvim.display()
    );
}

#[test]
fn install_unsupported_version_errors() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("install")
        .arg("v0.2.1")
        .assert()
        .failure()
        .stdout(predicate::str::contains("not supported"));
}

#[test]
fn install_short_hash_errors() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("install")
        .arg("a9a4c27")
        .assert()
        .failure()
        .stdout(predicate::str::contains("full commit hash"));
}

#[test]
fn use_short_hash_errors_when_not_installed() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("use")
        .arg("a9a4c27")
        .assert()
        .failure()
        .stdout(predicate::str::contains("full commit hash"));
}

#[test]
fn use_short_hash_of_installed_build_succeeds() {
    const FULL_HASH: &str = "a9a4c271b13fffba2a21567c86b0f40ae4c180a1";

    let workspace = TestWorkspace::new();
    workspace.fake_version("a9a4c27");
    fs::write(
        workspace
            .downloads_dir
            .join("a9a4c27")
            .join("full-hash.txt"),
        FULL_HASH,
    )
    .expect("failed to write full-hash.txt");

    workspace.bob().arg("use").arg("a9a4c27").assert().success();

    let used = fs::read_to_string(workspace.downloads_dir.join("used"))
        .expect("used file should have been written");
    assert_eq!(used.trim(), FULL_HASH);
}

#[test]
fn use_installed_version() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("install")
        .arg("v0.9.5")
        .assert()
        .success();
    workspace.bob().arg("use").arg("v0.9.5").assert().success();

    let used = fs::read_to_string(workspace.downloads_dir.join("used"))
        .expect("used file should have been written");
    assert_eq!(used.trim(), "v0.9.5");
}

#[test]
fn update_installed_version() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("install")
        .arg("v0.9.5")
        .assert()
        .success();
    workspace
        .bob()
        .args(["update", "v0.9.5"])
        .assert()
        .success();
}

#[test]
fn list_remote_shows_versions() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("list-remote")
        .assert()
        .success()
        .stdout(predicate::str::contains("v0."));
}

#[test]
fn sync_with_installed_version() {
    let mut workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("install")
        .arg("v0.9.5")
        .assert()
        .success();
    let sync_file = workspace.temp_dir.path().join("version.txt");
    fs::write(&sync_file, "v0.9.5\n").expect("failed to write sync file");
    workspace.write_toml(&format!(
        "version_sync_file_location = \"{}\"\n",
        common::path_string(&sync_file)
    ));
    workspace.bob().arg("sync").assert().success();
}
