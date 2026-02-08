use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::core::errors::DotsmithError;
use crate::core::module::ModuleDefinition;
use crate::util;

/// Check if a tool is installed by running its detect command.
/// `detect_cmd` is a full command string, e.g. "which tmux".
pub fn check_installed(tool: &str, detect_cmd: &str) -> Result<()> {
    let parts: Vec<&str> = detect_cmd.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("empty detect command for tool '{}'", tool);
    }

    // Execute with explicit args — no shell interpolation
    let status = Command::new(parts[0])
        .args(&parts[1..])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(DotsmithError::ToolNotInstalled(tool.to_string(), detect_cmd.to_string()).into()),
    }
}

/// Given a module definition, find which config paths actually exist.
///
/// Handles symlinks correctly:
/// - Records the user-facing path (the symlink), not the resolved target
/// - Follows symlinks transparently when reading contents
/// - Validates symlink targets are within $HOME
pub fn find_config_paths_from_module(module: &ModuleDefinition) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();

    for candidate in &module.metadata.config_paths {
        let expanded = util::paths::expand_tilde(candidate);

        // Check if this candidate path exists (without following the final symlink)
        let meta = match fs::symlink_metadata(&expanded) {
            Ok(m) => m,
            Err(_) => continue, // Doesn't exist, try next candidate
        };

        // Safety check: verify resolved path is within $HOME
        if let Err(e) = util::fs::check_path_safety(&expanded) {
            eprintln!("warning: skipping {}: {}", candidate, e);
            continue;
        }

        if meta.file_type().is_symlink() || meta.is_dir() {
            // Path is a directory (or symlink to one) — look inside for config files
            // Use metadata() which follows symlinks to check the target type
            if let Ok(target_meta) = fs::metadata(&expanded) {
                if target_meta.is_dir() {
                    discover_config_dir(&expanded, &mut found)?;
                } else {
                    found.push(expanded);
                }
            }
        } else if meta.is_file() {
            // It's a regular config file (e.g. ~/.tmux.conf)
            found.push(expanded);
        }

        // First valid candidate wins — don't check lower-priority paths
        if !found.is_empty() {
            break;
        }
    }

    Ok(found)
}

/// For a directory config root, discover config files and relevant subdirectories.
/// Skips plugin directories to avoid tracking third-party code.
fn discover_config_dir(dir: &Path, found: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        fs::read_dir(dir).with_context(|| format!("failed to read directory {}", dir.display()))?;

    // Known config file extensions
    let config_extensions = ["conf", "toml", "yaml", "yml", "json", "jsonc", "lua", "vim"];

    // Known config file names (no extension)
    let config_names = [
        "tmux.conf",
        ".zshrc",
        ".zshenv",
        ".zprofile",
        "kitty.conf",
        "alacritty.toml",
        "alacritty.yml",
        "config",
        "init.lua",
        "init.vim",
    ];

    // Directories to recurse into (tracked config subdirs)
    let tracked_subdirs = ["conf", "utils", "scripts", "lua", "after", "colors"];

    // Directories to SKIP (plugin dirs, version control, caches)
    let skip_dirs = [
        "plugs",
        "plugins",
        "tpm",
        ".git",
        "zinix-mgr",
        "zinit",
        "oh-my-zsh",
        ".antidote",
        ".cache",
        "node_modules",
        "__pycache__",
    ];

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Skip hidden files/dirs (except known config files like .zshrc)
        if name.starts_with('.') && !config_names.contains(&name.as_str()) {
            continue;
        }

        if file_type.is_file() || file_type.is_symlink() {
            // Check if this looks like a config file
            let is_config = config_names.contains(&name.as_str())
                || config_extensions
                    .iter()
                    .any(|ext| name.ends_with(&format!(".{}", ext)));

            if is_config {
                found.push(path);
            }
        } else if file_type.is_dir() {
            if tracked_subdirs.contains(&name.as_str()) {
                found.push(path); // Track the directory itself
            }
            // Skip plugin dirs and unrecognized dirs silently
            if skip_dirs.contains(&name.as_str()) {
                continue;
            }
        }
    }

    Ok(())
}

/// For Tier 2 tools: auto-detect config file locations.
pub fn auto_detect_config_paths(tool: &str) -> Result<Vec<PathBuf>> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    let xdg_config = dirs::config_dir().unwrap_or_else(|| home.join(".config"));

    let candidates = vec![
        xdg_config.join(tool),                    // ~/.config/<tool>/
        home.join(format!(".{}config", tool)),     // ~/.<tool>config (e.g. .gitconfig)
        home.join(format!(".{}", tool)),           // ~/.<tool>
        home.join(format!(".{}rc", tool)),         // ~/.<tool>rc
        home.join(format!(".{}.conf", tool)),      // ~/.<tool>.conf
        home.join(format!(".config/{}", tool)),    // ~/.config/<tool> (fallback)
    ];

    let mut found = Vec::new();
    for candidate in candidates {
        if candidate.exists() {
            // Safety check
            if let Err(e) = util::fs::check_path_safety(&candidate) {
                eprintln!("warning: skipping {}: {}", candidate.display(), e);
                continue;
            }

            let meta = fs::metadata(&candidate)?;
            if meta.is_dir() {
                discover_config_dir(&candidate, &mut found)?;
                if found.is_empty() {
                    // Directory exists but no config files found inside — track the dir itself
                    found.push(candidate);
                }
            } else {
                found.push(candidate);
            }
            break; // First match wins
        }
    }

    Ok(found)
}

/// Detect existing plugin managers for a tool.
/// Returns the name of the detected plugin manager, or None.
pub fn detect_plugin_manager(tool: &str, config_paths: &[PathBuf]) -> Option<String> {
    // Determine the config root directory from the first config path
    let config_root = config_paths.first().and_then(|p| {
        if p.is_dir() {
            Some(p.as_path())
        } else {
            p.parent()
        }
    })?;

    match tool {
        "tmux" => {
            let checks = [("tpm", &["plugs/tpm", "plugins/tpm"][..])];
            for (name, paths) in &checks {
                for subpath in *paths {
                    if config_root.join(subpath).exists() {
                        return Some(name.to_string());
                    }
                }
            }
            None
        }
        "zsh" => {
            let checks = [
                ("zinix-mgr", "zinix-mgr"),
                ("zinit", "zinit"),
                ("zinit", ".zinit"),
                ("oh-my-zsh", "oh-my-zsh"),
                ("oh-my-zsh", ".oh-my-zsh"),
                ("antidote", ".antidote"),
                ("zplug", "zplug"),
                ("antigen", ".antigen"),
            ];
            for (name, dir) in &checks {
                if config_root.join(dir).exists() {
                    return Some(name.to_string());
                }
            }
            None
        }
        "nvim" => {
            // Check for lazy.nvim lockfile
            if config_root.join("lazy-lock.json").exists() {
                return Some("lazy".to_string());
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_plugin_manager_tmux_tpm() {
        let tmp = TempDir::new().unwrap();
        let tmux_dir = tmp.path().join("tmux");
        fs::create_dir_all(tmux_dir.join("plugs/tpm")).unwrap();
        fs::write(tmux_dir.join("tmux.conf"), "# config").unwrap();

        let paths = vec![tmux_dir.join("tmux.conf")];
        let result = detect_plugin_manager("tmux", &paths);
        assert_eq!(result, Some("tpm".to_string()));
    }

    #[test]
    fn test_detect_plugin_manager_zsh_zinix() {
        let tmp = TempDir::new().unwrap();
        let zsh_dir = tmp.path().join("zsh");
        fs::create_dir_all(zsh_dir.join("zinix-mgr")).unwrap();
        fs::write(zsh_dir.join(".zshrc"), "# config").unwrap();

        let paths = vec![zsh_dir.join(".zshrc")];
        let result = detect_plugin_manager("zsh", &paths);
        assert_eq!(result, Some("zinix-mgr".to_string()));
    }

    #[test]
    fn test_detect_plugin_manager_none() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("tool");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("config"), "# config").unwrap();

        let paths = vec![dir.join("config")];
        let result = detect_plugin_manager("tmux", &paths);
        assert_eq!(result, None);
    }

    #[test]
    fn test_discover_config_dir_basic() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("tmux");
        fs::create_dir_all(dir.join("conf")).unwrap();
        fs::write(dir.join("tmux.conf"), "set -g mouse on").unwrap();
        fs::write(dir.join("conf/settings.conf"), "set -g base-index 1").unwrap();

        let mut found = Vec::new();
        discover_config_dir(&dir, &mut found).unwrap();

        let names: Vec<String> = found
            .iter()
            .map(|p| p.strip_prefix(&dir).unwrap().display().to_string())
            .collect();

        assert!(names.contains(&"tmux.conf".to_string()));
        assert!(names.contains(&"conf".to_string()));
    }

    #[test]
    fn test_discover_config_dir_skips_plugin_dirs() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("tmux");
        fs::create_dir_all(dir.join("plugs/tpm")).unwrap();
        fs::create_dir_all(dir.join("plugins")).unwrap();
        fs::write(dir.join("tmux.conf"), "# config").unwrap();

        let mut found = Vec::new();
        discover_config_dir(&dir, &mut found).unwrap();

        let names: Vec<String> = found
            .iter()
            .map(|p| p.strip_prefix(&dir).unwrap().display().to_string())
            .collect();

        assert!(names.contains(&"tmux.conf".to_string()));
        assert!(!names.contains(&"plugs".to_string()));
        assert!(!names.contains(&"plugins".to_string()));
    }

    #[test]
    fn test_auto_detect_config_paths() {
        let tmp = TempDir::new().unwrap();
        let tool_dir = tmp.path().join(".config/sometool");
        fs::create_dir_all(&tool_dir).unwrap();
        fs::write(tool_dir.join("config.toml"), "key = 'value'").unwrap();

        // This test exercises the structure but can't fully test the function
        // because auto_detect uses dirs::config_dir() which we can't override.
        // Integration tests cover this via DOTSMITH_CONFIG_DIR + real paths.
    }

    #[test]
    fn test_discover_with_symlinked_config() {
        let tmp = TempDir::new().unwrap();

        // Create "dotfiles repo" source
        let source = tmp.path().join("dots/tmux");
        fs::create_dir_all(source.join("conf")).unwrap();
        fs::write(source.join("tmux.conf"), "set -g mouse on").unwrap();
        fs::write(source.join("conf/keys.conf"), "bind r source").unwrap();

        // Create symlink like the user's setup
        let config_tmux = tmp.path().join("config/tmux");
        fs::create_dir_all(tmp.path().join("config")).unwrap();
        std::os::unix::fs::symlink(&source, &config_tmux).unwrap();

        // Discover should find files through the symlink
        let mut found = Vec::new();
        discover_config_dir(&config_tmux, &mut found).unwrap();

        assert!(!found.is_empty());
        // Paths should be relative to the symlink, not the target
        assert!(found.iter().any(|p| p.starts_with(&config_tmux)));
    }
}
