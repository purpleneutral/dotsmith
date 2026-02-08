use serde::{Deserialize, Serialize};

/// Metadata about a supported tool, loaded from module.toml.
/// Embedded in the binary for Tier 1 tools via `include_str!`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleDefinition {
    pub metadata: ModuleMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub homepage: String,

    /// Config file paths to check, in priority order.
    /// First one found wins. Supports tilde expansion.
    pub config_paths: Vec<String>,

    /// Shell command to detect if the tool is installed.
    pub detect_command: String,

    /// Shell command to reload config. `{config_path}` is replaced at runtime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reload_command: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reload_description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub man_page: Option<String>,

    /// Config file format (determines which parser to use).
    pub config_format: String,

    /// Whether dotsmith can manage plugins for this tool.
    #[serde(default)]
    pub plugins_supported: bool,

    /// Plugin directory path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_dir: Option<String>,

    /// Default config shipped with dotsmith (future use).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_config: Option<String>,

    /// Option categories for TUI grouping.
    #[serde(default)]
    pub categories: Vec<String>,
}

// ---------------------------------------------------------------------------
// Option database types
// ---------------------------------------------------------------------------

/// The complete option database for a tool, loaded from options.toml.
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionDatabase {
    pub options: Vec<OptionEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptionEntry {
    pub name: String,

    #[serde(rename = "type")]
    pub option_type: OptionType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,

    pub category: String,
    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OptionType {
    Boolean,
    String,
    Integer,
    Float,
    Enum,
    Color,
    Path,
    List,
    #[serde(rename = "keybinding")]
    KeyBinding,
}

// ---------------------------------------------------------------------------
// Module registry (embedded Tier 1 data)
// ---------------------------------------------------------------------------

/// Registry of built-in modules. Uses `include_str!` to embed module data
/// in the binary at compile time.
pub struct ModuleRegistry;

impl ModuleRegistry {
    /// Get the module definition for a built-in tool.
    pub fn get_builtin(name: &str) -> Option<ModuleDefinition> {
        let toml_str = match name {
            "tmux" => include_str!("../../data/modules/tmux/module.toml"),
            "zsh" => include_str!("../../data/modules/zsh/module.toml"),
            "git" => include_str!("../../data/modules/git/module.toml"),
            _ => return None,
        };
        match toml::from_str(toml_str) {
            Ok(def) => Some(def),
            Err(e) => {
                eprintln!("warning: failed to parse built-in module '{}': {}", name, e);
                None
            }
        }
    }

    /// Get the option database for a built-in tool.
    pub fn get_options(name: &str) -> Option<OptionDatabase> {
        let toml_str = match name {
            "tmux" => include_str!("../../data/modules/tmux/options.toml"),
            "zsh" => include_str!("../../data/modules/zsh/options.toml"),
            "git" => include_str!("../../data/modules/git/options.toml"),
            _ => return None,
        };
        match toml::from_str(toml_str) {
            Ok(db) => Some(db),
            Err(e) => {
                eprintln!(
                    "warning: failed to parse option database for '{}': {}",
                    name, e
                );
                None
            }
        }
    }

    /// List all built-in module names.
    #[allow(dead_code)]
    pub fn builtin_names() -> &'static [&'static str] {
        &["git", "tmux", "zsh"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_tmux_module() {
        let module = ModuleRegistry::get_builtin("tmux").expect("tmux module should exist");
        assert_eq!(module.metadata.name, "tmux");
        assert_eq!(module.metadata.display_name, "tmux");
        assert!(!module.metadata.config_paths.is_empty());
        assert!(module.metadata.plugins_supported);
    }

    #[test]
    fn test_load_tmux_options() {
        let db = ModuleRegistry::get_options("tmux").expect("tmux options should exist");
        assert!(!db.options.is_empty(), "should have at least one option");

        // Check that a known option exists
        let mouse = db.options.iter().find(|o| o.name == "mouse");
        assert!(mouse.is_some(), "should have 'mouse' option");

        let mouse = mouse.unwrap();
        assert_eq!(mouse.option_type, OptionType::Boolean);
        assert_eq!(mouse.category, "interaction");
    }

    #[test]
    fn test_unknown_module() {
        assert!(ModuleRegistry::get_builtin("nonexistent").is_none());
        assert!(ModuleRegistry::get_options("nonexistent").is_none());
    }

    #[test]
    fn test_builtin_names() {
        let names = ModuleRegistry::builtin_names();
        assert!(names.contains(&"tmux"));
    }
}
