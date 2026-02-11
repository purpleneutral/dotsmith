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
fn test_list_without_init_auto_initializes() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith-noinit");

    // Should auto-initialize and succeed
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("list")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No tools tracked"));

    assert!(config_dir.join("manifest.toml").exists());
    assert!(config_dir.join("config.toml").exists());
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

#[test]
fn test_doctor_empty() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("doctor")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No tools tracked"))
        .stdout(predicate::str::contains("Summary"));
}

#[test]
fn test_doctor_without_init_auto_initializes() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith-noinit");

    // Should auto-initialize and report OK
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("doctor")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("config directory"))
        .stdout(predicate::str::contains("Summary"));

    assert!(config_dir.join("manifest.toml").exists());
}

#[test]
fn test_search_mouse() {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["search", "mouse"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tmux"))
        .stdout(predicate::str::contains("mouse"))
        .stdout(predicate::str::contains("result"));
}

#[test]
fn test_search_no_results() {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["search", "zzzznonexistent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results"));
}

#[test]
fn test_edit_not_tracked() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["edit", "nonexistent"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not tracked"));
}

#[test]
fn test_watch_not_tracked() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["watch", "nonexistent"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not tracked"));
}

#[test]
fn test_mangen_output() {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("mangen")
        .assert()
        .success()
        .stdout(predicate::str::contains("dotsmith"));
}
