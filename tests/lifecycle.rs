mod common;

use common::TestWorkspace;
use predicates::prelude::*;
use std::fs;

#[test]
fn list_empty_shows_message() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("There are no versions installed"));
}

#[test]
fn list_shows_installed_version() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");
    workspace
        .bob()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("v0.9.5"))
        .stdout(predicate::str::contains("Installed"));
}

#[test]
fn use_no_install_writes_used_and_proxy() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");
    workspace
        .bob()
        .args(["use", "v0.9.5", "--no-install"])
        .assert()
        .success();

    let used = fs::read_to_string(workspace.downloads_dir.join("used"))
        .expect("used file should have been written");
    assert_eq!(used.trim(), "v0.9.5");

    let proxy = workspace.installation_dir.join(common::NVIM_BIN);
    assert!(
        proxy.exists(),
        "proxy binary should exist at {}",
        proxy.display()
    );
}

#[test]
fn list_marks_used_version() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");
    workspace
        .bob()
        .args(["use", "v0.9.5", "--no-install"])
        .assert()
        .success();
    workspace
        .bob()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Used"));
}

#[test]
fn use_is_idempotent() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");
    workspace
        .bob()
        .args(["use", "v0.9.5", "--no-install"])
        .assert()
        .success();
    workspace
        .bob()
        .args(["use", "v0.9.5", "--no-install"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already installed and used"));
}

#[test]
fn use_nightly_no_install() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("nightly");
    workspace
        .bob()
        .args(["use", "nightly", "--no-install"])
        .assert()
        .success();

    let used = fs::read_to_string(workspace.downloads_dir.join("used"))
        .expect("used file should have been written");
    assert_eq!(used.trim(), "nightly");
}

#[test]
fn uninstall_removes_version() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");
    workspace
        .bob()
        .arg("uninstall")
        .arg("v0.9.5")
        .assert()
        .success();
    assert!(
        !workspace.downloads_dir.join("v0.9.5").exists(),
        "version directory should have been removed"
    );
}

#[test]
fn uninstall_used_version_warns_and_keeps_version() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");
    workspace
        .bob()
        .args(["use", "v0.9.5", "--no-install"])
        .assert()
        .success();
    workspace
        .bob()
        .arg("uninstall")
        .arg("v0.9.5")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Switch to a different version before proceeding",
        ));
    assert!(workspace.downloads_dir.join("v0.9.5").exists());
}

#[test]
fn erase_removes_downloads_then_second_erase_fails() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");
    workspace.bob().arg("erase").assert().success();
    assert!(
        !workspace.downloads_dir.exists(),
        "downloads directory should have been removed"
    );
    workspace
        .bob()
        .arg("erase")
        .assert()
        .failure()
        .stdout(predicate::str::contains("doesn't exist"));
}

#[test]
fn sync_empty_file_errors() {
    let mut workspace = TestWorkspace::new();
    let sync_file = workspace.temp_dir.path().join("version.txt");
    fs::write(&sync_file, "").expect("failed to write sync file");
    workspace.write_toml(&format!(
        "version_sync_file_location = \"{}\"\n",
        common::path_string(&sync_file)
    ));
    workspace
        .bob()
        .arg("sync")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Sync file is empty"));
}

#[test]
fn sync_nightly_rollback_errors() {
    let mut workspace = TestWorkspace::new();
    let sync_file = workspace.temp_dir.path().join("version.txt");
    fs::write(&sync_file, "nightly-abc1234\n").expect("failed to write sync file");
    workspace.write_toml(&format!(
        "version_sync_file_location = \"{}\"\n",
        common::path_string(&sync_file)
    ));
    workspace
        .bob()
        .arg("sync")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Cannot sync nightly rollbacks"));
}

#[test]
fn sync_uses_installed_version() {
    let mut workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");
    let sync_file = workspace.temp_dir.path().join("version.txt");
    fs::write(&sync_file, "v0.9.5\n").expect("failed to write sync file");
    workspace.write_toml(&format!(
        "version_sync_file_location = \"{}\"\n",
        common::path_string(&sync_file)
    ));
    workspace.bob().arg("sync").assert().success();

    let used = fs::read_to_string(workspace.downloads_dir.join("used"))
        .expect("used file should have been written");
    assert_eq!(used.trim(), "v0.9.5");
}

#[test]
fn list_detects_nightly_rollbacks() {
    let workspace = TestWorkspace::new();
    workspace.fake_version("v0.9.5");
    let nightly_dir = workspace.downloads_dir.join("nightly-abc1234");
    fs::create_dir_all(&nightly_dir).expect("failed to create nightly rollback dir");
    fs::write(
        nightly_dir.join("bob.json"),
        r#"{"tag_name":"nightly-abc1234","target_commitish":"abcdef1","published_at":"2024-01-01T00:00:00Z"}"#,
    )
    .expect("failed to write bob.json");
    workspace
        .bob()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("nightly-abc1234"));
}
