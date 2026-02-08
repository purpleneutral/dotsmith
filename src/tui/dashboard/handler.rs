use crossterm::event::{KeyCode, KeyEvent};

use super::DashboardState;

/// Action returned by the dashboard key handler.
pub enum DashboardAction {
    /// No state change.
    None,
    /// User wants to explore the selected tool.
    Explore(String),
    /// Snapshot all tracked tools.
    SnapshotAll,
    /// Reload the selected tool.
    ReloadSelected(String),
    /// Show diff for the selected tool.
    ShowDiff(String),
    /// Show history for the selected tool.
    ShowHistory(String),
    /// Show plugins for the selected tool.
    ShowPlugins(String),
    /// Sync the git repo.
    SyncRepo,
    /// User wants to quit.
    Quit,
}

/// Handle a key event in the dashboard view.
pub fn handle_key(key: KeyEvent, state: &mut DashboardState) -> DashboardAction {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => DashboardAction::Quit,
        KeyCode::Char('j') | KeyCode::Down => {
            state.select_next();
            DashboardAction::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.select_prev();
            DashboardAction::None
        }
        KeyCode::Char('e') | KeyCode::Enter => {
            if let Some(tool) = state.selected_tool() {
                if tool.has_option_db {
                    DashboardAction::Explore(tool.name.clone())
                } else {
                    DashboardAction::None
                }
            } else {
                DashboardAction::None
            }
        }
        KeyCode::Char('s') => DashboardAction::SnapshotAll,
        KeyCode::Char('r') => state
            .selected_tool()
            .map(|t| DashboardAction::ReloadSelected(t.name.clone()))
            .unwrap_or(DashboardAction::None),
        KeyCode::Char('d') => state
            .selected_tool()
            .map(|t| DashboardAction::ShowDiff(t.name.clone()))
            .unwrap_or(DashboardAction::None),
        KeyCode::Char('h') => state
            .selected_tool()
            .map(|t| DashboardAction::ShowHistory(t.name.clone()))
            .unwrap_or(DashboardAction::None),
        KeyCode::Char('p') => state
            .selected_tool()
            .map(|t| DashboardAction::ShowPlugins(t.name.clone()))
            .unwrap_or(DashboardAction::None),
        KeyCode::Char('g') => DashboardAction::SyncRepo,
        _ => DashboardAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::manifest::{Manifest, ToolEntry};
    use crate::tui::dashboard::DashboardState;
    use chrono::Utc;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use std::collections::BTreeMap;

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn sample_state() -> DashboardState {
        let mut m = Manifest::default();
        m.tools.insert(
            "tmux".to_string(),
            ToolEntry {
                tier: 1,
                config_paths: vec!["~/.config/tmux/tmux.conf".to_string()],
                plugins_managed: false,
                plugin_manager: None,
                added_at: Utc::now(),
                last_snapshot: None,
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
        DashboardState::from_manifest(&m)
    }

    #[test]
    fn test_quit_q() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('q')), &mut state),
            DashboardAction::Quit
        ));
    }

    #[test]
    fn test_quit_esc() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Esc), &mut state),
            DashboardAction::Quit
        ));
    }

    #[test]
    fn test_navigate_down() {
        let mut state = sample_state();
        assert_eq!(state.selected, 0);
        handle_key(make_key(KeyCode::Char('j')), &mut state);
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn test_navigate_up() {
        let mut state = sample_state();
        state.selected = 1;
        handle_key(make_key(KeyCode::Char('k')), &mut state);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_explore_tier1() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Char('e')), &mut state);
        assert!(matches!(action, DashboardAction::Explore(name) if name == "tmux"));
    }

    #[test]
    fn test_explore_enter() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Enter), &mut state);
        assert!(matches!(action, DashboardAction::Explore(name) if name == "tmux"));
    }

    #[test]
    fn test_snapshot_all() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('s')), &mut state),
            DashboardAction::SnapshotAll
        ));
    }

    #[test]
    fn test_reload_selected() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Char('r')), &mut state);
        assert!(matches!(action, DashboardAction::ReloadSelected(name) if name == "tmux"));
    }

    #[test]
    fn test_show_diff() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Char('d')), &mut state);
        assert!(matches!(action, DashboardAction::ShowDiff(name) if name == "tmux"));
    }

    #[test]
    fn test_show_history() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Char('h')), &mut state);
        assert!(matches!(action, DashboardAction::ShowHistory(name) if name == "tmux"));
    }

    #[test]
    fn test_show_plugins() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Char('p')), &mut state);
        assert!(matches!(action, DashboardAction::ShowPlugins(name) if name == "tmux"));
    }

    #[test]
    fn test_sync_repo() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('g')), &mut state),
            DashboardAction::SyncRepo
        ));
    }

    #[test]
    fn test_unknown_key() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('x')), &mut state),
            DashboardAction::None
        ));
    }
}
