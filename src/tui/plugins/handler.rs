use crossterm::event::{KeyCode, KeyEvent};

use super::{PluginMode, PluginState};

pub enum PluginAction {
    None,
    Back,
    Quit,
    AddPlugin(String),
    RemovePlugin(String),
    UpdatePlugin(Option<String>),
}

pub fn handle_key(key: KeyEvent, state: &mut PluginState) -> PluginAction {
    match state.mode {
        PluginMode::List => handle_list_key(key, state),
        PluginMode::AddInput => handle_add_input_key(key, state),
    }
}

fn handle_list_key(key: KeyEvent, state: &mut PluginState) -> PluginAction {
    match key.code {
        KeyCode::Char('q') => PluginAction::Quit,
        KeyCode::Esc => PluginAction::Back,
        KeyCode::Char('j') | KeyCode::Down => {
            state.select_next();
            PluginAction::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.select_prev();
            PluginAction::None
        }
        KeyCode::Char('a') if state.supported => {
            state.mode = PluginMode::AddInput;
            state.input_buffer.clear();
            PluginAction::None
        }
        KeyCode::Char('d') if state.supported => state
            .selected_plugin()
            .map(|p| PluginAction::RemovePlugin(p.name.clone()))
            .unwrap_or(PluginAction::None),
        KeyCode::Char('u') if state.supported => state
            .selected_plugin()
            .map(|p| PluginAction::UpdatePlugin(Some(p.name.clone())))
            .unwrap_or(PluginAction::None),
        KeyCode::Char('U') if state.supported => PluginAction::UpdatePlugin(None),
        _ => PluginAction::None,
    }
}

fn handle_add_input_key(key: KeyEvent, state: &mut PluginState) -> PluginAction {
    match key.code {
        KeyCode::Esc => {
            state.mode = PluginMode::List;
            state.input_buffer.clear();
            PluginAction::None
        }
        KeyCode::Enter => {
            let input = state.input_buffer.trim().to_string();
            state.mode = PluginMode::List;
            state.input_buffer.clear();
            if input.is_empty() {
                PluginAction::None
            } else {
                PluginAction::AddPlugin(input)
            }
        }
        KeyCode::Backspace => {
            state.input_buffer.pop();
            PluginAction::None
        }
        KeyCode::Char(c) => {
            state.input_buffer.push(c);
            PluginAction::None
        }
        _ => PluginAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::plugins::{PluginMode, PluginRow, PluginState};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

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
    fn test_quit() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('q')), &mut state),
            PluginAction::Quit
        ));
    }

    #[test]
    fn test_back() {
        let mut state = sample_state();
        assert!(matches!(
            handle_key(make_key(KeyCode::Esc), &mut state),
            PluginAction::Back
        ));
    }

    #[test]
    fn test_enter_add_mode() {
        let mut state = sample_state();
        handle_key(make_key(KeyCode::Char('a')), &mut state);
        assert_eq!(state.mode, PluginMode::AddInput);
    }

    #[test]
    fn test_add_input_type_and_submit() {
        let mut state = sample_state();
        state.mode = PluginMode::AddInput;
        handle_key(make_key(KeyCode::Char('f')), &mut state);
        handle_key(make_key(KeyCode::Char('o')), &mut state);
        handle_key(make_key(KeyCode::Char('o')), &mut state);
        assert_eq!(state.input_buffer, "foo");

        let action = handle_key(make_key(KeyCode::Enter), &mut state);
        assert!(matches!(action, PluginAction::AddPlugin(s) if s == "foo"));
        assert_eq!(state.mode, PluginMode::List);
    }

    #[test]
    fn test_add_input_cancel() {
        let mut state = sample_state();
        state.mode = PluginMode::AddInput;
        handle_key(make_key(KeyCode::Char('x')), &mut state);
        handle_key(make_key(KeyCode::Esc), &mut state);
        assert_eq!(state.mode, PluginMode::List);
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_add_input_backspace() {
        let mut state = sample_state();
        state.mode = PluginMode::AddInput;
        handle_key(make_key(KeyCode::Char('a')), &mut state);
        handle_key(make_key(KeyCode::Char('b')), &mut state);
        handle_key(make_key(KeyCode::Backspace), &mut state);
        assert_eq!(state.input_buffer, "a");
    }

    #[test]
    fn test_add_empty_input() {
        let mut state = sample_state();
        state.mode = PluginMode::AddInput;
        let action = handle_key(make_key(KeyCode::Enter), &mut state);
        assert!(matches!(action, PluginAction::None));
    }

    #[test]
    fn test_remove_plugin() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Char('d')), &mut state);
        assert!(matches!(action, PluginAction::RemovePlugin(s) if s == "zsh-autosuggestions"));
    }

    #[test]
    fn test_update_plugin() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Char('u')), &mut state);
        assert!(
            matches!(action, PluginAction::UpdatePlugin(Some(s)) if s == "zsh-autosuggestions")
        );
    }

    #[test]
    fn test_update_all() {
        let mut state = sample_state();
        let action = handle_key(make_key(KeyCode::Char('U')), &mut state);
        assert!(matches!(action, PluginAction::UpdatePlugin(None)));
    }

    #[test]
    fn test_navigate() {
        let mut state = sample_state();
        handle_key(make_key(KeyCode::Char('j')), &mut state);
        assert_eq!(state.selected, 1);
        handle_key(make_key(KeyCode::Char('k')), &mut state);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_unsupported_tool_ignores_actions() {
        let mut state = PluginState {
            tool_name: "git".to_string(),
            plugins: vec![],
            selected: 0,
            mode: PluginMode::List,
            input_buffer: String::new(),
            supported: false,
        };
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('a')), &mut state),
            PluginAction::None
        ));
        assert!(matches!(
            handle_key(make_key(KeyCode::Char('d')), &mut state),
            PluginAction::None
        ));
    }
}
