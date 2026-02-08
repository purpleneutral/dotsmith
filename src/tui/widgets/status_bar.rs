use std::time::Instant;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

/// Toast severity level.
#[derive(Clone, Copy)]
pub enum ToastLevel {
    Success,
    Error,
}

/// A transient notification message with auto-expiry.
pub struct Toast {
    pub message: String,
    pub level: ToastLevel,
    pub created_at: Instant,
}

impl Toast {
    pub fn new(message: String, level: ToastLevel) -> Self {
        Self {
            message,
            level,
            created_at: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() >= 3
    }
}

/// Data for rendering the status bar.
pub struct StatusBarData<'a> {
    pub mode: &'a str,
    pub tool: Option<&'a str>,
    pub toast: Option<&'a Toast>,
}

/// Persistent bottom status bar: [MODE] tool | toast_message
pub struct StatusBar<'a> {
    pub data: StatusBarData<'a>,
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Mode label
        let mode_style = Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);

        let mut spans = vec![Span::styled(
            format!(" {} ", self.data.mode),
            mode_style,
        )];

        // Tool name
        if let Some(tool) = self.data.tool {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                tool.to_string(),
                Style::default().fg(Color::White),
            ));
        }

        // Toast message
        if let Some(toast) = self.data.toast {
            spans.push(Span::raw("  "));
            let toast_style = match toast.level {
                ToastLevel::Success => Style::default().fg(Color::Green),
                ToastLevel::Error => Style::default().fg(Color::Red),
            };
            let icon = match toast.level {
                ToastLevel::Success => "✓ ",
                ToastLevel::Error => "✗ ",
            };
            spans.push(Span::styled(icon, toast_style));
            spans.push(Span::styled(&toast.message, toast_style));
        }

        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}
