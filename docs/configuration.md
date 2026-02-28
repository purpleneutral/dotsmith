# Configuration Reference

dotsmith uses two configuration files: `config.toml` for its own settings and `manifest.toml` for tracking tools. Both live in the dotsmith config directory.

## config.toml

**Location:** `~/.config/dotsmith/config.toml`

dotsmith's own settings. Created automatically on `dotsmith init`. Safe to edit by hand.

```toml
[general]
configs_dir = "~/.config/dotsmith/configs"
repo_path = "~/dots"
```

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `general.configs_dir` | string | `~/.config/dotsmith/configs` | Base directory for managed config sources (reserved for future use) |
| `general.repo_path` | string | *(none)* | Path to the git repo for dotfile backups. Set by `dotsmith repo init`. |

## manifest.toml

**Location:** `~/.config/dotsmith/manifest.toml`

Tracks which tools are managed, their config paths, and plugin entries. Managed by dotsmith commands -- normally you don't need to edit this by hand, but it's safe to read.

### Example

```toml
[tools.tmux]
tier = 1
config_paths = ["~/.config/tmux/tmux.conf"]
plugins_managed = false
added_at = "2025-02-10T14:32:15Z"

[tools.tmux.plugins.tmux-resurrect]
repo = "tmux-plugins/tmux-resurrect"
init = "resurrect.tmux"
added_at = "2025-02-10T14:45:22Z"

[tools.zsh]
tier = 1
config_paths = ["~/.config/zsh/scripts", "~/.config/zsh/.zshenv", "~/.config/zsh/.zshrc"]
plugins_managed = false
plugin_manager = "zinit"
added_at = "2025-02-10T14:32:20Z"

[tools.ranger]
tier = 2
config_paths = ["~/.config/ranger"]
plugins_managed = false
added_at = "2025-02-10T15:00:00Z"
```

### Tool Entry Fields

| Field | Type | Description |
|-------|------|-------------|
| `tier` | integer | 1 = full option database, 2 = auto-detected |
| `config_paths` | string[] | Tilde-contracted paths to tracked config files/directories |
| `plugins_managed` | boolean | Whether dotsmith manages plugins for this tool |
| `plugin_manager` | string? | Detected external plugin manager name (if any) |
| `added_at` | datetime | When the tool was added |
| `last_snapshot` | datetime? | When the last snapshot was taken |

### Plugin Entry Fields

| Field | Type | Description |
|-------|------|-------------|
| `repo` | string | GitHub shorthand or full URL |
| `init` | string | Relative path to the init/source file |
| `added_at` | datetime | When the plugin was added |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `DOTSMITH_CONFIG_DIR` | Override the config directory (default: `~/.config/dotsmith`). Mainly useful for testing. |
| `EDITOR` | Editor used by `dotsmith edit`. Falls back to `$VISUAL`, then `vi`. |
| `VISUAL` | Fallback editor if `$EDITOR` is not set. |

## File Permissions

All files created by dotsmith use restrictive permissions:

- **Files:** `0600` (owner read/write only)
- **Directories:** `0700` (owner read/write/execute only)

This applies to the manifest, config, snapshot database, backups, generated files, and profiles.

## Path Handling

dotsmith stores paths in **tilde-contracted** form (e.g., `~/.config/tmux/tmux.conf`) for portability across environments. Paths are expanded to absolute form when accessed.

**Symlink tracking:** When you `dotsmith add` a tool whose config is a symlink (e.g., `~/.config/tmux` → `~/dotfiles/tmux`), dotsmith records the user-facing path (`~/.config/tmux`), not the symlink target. This means snapshots and diffs work against the path you expect.

## See Also

- [Getting Started](getting-started.md) -- initial setup and data directory layout
- [Command Reference](commands.md) -- commands that modify these files
