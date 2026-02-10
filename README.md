# dotsmith

[![CI](https://github.com/purpleneutral/dotsmith/actions/workflows/ci.yml/badge.svg)](https://github.com/purpleneutral/dotsmith/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0--alpha.8-orange.svg)](CHANGELOG.md)

The dotfile workbench -- explore, manage, and master your configs.

dotsmith is a CLI + TUI tool that does what no other dotfile manager does:

1. **Explores** -- shows what config options exist for your tools, which you're using, which you're missing
2. **Manages** -- handles change tracking with snapshots, diffs, rollback, and git repo sync
3. **Teaches** -- every option has a description, a "why you'd want this", and an example
4. **Reloads** -- apply config changes live without restarting your session

Think of it as an IDE for your dotfiles.

## Install

**One-liner** (downloads prebuilt binary or builds from source):

```sh
curl -sSf https://raw.githubusercontent.com/purpleneutral/dotsmith/main/install.sh | sh
```

**From source** (requires Rust 1.85+):

```sh
git clone https://github.com/purpleneutral/dotsmith.git
cd dotsmith
make install
```

**With cargo:**

```sh
cargo install --git https://github.com/purpleneutral/dotsmith.git
```

The binary installs to `~/.local/bin` by default. Override with `PREFIX`:

```sh
make install PREFIX=/usr/local
```

A man page is installed automatically with `make install`. You can also generate it separately:

```sh
make man              # generates dotsmith.1
man ./dotsmith.1      # preview locally
```

## Quick Start

```sh
# Initialize dotsmith
dotsmith init

# Start tracking tools
dotsmith add tmux
dotsmith add zsh
dotsmith add git

# Launch the TUI dashboard
dotsmith

# Explore config options interactively
dotsmith explore tmux
```

## TUI Dashboard

Running `dotsmith` with no arguments opens an interactive dashboard:

| Key | Action |
|-----|--------|
| `j/k` | Navigate tools |
| `e` | Explore config options (Tier 1 tools) |
| `s` | Snapshot all tracked configs |
| `r` | Reload selected tool |
| `d` | View diff since last snapshot |
| `h` | Browse snapshot history |
| `p` | Manage plugins |
| `g` | Sync dotfile git repo |
| `q` | Quit |

The TUI includes a scrollable diff viewer, snapshot history browser with rollback, and plugin management -- all without leaving the terminal.

In the explore view, press `g` to generate a commented config snippet file at `~/.config/dotsmith/generated/<tool>.<ext>` containing all currently visible options. Filter by category or search first to generate a focused snippet.

## Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize dotsmith configuration directory |
| `add <tool>` | Add a tool to dotsmith tracking |
| `remove <tool>` | Remove a tool (never touches your config files) |
| `list` | List all tracked tools with tier, paths, plugin manager |
| `status` | Health check -- verify tracked configs still exist |
| `doctor [tool]` | Deep health check -- installation, paths, syntax validation, snapshot freshness |
| `search <query>` | Search config options across all Tier 1 tool databases |
| `edit <tool>` | Open config in $EDITOR (auto-snapshots before editing) |
| `watch [tool]` | Watch tracked configs for changes and auto-snapshot on save |
| `snapshot [tool] [-m msg]` | Snapshot config files for rollback |
| `history <tool> [-l N]` | Show snapshot history |
| `diff [tool]` | Colored diff between current state and last snapshot |
| `rollback <id> [--dry-run]` | Restore a config file to a previous snapshot |
| `deploy <src> <dst> [--dry-run]` | Create config symlinks with backup |
| `deploy-remote <host> [--dry-run]` | Deploy tracked configs to a remote host via SSH |
| `reload <tool>` | Reload a running tool's configuration |
| `explore <tool>` | Interactive TUI explorer for config options |
| `plugins <tool> add <repo>` | Add a plugin (GitHub shorthand: `user/repo`) |
| `plugins <tool> remove <name>` | Remove a plugin |
| `plugins <tool> list` | List installed plugins |
| `plugins <tool> update [name]` | Update one or all plugins |
| `profile save <name>` | Save current configs as a named profile |
| `profile load <name>` | Restore config files from a saved profile |
| `profile list` | List saved profiles |
| `profile delete <name>` | Delete a saved profile |
| `repo init <path>` | Initialize a git repo for dotfile backups |
| `repo sync` | Copy tracked configs into the repo and commit |
| `repo status` | Show repo status |
| `completions <shell>` | Generate shell completions (bash, zsh, fish) |

## Shell Completions

Generate and install tab completions for your shell:

```sh
# Bash
dotsmith completions bash > ~/.local/share/bash-completion/completions/dotsmith

# Zsh
dotsmith completions zsh > ~/.local/share/zsh/site-functions/_dotsmith

# Fish
dotsmith completions fish > ~/.config/fish/completions/dotsmith.fish
```

## Tiered Support

**Tier 1** -- Full support with curated option databases:
- **tmux** -- 31 options across interaction, display, behavior, plugins, clipboard
- **zsh** -- 33 options across history, completion, prompt, navigation, globbing, safety, starship
- **git** -- 31 options across user, core, diff, merge, push, color, aliases, safety
- **kitty** -- 31 options across appearance, fonts, cursor, scrollback, mouse, performance, tabs
- **neovim** -- 31 options across ui, editing, search, indentation, completion, lsp, files
- **alacritty** -- 31 options across window, font, colors, cursor, scrolling, shell, keybindings
- **awesomewm** -- 31 options across general, tags, layouts, keybindings, rules, wibar, themes

**Tier 2** -- Auto-detected tracking for any tool:
- Automatically discovers config paths at `~/.config/<tool>/`, `~/.<tool>rc`, `~/.<tool>config`, etc.
- Snapshot, diff, rollback all work
- No curated option database (yet)

## Plugin Management

dotsmith includes built-in plugin management for zsh and tmux, inspired by [zsh_unplugged](https://github.com/mattmc3/zsh_unplugged). A plugin is just a git repo with a file to source -- no framework needed.

```sh
# Add plugins using GitHub shorthand
dotsmith plugins zsh add zsh-users/zsh-autosuggestions
dotsmith plugins tmux add tmux-plugins/tmux-sensible

# Then add ONE line to your .zshrc / tmux.conf:
#   source ~/.config/dotsmith/plugins/zsh/loader.zsh
#   source-file ~/.config/dotsmith/plugins/tmux/loader.conf
```

dotsmith clones with `--depth 1`, auto-detects init files (`*.plugin.zsh`, `*.tmux`, etc.), and generates a loader file that you source once. Updates are a single command: `dotsmith plugins zsh update`.

Supported tools: **zsh**, **tmux**. Existing plugin managers (TPM, zinit, etc.) are detected on `add` but never replaced -- opt in to dotsmith plugin management explicitly.

## Dotfile Repo Sync

Keep a git-backed copy of your configs for pushing to a remote:

```sh
# Create a repo
dotsmith repo init ~/dots

# Sync tracked configs into it (copies files, commits changes)
dotsmith repo sync

# Check status
dotsmith repo status

# Push whenever you're ready
cd ~/dots && git remote add origin <url> && git push
```

The `g` key on the TUI dashboard also triggers a sync.

## Configuration Profiles

Save and restore named configuration profiles -- great for switching between setups or migrating to a new machine:

```sh
# Save your current setup
dotsmith profile save workstation

# List saved profiles
dotsmith profile list

# Preview what loading would do
dotsmith profile load laptop --dry-run

# Restore configs from a profile (backs up existing files first)
dotsmith profile load workstation

# Load and also add tools that aren't currently tracked
dotsmith profile load workstation --add-untracked

# Remove a profile you no longer need
dotsmith profile delete old-setup
```

Profiles are stored at `~/.config/dotsmith/profiles/<name>/` and include the full manifest plus actual config file contents.

## Remote Deploy

Deploy your tracked configs to a remote host via SSH:

```sh
# Deploy all tracked configs to a remote host
dotsmith deploy-remote myserver

# Preview what would be copied
dotsmith deploy-remote myserver --dry-run

# Deploy specific tools only
dotsmith deploy-remote myserver --tool tmux --tool zsh

# Specify a user
dotsmith deploy-remote myserver --user alice
```

Uses your system `ssh` and `scp` commands, so your `~/.ssh/config` (aliases, ProxyJump, agent forwarding) is fully respected. Existing remote files are backed up before overwriting.

## How It Works

dotsmith tracks your configs **in-place**. It never copies, moves, or re-symlinks your files.

- `add` detects existing symlinks and records the user-facing path
- Existing plugin managers (TPM, zinix-mgr, oh-my-zsh, etc.) are detected but never replaced
- Snapshots are stored in a local SQLite database with SHA-256 content dedup
- All write operations (deploy, rollback) create backups first and support `--dry-run`
- Config files are never modified unless you explicitly run a write command

Data is stored at `~/.config/dotsmith/`:
- `manifest.toml` -- which tools are tracked (including plugin entries)
- `config.toml` -- dotsmith settings
- `snapshots.db` -- snapshot history (SQLite, WAL mode)
- `backups/` -- automatic backups from rollback/deploy
- `plugins/` -- cloned plugin repositories and loader files
- `generated/` -- config snippet files from TUI explore (`g` key)
- `profiles/` -- saved configuration profiles (manifest + file contents)

All dotsmith-created files use `0600`/`0700` permissions.

## Building

```sh
cargo build              # debug build
cargo build --release    # optimized build
cargo test               # run all tests (~420 tests)
cargo clippy             # lint check
make check               # clippy + tests together
```

## Project Status

**Current:** v0.1.0-alpha.8

- Phase 1: CLI skeleton, manifest, module system, tool detection
- Phase 2: Snapshots, diff, deploy, rollback, reload, zsh module
- Phase 3: Built-in plugin management for zsh and tmux
- Phase 4: TUI dashboard, option explorer, diff/history/plugin views, repo sync
- Phase 5a: Shell completions, kitty/neovim/alacritty/awesomewm Tier 1 modules, starship integration
- Quick wins: `doctor`, `search`, config generation from TUI explore
- Phase 5b: `edit`, `watch`, config validation, man page generation
- Phase 6: Git repo management (`repo init`, `repo sync`, `repo status`)
- Phase 7: Configuration profiles (save/load), remote deploy via SSH

See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.

## Support

If you find dotsmith useful, consider buying me a coffee:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-support-yellow?style=flat&logo=buy-me-a-coffee)](https://buymeacoffee.com/uniqueuserg)

## License

Copyright (c) 2026 purpleneutral

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, version 3.

See [LICENSE](LICENSE) for the full text.
