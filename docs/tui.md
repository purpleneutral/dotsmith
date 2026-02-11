# TUI Guide

dotsmith includes an interactive terminal UI built with [ratatui](https://ratatui.rs). It provides a dashboard for managing all your tools and an option explorer for discovering config settings.

## Launching

```sh
dotsmith          # open the dashboard
dotsmith explore tmux   # jump directly into the option explorer for a tool
```

## Dashboard

The dashboard is the default view when you run `dotsmith` with no arguments. It shows all tracked tools with their tier, config paths, plugin status, and last snapshot time.

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Select next tool |
| `k` / `↑` | Select previous tool |
| `e` / `Enter` | Explore config options (Tier 1 tools) |
| `s` | Snapshot all tracked configs |
| `r` | Reload selected tool |
| `d` | View diff since last snapshot |
| `h` | Browse snapshot history |
| `p` | Manage plugins for selected tool |
| `g` | Sync dotfile git repo |
| `q` / `Esc` | Quit |

### Toast Notifications

Actions like snapshotting, reloading, and syncing display toast notifications in the status bar. Toasts appear in green (success), yellow (warning), or red (error) and auto-expire after 3 seconds.

## Explore View

The option explorer is a three-panel interface for discovering and learning about config options. It's available for all Tier 1 tools.

```
+------------------+------------------+------------------+
| Categories       | Options          | Details          |
|                  |                  |                  |
| All (55)         | mouse            | mouse (boolean)  |
| appearance       | > status         |                  |
| > behavior       | default-shell    | Default: off     |
| interaction      | set-titles       |                  |
| keybindings      |                  | Enable mouse     |
| plugin:resurrect |                  | support for...   |
+------------------+------------------+------------------+
```

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Navigate down in the focused panel |
| `k` / `↑` | Navigate up in the focused panel |
| `Tab` | Cycle focus forward (Categories → Options → Details) |
| `Shift+Tab` | Cycle focus backward |
| `Enter` | Switch from Categories to Options panel |
| `/` | Enter search mode |
| `s` | Snapshot the current tool |
| `r` | Reload the current tool |
| `g` | Generate config snippet file |
| `Esc` | Return to dashboard (or cancel search) |
| `q` | Quit |

### Search

Press `/` to enter search mode. Type a query to filter options by name, description, and tags. Press `Enter` to confirm the filter or `Esc` to cancel and show all options again.

### Category Filtering

Select a category in the left panel to narrow the options list. The "All" category shows every option. Plugin options are grouped under `plugin:*` categories (e.g., `plugin:resurrect`, `plugin:autosuggestions`).

### Config Generation

Press `g` to generate a commented config snippet file at `~/.config/dotsmith/generated/<tool>.<ext>`. The generated file includes all currently visible options (respecting search and category filters) with descriptions, types, defaults, and examples -- all commented out for easy copy-paste.

Filter by category or search first to generate a focused snippet for just the options you care about.

## Diff View

The diff view shows a colored unified diff between the current state of a tool's config files and the last snapshot. Access it from the dashboard by pressing `d`.

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down one line |
| `k` / `↑` | Scroll up one line |
| `d` / `PageDown` | Page down |
| `u` / `PageUp` | Page up |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |
| `Esc` | Return to dashboard |
| `q` | Quit |

## History View

The history view shows snapshot entries for a tool. Access it from the dashboard by pressing `h`.

Each entry shows the snapshot ID, timestamp, content hash, file path, and optional message.

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Select next snapshot |
| `k` / `↑` | Select previous snapshot |
| `Enter` | View the selected snapshot's diff |
| `r` | Rollback to the selected snapshot |
| `Esc` | Return to dashboard |
| `q` | Quit |

## Plugins View

The plugins view lets you manage plugins for zsh and tmux directly from the TUI. Access it from the dashboard by pressing `p`.

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Select next plugin |
| `k` / `↑` | Select previous plugin |
| `a` | Add a plugin (enters input mode) |
| `d` | Remove selected plugin |
| `u` | Update selected plugin |
| `U` | Update all plugins |
| `i` | Toggle info panel |
| `Esc` | Return to dashboard |
| `q` | Quit |

### Adding a Plugin

Press `a` to enter input mode. Type the plugin repository (e.g., `zsh-users/zsh-autosuggestions`) and press `Enter` to add it. Press `Esc` to cancel.

### Info Panel

Press `i` to toggle a split view showing plugin details alongside the list. The info panel displays the plugin name, repository, URL, init file, and -- if a README is found in the plugin directory -- a description and configuration excerpt.

## Status Bar

The status bar at the bottom of the TUI shows the current view mode, the selected tool name, and toast notifications for completed actions.

## See Also

- [Command Reference](commands.md) -- CLI equivalents for all TUI actions
- [Plugin Management](plugins.md) -- detailed plugin guide
- [Snapshots & History](snapshots-and-history.md) -- snapshot workflow details
