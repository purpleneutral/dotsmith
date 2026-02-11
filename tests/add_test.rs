mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: init dotsmith in a temp dir
fn init_dotsmith(config_dir: &std::path::Path) {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("init")
        .env("DOTSMITH_CONFIG_DIR", config_dir)
        .assert()
        .success();
}

#[test]
fn test_add_tmux_tier1() {
    if !common::tool_has_config("tmux") {
        eprintln!("skipping: tmux config not found on this system");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["add", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Tier 1"))
        .stdout(predicate::str::contains("option"));
}

#[test]
fn test_add_duplicate_fails() {
    if !common::tool_has_config("tmux") {
        eprintln!("skipping: tmux config not found on this system");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    // First add succeeds
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["add", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // Second add fails
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["add", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already managed"));
}

#[test]
fn test_add_not_installed_fails() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["add", "nonexistent_tool_xyz_123"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn test_add_without_init_auto_initializes() {
    if !common::tool_has_config("tmux") {
        return;
    }

    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith-noinit");

    // Should auto-initialize and succeed
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["add", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added tmux"));

    assert!(config_dir.join("manifest.toml").exists());
    assert!(config_dir.join("config.toml").exists());
}

#[test]
fn test_add_updates_manifest() {
    if !common::tool_has_config("tmux") {
        eprintln!("skipping: tmux config not found on this system");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["add", "tmux"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // Read manifest and verify tmux is present
    let content = fs::read_to_string(config_dir.join("manifest.toml")).unwrap();
    assert!(
        content.contains("[tools.tmux]"),
        "manifest should contain tmux entry"
    );
    assert!(content.contains("tier = 1"), "tmux should be tier 1");
}

#[test]
fn test_add_detects_tpm_plugin_manager() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    // Only run this test if tmux is installed and has TPM
    let tmux_config = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
        .join("tmux");

    if tmux_config.join("plugs/tpm").exists() || tmux_config.join("plugins/tpm").exists() {
        Command::cargo_bin("dotsmith")
            .unwrap()
            .args(["add", "tmux"])
            .env("DOTSMITH_CONFIG_DIR", &config_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("plugin manager"));
    }
}
