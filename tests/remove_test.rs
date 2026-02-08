mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
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
fn test_remove_tool() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    // Add tmux first
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["add", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // Remove tmux
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["remove", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed tmux"))
        .stdout(predicate::str::contains("untouched"));

    // Verify manifest no longer contains tmux
    let content = fs::read_to_string(config_dir.join("manifest.toml")).unwrap();
    assert!(
        !content.contains("[tools.tmux]"),
        "manifest should not contain tmux after removal"
    );
}

#[test]
fn test_remove_nonexistent_fails() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["remove", "nonexistent"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not managed"));
}

#[test]
fn test_remove_does_not_touch_config_files() {
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

    // Verify the actual tmux config still exists before and after removal
    let tmux_config_path = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
        .join("tmux/tmux.conf");

    let existed_before = tmux_config_path.exists();

    // Remove tmux
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["remove", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    let exists_after = tmux_config_path.exists();

    // Config file state should be unchanged
    assert_eq!(
        existed_before, exists_after,
        "tmux config file existence should not change after remove"
    );
}
