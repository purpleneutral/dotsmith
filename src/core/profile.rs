use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::errors::DotsmithError;
use crate::core::manifest::{Manifest, ToolEntry};
use crate::util;

/// Metadata about a saved profile, serialized to profile.toml.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub tools: BTreeMap<String, ToolEntry>,
    pub checksums: BTreeMap<String, String>,
}

/// Summary of a profile for listing.
#[derive(Debug)]
pub struct ProfileSummary {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub tool_count: usize,
    pub file_count: usize,
}

/// Result of loading a profile.
#[derive(Debug)]
pub struct ProfileLoadResult {
    pub restored_files: usize,
    pub backed_up_files: usize,
    pub tools_added: Vec<String>,
    pub skipped_tools: Vec<String>,
}

/// Get the profiles directory path.
pub fn profiles_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("profiles")
}

/// Validate that a profile name is safe (no path traversal, no special chars).
fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 64 {
        return Err(DotsmithError::InvalidProfileName(name.to_string()).into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(DotsmithError::InvalidProfileName(name.to_string()).into());
    }
    if name == "." || name == ".." {
        return Err(DotsmithError::InvalidProfileName(name.to_string()).into());
    }
    Ok(())
}

/// Compute SHA-256 hash of a file's contents.
fn hash_file(path: &Path) -> Result<String> {
    let content = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Copy a single file into the profile's files directory, returning the relative key and hash.
fn copy_file_to_profile(
    file_path: &Path,
    tool_name: &str,
    files_dir: &Path,
) -> Result<Option<(String, String)>> {
    if !file_path.is_file() {
        return Ok(None);
    }

    let file_name = file_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("no filename for {}", file_path.display()))?;

    let tool_dir = files_dir.join(tool_name);
    fs::create_dir_all(&tool_dir)
        .with_context(|| format!("failed to create {}", tool_dir.display()))?;

    let dest = tool_dir.join(file_name);
    fs::copy(file_path, &dest)
        .with_context(|| format!("failed to copy {} to {}", file_path.display(), dest.display()))?;

    let hash = hash_file(&dest)?;
    let key = format!("{}/{}", tool_name, file_name.to_string_lossy());

    Ok(Some((key, hash)))
}

/// Walk a directory and copy all files into the profile.
fn copy_dir_to_profile(
    dir_path: &Path,
    tool_name: &str,
    files_dir: &Path,
) -> Result<Vec<(String, String)>> {
    let mut results = Vec::new();

    if !dir_path.is_dir() {
        return Ok(results);
    }

    let dir_name = dir_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("no dirname for {}", dir_path.display()))?;

    let dest_dir = files_dir.join(tool_name).join(dir_name);
    fs::create_dir_all(&dest_dir)
        .with_context(|| format!("failed to create {}", dest_dir.display()))?;

    for entry in fs::read_dir(dir_path)
        .with_context(|| format!("failed to read dir {}", dir_path.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("no filename"))?;
            let dest = dest_dir.join(file_name);
            fs::copy(&path, &dest).with_context(|| {
                format!("failed to copy {} to {}", path.display(), dest.display())
            })?;
            let hash = hash_file(&dest)?;
            let key = format!(
                "{}/{}/{}",
                tool_name,
                dir_name.to_string_lossy(),
                file_name.to_string_lossy()
            );
            results.push((key, hash));
        }
    }

    Ok(results)
}

/// Save the current manifest and config file contents as a named profile.
pub fn save_profile(
    config_dir: &Path,
    manifest: &Manifest,
    name: &str,
) -> Result<(usize, usize)> {
    validate_profile_name(name)?;

    let profile_dir = profiles_dir(config_dir).join(name);
    if profile_dir.exists() {
        return Err(DotsmithError::ProfileAlreadyExists(name.to_string()).into());
    }

    let files_dir = profile_dir.join("files");
    fs::create_dir_all(&files_dir)
        .with_context(|| format!("failed to create {}", files_dir.display()))?;

    let mut checksums = BTreeMap::new();
    let mut file_count = 0usize;

    for (tool_name, entry) in &manifest.tools {
        for config_path in &entry.config_paths {
            let expanded = util::paths::expand_tilde(config_path);
            if expanded.is_file() {
                if let Some((key, hash)) =
                    copy_file_to_profile(&expanded, tool_name, &files_dir)?
                {
                    checksums.insert(key, hash);
                    file_count += 1;
                }
            } else if expanded.is_dir() {
                let dir_results = copy_dir_to_profile(&expanded, tool_name, &files_dir)?;
                file_count += dir_results.len();
                for (key, hash) in dir_results {
                    checksums.insert(key, hash);
                }
            }
        }
    }

    let meta = ProfileMeta {
        name: name.to_string(),
        created_at: Utc::now(),
        tools: manifest.tools.clone(),
        checksums,
    };

    let toml_content = toml::to_string_pretty(&meta).context("failed to serialize profile")?;
    util::fs::atomic_write(&profile_dir.join("profile.toml"), &toml_content)
        .context("failed to write profile.toml")?;

    Ok((manifest.tools.len(), file_count))
}

/// Load a named profile, restoring config files with backup.
pub fn load_profile(
    config_dir: &Path,
    manifest: &mut Manifest,
    name: &str,
    add_untracked: bool,
) -> Result<ProfileLoadResult> {
    validate_profile_name(name)?;

    let profile_dir = profiles_dir(config_dir).join(name);
    if !profile_dir.exists() {
        return Err(DotsmithError::ProfileNotFound(name.to_string()).into());
    }

    let meta_path = profile_dir.join("profile.toml");
    let meta_content = fs::read_to_string(&meta_path)
        .with_context(|| format!("failed to read {}", meta_path.display()))?;
    let meta: ProfileMeta =
        toml::from_str(&meta_content).with_context(|| "failed to parse profile.toml")?;

    let files_dir = profile_dir.join("files");
    let backup_dir = config_dir.join("backups");
    fs::create_dir_all(&backup_dir)?;

    let mut result = ProfileLoadResult {
        restored_files: 0,
        backed_up_files: 0,
        tools_added: Vec::new(),
        skipped_tools: Vec::new(),
    };

    let mut manifest_changed = false;

    for (tool_name, tool_entry) in &meta.tools {
        let is_tracked = manifest.has_tool(tool_name);

        if !is_tracked && !add_untracked {
            result.skipped_tools.push(tool_name.clone());
            continue;
        }

        if !is_tracked {
            manifest.add_tool(tool_name, tool_entry.clone())?;
            result.tools_added.push(tool_name.clone());
            manifest_changed = true;
        }

        // Restore files for this tool
        for config_path in &tool_entry.config_paths {
            let target = util::paths::expand_tilde(config_path);

            // Try to find the source file in the profile
            if target.is_file() || !target.exists() {
                let file_name = match target.file_name() {
                    Some(f) => f,
                    None => continue,
                };
                let source = files_dir.join(tool_name).join(file_name);
                if source.is_file() {
                    // Backup existing file if present
                    if target.is_file() {
                        let backup_name = format!(
                            "{}.{}.bak",
                            file_name.to_string_lossy(),
                            chrono::Local::now().format("%Y%m%d_%H%M%S")
                        );
                        let backup_path = backup_dir.join(&backup_name);
                        fs::copy(&target, &backup_path).with_context(|| {
                            format!(
                                "failed to backup {} to {}",
                                target.display(),
                                backup_path.display()
                            )
                        })?;
                        result.backed_up_files += 1;
                    }

                    // Ensure parent directory exists
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::copy(&source, &target).with_context(|| {
                        format!(
                            "failed to restore {} to {}",
                            source.display(),
                            target.display()
                        )
                    })?;
                    result.restored_files += 1;
                }
            } else if target.is_dir() || !target.exists() {
                // Restore directory contents
                let dir_name = match target.file_name() {
                    Some(f) => f,
                    None => continue,
                };
                let source_dir = files_dir.join(tool_name).join(dir_name);
                if source_dir.is_dir() {
                    fs::create_dir_all(&target)?;
                    for entry in fs::read_dir(&source_dir)? {
                        let entry = entry?;
                        let src_file = entry.path();
                        if src_file.is_file() {
                            let dest_file = target.join(entry.file_name());
                            if dest_file.is_file() {
                                let backup_name = format!(
                                    "{}.{}.bak",
                                    entry.file_name().to_string_lossy(),
                                    chrono::Local::now().format("%Y%m%d_%H%M%S")
                                );
                                let backup_path = backup_dir.join(&backup_name);
                                fs::copy(&dest_file, &backup_path)?;
                                result.backed_up_files += 1;
                            }
                            fs::copy(&src_file, &dest_file)?;
                            result.restored_files += 1;
                        }
                    }
                }
            }
        }
    }

    if manifest_changed {
        manifest.save(config_dir)?;
    }

    Ok(result)
}

/// List all saved profiles.
pub fn list_profiles(config_dir: &Path) -> Result<Vec<ProfileSummary>> {
    let dir = profiles_dir(config_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();

    for entry in fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        let meta_path = entry.path().join("profile.toml");
        if !meta_path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&meta_path)
            && let Ok(meta) = toml::from_str::<ProfileMeta>(&content)
        {
            profiles.push(ProfileSummary {
                name: meta.name,
                created_at: meta.created_at,
                tool_count: meta.tools.len(),
                file_count: meta.checksums.len(),
            });
        }
    }

    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(profiles)
}

/// Delete a named profile.
pub fn delete_profile(config_dir: &Path, name: &str) -> Result<()> {
    validate_profile_name(name)?;

    let profile_dir = profiles_dir(config_dir).join(name);
    if !profile_dir.exists() {
        return Err(DotsmithError::ProfileNotFound(name.to_string()).into());
    }

    fs::remove_dir_all(&profile_dir)
        .with_context(|| format!("failed to delete profile at {}", profile_dir.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    fn sample_manifest(config_dir: &Path) -> Manifest {
        // Create a fake config file to track
        let fake_config = config_dir.join("fake_tool.conf");
        fs::write(&fake_config, "key = value\n").unwrap();

        let mut manifest = Manifest::default();
        let entry = ToolEntry {
            tier: 1,
            config_paths: vec![fake_config.to_string_lossy().to_string()],
            plugins_managed: false,
            plugin_manager: None,
            added_at: Utc::now(),
            last_snapshot: None,
            plugins: BTreeMap::new(),
        };
        manifest.add_tool("faketool", entry).unwrap();
        manifest
    }

    #[test]
    fn test_validate_profile_name_valid() {
        assert!(validate_profile_name("workstation").is_ok());
        assert!(validate_profile_name("my-laptop").is_ok());
        assert!(validate_profile_name("config_v2").is_ok());
        assert!(validate_profile_name("a").is_ok());
    }

    #[test]
    fn test_validate_profile_name_invalid() {
        assert!(validate_profile_name("").is_err());
        assert!(validate_profile_name("..").is_err());
        assert!(validate_profile_name("/etc").is_err());
        assert!(validate_profile_name("has space").is_err());
        assert!(validate_profile_name("has.dot").is_err());
        let long_name = "a".repeat(65);
        assert!(validate_profile_name(&long_name).is_err());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let mut manifest = sample_manifest(tmp.path());
        manifest.save(&config_dir).unwrap();

        // Save profile
        let (tools, files) = save_profile(&config_dir, &manifest, "test-profile").unwrap();
        assert_eq!(tools, 1);
        assert_eq!(files, 1);

        // Modify the tracked file
        let config_path = manifest.tools["faketool"].config_paths[0].clone();
        fs::write(&config_path, "modified = true\n").unwrap();

        // Load profile (restores original)
        let result = load_profile(&config_dir, &mut manifest, "test-profile", false).unwrap();
        assert_eq!(result.restored_files, 1);
        assert_eq!(result.backed_up_files, 1);

        // Verify file was restored
        let content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(content, "key = value\n");
    }

    #[test]
    fn test_save_creates_structure() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let manifest = sample_manifest(tmp.path());
        save_profile(&config_dir, &manifest, "my-setup").unwrap();

        let profile_dir = profiles_dir(&config_dir).join("my-setup");
        assert!(profile_dir.join("profile.toml").exists());
        assert!(profile_dir.join("files").is_dir());
        assert!(profile_dir.join("files").join("faketool").is_dir());
    }

    #[test]
    fn test_save_checksums() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let manifest = sample_manifest(tmp.path());
        save_profile(&config_dir, &manifest, "checksums-test").unwrap();

        let meta_path = profiles_dir(&config_dir)
            .join("checksums-test")
            .join("profile.toml");
        let content = fs::read_to_string(&meta_path).unwrap();
        let meta: ProfileMeta = toml::from_str(&content).unwrap();

        assert_eq!(meta.checksums.len(), 1);
        let (key, hash) = meta.checksums.iter().next().unwrap();
        assert!(key.starts_with("faketool/"));
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 hex
    }

    #[test]
    fn test_load_creates_backup() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let mut manifest = sample_manifest(tmp.path());
        manifest.save(&config_dir).unwrap();

        save_profile(&config_dir, &manifest, "backup-test").unwrap();

        let result = load_profile(&config_dir, &mut manifest, "backup-test", false).unwrap();
        assert_eq!(result.backed_up_files, 1);

        let backup_dir = config_dir.join("backups");
        assert!(backup_dir.is_dir());
        let backups: Vec<_> = fs::read_dir(&backup_dir).unwrap().collect();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_load_add_untracked() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let manifest = sample_manifest(tmp.path());
        manifest.save(&config_dir).unwrap();
        save_profile(&config_dir, &manifest, "untracked-test").unwrap();

        // Start with empty manifest
        let mut empty_manifest = Manifest::default();
        empty_manifest.save(&config_dir).unwrap();

        let result =
            load_profile(&config_dir, &mut empty_manifest, "untracked-test", true).unwrap();
        assert_eq!(result.tools_added.len(), 1);
        assert_eq!(result.tools_added[0], "faketool");
        assert!(empty_manifest.has_tool("faketool"));
    }

    #[test]
    fn test_load_skip_untracked() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let manifest = sample_manifest(tmp.path());
        manifest.save(&config_dir).unwrap();
        save_profile(&config_dir, &manifest, "skip-test").unwrap();

        let mut empty_manifest = Manifest::default();
        empty_manifest.save(&config_dir).unwrap();

        let result = load_profile(&config_dir, &mut empty_manifest, "skip-test", false).unwrap();
        assert_eq!(result.skipped_tools.len(), 1);
        assert_eq!(result.restored_files, 0);
    }

    #[test]
    fn test_list_empty() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let profiles = list_profiles(&config_dir).unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_list_multiple() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let manifest = sample_manifest(tmp.path());
        save_profile(&config_dir, &manifest, "alpha").unwrap();
        save_profile(&config_dir, &manifest, "beta").unwrap();

        let profiles = list_profiles(&config_dir).unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].name, "alpha");
        assert_eq!(profiles[1].name, "beta");
    }

    #[test]
    fn test_delete_profile() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let manifest = sample_manifest(tmp.path());
        save_profile(&config_dir, &manifest, "to-delete").unwrap();

        assert!(profiles_dir(&config_dir).join("to-delete").exists());
        delete_profile(&config_dir, "to-delete").unwrap();
        assert!(!profiles_dir(&config_dir).join("to-delete").exists());
    }

    #[test]
    fn test_delete_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let result = delete_profile(&config_dir, "nope");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_save_duplicate_error() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("dotsmith");
        fs::create_dir_all(&config_dir).unwrap();

        let manifest = sample_manifest(tmp.path());
        save_profile(&config_dir, &manifest, "dupe").unwrap();

        let result = save_profile(&config_dir, &manifest, "dupe");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }
}
