use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::{DiffLineKind, DiffState};
use crate::tui::widgets::help_bar::{HelpBar, HelpItem};

pub fn draw_diff(f: &mut Frame, area: Rect, state: &mut DiffState) {
    let chunks = Layout::vertical([
        Constraint::Min(5),    // diff content
        Constraint::Length(1), // help bar
    ])
    .split(area);

    state.visible_height = chunks[0].height.saturating_sub(2) as usize;

    if !state.has_changes {
        let msg = Paragraph::new("No changes detected since last snapshot")
            .block(
                Block::default()
                    .title(format!(" Diff: {} ", state.tool_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .alignment(Alignment::Center);
        f.render_widget(msg, chunks[0]);
    } else {
        let visible_lines: Vec<Line> = state
            .lines
            .iter()
            .skip(state.scroll_offset)
            .take(state.visible_height)
            .map(|dl| {
                let style = match dl.kind {
                    DiffLineKind::Header => {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    }
                    DiffLineKind::HunkHeader => Style::default().fg(Color::Cyan),
                    DiffLineKind::Added => Style::default().fg(Color::Green),
                    DiffLineKind::Removed => Style::default().fg(Color::Red),
                    DiffLineKind::Context => Style::default(),
                    DiffLineKind::Empty => Style::default(),
                };
                Line::from(Span::styled(dl.content.clone(), style))
            })
            .collect();

        let title = format!(
            " Diff: {} ({} file(s)) [{}/{}] ",
            state.tool_name,
            state.total_files,
            state.scroll_offset + 1,
            state.lines.len(),
        );

        let paragraph = Paragraph::new(visible_lines).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(paragraph, chunks[0]);
    }

    draw_help(f, chunks[1]);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help = HelpBar::new(vec![
        HelpItem {
            key: "j/k",
            action: "scroll",
        },
        HelpItem {
            key: "d/u",
            action: "page",
        },
        HelpItem {
            key: "g/G",
            action: "top/bottom",
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
