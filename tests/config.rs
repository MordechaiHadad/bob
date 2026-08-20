mod common;

use common::{TestWorkspace, path_string};
use predicates::prelude::*;

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
