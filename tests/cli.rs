mod common;

use common::TestWorkspace;
use predicates::prelude::*;

#[test]
fn help_flag_succeeds() {
    let workspace = TestWorkspace::new();
    workspace
        .bob()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("version manager for neovim"));
}

#[test]
fn version_flag_succeeds() {
    let workspace = TestWorkspace::new();
    workspace.bob().arg("--version").assert().success();
}

#[test]
fn no_subcommand_fails() {
    let workspace = TestWorkspace::new();
    workspace.bob().assert().failure();
}

#[test]
fn use_requires_version_argument() {
    let workspace = TestWorkspace::new();
    workspace.bob().arg("use").assert().failure();
}

#[test]
fn completions_generate_for_all_shells() {
    let workspace = TestWorkspace::new();
    for shell in ["bash", "elvish", "fish", "nushell", "power-shell", "zsh"] {
        workspace
            .bob()
            .args(["complete", shell])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}
