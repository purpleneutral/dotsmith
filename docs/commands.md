# Command Reference

Complete reference for all dotsmith CLI commands. Run `dotsmith --help` or `dotsmith <command> --help` for built-in usage info.

## Global Flags

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Suppress non-essential output |

## Setup

### `init`

Initialize the dotsmith configuration directory.

```sh
dotsmith init
```

Creates `~/.config/dotsmith/` with `manifest.toml`, `config.toml`, and `snapshots.db`. Idempotent -- safe to run multiple times. Note: most commands auto-initialize on first use, so explicit `init` is optional.

### `add`

Add a tool to dotsmith tracking.

```sh
dotsmith add <tool>
```

Automatically detects config paths, support tier, and existing plugin managers. Tier 1 tools get a curated option database; Tier 2 tools get auto-detected config paths.

```sh
dotsmith add tmux    # Tier 1: full option database
dotsmith add ranger  # Tier 2: auto-detected paths
```

### `remove`

Remove a tool from dotsmith tracking.

```sh
dotsmith remove <tool>
```

Removes the tool from the manifest. **Never touches your config files.**

### `list`

List all tracked tools with their tier, config paths, and plugin status.

```sh
dotsmith list
```

### `status`

Show health status of tracked configs -- verifies that tracked files still exist and flags warnings.

```sh
dotsmith status
```

## Snapshots & History

See [Snapshots & History](snapshots-and-history.md) for workflow details.

### `snapshot`

Take a snapshot of config files for later rollback.

```sh
dotsmith snapshot                        # snapshot all tracked tools
dotsmith snapshot tmux                   # snapshot a specific tool
dotsmith snapshot tmux -m "enabled mouse"  # attach a message
```

| Flag | Description |
|------|-------------|
| `-m, --message <msg>` | Message to attach to the snapshot |

Snapshots are deduplicated by content hash -- unchanged files don't create new entries.

### `history`

Show snapshot history for a tool.

```sh
dotsmith history tmux
dotsmith history tmux --limit 5
```

| Flag | Description |
|------|-------------|
| `-l, --limit <N>` | Maximum entries to show (default: 20) |

### `diff`

Show a colored unified diff between current config state and the last snapshot.

```sh
dotsmith diff          # diff all tracked tools
dotsmith diff tmux     # diff a specific tool
```

### `rollback`

Restore a config file to a specific snapshot. The snapshot ID comes from `history` output.

```sh
dotsmith rollback 5 --dry-run   # preview changes first
dotsmith rollback 5             # apply the rollback
```

| Flag | Description |
|------|-------------|
| `--dry-run` | Preview changes without applying them |

Existing files are backed up to `~/.config/dotsmith/backups/` before overwriting.

## Editing & Watching

### `edit`

Open a tool's config file in your editor. Automatically takes a snapshot before editing and detects changes afterward.

```sh
dotsmith edit tmux
```

Uses `$EDITOR`, then `$VISUAL`, then `vi` as fallback.

### `watch`

Watch tracked configs for changes and auto-snapshot on save.

```sh
dotsmith watch          # watch all tracked tools
dotsmith watch tmux     # watch a specific tool
```

Polls every 2 seconds. Detects actual content changes (not just mtime). Press `Ctrl-C` to stop.

## Exploration & Health

### `explore`

Launch the interactive TUI option explorer for a Tier 1 tool.

```sh
dotsmith explore tmux
```

Opens a three-panel interface with categories, options, and detailed descriptions. See [TUI Guide](tui.md#explore-view) for keybindings.

### `search`

Search config options across all Tier 1 tool databases.

```sh
dotsmith search mouse
dotsmith search resurrect
```

Matches option names, descriptions, categories, and tags. Plugin options include documentation URLs.

### `doctor`

Run deep health checks on tracked tools.

```sh
dotsmith doctor          # check all tools
dotsmith doctor tmux     # check a specific tool
```

Checks: tool installation, config paths existence, config syntax validation (Tier 1), snapshot freshness. Provides actionable hints for issues found.

## Deployment

### `deploy`

Create config symlinks from a source directory to a target location.

```sh
dotsmith deploy ~/dots/tmux ~/.config/tmux --dry-run
dotsmith deploy ~/dots/tmux ~/.config/tmux
```

| Flag | Description |
|------|-------------|
| `--dry-run` | Preview changes without applying them |

Existing files at the target are backed up before being replaced with symlinks.

### `deploy-remote`

Deploy tracked configs to a remote host via SSH.

```sh
dotsmith deploy-remote myserver
dotsmith deploy-remote myserver --dry-run
dotsmith deploy-remote myserver --tool tmux --tool zsh
dotsmith deploy-remote myserver --user alice
```

| Flag | Description |
|------|-------------|
| `--dry-run` | Preview what would be copied |
| `-t, --tool <name>` | Deploy only specific tools (repeatable) |
| `-u, --user <user>` | SSH user (defaults to current user / ssh config) |

Uses your system `ssh` and `scp` commands, so `~/.ssh/config` (aliases, ProxyJump, agent forwarding) is fully respected. Remote files are backed up as `.dotsmith-bak.<timestamp>` before overwriting.

See [Deploy & Profiles](deploy-and-profiles.md) for workflow details.

## Plugins

See [Plugin Management](plugins.md) for the full guide.

### `plugins add`

Add a plugin for a tool.

```sh
dotsmith plugins zsh add zsh-users/zsh-autosuggestions
dotsmith plugins tmux add tmux-plugins/tmux-sensible
```

Accepts GitHub shorthand (`user/repo`) or full HTTPS URLs.

### `plugins remove`

Remove an installed plugin.

```sh
dotsmith plugins zsh remove zsh-autosuggestions
```

### `plugins list`

List installed plugins with their repository and init file.

```sh
dotsmith plugins zsh list
```

### `plugins update`

Update one or all plugins.

```sh
dotsmith plugins zsh update                        # update all
dotsmith plugins zsh update zsh-autosuggestions     # update one
```

### `plugins info`

Show plugin details extracted from the plugin's README -- description, configuration excerpt, and URL.

```sh
dotsmith plugins zsh info                          # all plugins
dotsmith plugins zsh info zsh-autosuggestions       # specific plugin
```

## Profiles

See [Deploy & Profiles](deploy-and-profiles.md) for workflow details.

### `profile save`

Save current tracked configs as a named profile.

```sh
dotsmith profile save workstation
```

### `profile load`

Restore config files from a saved profile.

```sh
dotsmith profile load workstation --dry-run    # preview first
dotsmith profile load workstation              # restore configs
dotsmith profile load workstation --add-untracked  # also add new tools
```

| Flag | Description |
|------|-------------|
| `--dry-run` | Preview changes without applying them |
| `--add-untracked` | Add tools from the profile that aren't currently tracked |

Existing files are backed up before being overwritten.

### `profile list`

List saved profiles with creation date, tool count, and file count.

```sh
dotsmith profile list
```

### `profile delete`

Delete a saved profile.

```sh
dotsmith profile delete old-setup
```

## Git Repo

See [Deploy & Profiles](deploy-and-profiles.md#repo-sync) for workflow details.

### `repo init`

Initialize a git repo for dotfile backups.

```sh
dotsmith repo init ~/dots
```

Creates the directory, runs `git init`, and saves the path in `config.toml`.

### `repo sync`

Copy tracked configs into the repo and commit.

```sh
dotsmith repo sync
```

### `repo status`

Show the repo's git status.

```sh
dotsmith repo status
```

## Utilities

### `completions`

Generate shell completions.

```sh
dotsmith completions bash
dotsmith completions zsh
dotsmith completions fish
```

See [Getting Started](getting-started.md#shell-completions) for installation instructions.

### `reload`

Reload a running tool's configuration.

```sh
dotsmith reload tmux
```

Supported reload methods vary by tool -- tmux uses `source-file`, awesomewm uses `awesome-client`, kitty auto-reloads, etc. See [Supported Tools](supported-tools.md) for per-tool details.
