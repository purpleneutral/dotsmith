use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::core::errors::DotsmithError;
use crate::core::manifest::{Manifest, PluginEntry};
use crate::util;

/// Which tools support dotsmith-managed plugins.
const SUPPORTED_TOOLS: &[&str] = &["zsh", "tmux"];

/// Result of a plugin update operation.
#[derive(Debug)]
pub struct UpdateResult {
    pub name: String,
    pub updated: bool,
    pub old_commit: String,
    pub new_commit: String,
}

// ---------------------------------------------------------------------------
// Repo parsing
// ---------------------------------------------------------------------------

/// Parse a plugin repository specifier.
///
/// Accepts:
/// - GitHub shorthand: `"user/repo"`
/// - Full HTTPS URL: `"https://github.com/user/repo"` or `"https://github.com/user/repo.git"`
///
/// Returns `(clone_url, plugin_name)`.
pub fn parse_repo(repo: &str) -> Result<(String, String)> {
    if repo.starts_with("https://") || repo.starts_with("http://") || repo.starts_with("file://") {
        let name = repo
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .rsplit('/')
            .next()
            .ok_or_else(|| DotsmithError::InvalidPluginRepo(repo.to_string()))?
            .to_string();
        if name.is_empty() {
            return Err(DotsmithError::InvalidPluginRepo(repo.to_string()).into());
        }
        // Don't append .git to file:// URLs
        let url = if repo.starts_with("file://") || repo.ends_with(".git") {
            repo.to_string()
        } else {
            format!("{}.git", repo.trim_end_matches('/'))
        };
        Ok((url, name))
    } else {
        // GitHub shorthand: user/repo
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(DotsmithError::InvalidPluginRepo(repo.to_string()).into());
        }
        let name = parts[1].to_string();
        let url = format!("https://github.com/{}.git", repo);
        Ok((url, name))
    }
}

// ---------------------------------------------------------------------------
// Git operations
// ---------------------------------------------------------------------------

/// Check that git is installed.
pub fn check_git_installed() -> Result<()> {
    let status = std::process::Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(DotsmithError::GitNotInstalled.into()),
    }
}

/// Clone a plugin repository with `--depth 1`.
pub fn git_clone(url: &str, dest: &Path) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["clone", "--depth", "1"])
        .arg(url)
        .arg(dest)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .with_context(|| format!("failed to execute git clone for {}", url))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(
            DotsmithError::GitCloneFailed(url.to_string(), stderr.trim().to_string()).into(),
        );
    }
    Ok(())
}

/// Get the current HEAD commit hash of a git repository.
pub fn git_head_commit(repo_dir: &Path) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .context("failed to get git HEAD commit")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Pull latest changes for a plugin.
/// Returns `true` if changes were pulled, `false` if already up to date.
pub fn git_pull(repo_dir: &Path) -> Result<bool> {
    let before = git_head_commit(repo_dir)?;

    let output = std::process::Command::new("git")
        .args(["pull", "--ff-only"])
        .current_dir(repo_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .context("failed to execute git pull")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DotsmithError::GitPullFailed(
            repo_dir.display().to_string(),
            stderr.trim().to_string(),
        )
        .into());
    }

    let after = git_head_commit(repo_dir)?;
    Ok(before != after)
}

// ---------------------------------------------------------------------------
// Init file detection
// ---------------------------------------------------------------------------

/// Detect the init file for a plugin based on the tool type.
/// Returns the filename (not full path) of the init file.
pub fn detect_init_file(tool: &str, plugin_dir: &Path) -> Result<String> {
    let entries: Vec<String> = std::fs::read_dir(plugin_dir)
        .with_context(|| format!("failed to read plugin directory {}", plugin_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect();

    match tool {
        "zsh" => detect_zsh_init_file(&entries),
        "tmux" => detect_tmux_init_file(&entries),
        _ => Err(DotsmithError::PluginsNotSupported(tool.to_string()).into()),
    }
}

fn detect_zsh_init_file(files: &[String]) -> Result<String> {
    // Priority 1: *.plugin.zsh
    let mut plugin_zsh: Vec<&String> = files.iter().filter(|f| f.ends_with(".plugin.zsh")).collect();
    if !plugin_zsh.is_empty() {
        plugin_zsh.sort();
        return Ok(plugin_zsh[0].clone());
    }

    // Priority 2: *.zsh-theme
    let mut themes: Vec<&String> = files.iter().filter(|f| f.ends_with(".zsh-theme")).collect();
    if !themes.is_empty() {
        themes.sort();
        return Ok(themes[0].clone());
    }

    // Priority 3: init.zsh
    if files.iter().any(|f| f == "init.zsh") {
        return Ok("init.zsh".to_string());
    }

    // Priority 4: *.zsh (only if exactly one, excluding .plugin.zsh files)
    let zsh_files: Vec<&String> = files
        .iter()
        .filter(|f| f.ends_with(".zsh") && !f.ends_with(".plugin.zsh"))
        .collect();
    if zsh_files.len() == 1 {
        return Ok(zsh_files[0].clone());
    }

    Err(DotsmithError::PluginNoInitFile(
        "zsh plugin".to_string(),
        "*.plugin.zsh, *.zsh-theme, init.zsh, or a single *.zsh file".to_string(),
    )
    .into())
}

fn detect_tmux_init_file(files: &[String]) -> Result<String> {
    let mut tmux_files: Vec<&String> = files.iter().filter(|f| f.ends_with(".tmux")).collect();
    if !tmux_files.is_empty() {
        tmux_files.sort();
        return Ok(tmux_files[0].clone());
    }

    Err(DotsmithError::PluginNoInitFile(
        "tmux plugin".to_string(),
        "*.tmux".to_string(),
    )
    .into())
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Get the base plugins directory for a tool: `<config_dir>/plugins/<tool>/`
pub fn plugin_base_dir(config_dir: &Path, tool: &str) -> PathBuf {
    config_dir.join("plugins").join(tool)
}

/// Get the directory for a specific plugin: `<config_dir>/plugins/<tool>/<name>/`
pub fn plugin_dir(config_dir: &Path, tool: &str, name: &str) -> PathBuf {
    plugin_base_dir(config_dir, tool).join(name)
}

/// Get the path to the loader file for a tool.
pub fn loader_path(config_dir: &Path, tool: &str) -> PathBuf {
    let base = plugin_base_dir(config_dir, tool);
    match tool {
        "tmux" => base.join("loader.conf"),
        _ => base.join(format!("loader.{}", tool)),
    }
}

// ---------------------------------------------------------------------------
// Loader generation
// ---------------------------------------------------------------------------

/// Generate the loader file content for a tool.
pub fn generate_loader_content(
    tool: &str,
    config_dir: &Path,
    plugins: &BTreeMap<String, PluginEntry>,
) -> String {
    let mut lines = Vec::new();
    lines.push("# Auto-generated by dotsmith — do not edit manually".to_string());
    lines.push(format!("# Managed plugins for {}", tool));
    lines.push(String::new());

    let base = plugin_base_dir(config_dir, tool);

    for (name, entry) in plugins {
        let init_path = base.join(name).join(&entry.init);
        let contracted = util::paths::contract_tilde(&init_path);

        match tool {
            "tmux" => lines.push(format!("run-shell {}", contracted)),
            _ => lines.push(format!("source {}", contracted)),
        }
    }

    lines.push(String::new());
    lines.join("\n")
}

/// Write the loader file atomically.
pub fn write_loader(
    tool: &str,
    config_dir: &Path,
    plugins: &BTreeMap<String, PluginEntry>,
) -> Result<()> {
    let content = generate_loader_content(tool, config_dir, plugins);
    let path = loader_path(config_dir, tool);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    util::fs::atomic_write(&path, &content)
        .with_context(|| format!("failed to write loader at {}", path.display()))
}

// ---------------------------------------------------------------------------
// ZSH-specific: recompile .zwc files
// ---------------------------------------------------------------------------

/// Recompile `*.zwc` files in a zsh plugin directory after update.
/// Only recompiles if existing `.zwc` files are present (indicates the user
/// or plugin uses compiled zsh files).
///
/// Uses `zsh -c "zcompile ..."` because zcompile is a zsh builtin with no
/// standalone binary equivalent.
pub fn zsh_recompile_zwc(plugin_dir: &Path) -> Result<bool> {
    let has_zwc = std::fs::read_dir(plugin_dir)?
        .filter_map(|e| e.ok())
        .any(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext == "zwc")
        });

    if !has_zwc {
        return Ok(false);
    }

    let zsh_files: Vec<PathBuf> = std::fs::read_dir(plugin_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "zsh")
        })
        .collect();

    for zsh_file in &zsh_files {
        // zcompile is a zsh builtin — must invoke via zsh -c
        let _ = std::process::Command::new("zsh")
            .arg("-c")
            .arg(format!("zcompile {}", zsh_file.display()))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    Ok(true)
}

// ---------------------------------------------------------------------------
// High-level operations
// ---------------------------------------------------------------------------

/// Validate that plugin management is supported for a tool.
pub fn validate_tool_supported(tool: &str) -> Result<()> {
    if !SUPPORTED_TOOLS.contains(&tool) {
        return Err(DotsmithError::PluginsNotSupported(tool.to_string()).into());
    }
    Ok(())
}

/// Add a plugin for a tool.
///
/// 1. Parse repo specifier
/// 2. Verify tool is tracked and plugin not already installed
/// 3. `git clone --depth 1`
/// 4. Detect init file
/// 5. Register in manifest
/// 6. Regenerate loader
/// 7. Save manifest
///
/// Returns `(plugin_name, init_file)`.
pub fn add_plugin(
    config_dir: &Path,
    manifest: &mut Manifest,
    tool: &str,
    repo_spec: &str,
) -> Result<(String, String)> {
    validate_tool_supported(tool)?;
    check_git_installed()?;

    let (clone_url, name) = parse_repo(repo_spec)?;

    let tool_entry = manifest
        .get_tool_mut(tool)
        .ok_or_else(|| DotsmithError::ToolNotTracked(tool.to_string()))?;

    if tool_entry.plugins.contains_key(&name) {
        return Err(
            DotsmithError::PluginAlreadyInstalled(name.clone(), tool.to_string()).into(),
        );
    }

    // Clone the repository
    let dest = plugin_dir(config_dir, tool, &name);
    git_clone(&clone_url, &dest)?;

    // Detect init file
    let init_file = match detect_init_file(tool, &dest) {
        Ok(f) => f,
        Err(e) => {
            // Clean up cloned directory on init file detection failure
            let _ = std::fs::remove_dir_all(&dest);
            return Err(e);
        }
    };

    // Register in manifest
    let plugin_entry = PluginEntry {
        repo: repo_spec.to_string(),
        init: init_file.clone(),
        added_at: Utc::now(),
    };
    tool_entry.plugins.insert(name.clone(), plugin_entry);
    tool_entry.plugins_managed = true;
    tool_entry.plugin_manager = Some("dotsmith".to_string());

    // Regenerate loader
    write_loader(tool, config_dir, &tool_entry.plugins)?;

    // Save manifest
    manifest.save(config_dir)?;

    Ok((name, init_file))
}

/// Remove a plugin for a tool.
///
/// 1. Verify plugin exists
/// 2. Remove the cloned directory
/// 3. Remove from manifest
/// 4. Regenerate loader (or remove loader if no plugins remain)
/// 5. Save manifest
pub fn remove_plugin(
    config_dir: &Path,
    manifest: &mut Manifest,
    tool: &str,
    name: &str,
) -> Result<()> {
    validate_tool_supported(tool)?;

    let tool_entry = manifest
        .get_tool_mut(tool)
        .ok_or_else(|| DotsmithError::ToolNotTracked(tool.to_string()))?;

    if !tool_entry.plugins.contains_key(name) {
        return Err(
            DotsmithError::PluginNotInstalled(name.to_string(), tool.to_string()).into(),
        );
    }

    // Remove directory
    let dir = plugin_dir(config_dir, tool, name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .with_context(|| format!("failed to remove plugin directory {}", dir.display()))?;
    }

    // Remove from manifest
    tool_entry.plugins.remove(name);

    if tool_entry.plugins.is_empty() {
        tool_entry.plugins_managed = false;
        tool_entry.plugin_manager = None;
        // Remove the loader file
        let lp = loader_path(config_dir, tool);
        if lp.exists() {
            std::fs::remove_file(&lp).ok();
        }
    } else {
        write_loader(tool, config_dir, &tool_entry.plugins)?;
    }

    manifest.save(config_dir)?;
    Ok(())
}

/// List plugins for a tool. Returns `(name, repo, init_file)` tuples.
pub fn list_plugins(manifest: &Manifest, tool: &str) -> Result<Vec<(String, String, String)>> {
    validate_tool_supported(tool)?;

    let tool_entry = manifest
        .get_tool(tool)
        .ok_or_else(|| DotsmithError::ToolNotTracked(tool.to_string()))?;

    Ok(tool_entry
        .plugins
        .iter()
        .map(|(name, entry)| (name.clone(), entry.repo.clone(), entry.init.clone()))
        .collect())
}

/// Update one or all plugins for a tool.
pub fn update_plugins(
    config_dir: &Path,
    manifest: &Manifest,
    tool: &str,
    name: Option<&str>,
) -> Result<Vec<UpdateResult>> {
    validate_tool_supported(tool)?;
    check_git_installed()?;

    let tool_entry = manifest
        .get_tool(tool)
        .ok_or_else(|| DotsmithError::ToolNotTracked(tool.to_string()))?;

    let plugins_to_update: Vec<(&String, &PluginEntry)> = match name {
        Some(n) => {
            let entry = tool_entry
                .plugins
                .get(n)
                .ok_or_else(|| DotsmithError::PluginNotInstalled(n.to_string(), tool.to_string()))?;
            vec![(tool_entry.plugins.keys().find(|k| k.as_str() == n).unwrap(), entry)]
        }
        None => tool_entry.plugins.iter().collect(),
    };

    let mut results = Vec::new();

    for (plugin_name, _entry) in &plugins_to_update {
        let dir = plugin_dir(config_dir, tool, plugin_name);

        if !dir.exists() {
            results.push(UpdateResult {
                name: plugin_name.to_string(),
                updated: false,
                old_commit: "missing".to_string(),
                new_commit: "missing".to_string(),
            });
            continue;
        }

        let old_commit = git_head_commit(&dir).unwrap_or_default();
        let updated = git_pull(&dir)?;
        let new_commit = git_head_commit(&dir).unwrap_or_default();

        if updated && tool == "zsh" {
            zsh_recompile_zwc(&dir)?;
        }

        results.push(UpdateResult {
            name: plugin_name.to_string(),
            updated,
            old_commit,
            new_commit,
        });
    }

    Ok(results)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // -- parse_repo tests --

    #[test]
    fn test_parse_repo_shorthand() {
        let (url, name) = parse_repo("zsh-users/zsh-autosuggestions").unwrap();
        assert_eq!(url, "https://github.com/zsh-users/zsh-autosuggestions.git");
        assert_eq!(name, "zsh-autosuggestions");
    }

    #[test]
    fn test_parse_repo_full_url() {
        let (url, name) = parse_repo("https://github.com/foo/bar").unwrap();
        assert_eq!(url, "https://github.com/foo/bar.git");
        assert_eq!(name, "bar");
    }

    #[test]
    fn test_parse_repo_full_url_with_git_suffix() {
        let (url, name) = parse_repo("https://github.com/foo/bar.git").unwrap();
        assert_eq!(url, "https://github.com/foo/bar.git");
        assert_eq!(name, "bar");
    }

    #[test]
    fn test_parse_repo_full_url_trailing_slash() {
        let (url, name) = parse_repo("https://github.com/foo/bar/").unwrap();
        assert_eq!(url, "https://github.com/foo/bar.git");
        assert_eq!(name, "bar");
    }

    #[test]
    fn test_parse_repo_invalid_no_slash() {
        assert!(parse_repo("justname").is_err());
    }

    #[test]
    fn test_parse_repo_invalid_empty_user() {
        assert!(parse_repo("/repo").is_err());
    }

    #[test]
    fn test_parse_repo_invalid_empty_repo() {
        assert!(parse_repo("user/").is_err());
    }

    #[test]
    fn test_parse_repo_invalid_three_parts() {
        assert!(parse_repo("a/b/c").is_err());
    }

    // -- detect_init_file tests --

    #[test]
    fn test_detect_zsh_init_plugin_zsh() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("zsh-autosuggestions.plugin.zsh"),
            "# plugin",
        )
        .unwrap();
        std::fs::write(tmp.path().join("README.md"), "# readme").unwrap();

        let result = detect_init_file("zsh", tmp.path()).unwrap();
        assert_eq!(result, "zsh-autosuggestions.plugin.zsh");
    }

    #[test]
    fn test_detect_zsh_init_theme() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("pure.zsh-theme"), "# theme").unwrap();
        std::fs::write(tmp.path().join("async.zsh"), "# async helper").unwrap();
        std::fs::write(tmp.path().join("pure.zsh"), "# setup").unwrap();

        let result = detect_init_file("zsh", tmp.path()).unwrap();
        assert_eq!(result, "pure.zsh-theme");
    }

    #[test]
    fn test_detect_zsh_init_init_zsh() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("init.zsh"), "# init").unwrap();
        std::fs::write(tmp.path().join("helpers.zsh"), "# helpers").unwrap();
        std::fs::write(tmp.path().join("utils.zsh"), "# utils").unwrap();

        let result = detect_init_file("zsh", tmp.path()).unwrap();
        assert_eq!(result, "init.zsh");
    }

    #[test]
    fn test_detect_zsh_init_single_zsh() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("foo.zsh"), "# single zsh file").unwrap();
        std::fs::write(tmp.path().join("README.md"), "# readme").unwrap();

        let result = detect_init_file("zsh", tmp.path()).unwrap();
        assert_eq!(result, "foo.zsh");
    }

    #[test]
    fn test_detect_zsh_init_ambiguous_fails() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("a.zsh"), "# a").unwrap();
        std::fs::write(tmp.path().join("b.zsh"), "# b").unwrap();

        let result = detect_init_file("zsh", tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_tmux_init() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("sensible.tmux"), "#!/usr/bin/env bash").unwrap();
        std::fs::write(tmp.path().join("README.md"), "# readme").unwrap();

        let result = detect_init_file("tmux", tmp.path()).unwrap();
        assert_eq!(result, "sensible.tmux");
    }

    #[test]
    fn test_detect_tmux_init_none_fails() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("README.md"), "# readme").unwrap();

        let result = detect_init_file("tmux", tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_unsupported_tool_fails() {
        let tmp = TempDir::new().unwrap();
        let result = detect_init_file("git", tmp.path());
        assert!(result.is_err());
    }

    // -- loader generation tests --

    #[test]
    fn test_generate_loader_zsh() {
        let tmp = TempDir::new().unwrap();
        let mut plugins = BTreeMap::new();
        plugins.insert(
            "zsh-autosuggestions".to_string(),
            PluginEntry {
                repo: "zsh-users/zsh-autosuggestions".to_string(),
                init: "zsh-autosuggestions.plugin.zsh".to_string(),
                added_at: Utc::now(),
            },
        );

        let content = generate_loader_content("zsh", tmp.path(), &plugins);
        assert!(content.contains("# Auto-generated by dotsmith"));
        assert!(content.contains("source "));
        assert!(content.contains("zsh-autosuggestions/zsh-autosuggestions.plugin.zsh"));
        assert!(!content.contains("run-shell"));
    }

    #[test]
    fn test_generate_loader_tmux() {
        let tmp = TempDir::new().unwrap();
        let mut plugins = BTreeMap::new();
        plugins.insert(
            "tmux-sensible".to_string(),
            PluginEntry {
                repo: "tmux-plugins/tmux-sensible".to_string(),
                init: "sensible.tmux".to_string(),
                added_at: Utc::now(),
            },
        );

        let content = generate_loader_content("tmux", tmp.path(), &plugins);
        assert!(content.contains("# Auto-generated by dotsmith"));
        assert!(content.contains("run-shell "));
        assert!(content.contains("tmux-sensible/sensible.tmux"));
        assert!(!content.contains("source "));
    }

    #[test]
    fn test_generate_loader_empty() {
        let tmp = TempDir::new().unwrap();
        let plugins = BTreeMap::new();
        let content = generate_loader_content("zsh", tmp.path(), &plugins);
        assert!(content.contains("# Auto-generated by dotsmith"));
        assert!(!content.contains("source "));
    }

    // -- path helper tests --

    #[test]
    fn test_plugin_dir_paths() {
        let base = Path::new("/home/user/.config/dotsmith");
        assert_eq!(
            plugin_base_dir(base, "zsh"),
            PathBuf::from("/home/user/.config/dotsmith/plugins/zsh")
        );
        assert_eq!(
            plugin_dir(base, "zsh", "zsh-autosuggestions"),
            PathBuf::from("/home/user/.config/dotsmith/plugins/zsh/zsh-autosuggestions")
        );
    }

    #[test]
    fn test_loader_path_zsh() {
        let base = Path::new("/home/user/.config/dotsmith");
        assert_eq!(
            loader_path(base, "zsh"),
            PathBuf::from("/home/user/.config/dotsmith/plugins/zsh/loader.zsh")
        );
    }

    #[test]
    fn test_loader_path_tmux() {
        let base = Path::new("/home/user/.config/dotsmith");
        assert_eq!(
            loader_path(base, "tmux"),
            PathBuf::from("/home/user/.config/dotsmith/plugins/tmux/loader.conf")
        );
    }

    // -- validate_tool_supported tests --

    #[test]
    fn test_validate_supported() {
        assert!(validate_tool_supported("zsh").is_ok());
        assert!(validate_tool_supported("tmux").is_ok());
    }

    #[test]
    fn test_validate_unsupported() {
        assert!(validate_tool_supported("git").is_err());
        assert!(validate_tool_supported("nvim").is_err());
    }
}
