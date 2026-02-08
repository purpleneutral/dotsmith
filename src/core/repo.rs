use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::core::manifest::Manifest;
use crate::util;

/// Result of a repo sync operation.
#[derive(Debug)]
pub struct SyncResult {
    pub files_copied: usize,
    pub committed: bool,
}

/// Result of a repo status check.
#[derive(Debug)]
pub struct RepoStatus {
    pub initialized: bool,
    pub changed_files: usize,
}

/// Initialize a git repo at the given path for storing dotfile backups.
pub fn init_repo(repo_path: &Path) -> Result<()> {
    std::fs::create_dir_all(repo_path)
        .with_context(|| format!("Failed to create repo directory: {}", repo_path.display()))?;

    let git_dir = repo_path.join(".git");
    if git_dir.exists() {
        return Ok(());
    }

    let output = Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .context("Failed to run git init")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git init failed: {}", stderr.trim());
    }

    Ok(())
}

/// Sync tracked config files into the repo directory, then commit if changes exist.
pub fn sync_repo(repo_path: &Path, manifest: &Manifest) -> Result<SyncResult> {
    if !repo_path.join(".git").exists() {
        bail!(
            "No git repo at {}. Run `dotsmith repo init` first.",
            repo_path.display()
        );
    }

    let mut files_copied = 0;

    for (tool_name, entry) in &manifest.tools {
        let tool_dir = repo_path.join(tool_name);
        std::fs::create_dir_all(&tool_dir)?;

        for config_path in &entry.config_paths {
            let expanded = util::paths::expand_tilde(config_path);
            let src = Path::new(&expanded);
            if !src.exists() {
                continue;
            }

            if src.is_dir() {
                copy_dir_recursive(src, &tool_dir)?;
                files_copied += 1;
            } else {
                let file_name = src
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| config_path.clone());
                let dest = tool_dir.join(&file_name);
                std::fs::copy(src, &dest).with_context(|| {
                    format!("Failed to copy {} to {}", src.display(), dest.display())
                })?;
                files_copied += 1;
            }
        }
    }

    // git add -A
    let output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo_path)
        .output()
        .context("Failed to run git add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git add failed: {}", stderr.trim());
    }

    // Check if there are staged changes
    let status = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(repo_path)
        .status()
        .context("Failed to run git diff --cached")?;

    if status.success() {
        // No changes to commit
        return Ok(SyncResult {
            files_copied,
            committed: false,
        });
    }

    // Commit
    let msg = format!(
        "dotsmith sync: {} tool(s), {} file(s)",
        manifest.tools.len(),
        files_copied
    );
    let output = Command::new("git")
        .args(["commit", "-m", &msg])
        .current_dir(repo_path)
        .output()
        .context("Failed to run git commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git commit failed: {}", stderr.trim());
    }

    Ok(SyncResult {
        files_copied,
        committed: true,
    })
}

/// Get the status of the repo (number of changed files).
pub fn repo_status(repo_path: &Path) -> Result<RepoStatus> {
    if !repo_path.join(".git").exists() {
        return Ok(RepoStatus {
            initialized: false,
            changed_files: 0,
        });
    }

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .context("Failed to run git status")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let changed_files = stdout.lines().filter(|l| !l.is_empty()).count();

    Ok(RepoStatus {
        initialized: true,
        changed_files,
    })
}

/// Recursively copy a directory's contents into a target directory.
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let target = dest.join(&file_name);

        if path.is_dir() {
            // Skip .git directories
            if file_name == ".git" {
                continue;
            }
            std::fs::create_dir_all(&target)?;
            copy_dir_recursive(&path, &target)?;
        } else {
            std::fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_repo_creates_git_dir() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("test-repo");
        init_repo(&repo_path).unwrap();
        assert!(repo_path.join(".git").exists());
    }

    #[test]
    fn test_init_repo_idempotent() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("test-repo");
        init_repo(&repo_path).unwrap();
        init_repo(&repo_path).unwrap(); // second call should be fine
        assert!(repo_path.join(".git").exists());
    }

    #[test]
    fn test_sync_repo_no_repo() {
        let tmp = TempDir::new().unwrap();
        let result = sync_repo(tmp.path(), &Manifest::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_repo_empty_manifest() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("repo");
        init_repo(&repo_path).unwrap();

        // Need an initial commit for git diff --cached to work
        std::fs::write(repo_path.join(".gitkeep"), "").unwrap();
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        let result = sync_repo(&repo_path, &Manifest::default()).unwrap();
        assert_eq!(result.files_copied, 0);
        assert!(!result.committed);
    }

    #[test]
    fn test_sync_repo_with_file() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("repo");
        init_repo(&repo_path).unwrap();

        // Create a config file to sync
        let config_dir = tmp.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_file = config_dir.join("test.conf");
        std::fs::write(&config_file, "hello = world\n").unwrap();

        // Build manifest pointing to the file
        use crate::core::manifest::{Manifest, ToolEntry};
        use chrono::Utc;
        use std::collections::BTreeMap;

        let mut manifest = Manifest::default();
        manifest.tools.insert(
            "test".to_string(),
            ToolEntry {
                tier: 2,
                config_paths: vec![config_file.to_string_lossy().to_string()],
                plugins_managed: false,
                plugin_manager: None,
                added_at: Utc::now(),
                last_snapshot: None,
                plugins: BTreeMap::new(),
            },
        );

        let result = sync_repo(&repo_path, &manifest).unwrap();
        assert_eq!(result.files_copied, 1);
        assert!(result.committed);

        // Second sync with no changes
        let result2 = sync_repo(&repo_path, &manifest).unwrap();
        assert!(!result2.committed);
    }

    #[test]
    fn test_repo_status_no_repo() {
        let tmp = TempDir::new().unwrap();
        let status = repo_status(tmp.path()).unwrap();
        assert!(!status.initialized);
    }

    #[test]
    fn test_repo_status_clean() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("repo");
        init_repo(&repo_path).unwrap();

        let status = repo_status(&repo_path).unwrap();
        assert!(status.initialized);
    }
}
