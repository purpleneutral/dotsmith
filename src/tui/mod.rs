mod dashboard;
mod diff;
mod event;
mod explore;
mod history;
mod plugins;
mod terminal;
mod widgets;

use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::core::config::DotsmithConfig;
use crate::core::manifest::Manifest;
use crate::core::snapshot::SnapshotEngine;
use crate::util;

use dashboard::DashboardState;
use dashboard::handler::{DashboardAction, handle_key as dashboard_handle_key};
use dashboard::view::draw_dashboard;
use diff::DiffState;
use diff::handler::{DiffAction, handle_key as diff_handle_key};
use diff::view::draw_diff;
use explore::ExploreState;
use explore::handler::{ExploreAction, handle_key as explore_handle_key};
use explore::view::draw_explore;
use history::HistoryState;
use history::handler::{HistoryAction, handle_key as history_handle_key};
use history::view::draw_history;
use plugins::PluginState;
use plugins::handler::{PluginAction, handle_key as plugin_handle_key};
use plugins::view::draw_plugins;
use widgets::status_bar::{StatusBar, StatusBarData, Toast, ToastLevel};

/// Which view is currently active.
#[derive(Clone, Copy)]
enum CurrentView {
    Dashboard,
    Explore,
    Diff,
    History,
    Plugins,
}

/// Top-level app state.
struct App {
    current_view: CurrentView,
    dashboard: DashboardState,
    explore: Option<ExploreState>,
    diff_view: Option<DiffState>,
    history_view: Option<HistoryState>,
    plugins_view: Option<PluginState>,
    return_view: Option<CurrentView>,
    should_quit: bool,
    toast: Option<Toast>,

    // Shared state
    config_dir: PathBuf,
    manifest: Manifest,
    snapshot_engine: SnapshotEngine,
    config: DotsmithConfig,
}

impl App {
    fn toast_success(&mut self, msg: impl Into<String>) {
        self.toast = Some(Toast::new(msg.into(), ToastLevel::Success));
    }

    fn toast_error(&mut self, msg: impl Into<String>) {
        self.toast = Some(Toast::new(msg.into(), ToastLevel::Error));
    }

    fn expire_toast(&mut self) {
        if let Some(ref toast) = self.toast {
            if toast.is_expired() {
                self.toast = None;
            }
        }
    }

    fn current_tool_name(&self) -> Option<&str> {
        match self.current_view {
            CurrentView::Dashboard => self
                .dashboard
                .selected_tool()
                .map(|t| t.name.as_str()),
            CurrentView::Explore => self.explore.as_ref().map(|e| e.tool_name.as_str()),
            CurrentView::Diff => self.diff_view.as_ref().map(|d| d.tool_name.as_str()),
            CurrentView::History => self
                .history_view
                .as_ref()
                .map(|h| h.tool_name.as_str()),
            CurrentView::Plugins => self
                .plugins_view
                .as_ref()
                .map(|p| p.tool_name.as_str()),
        }
    }

    fn mode_label(&self) -> &str {
        match self.current_view {
            CurrentView::Dashboard => "DASHBOARD",
            CurrentView::Explore => "EXPLORE",
            CurrentView::Diff => "DIFF",
            CurrentView::History => "HISTORY",
            CurrentView::Plugins => "PLUGINS",
        }
    }

    fn refresh_dashboard(&mut self) {
        self.dashboard = DashboardState::from_manifest(&self.manifest);
    }
}

/// Entry point for the TUI.
/// - `None` → open the dashboard
/// - `Some(tool)` → open the explorer directly for that tool
pub fn run(tool: Option<&str>) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir).unwrap_or_default();
    let snapshot_engine = SnapshotEngine::open(&config_dir)?;
    let config = DotsmithConfig::load(&config_dir);
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
        diff_view: None,
        history_view: None,
        plugins_view: None,
        return_view: None,
        should_quit: false,
        toast: None,
        config_dir,
        manifest,
        snapshot_engine,
        config,
    };

    let mut terminal = terminal::init()?;

    let result = run_loop(&mut terminal, &mut app);

    terminal::restore()?;

    result
}

fn run_loop(terminal: &mut terminal::Tui, app: &mut App) -> Result<()> {
    loop {
        app.expire_toast();

        // Draw
        terminal.draw(|f| {
            let area = f.area();
            let chunks = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Min(3),
                ratatui::layout::Constraint::Length(1),
            ])
            .split(area);

            match app.current_view {
                CurrentView::Dashboard => draw_dashboard(f, chunks[0], &app.dashboard),
                CurrentView::Explore => {
                    if let Some(ref state) = app.explore {
                        draw_explore(f, chunks[0], state);
                    }
                }
                CurrentView::Diff => {
                    if let Some(ref mut state) = app.diff_view {
                        draw_diff(f, chunks[0], state);
                    }
                }
                CurrentView::History => {
                    if let Some(ref state) = app.history_view {
                        draw_history(f, chunks[0], state);
                    }
                }
                CurrentView::Plugins => {
                    if let Some(ref state) = app.plugins_view {
                        draw_plugins(f, chunks[0], state);
                    }
                }
            }

            // Status bar
            let tool_name = app.current_tool_name().map(|s| s.to_string());
            let status = StatusBar {
                data: StatusBarData {
                    mode: app.mode_label(),
                    tool: tool_name.as_deref(),
                    toast: app.toast.as_ref(),
                },
            };
            f.render_widget(status, chunks[1]);
        })?;

        // Poll event
        if let Some(key) = event::next_key_event()? {
            match app.current_view {
                CurrentView::Dashboard => handle_dashboard_action(key, app),
                CurrentView::Explore => handle_explore_action(key, app),
                CurrentView::Diff => handle_diff_action(key, app),
                CurrentView::History => handle_history_action(key, app),
                CurrentView::Plugins => handle_plugin_action(key, app),
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_dashboard_action(key: crossterm::event::KeyEvent, app: &mut App) {
    let action = dashboard_handle_key(key, &mut app.dashboard);
    match action {
        DashboardAction::Quit => app.should_quit = true,
        DashboardAction::Explore(tool_name) => {
            if let Some(state) = ExploreState::new(&tool_name) {
                app.explore = Some(state);
                app.current_view = CurrentView::Explore;
            }
        }
        DashboardAction::SnapshotAll => {
            match app
                .snapshot_engine
                .snapshot_all(&app.manifest, Some("TUI snapshot"))
            {
                Ok(count) => {
                    // Reload manifest to pick up updated last_snapshot timestamps
                    if let Ok(m) = Manifest::load(&app.config_dir) {
                        app.manifest = m;
                        app.refresh_dashboard();
                    }
                    app.toast_success(format!("Snapshotted {} file(s)", count));
                }
                Err(e) => app.toast_error(format!("Snapshot failed: {}", e)),
            }
        }
        DashboardAction::ReloadSelected(tool_name) => {
            match crate::core::reload::reload_tool(&tool_name, None) {
                Ok(msg) => app.toast_success(msg),
                Err(e) => app.toast_error(format!("Reload failed: {}", e)),
            }
        }
        DashboardAction::ShowDiff(tool_name) => {
            let state =
                DiffState::from_tool(&tool_name, &app.snapshot_engine, &app.manifest);
            app.diff_view = Some(state);
            app.return_view = Some(CurrentView::Dashboard);
            app.current_view = CurrentView::Diff;
        }
        DashboardAction::ShowHistory(tool_name) => {
            let state = HistoryState::new(&tool_name, &app.snapshot_engine);
            app.history_view = Some(state);
            app.current_view = CurrentView::History;
        }
        DashboardAction::ShowPlugins(tool_name) => {
            let state = PluginState::new(&tool_name, &app.manifest, Some(&app.config_dir));
            app.plugins_view = Some(state);
            app.current_view = CurrentView::Plugins;
        }
        DashboardAction::SyncRepo => {
            if let Some(ref repo_path_str) = app.config.general.repo_path {
                let expanded = util::paths::expand_tilde(repo_path_str);
                match crate::core::repo::sync_repo(&expanded, &app.manifest) {
                    Ok(result) => {
                        if result.committed {
                            app.toast_success(format!(
                                "Synced {} file(s), committed",
                                result.files_copied
                            ));
                        } else {
                            app.toast_success("Repo up to date");
                        }
                    }
                    Err(e) => app.toast_error(format!("Sync failed: {}", e)),
                }
            } else {
                app.toast_error("No repo configured. Run `dotsmith repo init <path>` first.");
            }
        }
        DashboardAction::None => {}
    }
}

fn handle_explore_action(key: crossterm::event::KeyEvent, app: &mut App) {
    if let Some(ref mut state) = app.explore {
        let action = explore_handle_key(key, state);
        match action {
            ExploreAction::Quit => app.should_quit = true,
            ExploreAction::Back => {
                app.explore = None;
                app.current_view = CurrentView::Dashboard;
            }
            ExploreAction::Snapshot(tool_name) => {
                match app
                    .snapshot_engine
                    .snapshot_all(&app.manifest, Some("TUI snapshot"))
                {
                    Ok(count) => {
                        if let Ok(m) = Manifest::load(&app.config_dir) {
                            app.manifest = m;
                            app.refresh_dashboard();
                        }
                        app.toast_success(format!(
                            "Snapshotted {} ({} files)",
                            tool_name, count
                        ));
                    }
                    Err(e) => app.toast_error(format!("Snapshot failed: {}", e)),
                }
            }
            ExploreAction::Reload(tool_name) => {
                match crate::core::reload::reload_tool(&tool_name, None) {
                    Ok(msg) => app.toast_success(msg),
                    Err(e) => app.toast_error(format!("Reload failed: {}", e)),
                }
            }
            ExploreAction::GenerateConfig(ref tool_name) => {
                generate_config(app, tool_name);
            }
            ExploreAction::None => {}
        }
    }
}

fn generate_config(app: &mut App, tool_name: &str) {
    use crate::core::module::ModuleRegistry;

    // Get filtered options from explore state
    let Some(ref state) = app.explore else {
        return;
    };

    let filtered_options: Vec<_> = state
        .filtered_indices
        .iter()
        .map(|&i| &state.all_options[i])
        .collect();

    if filtered_options.is_empty() {
        app.toast_error("No options to generate");
        return;
    }

    // Determine file extension from config_format
    let ext = match ModuleRegistry::get_builtin(tool_name) {
        Some(module) => match module.metadata.config_format.as_str() {
            "tmux" => "conf",
            "git" => "gitconfig",
            "shell" => "zsh",
            "lua" => "lua",
            "key-value" => "conf",
            "toml" => "toml",
            _ => "conf",
        },
        None => "conf",
    };

    // Build commented config content
    let mut content = format!(
        "# Generated by dotsmith — {} configuration options\n",
        tool_name
    );
    content.push_str(&format!(
        "# {} option(s) included\n",
        filtered_options.len()
    ));
    content.push('\n');

    let comment = match ext {
        "lua" => "--",
        _ => "#",
    };

    let mut current_category = String::new();
    for opt in &filtered_options {
        if opt.category != current_category {
            current_category = opt.category.clone();
            content.push_str(&format!(
                "{} Category: {}\n{}\n",
                comment, current_category, comment
            ));
        }

        let type_str = format!("{:?}", opt.option_type).to_lowercase();
        let default_str = opt
            .default
            .as_deref()
            .map(|d| format!(", default: {}", d))
            .unwrap_or_default();

        content.push_str(&format!(
            "{} {} ({}{})\n",
            comment, opt.name, type_str, default_str
        ));
        content.push_str(&format!("{} {}\n", comment, opt.description));

        if let Some(ref example) = opt.example {
            content.push_str(&format!("{} {}\n", comment, example));
        }

        content.push_str(&format!("{}\n", comment));
    }

    // Write to generated/ directory
    let generated_dir = app.config_dir.join("generated");
    if let Err(e) = std::fs::create_dir_all(&generated_dir) {
        app.toast_error(format!("Failed to create directory: {}", e));
        return;
    }

    let file_path = generated_dir.join(format!("{}.{}", tool_name, ext));
    match crate::util::fs::atomic_write(&file_path, &content) {
        Ok(()) => {
            let display_path = crate::util::paths::contract_tilde(&file_path);
            app.toast_success(format!(
                "Generated {} ({} options)",
                display_path,
                filtered_options.len()
            ));
        }
        Err(e) => app.toast_error(format!("Write failed: {}", e)),
    }
}

fn handle_diff_action(key: crossterm::event::KeyEvent, app: &mut App) {
    if let Some(ref mut state) = app.diff_view {
        let action = diff_handle_key(key, state);
        match action {
            DiffAction::Quit => app.should_quit = true,
            DiffAction::Back => {
                app.diff_view = None;
                app.current_view = app.return_view.take().unwrap_or(CurrentView::Dashboard);
            }
            DiffAction::None => {}
        }
    }
}

fn handle_history_action(key: crossterm::event::KeyEvent, app: &mut App) {
    let action = {
        let Some(ref mut state) = app.history_view else {
            return;
        };
        history_handle_key(key, state)
    };

    match action {
        HistoryAction::Quit => app.should_quit = true,
        HistoryAction::Back => {
            app.history_view = None;
            app.current_view = CurrentView::Dashboard;
        }
        HistoryAction::ViewSnapshot(id) => {
            let tool = app
                .history_view
                .as_ref()
                .map(|s| s.tool_name.clone())
                .unwrap_or_default();
            match app.snapshot_engine.get_snapshot(id) {
                Ok(Some((file_path, snapshot_content))) => {
                    let expanded = util::paths::expand_tilde(&file_path);
                    let current = std::fs::read_to_string(&expanded).unwrap_or_default();
                    let diff = DiffState::from_strings(
                        &tool,
                        &file_path,
                        &snapshot_content,
                        &current,
                    );
                    app.diff_view = Some(diff);
                    app.return_view = Some(CurrentView::History);
                    app.current_view = CurrentView::Diff;
                }
                Ok(None) => app.toast_error("Snapshot not found"),
                Err(e) => app.toast_error(format!("Failed to load snapshot: {}", e)),
            }
        }
        HistoryAction::Rollback(id) => {
            let backup_dir = app.config_dir.join("backups");
            match app.snapshot_engine.rollback(id, &backup_dir) {
                Ok(file_path) => {
                    app.toast_success(format!("Rolled back {}", file_path));
                    // Refresh history
                    let tool = app
                        .history_view
                        .as_ref()
                        .map(|s| s.tool_name.clone())
                        .unwrap_or_default();
                    app.history_view =
                        Some(HistoryState::new(&tool, &app.snapshot_engine));
                }
                Err(e) => app.toast_error(format!("Rollback failed: {}", e)),
            }
        }
        HistoryAction::None => {}
    }
}

fn handle_plugin_action(key: crossterm::event::KeyEvent, app: &mut App) {
    let action = {
        let Some(ref mut state) = app.plugins_view else {
            return;
        };
        plugin_handle_key(key, state)
    };

    match action {
        PluginAction::Quit => app.should_quit = true,
        PluginAction::Back => {
            app.plugins_view = None;
            app.current_view = CurrentView::Dashboard;
        }
        PluginAction::AddPlugin(repo_spec) => {
            let tool = app
                .plugins_view
                .as_ref()
                .map(|s| s.tool_name.clone())
                .unwrap_or_default();
            match crate::core::plugin::add_plugin(
                &app.config_dir,
                &mut app.manifest,
                &tool,
                &repo_spec,
            ) {
                Ok((name, _init)) => {
                    app.toast_success(format!("Added plugin: {}", name));
                    app.plugins_view = Some(PluginState::new(&tool, &app.manifest, Some(&app.config_dir)));
                    app.refresh_dashboard();
                }
                Err(e) => app.toast_error(format!("Add failed: {}", e)),
            }
        }
        PluginAction::RemovePlugin(name) => {
            let tool = app
                .plugins_view
                .as_ref()
                .map(|s| s.tool_name.clone())
                .unwrap_or_default();
            match crate::core::plugin::remove_plugin(
                &app.config_dir,
                &mut app.manifest,
                &tool,
                &name,
            ) {
                Ok(()) => {
                    app.toast_success(format!("Removed plugin: {}", name));
                    app.plugins_view = Some(PluginState::new(&tool, &app.manifest, Some(&app.config_dir)));
                    app.refresh_dashboard();
                }
                Err(e) => app.toast_error(format!("Remove failed: {}", e)),
            }
        }
        PluginAction::UpdatePlugin(name) => {
            let tool = app
                .plugins_view
                .as_ref()
                .map(|s| s.tool_name.clone())
                .unwrap_or_default();
            match crate::core::plugin::update_plugins(
                &app.config_dir,
                &app.manifest,
                &tool,
                name.as_deref(),
            ) {
                Ok(results) => {
                    let updated: Vec<_> =
                        results.iter().filter(|r| r.updated).collect();
                    if updated.is_empty() {
                        app.toast_success("All plugins up to date");
                    } else {
                        app.toast_success(format!(
                            "Updated {} plugin(s)",
                            updated.len()
                        ));
                    }
                }
                Err(e) => app.toast_error(format!("Update failed: {}", e)),
            }
        }
        PluginAction::None => {}
    }
}
