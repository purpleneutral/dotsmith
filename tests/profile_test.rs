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

/// Create a fake tool config and add it to dotsmith.
fn add_fake_tool(config_dir: &std::path::Path, tool_file: &std::path::Path) {
    // Write a manifest entry manually since we can't use `add` without a real tool
    let manifest_path = config_dir.join("manifest.toml");
    let manifest = format!(
        r#"[tools.faketool]
tier = 2
config_paths = ["{}"]
plugins_managed = false
added_at = "2026-01-01T00:00:00Z"
"#,
        tool_file.display()
    );
    fs::write(&manifest_path, manifest).unwrap();
}

#[test]
fn test_profile_save() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    // Create a fake config file and add it
    let tool_file = tmp.path().join("fake.conf");
    fs::write(&tool_file, "setting = true\n").unwrap();
    add_fake_tool(&config_dir, &tool_file);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "save", "test-profile"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Saved profile"))
        .stdout(predicate::str::contains("test-profile"));
}

#[test]
fn test_profile_list() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    let tool_file = tmp.path().join("fake.conf");
    fs::write(&tool_file, "setting = true\n").unwrap();
    add_fake_tool(&config_dir, &tool_file);

    // Save a profile
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "save", "my-setup"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // List should show it
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "list"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("my-setup"));
}

#[test]
fn test_profile_list_empty() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "list"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No profiles"));
}

#[test]
fn test_profile_load() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    let tool_file = tmp.path().join("fake.conf");
    fs::write(&tool_file, "original = true\n").unwrap();
    add_fake_tool(&config_dir, &tool_file);

    // Save profile with original content
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "save", "restore-test"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // Modify the file
    fs::write(&tool_file, "modified = true\n").unwrap();

    // Load profile to restore
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "load", "restore-test"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded profile"))
        .stdout(predicate::str::contains("restored"));

    // Verify file was restored
    let content = fs::read_to_string(&tool_file).unwrap();
    assert_eq!(content, "original = true\n");
}

#[test]
fn test_profile_load_dry_run() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    let tool_file = tmp.path().join("fake.conf");
    fs::write(&tool_file, "original = true\n").unwrap();
    add_fake_tool(&config_dir, &tool_file);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "save", "dry-test"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    // Modify the file
    fs::write(&tool_file, "modified = true\n").unwrap();

    // Dry run should not restore
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "load", "dry-test", "--dry-run"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // Verify file was NOT restored
    let content = fs::read_to_string(&tool_file).unwrap();
    assert_eq!(content, "modified = true\n");
}

#[test]
fn test_profile_delete() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    let tool_file = tmp.path().join("fake.conf");
    fs::write(&tool_file, "setting = true\n").unwrap();
    add_fake_tool(&config_dir, &tool_file);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "save", "to-delete"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "delete", "to-delete"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted profile"));

    // List should be empty now
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "list"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("No profiles"));
}

#[test]
fn test_profile_save_duplicate_fails() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    let tool_file = tmp.path().join("fake.conf");
    fs::write(&tool_file, "setting = true\n").unwrap();
    add_fake_tool(&config_dir, &tool_file);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "save", "dupe"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .success();

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "save", "dupe"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_profile_load_nonexistent_fails() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["profile", "load", "nope"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
