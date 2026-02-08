use crossterm::event::{KeyCode, KeyEvent};

use super::HistoryState;

pub enum HistoryAction {
    None,
    Back,
    Quit,
    ViewSnapshot(i64),
    Rollback(i64),
}

pub fn handle_key(key: KeyEvent, state: &mut HistoryState) -> HistoryAction {
    match key.code {
        KeyCode::Char('q') => HistoryAction::Quit,
        KeyCode::Esc => HistoryAction::Back,
        KeyCode::Char('j') | KeyCode::Down => {
            state.select_next();
            HistoryAction::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.select_prev();
            HistoryAction::None
        }
        KeyCode::Enter => state
            .selected_entry()
            .map(|e| HistoryAction::ViewSnapshot(e.id))
            .unwrap_or(HistoryAction::None),
        KeyCode::Char('r') => state
            .selected_entry()
            .map(|e| HistoryAction::Rollback(e.id))
            .unwrap_or(HistoryAction::None),
        _ => HistoryAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::snapshot::SnapshotSummary;
    use crate::tui::history::HistoryState;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn sample_state() -> HistoryState {
        HistoryState {
            tool_name: "tmux".to_string(),
            entries: vec![
                SnapshotSummary {
                    id: 3,
                    tool: "tmux".into(),
                    file_path: "tmux.conf".into(),
                    hash: "abc12345".into(),
                    message: Some("test".into()),
                    created_at: "2026-02-08".into(),
                },
                SnapshotSummary {
                    id: 2,
                    tool: "tmux".into(),
                    file_path: "tmux.conf".into(),
                    hash: "def67890".into(),
                    message: None,
                    created_at: "2026-02-07".into(),
                },
            ],
            selected: 0,
        }
    }

    #[test]
    fn test_quit() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('q')), &mut state),
            HistoryAction::Quit
        ));
    }

    #[test]
    fn test_back() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Esc), &mut state),
            HistoryAction::Back
        ));
    }

    #[test]
    fn test_view_snapshot() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Enter), &mut state);
        assert!(matches!(action, HistoryAction::ViewSnapshot(3)));
    }

    #[test]
    fn test_rollback() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Char('r')), &mut state);
        assert!(matches!(action, HistoryAction::Rollback(3)));
    }

    #[test]
    fn test_navigate() {
        let mut state = sample_state();
        handle_key(make_key(KeyCode::Char('j')), &mut state);
        assert_eq!(state.selected, 1);
        handle_key(make_key(KeyCode::Char('k')), &mut state);
        assert_eq!(state.selected, 0);
    }
}
