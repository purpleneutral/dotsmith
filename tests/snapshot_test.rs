use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn dotsmith() -> Command {
    Command::cargo_bin("dotsmith").unwrap()
}

fn init_env(tmp: &TempDir) -> String {
    let config_dir = tmp.path().join("config");
    let dir_str = config_dir.display().to_string();

    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &dir_str)
        .args(["init"])
        .assert()
        .success();

    dir_str
}

fn add_tool_with_config(tmp: &TempDir, config_dir: &str, tool: &str) -> String {
    // Create a config file for the tool
    let tool_dir = tmp.path().join(format!("dot-{}", tool));
    fs::create_dir_all(&tool_dir).unwrap();
    let conf_path = tool_dir.join("config.conf");
    fs::write(&conf_path, format!("# {} config\noption1 = true\n", tool)).unwrap();

    // Manually write the manifest to track this tool
    let manifest_path = format!("{}/manifest.toml", config_dir);
    let path_str = conf_path.display().to_string();
    let manifest_content = format!(
        r#"[tools.{}]
tier = 2
config_paths = ["{}"]
plugins_managed = false
added_at = "2026-01-01T00:00:00Z"
"#,
        tool, path_str
    );
    fs::write(&manifest_path, &manifest_content).unwrap();

    path_str
}

#[test]
fn test_snapshot_tool() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);
    let _conf_path = add_tool_with_config(&tmp, &config_dir, "testtool");

    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["snapshot", "testtool", "-m", "initial snapshot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Snapshotted"));
}

#[test]
fn test_snapshot_all() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);
    let _conf_path = add_tool_with_config(&tmp, &config_dir, "testtool");

    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["snapshot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Snapshotted"));
}

#[test]
fn test_snapshot_dedup() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);
    let _conf_path = add_tool_with_config(&tmp, &config_dir, "testtool");

    // First snapshot
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["snapshot", "testtool", "-m", "first"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Snapshotted"));

    // Second snapshot with same content — should report no changes
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["snapshot", "testtool", "-m", "second"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes"));
}

#[test]
fn test_history() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);
    let _conf_path = add_tool_with_config(&tmp, &config_dir, "testtool");

    // Take a snapshot
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["snapshot", "testtool", "-m", "test snapshot"])
        .assert()
        .success();

    // Check history
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["history", "testtool"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#1"))
        .stdout(predicate::str::contains("test snapshot"));
}

#[test]
fn test_history_empty() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);
    let _conf_path = add_tool_with_config(&tmp, &config_dir, "testtool");

    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["history", "testtool"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No snapshots"));
}

#[test]
fn test_diff_no_snapshot() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);
    let _conf_path = add_tool_with_config(&tmp, &config_dir, "testtool");

    // Diff with no prior snapshot — shows full file as new
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["diff", "testtool"])
        .assert()
        .success();
}

#[test]
fn test_diff_after_change() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);
    let conf_path = add_tool_with_config(&tmp, &config_dir, "testtool");

    // Take initial snapshot
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["snapshot", "testtool"])
        .assert()
        .success();

    // Modify the file
    fs::write(&conf_path, "# modified config\noption1 = false\n").unwrap();

    // Diff should show changes
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["diff", "testtool"])
        .assert()
        .success()
        .stdout(predicate::str::contains("config.conf"));
}

#[test]
fn test_rollback_dry_run() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);
    let conf_path = add_tool_with_config(&tmp, &config_dir, "testtool");

    // Snapshot original
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["snapshot", "testtool", "-m", "original"])
        .assert()
        .success();

    // Modify the file
    fs::write(&conf_path, "# modified\n").unwrap();

    // Dry-run rollback should not modify the file
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["rollback", "1", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // File should still have modified content
    let content = fs::read_to_string(&conf_path).unwrap();
    assert_eq!(content, "# modified\n");
}

#[test]
fn test_rollback_execute() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);
    let conf_path = add_tool_with_config(&tmp, &config_dir, "testtool");

    let original_content = fs::read_to_string(&conf_path).unwrap();

    // Snapshot original
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["snapshot", "testtool", "-m", "original"])
        .assert()
        .success();

    // Modify the file
    fs::write(&conf_path, "# modified\n").unwrap();

    // Rollback to snapshot #1
    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["rollback", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rolled back"));

    // File should be back to original
    let restored = fs::read_to_string(&conf_path).unwrap();
    assert_eq!(restored, original_content);

    // Backup should exist
    let backup_dir = format!("{}/backups", config_dir);
    assert!(std::path::Path::new(&backup_dir).exists());
}

#[test]
fn test_deploy_dry_run() {
    // Deploy tests need paths within $HOME for safety check
    let home = dirs::home_dir().expect("HOME must be set");
    let tmp = TempDir::new_in(&home).unwrap();
    let config_dir = init_env(&tmp);

    let source = tmp.path().join("source_dir");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("config.conf"), "content").unwrap();

    let target = tmp.path().join("target_link");

    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args([
            "deploy",
            &source.display().to_string(),
            &target.display().to_string(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // Target should NOT exist (dry run)
    assert!(!target.exists());
}

#[test]
fn test_deploy_creates_symlink() {
    // Deploy tests need paths within $HOME for safety check
    let home = dirs::home_dir().expect("HOME must be set");
    let tmp = TempDir::new_in(&home).unwrap();
    let config_dir = init_env(&tmp);

    let source = tmp.path().join("source_dir");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("config.conf"), "content").unwrap();

    let target = tmp.path().join("target_link");

    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args([
            "deploy",
            &source.display().to_string(),
            &target.display().to_string(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deploy complete"));

    // Target should be a symlink to source
    assert!(target.is_symlink());
    assert_eq!(fs::read_link(&target).unwrap(), source);
}

#[test]
fn test_snapshot_untracked_tool_fails() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);

    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["snapshot", "nonexistent"])
        .assert()
        .failure();
}

#[test]
fn test_reload_untracked_tool_fails() {
    let tmp = TempDir::new().unwrap();
    let config_dir = init_env(&tmp);

    dotsmith()
        .env("DOTSMITH_CONFIG_DIR", &config_dir)
        .args(["reload", "nonexistent"])
        .assert()
        .failure();
}
