use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

/// A key-action pair for the help bar.
pub struct HelpItem {
    pub key: &'static str,
    pub action: &'static str,
}

/// A bottom help bar that renders `[key] action` pairs.
pub struct HelpBar {
    items: Vec<HelpItem>,
}

impl HelpBar {
    pub fn new(items: Vec<HelpItem>) -> Self {
        Self { items }
    }
}

impl Widget for HelpBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans = Vec::new();
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                format!("[{}]", item.key),
                Style::default().fg(Color::Yellow),
            ));
            spans.push(Span::raw(format!(" {}", item.action)));
        }
        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}
