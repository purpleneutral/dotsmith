use serde::{Deserialize, Serialize};

/// Dotsmith's own configuration, stored at <config_dir>/config.toml
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DotsmithConfig {
    #[serde(default)]
    pub general: GeneralConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Where managed config source files live (for future deploy command).
    #[serde(default = "default_configs_dir")]
    pub configs_dir: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            configs_dir: default_configs_dir(),
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
    }

    #[test]
    fn test_config_default_values() {
        let config = DotsmithConfig::default();
        assert_eq!(config.general.configs_dir, "~/.config/dotsmith/configs");
    }
}
