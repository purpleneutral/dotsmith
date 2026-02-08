use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
};

use super::HistoryState;
use crate::tui::widgets::help_bar::{HelpBar, HelpItem};

pub fn draw_history(f: &mut Frame, area: Rect, state: &HistoryState) {
    let chunks = Layout::vertical([
        Constraint::Min(5),    // table
        Constraint::Length(1), // help bar
    ])
    .split(area);

    if state.entries.is_empty() {
        let msg = ratatui::widgets::Paragraph::new("No snapshots yet. Press 's' on the dashboard to create one.")
            .block(
                Block::default()
                    .title(format!(" History: {} ", state.tool_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(msg, chunks[0]);
    } else {
        let header = Row::new(vec![
            Cell::from("ID"),
            Cell::from("Date"),
            Cell::from("File"),
            Cell::from("Hash"),
            Cell::from("Message"),
        ])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        let rows: Vec<Row> = state
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let style = if i == state.selected {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    Cell::from(format!("#{}", entry.id)),
                    Cell::from(entry.created_at.clone()),
                    Cell::from(entry.file_path.clone()),
                    Cell::from(entry.hash.get(..8).unwrap_or(&entry.hash).to_string()),
                    Cell::from(
                        entry
                            .message
                            .as_deref()
                            .unwrap_or("-")
                            .to_string(),
                    ),
                ])
                .style(style)
            })
            .collect();

        let widths = [
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Min(15),
        ];

        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .title(Line::from(Span::styled(
                    format!(
                        " History: {} ({} snapshots) ",
                        state.tool_name,
                        state.entries.len()
                    ),
                    Style::default().add_modifier(Modifier::BOLD),
                )))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(table, chunks[0]);
    }

    draw_help(f, chunks[1]);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help = HelpBar::new(vec![
        HelpItem {
            key: "j/k",
            action: "navigate",
        },
        HelpItem {
            key: "Enter",
            action: "view diff",
        },
        HelpItem {
            key: "r",
            action: "rollback",
        },
        HelpItem {
            key: "Esc",
            action: "back",
        },
        HelpItem {
            key: "q",
            action: "quit",
        },
    ]);
    f.render_widget(help, area);
}
