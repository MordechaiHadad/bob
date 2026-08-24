mod common;

use common::{TestWorkspace, path_string};
use predicates::prelude::*;
use std::fs;

#[test]
fn toml_config_is_honored() {
    let mut workspace = TestWorkspace::new();
    workspace.write_toml("");
    workspace.fake_version("v0.9.5");
    workspace
        .bob()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("v0.9.5"));
}

#[test]
fn json_config_is_honored() {
    let mut workspace = TestWorkspace::new();
    workspace.write_json("");
    workspace.fake_version("v0.9.5");
    workspace
        .bob()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("v0.9.5"));
}

#[test]
fn missing_downloads_dir_errors() {
    let mut workspace = TestWorkspace::new();
    let missing = workspace.temp_dir.path().join("does-not-exist");
    workspace.write_raw(
        "config.toml",
        &format!(
            "downloads_location = \"{}\"\ninstallation_location = \"{}\"\nadd_neovim_binary_to_path = false\nignore_running_instances = true\n",
            path_string(&missing),
            path_string(&workspace.installation_dir)
        ),
    );
    workspace
        .bob()
        .arg("list")
        .assert()
        .failure()
        .stdout(predicate::str::contains("doesn't exist"));
}

#[test]
fn env_var_substitution_in_config() {
    let mut workspace = TestWorkspace::new();
    let marker = "BOB_INTEGRATION_TEST_DIR";
    let marker_value = path_string(&workspace.downloads_dir);
    workspace.write_raw(
        "config.toml",
        &format!(
            "downloads_location = \"${marker}\"\ninstallation_location = \"{}\"\nadd_neovim_binary_to_path = false\nignore_running_instances = true\n",
            path_string(&workspace.installation_dir)
        ),
    );
    workspace.fake_version("v0.9.5");
    workspace
        .bob_with_envs(&[(marker, &marker_value)])
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("v0.9.5"));
}

#[test]
fn missing_bob_config_creates_toml_template() {
    let workspace = TestWorkspace::new();
    let config_path = workspace.temp_dir.path().join("nested").join("config.toml");

    workspace
        .bob_sandboxed()
        .env("BOB_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success();

    let contents = fs::read_to_string(&config_path).expect("config file should be created");
    assert!(
        contents.contains("# rollback_limit = 3"),
        "expected the documented template, got: {contents}"
    );

    workspace
        .bob_sandboxed()
        .env("BOB_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success();
}

#[test]
fn missing_json_bob_config_creates_json_defaults() {
    let workspace = TestWorkspace::new();
    let config_path = workspace.temp_dir.path().join("fresh.json");

    workspace
        .bob_sandboxed()
        .env("BOB_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success();

    let contents = fs::read_to_string(&config_path).expect("json config file should be created");
    assert_eq!(contents.trim(), "{}");
}

#[cfg(unix)]
#[test]
fn unreadable_bob_config_falls_back_to_defaults() {
    let workspace = TestWorkspace::new();
    let blocker = workspace.temp_dir.path().join("blocker");
    fs::write(&blocker, "not a directory").expect("failed to write blocker file");

    let config_path = blocker.join("config.toml");

    workspace
        .bob_sandboxed()
        .env("BOB_CONFIG", &config_path)
        .arg("list")
        .assert()
        .success();

    assert!(
        !config_path.exists(),
        "bob must not create a config when the path is unusable"
    );
}

#[cfg(unix)]
#[test]
fn first_run_creates_config_at_default_location() {
    let workspace = TestWorkspace::new();

    workspace.bob_sandboxed().arg("list").assert().success();

    let expected = workspace
        .temp_dir
        .path()
        .join("config-home")
        .join("bob")
        .join("config.toml");
    let contents = fs::read_to_string(expected).expect("default config.toml should be created");
    assert!(
        contents.contains("# rollback_limit = 3"),
        "expected the documented template, got: {contents}"
    );
}
