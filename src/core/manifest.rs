use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::errors::DotsmithError;
use crate::util;

/// The root manifest file tracking all tools managed by dotsmith.
/// Stored at `<config_dir>/manifest.toml`.
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Manifest {
    #[serde(default)]
    pub tools: BTreeMap<String, ToolEntry>,
}

/// A single tool entry in the manifest.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ToolEntry {
    /// Support tier (1 = full, 2 = auto-detected, 3 = user-enriched)
    pub tier: u8,

    /// Paths to tracked config files/directories.
    /// Stored as tilde-contracted paths for portability.
    pub config_paths: Vec<String>,

    /// Whether dotsmith manages plugins for this tool.
    pub plugins_managed: bool,

    /// If an external plugin manager is detected, its name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_manager: Option<String>,

    /// When this tool was added to dotsmith.
    pub added_at: DateTime<Utc>,

    /// When the last snapshot was taken.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_snapshot: Option<DateTime<Utc>>,
}

impl Manifest {
    /// Load manifest from the dotsmith config directory.
    /// Returns `NotInitialized` error if the manifest file doesn't exist.
    pub fn load(config_dir: &Path) -> Result<Self> {
        let path = config_dir.join("manifest.toml");
        if !path.exists() {
            return Err(DotsmithError::NotInitialized.into());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let manifest: Manifest = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(manifest)
    }

    /// Write manifest to disk atomically (write to .tmp, then rename).
    /// Creates parent directories as needed. Sets file permissions to 0600.
    pub fn save(&self, config_dir: &Path) -> Result<()> {
        fs::create_dir_all(config_dir)
            .with_context(|| format!("failed to create {}", config_dir.display()))?;

        let path = config_dir.join("manifest.toml");
        let content = toml::to_string_pretty(self).context("failed to serialize manifest")?;

        util::fs::atomic_write(&path, &content)
            .context("failed to write manifest.toml")?;

        Ok(())
    }

    /// Add a tool entry. Returns error if the tool is already tracked.
    pub fn add_tool(&mut self, name: &str, entry: ToolEntry) -> Result<()> {
        if self.tools.contains_key(name) {
            return Err(DotsmithError::ToolAlreadyTracked(name.to_string()).into());
        }
        self.tools.insert(name.to_string(), entry);
        Ok(())
    }

    /// Remove a tool entry. Returns the removed entry, or error if not tracked.
    pub fn remove_tool(&mut self, name: &str) -> Result<ToolEntry> {
        self.tools
            .remove(name)
            .ok_or_else(|| DotsmithError::ToolNotTracked(name.to_string()).into())
    }

    /// Check if a tool is tracked.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get a reference to a tool entry.
    #[allow(dead_code)]
    pub fn get_tool(&self, name: &str) -> Option<&ToolEntry> {
        self.tools.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn sample_entry() -> ToolEntry {
        ToolEntry {
            tier: 1,
            config_paths: vec!["~/.config/tmux/tmux.conf".to_string()],
            plugins_managed: false,
            plugin_manager: Some("tpm".to_string()),
            added_at: Utc::now(),
            last_snapshot: None,
        }
    }

    #[test]
    fn test_manifest_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let mut manifest = Manifest::default();
        manifest.add_tool("tmux", sample_entry()).unwrap();

        manifest.save(tmp.path()).unwrap();
        let loaded = Manifest::load(tmp.path()).unwrap();

        assert_eq!(manifest, loaded);
    }

    #[test]
    fn test_add_tool() {
        let mut manifest = Manifest::default();
        manifest.add_tool("tmux", sample_entry()).unwrap();

        assert!(manifest.has_tool("tmux"));
        assert_eq!(manifest.get_tool("tmux").unwrap().tier, 1);
    }

    #[test]
    fn test_add_duplicate_tool() {
        let mut manifest = Manifest::default();
        manifest.add_tool("tmux", sample_entry()).unwrap();

        let result = manifest.add_tool("tmux", sample_entry());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already managed"));
    }

    #[test]
    fn test_remove_tool() {
        let mut manifest = Manifest::default();
        manifest.add_tool("tmux", sample_entry()).unwrap();

        let removed = manifest.remove_tool("tmux").unwrap();
        assert_eq!(removed.tier, 1);
        assert!(!manifest.has_tool("tmux"));
    }

    #[test]
    fn test_remove_nonexistent_tool() {
        let mut manifest = Manifest::default();
        let result = manifest.remove_tool("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not managed"));
    }

    #[test]
    fn test_load_missing_manifest() {
        let tmp = TempDir::new().unwrap();
        let result = Manifest::load(tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not initialized"));
    }

    #[test]
    fn test_empty_manifest() {
        let tmp = TempDir::new().unwrap();
        let manifest = Manifest::default();
        manifest.save(tmp.path()).unwrap();
        let loaded = Manifest::load(tmp.path()).unwrap();
        assert!(loaded.tools.is_empty());
    }

    #[test]
    fn test_manifest_file_permissions() {
        let tmp = TempDir::new().unwrap();
        let manifest = Manifest::default();
        manifest.save(tmp.path()).unwrap();

        let path = tmp.path().join("manifest.toml");
        let metadata = fs::metadata(&path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "manifest.toml should have 0600 permissions");
    }

    #[test]
    fn test_tools_sorted_alphabetically() {
        let mut manifest = Manifest::default();
        manifest.add_tool("zsh", sample_entry()).unwrap();
        manifest.add_tool("git", sample_entry()).unwrap();
        manifest.add_tool("tmux", sample_entry()).unwrap();

        let names: Vec<&String> = manifest.tools.keys().collect();
        assert_eq!(names, vec!["git", "tmux", "zsh"]);
    }
}
