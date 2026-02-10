use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use super::{ExploreState, Panel};
use crate::tui::widgets::help_bar::{HelpBar, HelpItem};

pub fn draw_explore(f: &mut Frame, area: Rect, state: &ExploreState) {
    let chunks = Layout::vertical([
        Constraint::Min(5),    // panels
        Constraint::Length(1), // help / search bar
    ])
    .split(area);

    draw_panels(f, chunks[0], state);

    if state.search_mode {
        draw_search_bar(f, chunks[1], state);
    } else {
        draw_help(f, chunks[1]);
    }
}

fn draw_panels(f: &mut Frame, area: Rect, state: &ExploreState) {
    let cols = Layout::horizontal([
        Constraint::Length(16),  // categories
        Constraint::Percentage(40), // options
        Constraint::Min(20),    // details
    ])
    .split(area);

    draw_categories(f, cols[0], state);
    draw_options(f, cols[1], state);
    draw_details(f, cols[2], state);
}

fn draw_categories(f: &mut Frame, area: Rect, state: &ExploreState) {
    let focused = state.focus == Panel::Categories;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = state
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let style = if i == state.category_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            let text = format!("{} ({})", cat.name, cat.count);
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(Line::from(Span::styled(
                format!(" {} ", state.tool_name),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    f.render_widget(list, area);
}

fn draw_options(f: &mut Frame, area: Rect, state: &ExploreState) {
    let focused = state.focus == Panel::Options;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let count = state.filtered_indices.len();
    let title = if state.search_query.is_empty() {
        format!(" Options [{}] ", count)
    } else {
        format!(" Options [{}] \"{}\" ", count, state.search_query)
    };

    let items: Vec<ListItem> = state
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(i, &opt_idx)| {
            let opt = &state.all_options[opt_idx];
            let style = if i == state.option_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            // Show name + abbreviated category
            let cat_abbrev: String = opt.category.chars().take(5).collect();
            let text = format!("{:<24} {}", opt.name, cat_abbrev);
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(Line::from(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    f.render_widget(list, area);
}

fn draw_details(f: &mut Frame, area: Rect, state: &ExploreState) {
    let focused = state.focus == Panel::Details;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let content = if let Some(opt) = state.selected_option() {
        let mut lines = vec![
            Line::from(Span::styled(
                opt.name.clone(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:?}", opt.option_type).to_lowercase()),
            ]),
        ];

        if let Some(ref default) = opt.default {
            if !default.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Default: ", Style::default().fg(Color::Yellow)),
                    Span::raw(default.clone()),
                ]));
            }
        }

        if let Some(ref values) = opt.values {
            lines.push(Line::from(vec![
                Span::styled("Values: ", Style::default().fg(Color::Yellow)),
                Span::raw(values.join(", ")),
            ]));
        }

        if let Some(ref since) = opt.since {
            lines.push(Line::from(vec![
                Span::styled("Since: ", Style::default().fg(Color::Yellow)),
                Span::raw(since.clone()),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(opt.description.clone()));

        if let Some(ref why) = opt.why {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Why:",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(why.clone()));
        }

        if let Some(ref example) = opt.example {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Example:",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {example}"),
                Style::default().fg(Color::White),
            )));
        }

        if let Some(ref tags) = opt.tags {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Tags: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    tags.join(", "),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        if let Some(ref related) = opt.related {
            lines.push(Line::from(vec![
                Span::styled("Related: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    related.join(", "),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        lines
    } else {
        vec![Line::from(Span::styled(
            "No option selected",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let block = Block::default()
        .title(Line::from(Span::styled(
            " Details ",
            Style::default().add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(content).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn draw_search_bar(f: &mut Frame, area: Rect, state: &ExploreState) {
    let line = Line::from(vec![
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::raw(&state.search_query),
        Span::styled("_", Style::default().fg(Color::Yellow)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help = HelpBar::new(vec![
        HelpItem { key: "/", action: "search" },
        HelpItem { key: "Tab", action: "panel" },
        HelpItem { key: "s", action: "snapshot" },
        HelpItem { key: "r", action: "reload" },
        HelpItem { key: "g", action: "generate" },
        HelpItem { key: "Esc", action: "back" },
        HelpItem { key: "q", action: "quit" },
    ]);
    f.render_widget(help, area);
}
