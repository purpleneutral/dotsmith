pub mod handler;
pub mod view;

use chrono::{DateTime, Utc};

use crate::core::manifest::Manifest;
use crate::core::module::ModuleRegistry;

/// A row in the dashboard table.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolRow {
    pub name: String,
    pub tier: u8,
    pub tier_label: String,
    pub config_path_count: usize,
    pub plugin_count: usize,
    pub last_snapshot: Option<DateTime<Utc>>,
    pub has_option_db: bool,
}

/// State for the dashboard view.
pub struct DashboardState {
    pub tools: Vec<ToolRow>,
    pub selected: usize,
}

impl DashboardState {
    /// Build dashboard state from a loaded manifest.
    pub fn from_manifest(manifest: &Manifest) -> Self {
        let tools: Vec<ToolRow> = manifest
            .tools
            .iter()
            .map(|(name, entry)| {
                let has_option_db = ModuleRegistry::get_options(name).is_some();
                let tier_label = match entry.tier {
                    1 => "Full".to_string(),
                    2 => "Auto".to_string(),
                    _ => format!("T{}", entry.tier),
                };
                ToolRow {
                    name: name.clone(),
                    tier: entry.tier,
                    tier_label,
                    config_path_count: entry.config_paths.len(),
                    plugin_count: entry.plugins.len(),
                    last_snapshot: entry.last_snapshot,
                    has_option_db,
                }
            })
            .collect();

        Self { tools, selected: 0 }
    }

    pub fn select_next(&mut self) {
        if !self.tools.is_empty() {
            self.selected = (self.selected + 1).min(self.tools.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Get the currently selected tool name, if any.
    pub fn selected_tool(&self) -> Option<&ToolRow> {
        self.tools.get(self.selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::manifest::{Manifest, ToolEntry};
    use chrono::Utc;
    use std::collections::BTreeMap;

    fn sample_manifest() -> Manifest {
        let mut m = Manifest::default();
        m.tools.insert(
            "tmux".to_string(),
            ToolEntry {
                tier: 1,
                config_paths: vec![
                    "~/.config/tmux/tmux.conf".to_string(),
                    "~/.tmux.conf".to_string(),
                ],
                plugins_managed: false,
                plugin_manager: None,
                added_at: Utc::now(),
                last_snapshot: Some(Utc::now()),
                plugins: BTreeMap::new(),
            },
        );
        m.tools.insert(
            "zsh".to_string(),
            ToolEntry {
                tier: 1,
                config_paths: vec!["~/.config/zsh/.zshrc".to_string()],
                plugins_managed: false,
                plugin_manager: None,
                added_at: Utc::now(),
                last_snapshot: None,
                plugins: BTreeMap::new(),
            },
        );
        m
    }

    #[test]
    fn test_from_manifest() {
        let manifest = sample_manifest();
        let state = DashboardState::from_manifest(&manifest);
        assert_eq!(state.tools.len(), 2);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_tool_rows() {
        let manifest = sample_manifest();
        let state = DashboardState::from_manifest(&manifest);

        let tmux = state.tools.iter().find(|t| t.name == "tmux").unwrap();
        assert_eq!(tmux.tier, 1);
        assert_eq!(tmux.tier_label, "Full");
        assert_eq!(tmux.config_path_count, 2);
        assert!(tmux.has_option_db);
        assert!(tmux.last_snapshot.is_some());

        let zsh = state.tools.iter().find(|t| t.name == "zsh").unwrap();
        assert_eq!(zsh.config_path_count, 1);
        assert!(zsh.last_snapshot.is_none());
    }

    #[test]
    fn test_navigation() {
        let manifest = sample_manifest();
        let mut state = DashboardState::from_manifest(&manifest);

        assert_eq!(state.selected, 0);
        state.select_next();
        assert_eq!(state.selected, 1);
        state.select_next();
        assert_eq!(state.selected, 1); // clamped
        state.select_prev();
        assert_eq!(state.selected, 0);
        state.select_prev();
        assert_eq!(state.selected, 0); // clamped
    }

    #[test]
    fn test_empty_manifest() {
        let manifest = Manifest::default();
        let state = DashboardState::from_manifest(&manifest);
        assert!(state.tools.is_empty());
        assert_eq!(state.selected, 0);
        assert!(state.selected_tool().is_none());
    }
}
