# dotsmith

The dotfile workbench -- explore, manage, and master your configs.

dotsmith is a CLI tool that does four things no other dotfile manager does:

1. **Explores** -- shows what config options exist for your tools, which you're using, which you're missing
2. **Manages** -- handles config deployment, change tracking with snapshots, diffs, and rollback
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

## Quick Start

```sh
# Initialize dotsmith
dotsmith init

# Start tracking tools
dotsmith add tmux
dotsmith add zsh
dotsmith add git

# See what's tracked
dotsmith list
dotsmith status

# Snapshot your configs
dotsmith snapshot -m "before changes"

# Make changes, then see what changed
dotsmith diff tmux

# Rollback if something broke
dotsmith rollback 1 --dry-run   # preview first
dotsmith rollback 1             # restore

# Deploy symlinks
dotsmith deploy ~/.config/oz/dots/tmux ~/.config/tmux --dry-run
dotsmith deploy ~/.config/oz/dots/tmux ~/.config/tmux

# Reload a running tool
dotsmith reload tmux

# Manage plugins (replaces tpm, zinit, etc.)
dotsmith plugins zsh add zsh-users/zsh-autosuggestions
dotsmith plugins zsh add zsh-users/zsh-syntax-highlighting
dotsmith plugins zsh list
dotsmith plugins zsh update
dotsmith plugins tmux add tmux-plugins/tmux-sensible
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize dotsmith configuration directory |
| `add <tool>` | Add a tool to dotsmith tracking |
| `remove <tool>` | Remove a tool (never touches your config files) |
| `list` | List all tracked tools with tier, paths, plugin manager |
| `status` | Health check -- verify tracked configs still exist |
| `snapshot [tool] [-m msg]` | Snapshot config files for rollback |
| `history <tool> [-l N]` | Show snapshot history |
| `diff [tool]` | Colored diff between current state and last snapshot |
| `rollback <id> [--dry-run]` | Restore a config file to a previous snapshot |
| `deploy <src> <dst> [--dry-run]` | Create config symlinks with backup |
| `reload <tool>` | Reload a running tool's configuration |
| `plugins <tool> add <repo>` | Add a plugin (GitHub shorthand: `user/repo`) |
| `plugins <tool> remove <name>` | Remove a plugin |
| `plugins <tool> list` | List installed plugins |
| `plugins <tool> update [name]` | Update one or all plugins |

## Tiered Support

**Tier 1** -- Full support with curated option databases:
- **tmux** -- 31 options across interaction, display, behavior, plugins, clipboard
- **zsh** -- 31 options across history, completion, prompt, navigation, globbing, safety

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

All dotsmith-created files use `0600`/`0700` permissions.

## Building

```sh
cargo build              # debug build
cargo build --release    # optimized build
cargo test               # run all tests
cargo clippy             # lint check
make check               # clippy + tests
```

## Project Status

**Current:** v0.1.0-alpha.2 (Phase 3 complete)

- Phase 1: CLI skeleton, manifest, module system, tool detection
- Phase 2: Snapshots, diff, deploy, rollback, reload, zsh module
- Phase 3: Built-in plugin management for zsh and tmux
- Phase 4: TUI explorer (planned)
- Phase 5: Polish, completions, distribution (planned)

## License

Copyright (c) 2026 purpleneutral

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, version 3.

See [LICENSE](LICENSE) for the full text.
