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
fn test_deploy_remote_not_initialized() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith-noinit");

    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["deploy-remote", "example.com", "--dry-run"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn test_deploy_remote_empty_manifest() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("dotsmith");
    init_dotsmith(&config_dir);

    // With empty manifest, should either say "No files" or fail on ssh check
    let output = Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["deploy-remote", "example.com", "--dry-run"])
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either "No files to deploy" (ssh available) or "ssh" error (ssh not available)
    assert!(
        stdout.contains("No files") || stderr.contains("ssh"),
        "Expected 'No files' or 'ssh' error, got stdout: {}, stderr: {}",
        stdout,
        stderr,
    );
}

#[test]
fn test_deploy_remote_help() {
    Command::cargo_bin("dotsmith")
        .unwrap()
        .args(["deploy-remote", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("remote host"))
        .stdout(predicate::str::contains("--dry-run"));
}
