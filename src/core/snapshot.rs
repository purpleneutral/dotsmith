use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

use crate::core::manifest::Manifest;
use crate::util;

/// Summary of a single snapshot for display.
#[derive(Debug)]
pub struct SnapshotSummary {
    pub id: i64,
    #[allow(dead_code)]
    pub tool: String,
    pub file_path: String,
    pub hash: String,
    pub message: Option<String>,
    pub created_at: String,
}

/// A file diff between two states.
#[derive(Debug)]
pub struct FileDiff {
    pub file_path: String,
    pub old_content: String,
    pub new_content: String,
}

/// The snapshot engine manages point-in-time copies of config files in SQLite.
pub struct SnapshotEngine {
    conn: Connection,
}

impl SnapshotEngine {
    /// Open (or create) the snapshot database at `<config_dir>/snapshots.db`.
    /// Sets 0600 permissions on the DB file.
    pub fn open(config_dir: &Path) -> Result<Self> {
        let db_path = config_dir.join("snapshots.db");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("failed to open snapshot database at {}", db_path.display()))?;

        // Set file permissions to owner-only (may fail on first creation race, that's ok)
        if db_path.exists() {
            let _ = fs::set_permissions(&db_path, fs::Permissions::from_mode(0o600));
        }

        // Enable WAL mode for better concurrent reads
        conn.pragma_update(None, "journal_mode", "WAL")?;

        let engine = Self { conn };
        engine.init_schema()?;
        Ok(engine)
    }

    /// Create the schema if it doesn't exist.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                tool        TEXT NOT NULL,
                file_path   TEXT NOT NULL,
                content     TEXT NOT NULL,
                hash        TEXT NOT NULL,
                message     TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(tool, file_path, hash)
            );
            CREATE INDEX IF NOT EXISTS idx_snapshots_tool ON snapshots(tool);
            CREATE INDEX IF NOT EXISTS idx_snapshots_created ON snapshots(created_at);",
        )?;
        Ok(())
    }

    /// Take a snapshot of all config files for a tool.
    /// Returns the number of files snapshotted (skips unchanged files via hash dedup).
    pub fn snapshot_tool(
        &self,
        tool: &str,
        config_paths: &[String],
        message: Option<&str>,
    ) -> Result<usize> {
        let mut count = 0;

        for path_str in config_paths {
            let path = util::paths::expand_tilde(path_str);

            if path.is_dir() {
                // Snapshot all files within the directory
                count += self.snapshot_directory(tool, &path, message)?;
            } else if path.is_file()
                && self.snapshot_file(tool, &path, message)?
            {
                count += 1;
            }
            // Skip non-existent paths silently (status command warns about these)
        }

        Ok(count)
    }

    /// Snapshot all config files in a directory (non-recursive for now).
    fn snapshot_directory(
        &self,
        tool: &str,
        dir: &Path,
        message: Option<&str>,
    ) -> Result<usize> {
        let mut count = 0;
        if !dir.is_dir() {
            return Ok(0);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && self.snapshot_file(tool, &path, message)?
            {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Snapshot a single file. Returns true if a new snapshot was created,
    /// false if the content hasn't changed since the last snapshot.
    fn snapshot_file(
        &self,
        tool: &str,
        path: &Path,
        message: Option<&str>,
    ) -> Result<bool> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        let hash = compute_hash(&content);
        let path_str = util::paths::contract_tilde(path);

        // INSERT OR IGNORE — skips if this exact content was already snapshotted
        let rows = self.conn.execute(
            "INSERT OR IGNORE INTO snapshots (tool, file_path, content, hash, message)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![tool, path_str, content, hash, message],
        )?;

        Ok(rows > 0)
    }

    /// Take snapshots of ALL tracked tools.
    pub fn snapshot_all(
        &self,
        manifest: &Manifest,
        message: Option<&str>,
    ) -> Result<usize> {
        let mut total = 0;
        for (name, entry) in &manifest.tools {
            total += self.snapshot_tool(name, &entry.config_paths, message)?;
        }
        Ok(total)
    }

    /// Get the diff between the current file state and the last snapshot for a tool.
    pub fn diff_current(&self, tool: &str, config_paths: &[String]) -> Result<Vec<FileDiff>> {
        let mut diffs = Vec::new();

        for path_str in config_paths {
            let path = util::paths::expand_tilde(path_str);

            if path.is_dir() {
                self.diff_directory(tool, &path, &mut diffs)?;
            } else if path.is_file()
                && let Some(diff) = self.diff_file(tool, &path)?
            {
                diffs.push(diff);
            }
        }

        Ok(diffs)
    }

    /// Diff all files in a directory against their last snapshots.
    fn diff_directory(
        &self,
        tool: &str,
        dir: &Path,
        diffs: &mut Vec<FileDiff>,
    ) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && let Some(diff) = self.diff_file(tool, &path)?
            {
                diffs.push(diff);
            }
        }

        Ok(())
    }

    /// Diff a single file against its last snapshot. Returns None if unchanged.
    fn diff_file(&self, tool: &str, path: &Path) -> Result<Option<FileDiff>> {
        let current_content = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        let path_str = util::paths::contract_tilde(path);

        // Get the last snapshot for this file
        let last_content: Option<String> = self
            .conn
            .query_row(
                "SELECT content FROM snapshots
                 WHERE tool = ?1 AND file_path = ?2
                 ORDER BY id DESC LIMIT 1",
                params![tool, path_str],
                |row| row.get(0),
            )
            .ok();

        let old_content = last_content.unwrap_or_default();

        if old_content == current_content {
            return Ok(None);
        }

        Ok(Some(FileDiff {
            file_path: path_str,
            old_content,
            new_content: current_content,
        }))
    }

    /// List snapshot history for a tool.
    pub fn history(&self, tool: &str, limit: usize) -> Result<Vec<SnapshotSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tool, file_path, hash, message, created_at
             FROM snapshots
             WHERE tool = ?1
             ORDER BY id DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![tool, limit as i64], |row| {
            Ok(SnapshotSummary {
                id: row.get(0)?,
                tool: row.get(1)?,
                file_path: row.get(2)?,
                hash: row.get(3)?,
                message: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        let mut summaries = Vec::new();
        for row in rows {
            summaries.push(row?);
        }

        Ok(summaries)
    }

    /// Get snapshot content by ID (for rollback).
    pub fn get_snapshot(&self, snapshot_id: i64) -> Result<Option<(String, String)>> {
        let result = self
            .conn
            .query_row(
                "SELECT file_path, content FROM snapshots WHERE id = ?1",
                params![snapshot_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .ok();

        Ok(result)
    }

    /// Rollback a file to a specific snapshot.
    /// Creates a backup of the current file first.
    pub fn rollback(&self, snapshot_id: i64, backup_dir: &Path) -> Result<String> {
        let (path_str, content) = self
            .get_snapshot(snapshot_id)?
            .ok_or_else(|| anyhow::anyhow!("snapshot {} not found", snapshot_id))?;

        let path = util::paths::expand_tilde(&path_str);

        // Create backup of current state
        if path.exists() {
            fs::create_dir_all(backup_dir)?;
            let backup_name = format!(
                "{}.{}.bak",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file"),
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            );
            let backup_path = backup_dir.join(backup_name);
            fs::copy(&path, &backup_path)
                .with_context(|| format!("failed to backup {}", path.display()))?;
        }

        // Write the snapshot content back
        util::fs::atomic_write(&path, &content)?;

        Ok(path_str)
    }
}

/// Compute SHA-256 hash of content.
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, SnapshotEngine, TempDir) {
        let config_tmp = TempDir::new().unwrap();
        let engine = SnapshotEngine::open(config_tmp.path()).unwrap();
        let files_tmp = TempDir::new().unwrap();
        (config_tmp, engine, files_tmp)
    }

    #[test]
    fn test_snapshot_and_history() {
        let (_config_tmp, engine, files_tmp) = setup();

        // Create a config file
        let conf = files_tmp.path().join("tmux.conf");
        fs::write(&conf, "set -g mouse on\n").unwrap();

        let path_str = util::paths::contract_tilde(&conf);

        // Take a snapshot
        let count = engine
            .snapshot_tool("tmux", &[path_str.clone()], Some("initial"))
            .unwrap();
        assert_eq!(count, 1);

        // History should have one entry
        let history = engine.history("tmux", 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].tool, "tmux");
        assert_eq!(history[0].message, Some("initial".to_string()));
    }

    #[test]
    fn test_snapshot_dedup() {
        let (_config_tmp, engine, files_tmp) = setup();

        let conf = files_tmp.path().join("tmux.conf");
        fs::write(&conf, "set -g mouse on\n").unwrap();
        let path_str = util::paths::contract_tilde(&conf);

        // First snapshot creates a new entry
        let count1 = engine
            .snapshot_tool("tmux", &[path_str.clone()], Some("first"))
            .unwrap();
        assert_eq!(count1, 1);

        // Second snapshot with same content is skipped (dedup via hash)
        let count2 = engine
            .snapshot_tool("tmux", &[path_str], Some("second"))
            .unwrap();
        assert_eq!(count2, 0);

        // Only one entry in history
        let history = engine.history("tmux", 10).unwrap();
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_snapshot_after_change() {
        let (_config_tmp, engine, files_tmp) = setup();

        let conf = files_tmp.path().join("tmux.conf");
        fs::write(&conf, "set -g mouse on\n").unwrap();
        let path_str = util::paths::contract_tilde(&conf);

        engine
            .snapshot_tool("tmux", &[path_str.clone()], Some("v1"))
            .unwrap();

        // Change the file
        fs::write(&conf, "set -g mouse off\n").unwrap();

        let count = engine
            .snapshot_tool("tmux", &[path_str], Some("v2"))
            .unwrap();
        assert_eq!(count, 1);

        let history = engine.history("tmux", 10).unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_diff_current() {
        let (_config_tmp, engine, files_tmp) = setup();

        let conf = files_tmp.path().join("tmux.conf");
        fs::write(&conf, "set -g mouse on\n").unwrap();
        let path_str = util::paths::contract_tilde(&conf);

        // Snapshot the original
        engine
            .snapshot_tool("tmux", &[path_str.clone()], None)
            .unwrap();

        // No diff when unchanged
        let diffs = engine.diff_current("tmux", &[path_str.clone()]).unwrap();
        assert!(diffs.is_empty());

        // Change the file
        fs::write(&conf, "set -g mouse off\n").unwrap();

        // Now there should be a diff
        let diffs = engine.diff_current("tmux", &[path_str]).unwrap();
        assert_eq!(diffs.len(), 1);
        assert!(diffs[0].old_content.contains("on"));
        assert!(diffs[0].new_content.contains("off"));
    }

    #[test]
    fn test_diff_no_snapshot() {
        let (_config_tmp, engine, files_tmp) = setup();

        let conf = files_tmp.path().join("tmux.conf");
        fs::write(&conf, "set -g mouse on\n").unwrap();
        let path_str = util::paths::contract_tilde(&conf);

        // Diff with no prior snapshot — should show full file as new
        let diffs = engine.diff_current("tmux", &[path_str]).unwrap();
        assert_eq!(diffs.len(), 1);
        assert!(diffs[0].old_content.is_empty());
    }

    #[test]
    fn test_rollback() {
        let (config_tmp, engine, files_tmp) = setup();

        let conf = files_tmp.path().join("tmux.conf");
        fs::write(&conf, "set -g mouse on\n").unwrap();
        let path_str = util::paths::contract_tilde(&conf);

        // Snapshot v1
        engine
            .snapshot_tool("tmux", &[path_str.clone()], Some("v1"))
            .unwrap();

        // Change file
        fs::write(&conf, "set -g mouse off\n").unwrap();

        // Snapshot v2
        engine
            .snapshot_tool("tmux", &[path_str], Some("v2"))
            .unwrap();

        // Rollback to v1 (id=1)
        let backup_dir = config_tmp.path().join("backups");
        engine.rollback(1, &backup_dir).unwrap();

        // File should be back to v1 content
        let content = fs::read_to_string(&conf).unwrap();
        assert_eq!(content, "set -g mouse on\n");

        // Backup should exist
        assert!(backup_dir.exists());
        let backups: Vec<_> = fs::read_dir(&backup_dir).unwrap().collect();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_db_permissions() {
        let config_tmp = TempDir::new().unwrap();
        let _engine = SnapshotEngine::open(config_tmp.path()).unwrap();

        let db_path = config_tmp.path().join("snapshots.db");
        let mode = fs::metadata(&db_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "snapshots.db should have 0600 permissions");
    }
}
