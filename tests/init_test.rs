mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_init_creates_directory_structure() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");

    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("init")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized dotsmith"));

    assert!(config_dir.join("manifest.toml").exists());
    assert!(config_dir.join("config.toml").exists());
}

#[test]
fn test_init_creates_valid_manifest() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");

    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("init")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    let content = fs::read_to_string(config_dir.join("manifest.toml")).unwrap();
    let _manifest: toml::Value = toml::from_str(&content).expect("manifest should be valid TOML");
}

#[test]
fn test_init_creates_valid_config() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");

    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("init")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    let content = fs::read_to_string(config_dir.join("config.toml")).unwrap();
    let _config: toml::Value = toml::from_str(&content).expect("config should be valid TOML");
}

#[test]
fn test_init_twice_succeeds() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");

    // First init succeeds
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("init")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized dotsmith"));

    // Second init also succeeds (idempotent)
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("init")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("already initialized"));
}

#[test]
fn test_init_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");

    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("init")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // Check directory permissions
    let dir_mode = fs::metadata(&config_dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(dir_mode, 0o700, "config dir should be 0700");

    // Check file permissions
    let manifest_mode = fs::metadata(config_dir.join("manifest.toml"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(manifest_mode, 0o600, "manifest.toml should be 0600");
}

#[test]
fn test_auto_init_on_list() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");

    // Run list without prior init — should auto-initialize and succeed
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("list")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // Verify config files were created
    assert!(config_dir.join("manifest.toml").exists());
    assert!(config_dir.join("config.toml").exists());
}

#[test]
fn test_auto_init_on_status() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");

    // Run status without prior init — should auto-initialize and succeed
    Command::cargo_bin("dotsmith")
        .unwrap()
        .arg("status")
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    assert!(config_dir.join("manifest.toml").exists());
    assert!(config_dir.join("config.toml").exists());
}
