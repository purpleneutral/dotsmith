use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Result of a deploy operation for a single path.
#[derive(Debug)]
pub struct DeployAction {
    pub source: PathBuf,
    pub target: PathBuf,
    pub action: DeployActionType,
}

#[derive(Debug, PartialEq)]
pub enum DeployActionType {
    /// Create a new symlink (target doesn't exist)
    CreateSymlink,
    /// Symlink already points to the correct source
    AlreadyCorrect,
    /// Target exists as a regular file/dir — needs backup before linking
    BackupAndLink,
    /// Symlink exists but points elsewhere — needs relink
    Relink,
    /// Source doesn't exist — skip
    SourceMissing,
}

/// Plan a deploy operation for a tool without executing it.
/// This analyzes what would happen if we deployed symlinks.
///
/// `source_dir` is where the config files live (e.g., `~/.config/oz/dots/tmux`)
/// `target_dir` is where the symlinks should be created (e.g., `~/.config/tmux`)
pub fn plan_deploy(
    source_dir: &Path,
    target_dir: &Path,
) -> Result<Vec<DeployAction>> {
    let mut actions = Vec::new();

    if !source_dir.exists() {
        actions.push(DeployAction {
            source: source_dir.to_path_buf(),
            target: target_dir.to_path_buf(),
            action: DeployActionType::SourceMissing,
        });
        return Ok(actions);
    }

    // If the source is a single file, plan a single symlink
    if source_dir.is_file() {
        let action = classify_target(source_dir, target_dir);
        actions.push(DeployAction {
            source: source_dir.to_path_buf(),
            target: target_dir.to_path_buf(),
            action,
        });
        return Ok(actions);
    }

    // For directories, plan a directory-level symlink
    let action = classify_target(source_dir, target_dir);
    actions.push(DeployAction {
        source: source_dir.to_path_buf(),
        target: target_dir.to_path_buf(),
        action,
    });

    Ok(actions)
}

/// Classify what action is needed for a target path.
fn classify_target(source: &Path, target: &Path) -> DeployActionType {
    // Check symlink_metadata first (doesn't follow symlinks)
    match fs::symlink_metadata(target) {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                // Symlink exists — check if it points to the right place
                if let Ok(current_target) = fs::read_link(target) {
                    if current_target == source {
                        DeployActionType::AlreadyCorrect
                    } else {
                        DeployActionType::Relink
                    }
                } else {
                    DeployActionType::Relink
                }
            } else {
                // Regular file/dir exists at target — needs backup
                DeployActionType::BackupAndLink
            }
        }
        Err(_) => {
            // Target doesn't exist — create new symlink
            DeployActionType::CreateSymlink
        }
    }
}

/// Execute a deploy plan. Creates backups before modifying anything.
/// Returns a list of paths that were backed up.
pub fn execute_deploy(
    actions: &[DeployAction],
    backup_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let mut backed_up = Vec::new();

    for action in actions {
        match action.action {
            DeployActionType::CreateSymlink => {
                // Ensure parent directory exists
                if let Some(parent) = action.target.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("failed to create parent dir {}", parent.display())
                    })?;
                }

                unix_fs::symlink(&action.source, &action.target).with_context(|| {
                    format!(
                        "failed to create symlink {} -> {}",
                        action.target.display(),
                        action.source.display()
                    )
                })?;
            }
            DeployActionType::BackupAndLink => {
                // Backup existing file/dir
                let backup_path = backup_target(&action.target, backup_dir)?;
                backed_up.push(backup_path);

                // Create symlink
                unix_fs::symlink(&action.source, &action.target).with_context(|| {
                    format!(
                        "failed to create symlink {} -> {}",
                        action.target.display(),
                        action.source.display()
                    )
                })?;
            }
            DeployActionType::Relink => {
                // Remove old symlink and create new one
                fs::remove_file(&action.target).with_context(|| {
                    format!("failed to remove old symlink {}", action.target.display())
                })?;

                unix_fs::symlink(&action.source, &action.target).with_context(|| {
                    format!(
                        "failed to create symlink {} -> {}",
                        action.target.display(),
                        action.source.display()
                    )
                })?;
            }
            DeployActionType::AlreadyCorrect | DeployActionType::SourceMissing => {
                // No action needed
            }
        }
    }

    Ok(backed_up)
}

/// Backup a file or directory to the backup directory.
/// Returns the backup path.
fn backup_target(target: &Path, backup_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(backup_dir)
        .with_context(|| format!("failed to create backup dir {}", backup_dir.display()))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let backup_name = format!("{}.{}.bak", name, timestamp);
    let backup_path = backup_dir.join(backup_name);

    if target.is_dir() {
        // For directories, rename the whole thing
        fs::rename(target, &backup_path).with_context(|| {
            format!(
                "failed to backup directory {} to {}",
                target.display(),
                backup_path.display()
            )
        })?;
    } else {
        fs::rename(target, &backup_path).with_context(|| {
            format!(
                "failed to backup {} to {}",
                target.display(),
                backup_path.display()
            )
        })?;
    }

    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plan_deploy_new_symlink() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        let target = tmp.path().join("target");

        let actions = plan_deploy(&source, &target).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, DeployActionType::CreateSymlink);
    }

    #[test]
    fn test_plan_deploy_already_correct() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        let target = tmp.path().join("target");

        unix_fs::symlink(&source, &target).unwrap();

        let actions = plan_deploy(&source, &target).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, DeployActionType::AlreadyCorrect);
    }

    #[test]
    fn test_plan_deploy_backup_needed() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        let target = tmp.path().join("target");
        fs::create_dir_all(&target).unwrap();

        let actions = plan_deploy(&source, &target).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, DeployActionType::BackupAndLink);
    }

    #[test]
    fn test_plan_deploy_relink() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        let wrong = tmp.path().join("wrong");
        fs::create_dir_all(&wrong).unwrap();
        let target = tmp.path().join("target");
        unix_fs::symlink(&wrong, &target).unwrap();

        let actions = plan_deploy(&source, &target).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, DeployActionType::Relink);
    }

    #[test]
    fn test_plan_deploy_source_missing() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("nonexistent");
        let target = tmp.path().join("target");

        let actions = plan_deploy(&source, &target).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, DeployActionType::SourceMissing);
    }

    #[test]
    fn test_execute_deploy_create() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source");
        fs::write(&source, "content").unwrap();
        let target = tmp.path().join("target");
        let backup_dir = tmp.path().join("backups");

        let actions = vec![DeployAction {
            source: source.clone(),
            target: target.clone(),
            action: DeployActionType::CreateSymlink,
        }];

        let backed_up = execute_deploy(&actions, &backup_dir).unwrap();
        assert!(backed_up.is_empty());
        assert!(target.is_symlink());
        assert_eq!(fs::read_link(&target).unwrap(), source);
    }

    #[test]
    fn test_execute_deploy_backup_and_link() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source");
        fs::write(&source, "new content").unwrap();
        let target = tmp.path().join("target");
        fs::write(&target, "old content").unwrap();
        let backup_dir = tmp.path().join("backups");

        let actions = vec![DeployAction {
            source: source.clone(),
            target: target.clone(),
            action: DeployActionType::BackupAndLink,
        }];

        let backed_up = execute_deploy(&actions, &backup_dir).unwrap();
        assert_eq!(backed_up.len(), 1);
        assert!(target.is_symlink());
        // Original content should be in backup
        let backup_content = fs::read_to_string(&backed_up[0]).unwrap();
        assert_eq!(backup_content, "old content");
    }
}
