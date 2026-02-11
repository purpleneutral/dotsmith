use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

use super::{PluginMode, PluginState};
use crate::tui::widgets::help_bar::{HelpBar, HelpItem};

pub fn draw_plugins(f: &mut Frame, area: Rect, state: &PluginState) {
    let chunks = Layout::vertical([
        Constraint::Min(5),    // content
        Constraint::Length(1), // help/input bar
    ])
    .split(area);

    if !state.supported {
        let msg = Paragraph::new(format!(
            "Plugin management is not supported for '{}'",
            state.tool_name
        ))
        .block(
            Block::default()
                .title(format!(" Plugins: {} ", state.tool_name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);
        f.render_widget(msg, chunks[0]);
        draw_help_minimal(f, chunks[1]);
        return;
    }

    if state.plugins.is_empty() {
        let msg = Paragraph::new("No plugins installed. Press 'a' to add one.")
            .block(
                Block::default()
                    .title(format!(" Plugins: {} ", state.tool_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .alignment(Alignment::Center);
        f.render_widget(msg, chunks[0]);
    } else if state.show_info {
        // Split into list + detail panel
        let cols = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(chunks[0]);

        draw_plugin_list(f, cols[0], state);
        draw_plugin_detail(f, cols[1], state);
    } else {
        draw_plugin_table(f, chunks[0], state);
    }

    match state.mode {
        PluginMode::List => draw_help(f, chunks[1]),
        PluginMode::AddInput => draw_input(f, chunks[1], state),
    }
}

fn draw_plugin_table(f: &mut Frame, area: Rect, state: &PluginState) {
    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("Repo"),
        Cell::from("Init File"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = state
        .plugins
        .iter()
        .enumerate()
        .map(|(i, plugin)| {
            let style = if i == state.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(plugin.name.clone()),
                Cell::from(plugin.repo.clone()),
                Cell::from(plugin.init.clone()),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Min(30),
        Constraint::Min(25),
    ];

    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(Line::from(Span::styled(
                format!(
                    " Plugins: {} ({}) ",
                    state.tool_name,
                    state.plugins.len()
                ),
                Style::default().add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(table, area);
}

fn draw_plugin_list(f: &mut Frame, area: Rect, state: &PluginState) {
    let rows: Vec<Row> = state
        .plugins
        .iter()
        .enumerate()
        .map(|(i, plugin)| {
            let style = if i == state.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![Cell::from(plugin.name.clone())]).style(style)
        })
        .collect();

    let widths = [Constraint::Min(10)];

    let table = Table::new(rows, widths).block(
        Block::default()
            .title(Line::from(Span::styled(
                format!(
                    " Plugins: {} ({}) ",
                    state.tool_name,
                    state.plugins.len()
                ),
                Style::default().add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(table, area);
}

fn draw_plugin_detail(f: &mut Frame, area: Rect, state: &PluginState) {
    let content = if let Some(plugin) = state.selected_plugin() {
        let mut lines = vec![
            Line::from(Span::styled(
                plugin.name.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Repo: ", Style::default().fg(Color::Yellow)),
                Span::raw(plugin.repo.clone()),
            ]),
            Line::from(vec![
                Span::styled("URL:  ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    plugin.url.clone(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]),
            Line::from(vec![
                Span::styled("Init: ", Style::default().fg(Color::Yellow)),
                Span::raw(plugin.init.clone()),
            ]),
        ];

        if let Some(ref desc) = plugin.description {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Description:",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(desc.clone()));
        }

        if let Some(ref config) = plugin.config_excerpt {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Configuration:",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
            for line in config.lines().take(15) {
                lines.push(Line::from(line.to_string()));
            }
        }

        if plugin.description.is_none() && plugin.config_excerpt.is_none() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "No README found in plugin directory.",
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    } else {
        vec![Line::from(Span::styled(
            "No plugin selected",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let block = Block::default()
        .title(Line::from(Span::styled(
            " Info ",
            Style::default().add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(content).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help = HelpBar::new(vec![
        HelpItem {
            key: "a",
            action: "add",
        },
        HelpItem {
            key: "d",
            action: "remove",
        },
        HelpItem {
            key: "u",
            action: "update",
        },
        HelpItem {
            key: "U",
            action: "update all",
        },
        HelpItem {
            key: "i",
            action: "info",
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

fn draw_help_minimal(f: &mut Frame, area: Rect) {
    let help = HelpBar::new(vec![
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

fn draw_input(f: &mut Frame, area: Rect, state: &PluginState) {
    let line = Line::from(vec![
        Span::styled(
            "Add plugin (user/repo): ",
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(&state.input_buffer),
        Span::styled("_", Style::default().fg(Color::Yellow)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}
