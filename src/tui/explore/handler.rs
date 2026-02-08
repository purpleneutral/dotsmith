use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{ExploreState, Panel};

/// Action returned by the explore key handler.
pub enum ExploreAction {
    /// No state change.
    None,
    /// User wants to go back to the dashboard.
    Back,
    /// User wants to quit.
    Quit,
}

/// Handle a key event in the explore view.
pub fn handle_key(key: KeyEvent, state: &mut ExploreState) -> ExploreAction {
    if state.search_mode {
        return handle_search_key(key, state);
    }

    match key.code {
        KeyCode::Char('q') => ExploreAction::Quit,
        KeyCode::Esc => ExploreAction::Back,
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                state.cycle_focus_backward();
            } else {
                state.cycle_focus_forward();
            }
            ExploreAction::None
        }
        KeyCode::BackTab => {
            state.cycle_focus_backward();
            ExploreAction::None
        }
        KeyCode::Char('j') | KeyCode::Down => {
            match state.focus {
                Panel::Categories => state.select_next_category(),
                Panel::Options | Panel::Details => state.select_next_option(),
            }
            ExploreAction::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            match state.focus {
                Panel::Categories => state.select_prev_category(),
                Panel::Options | Panel::Details => state.select_prev_option(),
            }
            ExploreAction::None
        }
        KeyCode::Enter => {
            if state.focus == Panel::Categories {
                state.focus = Panel::Options;
            }
            ExploreAction::None
        }
        KeyCode::Char('/') => {
            state.search_mode = true;
            ExploreAction::None
        }
        _ => ExploreAction::None,
    }
}

/// Handle keys while in search mode.
fn handle_search_key(key: KeyEvent, state: &mut ExploreState) -> ExploreAction {
    match key.code {
        KeyCode::Esc => {
            // Cancel search, clear query
            state.search_mode = false;
            state.search_query.clear();
            state.apply_filters();
            ExploreAction::None
        }
        KeyCode::Enter => {
            // Confirm search, exit search mode but keep filter
            state.search_mode = false;
            ExploreAction::None
        }
        KeyCode::Backspace => {
            state.search_query.pop();
            state.apply_filters();
            ExploreAction::None
        }
        KeyCode::Char(c) => {
            state.search_query.push(c);
            state.apply_filters();
            ExploreAction::None
        }
        _ => ExploreAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::explore::ExploreState;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn make_key_shift(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn sample_state() -> ExploreState {
        ExploreState::new("tmux").unwrap()
    }

    #[test]
    fn test_quit() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('q')), &mut state),
            ExploreAction::Quit
        ));
    }

    #[test]
    fn test_back() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Esc), &mut state),
            ExploreAction::Back
        ));
    }

    #[test]
    fn test_tab_focus() {
        let mut state = sample_state();
        assert_eq!(state.focus, Panel::Categories);
        handle_key(make_key(KeyCode::Tab), &mut state);
        assert_eq!(state.focus, Panel::Options);
        handle_key(make_key(KeyCode::Tab), &mut state);
        assert_eq!(state.focus, Panel::Details);
        handle_key(make_key(KeyCode::Tab), &mut state);
        assert_eq!(state.focus, Panel::Categories);
    }

    #[test]
    fn test_shift_tab_focus() {
        let mut state = sample_state();
        handle_key(make_key_shift(KeyCode::Tab), &mut state);
        assert_eq!(state.focus, Panel::Details);
    }

    #[test]
    fn test_backtab_focus() {
        let mut state = sample_state();
        handle_key(make_key(KeyCode::BackTab), &mut state);
        assert_eq!(state.focus, Panel::Details);
    }

    #[test]
    fn test_navigate_categories() {
        let mut state = sample_state();
        assert_eq!(state.category_selected, 0);
        handle_key(make_key(KeyCode::Char('j')), &mut state);
        assert_eq!(state.category_selected, 1);
        handle_key(make_key(KeyCode::Char('k')), &mut state);
        assert_eq!(state.category_selected, 0);
    }

    #[test]
    fn test_navigate_options() {
        let mut state = sample_state();
        state.focus = Panel::Options;
        assert_eq!(state.option_selected, 0);
        handle_key(make_key(KeyCode::Char('j')), &mut state);
        assert_eq!(state.option_selected, 1);
    }

    #[test]
    fn test_enter_search() {
        let mut state = sample_state();
        handle_key(make_key(KeyCode::Char('/')), &mut state);
        assert!(state.search_mode);
    }

    #[test]
    fn test_search_type_and_confirm() {
        let mut state = sample_state();
        state.search_mode = true;
        handle_key(make_key(KeyCode::Char('m')), &mut state);
        handle_key(make_key(KeyCode::Char('o')), &mut state);
        assert_eq!(state.search_query, "mo");
        assert!(state.search_mode);

        handle_key(make_key(KeyCode::Enter), &mut state);
        assert!(!state.search_mode);
        assert_eq!(state.search_query, "mo"); // kept
    }

    #[test]
    fn test_search_cancel() {
        let mut state = sample_state();
        state.search_mode = true;
        handle_key(make_key(KeyCode::Char('x')), &mut state);
        assert_eq!(state.search_query, "x");

        handle_key(make_key(KeyCode::Esc), &mut state);
        assert!(!state.search_mode);
        assert!(state.search_query.is_empty()); // cleared
    }

    #[test]
    fn test_search_backspace() {
        let mut state = sample_state();
        state.search_mode = true;
        handle_key(make_key(KeyCode::Char('a')), &mut state);
        handle_key(make_key(KeyCode::Char('b')), &mut state);
        assert_eq!(state.search_query, "ab");

        handle_key(make_key(KeyCode::Backspace), &mut state);
        assert_eq!(state.search_query, "a");
    }

    #[test]
    fn test_enter_on_category_switches_to_options() {
        let mut state = sample_state();
        assert_eq!(state.focus, Panel::Categories);
        handle_key(make_key(KeyCode::Enter), &mut state);
        assert_eq!(state.focus, Panel::Options);
    }

    #[test]
    fn test_q_in_search_mode_types_q() {
        let mut state = sample_state();
        state.search_mode = true;
        let action = handle_key(make_key(KeyCode::Char('q')), &mut state);
        assert!(matches!(action, ExploreAction::None));
        assert_eq!(state.search_query, "q");
    }
}
