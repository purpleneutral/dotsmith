pub mod handler;
pub mod view;

use crate::core::manifest::Manifest;
use crate::core::plugin;

#[derive(Debug, Clone)]
pub struct PluginRow {
    pub name: String,
    pub repo: String,
    pub init: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PluginMode {
    List,
    AddInput,
}

pub struct PluginState {
    pub tool_name: String,
    pub plugins: Vec<PluginRow>,
    pub selected: usize,
    pub mode: PluginMode,
    pub input_buffer: String,
    pub supported: bool,
}

impl PluginState {
    pub fn new(tool: &str, manifest: &Manifest) -> Self {
        let supported = plugin::validate_tool_supported(tool).is_ok();
        let plugins = if supported {
            plugin::list_plugins(manifest, tool)
                .unwrap_or_default()
                .into_iter()
                .map(|(name, repo, init)| PluginRow { name, repo, init })
                .collect()
        } else {
            Vec::new()
        };

        Self {
            tool_name: tool.to_string(),
            plugins,
            selected: 0,
            mode: PluginMode::List,
            input_buffer: String::new(),
            supported,
        }
    }

    pub fn select_next(&mut self) {
        if !self.plugins.is_empty() {
            self.selected = (self.selected + 1).min(self.plugins.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn selected_plugin(&self) -> Option<&PluginRow> {
        self.plugins.get(self.selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> PluginState {
        PluginState {
            tool_name: "zsh".to_string(),
            plugins: vec![
                PluginRow {
                    name: "zsh-autosuggestions".into(),
                    repo: "zsh-users/zsh-autosuggestions".into(),
                    init: "zsh-autosuggestions.plugin.zsh".into(),
                },
                PluginRow {
                    name: "zsh-syntax-highlighting".into(),
                    repo: "zsh-users/zsh-syntax-highlighting".into(),
                    init: "zsh-syntax-highlighting.plugin.zsh".into(),
                },
            ],
            selected: 0,
            mode: PluginMode::List,
            input_buffer: String::new(),
            supported: true,
        }
    }

    #[test]
    fn test_navigation() {
        let mut state = sample_state();
        assert_eq!(state.selected, 0);
        state.select_next();
        assert_eq!(state.selected, 1);
        state.select_next();
        assert_eq!(state.selected, 1); // clamped
        state.select_prev();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_selected_plugin() {
        let state = sample_state();
        let p = state.selected_plugin().unwrap();
        assert_eq!(p.name, "zsh-autosuggestions");
    }

    #[test]
    fn test_empty_plugins() {
        let state = PluginState {
            tool_name: "git".to_string(),
            plugins: vec![],
            selected: 0,
            mode: PluginMode::List,
            input_buffer: String::new(),
            supported: false,
        };
        assert!(state.selected_plugin().is_none());
        assert!(!state.supported);
    }
}
