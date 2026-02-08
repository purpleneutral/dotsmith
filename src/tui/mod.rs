mod dashboard;
mod event;
mod explore;
mod terminal;
mod widgets;

use anyhow::{Result, bail};

use crate::core::manifest::Manifest;
use crate::util;

use dashboard::DashboardState;
use dashboard::handler::{DashboardAction, handle_key as dashboard_handle_key};
use dashboard::view::draw_dashboard;
use explore::ExploreState;
use explore::handler::{ExploreAction, handle_key as explore_handle_key};
use explore::view::draw_explore;

/// Which view is currently active.
enum CurrentView {
    Dashboard,
    Explore,
}

/// Top-level app state.
struct App {
    current_view: CurrentView,
    dashboard: DashboardState,
    explore: Option<ExploreState>,
    should_quit: bool,
}

/// Entry point for the TUI.
/// - `None` → open the dashboard
/// - `Some(tool)` → open the explorer directly for that tool
pub fn run(tool: Option<&str>) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir).unwrap_or_default();
    let dashboard = DashboardState::from_manifest(&manifest);

    let (current_view, explore) = if let Some(tool_name) = tool {
        let explore = ExploreState::new(tool_name);
        if explore.is_none() {
            bail!(
                "'{}' has no option database. Only Tier 1 tools (tmux, zsh, git) support explore.",
                tool_name
            );
        }
        (CurrentView::Explore, explore)
    } else {
        (CurrentView::Dashboard, None)
    };

    let mut app = App {
        current_view,
        dashboard,
        explore,
        should_quit: false,
    };

    let mut terminal = terminal::init()?;

    let result = run_loop(&mut terminal, &mut app);

    terminal::restore()?;

    result
}

fn run_loop(terminal: &mut terminal::Tui, app: &mut App) -> Result<()> {
    loop {
        // Draw
        terminal.draw(|f| {
            let area = f.area();
            match app.current_view {
                CurrentView::Dashboard => draw_dashboard(f, area, &app.dashboard),
                CurrentView::Explore => {
                    if let Some(ref state) = app.explore {
                        draw_explore(f, area, state);
                    }
                }
            }
        })?;

        // Poll event
        if let Some(key) = event::next_key_event()? {
            match app.current_view {
                CurrentView::Dashboard => {
                    let action = dashboard_handle_key(key, &mut app.dashboard);
                    match action {
                        DashboardAction::Quit => app.should_quit = true,
                        DashboardAction::Explore(tool_name) => {
                            if let Some(state) = ExploreState::new(&tool_name) {
                                app.explore = Some(state);
                                app.current_view = CurrentView::Explore;
                            }
                        }
                        DashboardAction::None => {}
                    }
                }
                CurrentView::Explore => {
                    if let Some(ref mut state) = app.explore {
                        let action = explore_handle_key(key, state);
                        match action {
                            ExploreAction::Quit => app.should_quit = true,
                            ExploreAction::Back => {
                                app.explore = None;
                                app.current_view = CurrentView::Dashboard;
                            }
                            ExploreAction::None => {}
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
