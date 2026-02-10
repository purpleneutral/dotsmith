use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use sha2::{Digest, Sha256};

use crate::core::manifest::Manifest;
use crate::core::snapshot::SnapshotEngine;
use crate::util;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

struct FileState {
    mtime: SystemTime,
    hash: String,
    tool: String,
}

pub fn run(verbose: bool, tool: Option<&str>) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir)?;

    // Collect files to watch
    let tools_to_watch: Vec<(&String, &crate::core::manifest::ToolEntry)> = match tool {
        Some(name) => {
            manifest
                .tools
                .get_key_value(name)
                .map(|(k, v)| vec![(k, v)])
                .ok_or_else(|| anyhow::anyhow!("'{}' is not tracked by dotsmith", name))?
        }
        None => manifest.tools.iter().collect(),
    };

    // Build initial file state
    let mut state = build_file_state(&tools_to_watch);

    let file_count = state.len();
    let tool_count = tools_to_watch.len();

    if file_count == 0 {
        println!("No config files to watch.");
        return Ok(());
    }

    println!(
        "  Watching {} file(s) for {} tool(s)... (Ctrl-C to stop)",
        file_count.to_string().cyan(),
        tool_count.to_string().cyan()
    );

    if verbose {
        for (path, fs) in &state {
            println!("    {} [{}]", util::paths::contract_tilde(path), fs.tool);
        }
    }

    println!();

    let snapshot_engine = SnapshotEngine::open(&config_dir)?;

    loop {
        std::thread::sleep(POLL_INTERVAL);

        let mut changed_tools: HashMap<String, Vec<String>> = HashMap::new();

        for (path, file_state) in state.iter_mut() {
            let Ok(metadata) = fs::metadata(path) else {
                continue;
            };
            let Ok(mtime) = metadata.modified() else {
                continue;
            };

            if mtime == file_state.mtime {
                continue;
            }

            // mtime changed — check hash
            let new_hash = hash_file(path);
            if new_hash == file_state.hash {
                file_state.mtime = mtime;
                continue;
            }

            // File actually changed
            let now = Local::now().format("%H:%M:%S");
            let display_path = util::paths::contract_tilde(path);
            println!(
                "  {} {} {} changed",
                format!("[{}]", now).dimmed(),
                file_state.tool.cyan(),
                display_path
            );

            changed_tools
                .entry(file_state.tool.clone())
                .or_default()
                .push(display_path);

            file_state.mtime = mtime;
            file_state.hash = new_hash;
        }

        // Snapshot changed tools
        for tool_name in changed_tools.keys() {
            if let Some(entry) = manifest.tools.get(tool_name.as_str()) {
                match snapshot_engine.snapshot_tool(
                    tool_name,
                    &entry.config_paths,
                    Some("auto-snapshot (watch)"),
                ) {
                    Ok(count) => {
                        let now = Local::now().format("%H:%M:%S");
                        println!(
                            "  {} {} snapshotted {} file(s)",
                            format!("[{}]", now).dimmed(),
                            "OK".green().bold(),
                            count
                        );
                    }
                    Err(e) => {
                        eprintln!("  {} snapshot failed for {}: {}", "!!".yellow(), tool_name, e);
                    }
                }
            }
        }
    }
}

fn build_file_state(
    tools: &[(&String, &crate::core::manifest::ToolEntry)],
) -> HashMap<PathBuf, FileState> {
    let mut state = HashMap::new();

    for (tool_name, entry) in tools {
        for path_str in &entry.config_paths {
            let path = util::paths::expand_tilde(path_str);
            if !path.is_file() {
                continue;
            }

            let mtime = fs::metadata(&path)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            let hash = hash_file(&path);

            state.insert(
                path,
                FileState {
                    mtime,
                    hash,
                    tool: tool_name.to_string(),
                },
            );
        }
    }

    state
}

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
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    fn sample_entry(paths: Vec<String>) -> crate::core::manifest::ToolEntry {
        crate::core::manifest::ToolEntry {
            tier: 1,
            config_paths: paths,
            plugins_managed: false,
            plugin_manager: None,
            added_at: chrono::Utc::now(),
            last_snapshot: None,
            plugins: BTreeMap::new(),
        }
    }

    #[test]
    fn test_build_file_state() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("config.conf");
        std::fs::write(&file, "test content").unwrap();

        let tool_name = "test-tool".to_string();
        let entry = sample_entry(vec![file.to_string_lossy().to_string()]);
        let tools: Vec<(&String, &crate::core::manifest::ToolEntry)> = vec![(&tool_name, &entry)];

        let state = build_file_state(&tools);
        assert_eq!(state.len(), 1);
        assert!(state.contains_key(&file));
        assert_eq!(state[&file].tool, "test-tool");
        assert!(!state[&file].hash.is_empty());
    }

    #[test]
    fn test_build_file_state_skips_dirs() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("subdir");
        std::fs::create_dir(&dir).unwrap();

        let tool_name = "test-tool".to_string();
        let entry = sample_entry(vec![dir.to_string_lossy().to_string()]);
        let tools: Vec<(&String, &crate::core::manifest::ToolEntry)> = vec![(&tool_name, &entry)];

        let state = build_file_state(&tools);
        assert!(state.is_empty());
    }

    #[test]
    fn test_detect_change() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("config.conf");
        std::fs::write(&file, "original").unwrap();

        let hash_before = hash_file(&file);
        std::fs::write(&file, "modified").unwrap();
        let hash_after = hash_file(&file);

        assert_ne!(hash_before, hash_after);
    }

    #[test]
    fn test_no_change() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("config.conf");
        std::fs::write(&file, "content").unwrap();

        let h1 = hash_file(&file);
        let h2 = hash_file(&file);
        assert_eq!(h1, h2);
    }
}
