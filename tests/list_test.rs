mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn init_dotsmith(config_dir: &std::path::Path) {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("init")
        .env("DOTSMITH_CONFIG_DIR", config_dir)
        .assert()
        .success();
}

#[test]
fn test_list_empty() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("list")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No tools tracked"));
}

#[test]
fn test_list_with_tools() {
    if !common::tool_has_config("tmux") {
        return; // skip on CI / systems without tmux config
    }

    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    // Add tmux
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["add", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // List should show tmux
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("list")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("tmux"))
        .stdout(predicate::str::contains("Tier 1"));
}

#[test]
fn test_list_without_init_fails() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith-noinit");

    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("list")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn test_status_with_tools() {
    if !common::tool_has_config("tmux") {
        return; // skip on CI / systems without tmux config
    }

    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    // Add tmux
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["add", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // Status should show tmux as healthy
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("status")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("tmux"));
}

#[test]
fn test_help_output() {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("dotfile workbench"));
}

#[test]
fn test_completions_bash() {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dotsmith"));
}

#[test]
fn test_completions_zsh() {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dotsmith"));
}

#[test]
fn test_completions_fish() {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dotsmith"));
}

#[test]
fn test_version_output() {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}
