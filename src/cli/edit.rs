use std::fs;
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;
use sha2::{Digest, Sha256};

use crate::core::manifest::Manifest;
use crate::core::snapshot::SnapshotEngine;
use crate::util;

pub fn run(verbose: bool, tool: &str) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir)?;

    let entry = manifest
        .tools
        .get(tool)
        .ok_or_else(|| anyhow::anyhow!("'{}' is not tracked by dotsmith", tool))?;

    // Find first config file (not directory)
    let file_path = pick_first_file(&entry.config_paths)
        .ok_or_else(|| anyhow::anyhow!("no config file found for '{}'", tool))?;

    if verbose {
        println!("  editing {}", file_path.display());
    }

    // Auto-snapshot before editing
    let snapshot_engine = SnapshotEngine::open(&config_dir)?;
    match snapshot_engine.snapshot_tool(tool, &entry.config_paths, Some("pre-edit snapshot")) {
        Ok(count) => {
            if verbose {
                println!("  snapshotted {} file(s) before editing", count);
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("  warning: pre-edit snapshot failed: {}", e);
            }
        }
    }

    // Hash before editing
    let hash_before = hash_file(&file_path);

    // Open editor
    let editor = find_editor();
    if verbose {
        println!("  using editor: {}", editor);
    }

    let status = Command::new(&editor)
        .arg(&file_path)
        .status()
        .with_context(|| format!("failed to launch editor '{}'", editor))?;

    if !status.success() {
        anyhow::bail!("editor exited with non-zero status");
    }

    // Check if file changed
    let hash_after = hash_file(&file_path);
    if hash_before != hash_after {
        println!(
            "  {} modified — run {} to review",
            util::paths::contract_tilde(&file_path).bold(),
            format!("dotsmith diff {}", tool).cyan()
        );
    } else {
        println!("  no changes detected");
    }

    Ok(())
}

/// Find the user's preferred editor.
fn find_editor() -> String {
    if let Ok(editor) = std::env::var("EDITOR") {
        if !editor.is_empty() {
            return editor;
        }
    }
    if let Ok(visual) = std::env::var("VISUAL") {
        if !visual.is_empty() {
            return visual;
        }
    }
    "vi".to_string()
}

/// Pick the first config file (not directory) from the list.
fn pick_first_file(config_paths: &[String]) -> Option<std::path::PathBuf> {
    for path_str in config_paths {
        let path = util::paths::expand_tilde(path_str);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

/// Compute SHA-256 hash of a file, or empty string if unreadable.
fn hash_file(path: &std::path::Path) -> String {
    match fs::read(path) {
        Ok(content) => {
            let mut hasher = Sha256::new();
            hasher.update(&content);
            format!("{:x}", hasher.finalize())
        }
        Err(_) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_editor_env() {
        let original = std::env::var("EDITOR").ok();
        // SAFETY: Test isolation — no concurrent tests depend on EDITOR
        unsafe {
            std::env::set_var("EDITOR", "nano");
        }
        assert_eq!(find_editor(), "nano");
        unsafe {
            match original {
                Some(val) => std::env::set_var("EDITOR", val),
                None => std::env::remove_var("EDITOR"),
            }
        }
    }

    #[test]
    fn test_pick_first_file() {
        let tmp = TempDir::new().unwrap();
        let dir_path = tmp.path().join("subdir");
        std::fs::create_dir(&dir_path).unwrap();
        let file_path = tmp.path().join("config.conf");
        std::fs::write(&file_path, "test").unwrap();

        let paths = vec![
            dir_path.to_string_lossy().to_string(),
            file_path.to_string_lossy().to_string(),
        ];
        let result = pick_first_file(&paths);
        assert_eq!(result, Some(file_path));
    }

    #[test]
    fn test_pick_first_file_empty() {
        let paths: Vec<String> = vec![];
        assert!(pick_first_file(&paths).is_none());
    }

    #[test]
    fn test_pick_first_file_no_files() {
        let paths = vec!["/nonexistent/path".to_string()];
        assert!(pick_first_file(&paths).is_none());
    }

    #[test]
    fn test_hash_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.txt");
        std::fs::write(&path, "hello").unwrap();

        let h1 = hash_file(&path);
        assert!(!h1.is_empty());

        // Same content = same hash
        let h2 = hash_file(&path);
        assert_eq!(h1, h2);

        // Different content = different hash
        std::fs::write(&path, "world").unwrap();
        let h3 = hash_file(&path);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_hash_file_missing() {
        let h = hash_file(std::path::Path::new("/nonexistent/file"));
        assert!(h.is_empty());
    }
}
