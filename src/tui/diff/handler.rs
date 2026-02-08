use crossterm::event::{KeyCode, KeyEvent};

use super::DiffState;

pub enum DiffAction {
    None,
    Back,
    Quit,
}

pub fn handle_key(key: KeyEvent, state: &mut DiffState) -> DiffAction {
    match key.code {
        KeyCode::Char('q') => DiffAction::Quit,
        KeyCode::Esc => DiffAction::Back,
        KeyCode::Char('j') | KeyCode::Down => {
            state.scroll_down();
            DiffAction::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.scroll_up();
            DiffAction::None
        }
        KeyCode::Char('d') | KeyCode::PageDown => {
            state.page_down();
            DiffAction::None
        }
        KeyCode::Char('u') | KeyCode::PageUp => {
            state.page_up();
            DiffAction::None
        }
        KeyCode::Char('g') | KeyCode::Home => {
            state.scroll_to_top();
            DiffAction::None
        }
        KeyCode::Char('G') | KeyCode::End => {
            state.scroll_to_end();
            DiffAction::None
        }
        _ => DiffAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::diff::DiffState;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn sample_state() -> DiffState {
        DiffState::from_strings("tmux", "tmux.conf", "old line\n", "new line\n")
    }

    #[test]
    fn test_quit() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('q')), &mut state),
            DiffAction::Quit
        ));
    }

    #[test]
    fn test_back() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Esc), &mut state),
            DiffAction::Back
        ));
    }

    #[test]
    fn test_scroll() {
        let mut state = sample_state();
        state.visible_height = 2;
        handle_key(make_key(KeyCode::Char('j')), &mut state);
        assert!(state.scroll_offset > 0 || state.lines.len() <= state.visible_height);
    }
}
