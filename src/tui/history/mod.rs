pub mod handler;
pub mod view;

use crate::core::snapshot::{SnapshotEngine, SnapshotSummary};

/// State for the history view.
pub struct HistoryState {
    pub tool_name: String,
    pub entries: Vec<SnapshotSummary>,
    pub selected: usize,
}

impl HistoryState {
    pub fn new(tool: &str, engine: &SnapshotEngine) -> Self {
        let entries = engine.history(tool, 50).unwrap_or_default();
        Self {
            tool_name: tool.to_string(),
            entries,
            selected: 0,
        }
    }

    pub fn select_next(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1).min(self.entries.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn selected_entry(&self) -> Option<&SnapshotSummary> {
        self.entries.get(self.selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // HistoryState::new requires a real DB, so we test navigation with a manually built state
    fn sample_state() -> HistoryState {
        let entries = vec![
            SnapshotSummary {
                id: 3,
                tool: "tmux".into(),
                file_path: "~/.config/tmux/tmux.conf".into(),
                hash: "abc12345".into(),
                message: Some("test snapshot".into()),
                created_at: "2026-02-08 12:00:00".into(),
            },
            SnapshotSummary {
                id: 2,
                tool: "tmux".into(),
                file_path: "~/.config/tmux/tmux.conf".into(),
                hash: "def67890".into(),
                message: None,
                created_at: "2026-02-07 12:00:00".into(),
            },
        ];
        HistoryState {
            tool_name: "tmux".to_string(),
            entries,
            selected: 0,
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
        state.select_prev();
        assert_eq!(state.selected, 0); // clamped
    }

    #[test]
    fn test_selected_entry() {
        let state = sample_state();
        let entry = state.selected_entry().unwrap();
        assert_eq!(entry.id, 3);
    }

    #[test]
    fn test_empty_state() {
        let state = HistoryState {
            tool_name: "tmux".to_string(),
            entries: vec![],
            selected: 0,
        };
        assert!(state.selected_entry().is_none());
    }
}
