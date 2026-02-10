use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::core::errors::DotsmithError;
use crate::core::manifest::Manifest;
use crate::util;

/// A planned remote deploy action for a single file.
#[derive(Debug)]
pub struct RemoteDeployAction {
    pub local_path: PathBuf,
    pub remote_path: String,
    pub tool: String,
    pub remote_exists: bool,
}

/// Summary returned after executing a remote deploy.
#[derive(Debug)]
pub struct RemoteDeployResult {
    pub files_copied: usize,
    pub files_backed_up: usize,
    pub files_skipped: usize,
}

/// Options for a remote deploy operation.
pub struct RemoteDeployOpts<'a> {
    pub host: &'a str,
    pub user: Option<&'a str>,
    pub tools: Option<Vec<&'a str>>,
    pub dry_run: bool,
}

/// Build the SSH destination string.
fn ssh_dest(host: &str, user: Option<&str>) -> String {
    match user {
        Some(u) => format!("{}@{}", u, host),
        None => host.to_string(),
    }
}

/// Check if ssh is available on the system.
fn check_ssh_installed() -> Result<()> {
    let status = Command::new("ssh")
        .arg("-V")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(DotsmithError::SshNotInstalled.into()),
    }
}

/// Check if a remote file exists.
fn remote_file_exists(dest: &str, remote_path: &str) -> bool {
    Command::new("ssh")
        .args([
            "-o", "BatchMode=yes",
            "-o", "ConnectTimeout=5",
            dest,
            &format!("test -e '{}'", remote_path),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Create a backup of a remote file.
fn remote_backup(dest: &str, remote_path: &str) -> Result<String> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = format!("{}.dotsmith-bak.{}", remote_path, timestamp);

    let status = Command::new("ssh")
        .args([
            "-o", "BatchMode=yes",
            dest,
            &format!("cp -a '{}' '{}'", remote_path, backup_path),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("failed to run ssh for backup")?;

    if !status.success() {
        anyhow::bail!("failed to backup remote file: {}", remote_path);
    }

    Ok(backup_path)
}

/// Ensure a remote directory exists.
fn remote_mkdir_p(dest: &str, remote_dir: &str) -> Result<()> {
    let status = Command::new("ssh")
        .args([
            "-o", "BatchMode=yes",
            dest,
            &format!("mkdir -p '{}'", remote_dir),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("failed to run ssh for mkdir")?;

    if !status.success() {
        anyhow::bail!("failed to create remote directory: {}", remote_dir);
    }

    Ok(())
}

/// Copy a local file to the remote host via scp.
fn scp_file(local_path: &Path, dest: &str, remote_path: &str) -> Result<()> {
    let status = Command::new("scp")
        .args([
            "-q",
            "-o", "BatchMode=yes",
            &local_path.to_string_lossy(),
            &format!("{}:{}", dest, remote_path),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("failed to run scp")?;

    if !status.success() {
        return Err(DotsmithError::ScpFailed(
            dest.to_string(),
            local_path.display().to_string(),
            "scp exited with non-zero status".to_string(),
        )
        .into());
    }

    Ok(())
}

/// Collect files from a directory for deployment.
fn collect_dir_files(dir: &Path, tool_name: &str, config_path: &str) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();

    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };

    let dir_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // The config_path is tilde-contracted (e.g., "~/.config/tmux")
    // Files inside get remote paths like "~/.config/tmux/file.conf"
    let _ = tool_name; // used for display, not path construction

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            // Remote path is the dir path + filename
            let remote = if config_path.ends_with('/') {
                format!("{}{}", config_path, file_name)
            } else {
                format!("{}/{}", config_path, file_name)
            };
            files.push((path, remote));
        }
    }

    let _ = dir_name;
    files
}

/// Plan a remote deploy operation.
pub fn plan_remote_deploy(
    manifest: &Manifest,
    opts: &RemoteDeployOpts,
) -> Result<Vec<RemoteDeployAction>> {
    check_ssh_installed()?;

    let dest = ssh_dest(opts.host, opts.user);
    let mut actions = Vec::new();

    let tools: Vec<(&String, &crate::core::manifest::ToolEntry)> = match &opts.tools {
        Some(filter) => manifest
            .tools
            .iter()
            .filter(|(name, _)| filter.contains(&name.as_str()))
            .collect(),
        None => manifest.tools.iter().collect(),
    };

    for (tool_name, entry) in &tools {
        for config_path in &entry.config_paths {
            let local = util::paths::expand_tilde(config_path);

            if local.is_file() {
                let exists = if !opts.dry_run {
                    remote_file_exists(&dest, config_path)
                } else {
                    // In dry-run, still check for accurate display
                    remote_file_exists(&dest, config_path)
                };

                actions.push(RemoteDeployAction {
                    local_path: local,
                    remote_path: config_path.clone(),
                    tool: tool_name.to_string(),
                    remote_exists: exists,
                });
            } else if local.is_dir() {
                let dir_files = collect_dir_files(&local, tool_name, config_path);
                for (file_path, remote_path) in dir_files {
                    let exists = remote_file_exists(&dest, &remote_path);

                    actions.push(RemoteDeployAction {
                        local_path: file_path,
                        remote_path,
                        tool: tool_name.to_string(),
                        remote_exists: exists,
                    });
                }
            }
            // Skip missing local files silently
        }
    }

    Ok(actions)
}

/// Execute a remote deploy plan.
pub fn execute_remote_deploy(
    actions: &[RemoteDeployAction],
    host: &str,
    user: Option<&str>,
) -> Result<RemoteDeployResult> {
    let dest = ssh_dest(host, user);
    let mut result = RemoteDeployResult {
        files_copied: 0,
        files_backed_up: 0,
        files_skipped: 0,
    };

    for action in actions {
        // Ensure remote parent directory exists
        if let Some(parent) = Path::new(&action.remote_path).parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() {
                remote_mkdir_p(&dest, &parent_str)?;
            }
        }

        // Backup existing remote file
        if action.remote_exists {
            remote_backup(&dest, &action.remote_path)?;
            result.files_backed_up += 1;
        }

        // Copy file
        scp_file(&action.local_path, &dest, &action.remote_path)?;
        result.files_copied += 1;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_dest_with_user() {
        assert_eq!(ssh_dest("example.com", Some("alice")), "alice@example.com");
    }

    #[test]
    fn test_ssh_dest_without_user() {
        assert_eq!(ssh_dest("example.com", None), "example.com");
    }

    #[test]
    fn test_plan_empty_manifest() {
        let manifest = Manifest::default();
        let opts = RemoteDeployOpts {
            host: "example.com",
            user: None,
            tools: None,
            dry_run: true,
        };

        // This will fail if ssh is not installed, which is expected in CI
        // The plan itself should produce no actions for an empty manifest
        match plan_remote_deploy(&manifest, &opts) {
            Ok(actions) => assert!(actions.is_empty()),
            Err(e) => {
                // SSH not installed is acceptable in test environments
                assert!(e.to_string().contains("ssh"));
            }
        }
    }

    #[test]
    fn test_plan_with_tool_filter() {
        use crate::core::manifest::ToolEntry;
        use chrono::Utc;
        use std::collections::BTreeMap;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let file1 = tmp.path().join("tool1.conf");
        let file2 = tmp.path().join("tool2.conf");
        std::fs::write(&file1, "content1").unwrap();
        std::fs::write(&file2, "content2").unwrap();

        let mut manifest = Manifest::default();
        manifest
            .add_tool(
                "tool1",
                ToolEntry {
                    tier: 2,
                    config_paths: vec![file1.to_string_lossy().to_string()],
                    plugins_managed: false,
                    plugin_manager: None,
                    added_at: Utc::now(),
                    last_snapshot: None,
                    plugins: BTreeMap::new(),
                },
            )
            .unwrap();
        manifest
            .add_tool(
                "tool2",
                ToolEntry {
                    tier: 2,
                    config_paths: vec![file2.to_string_lossy().to_string()],
                    plugins_managed: false,
                    plugin_manager: None,
                    added_at: Utc::now(),
                    last_snapshot: None,
                    plugins: BTreeMap::new(),
                },
            )
            .unwrap();

        let opts = RemoteDeployOpts {
            host: "example.com",
            user: None,
            tools: Some(vec!["tool1"]),
            dry_run: true,
        };

        match plan_remote_deploy(&manifest, &opts) {
            Ok(actions) => {
                // Only tool1 should be included
                assert!(actions.iter().all(|a| a.tool == "tool1"));
            }
            Err(e) => {
                // SSH not installed is acceptable
                assert!(e.to_string().contains("ssh"));
            }
        }
    }

    #[test]
    fn test_plan_skips_missing_local_files() {
        use crate::core::manifest::ToolEntry;
        use chrono::Utc;
        use std::collections::BTreeMap;

        let mut manifest = Manifest::default();
        manifest
            .add_tool(
                "missing",
                ToolEntry {
                    tier: 2,
                    config_paths: vec!["/nonexistent/path/file.conf".to_string()],
                    plugins_managed: false,
                    plugin_manager: None,
                    added_at: Utc::now(),
                    last_snapshot: None,
                    plugins: BTreeMap::new(),
                },
            )
            .unwrap();

        let opts = RemoteDeployOpts {
            host: "example.com",
            user: None,
            tools: None,
            dry_run: true,
        };

        match plan_remote_deploy(&manifest, &opts) {
            Ok(actions) => {
                // Missing local files should be skipped
                assert!(actions.is_empty());
            }
            Err(e) => {
                assert!(e.to_string().contains("ssh"));
            }
        }
    }
}
