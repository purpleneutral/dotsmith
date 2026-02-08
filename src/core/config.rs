use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::util;

/// Dotsmith's own configuration, stored at <config_dir>/config.toml
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DotsmithConfig {
    #[serde(default)]
    pub general: GeneralConfig,
}

impl DotsmithConfig {
    /// Load config from disk. Returns default if file doesn't exist.
    pub fn load(config_dir: &Path) -> Self {
        let path = config_dir.join("config.toml");
        if let Ok(content) = std::fs::read_to_string(&path) {
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save config to disk atomically.
    pub fn save(&self, config_dir: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).context("failed to serialize config")?;
        util::fs::atomic_write(&config_dir.join("config.toml"), &content)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Where managed config source files live (for future deploy command).
    #[serde(default = "default_configs_dir")]
    pub configs_dir: String,

    /// Path to the git repo for dotfile backups.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_path: Option<String>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            configs_dir: default_configs_dir(),
            repo_path: None,
        }
    }
}

fn default_configs_dir() -> String {
    "~/.config/dotsmith/configs".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_roundtrip() {
        let config = DotsmithConfig::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: DotsmithConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized.general.configs_dir,
            "~/.config/dotsmith/configs"
        );
        assert!(deserialized.general.repo_path.is_none());
    }

    #[test]
    fn test_config_with_repo_path() {
        let mut config = DotsmithConfig::default();
        config.general.repo_path = Some("~/dots".to_string());
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: DotsmithConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.general.repo_path.as_deref(), Some("~/dots"));
    }

    #[test]
    fn test_config_default_values() {
        let config = DotsmithConfig::default();
        assert_eq!(config.general.configs_dir, "~/.config/dotsmith/configs");
        assert!(config.general.repo_path.is_none());
    }
}
