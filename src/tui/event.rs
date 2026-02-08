use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

/// Poll for the next key press event.
/// Returns `None` if no event is available within the tick interval (250ms).
/// Filters to `KeyEventKind::Press` only to avoid double-firing on crossterm 0.28.
pub fn next_key_event() -> Result<Option<KeyEvent>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(Some(key));
            }
        }
    }
    Ok(None)
}
