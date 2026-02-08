use chrono::Utc;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
};

use super::DashboardState;
use crate::tui::widgets::help_bar::{HelpBar, HelpItem};

pub fn draw_dashboard(f: &mut Frame, area: Rect, state: &DashboardState) {
    let chunks = Layout::vertical([
        Constraint::Min(5),    // table
        Constraint::Length(1), // help bar
    ])
    .split(area);

    draw_table(f, chunks[0], state);
    draw_help(f, chunks[1]);
}

fn draw_table(f: &mut Frame, area: Rect, state: &DashboardState) {
    let header = Row::new(vec![
        Cell::from("Tool"),
        Cell::from("Tier"),
        Cell::from("Paths"),
        Cell::from("Plugins"),
        Cell::from("Last Snapshot"),
    ])
    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = state
        .tools
        .iter()
        .enumerate()
        .map(|(i, tool)| {
            let style = if i == state.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let snapshot_text = match tool.last_snapshot {
                Some(dt) => format_relative_time(dt),
                None => "never".to_string(),
            };

            let plugin_text = if tool.plugin_count > 0 {
                tool.plugin_count.to_string()
            } else {
                "-".to_string()
            };

            Row::new(vec![
                Cell::from(tool.name.clone()),
                Cell::from(tool.tier_label.clone()),
                Cell::from(tool.config_path_count.to_string()),
                Cell::from(plugin_text),
                Cell::from(snapshot_text),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Min(12),
        Constraint::Length(6),
        Constraint::Length(7),
        Constraint::Length(9),
        Constraint::Min(15),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(" dotsmith ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .row_highlight_style(Style::default());

    f.render_widget(table, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help = HelpBar::new(vec![
        HelpItem { key: "j/k", action: "navigate" },
        HelpItem { key: "e", action: "explore" },
        HelpItem { key: "q", action: "quit" },
    ]);
    f.render_widget(help, area);
}

/// Format a datetime as a relative time string (e.g., "2h ago", "3d ago").
fn format_relative_time(dt: chrono::DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 30 {
        format!("{}d ago", duration.num_days())
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_format_relative_time() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now), "just now");
        assert_eq!(
            format_relative_time(now - Duration::minutes(5)),
            "5m ago"
        );
        assert_eq!(
            format_relative_time(now - Duration::hours(3)),
            "3h ago"
        );
        assert_eq!(
            format_relative_time(now - Duration::days(7)),
            "7d ago"
        );
    }
}
