# Getting Started

This guide covers installing dotsmith, running it for the first time, and understanding how it works.

## Prerequisites

- **Prebuilt binary:** No dependencies -- just download and run.
- **From source:** Rust 1.85+ via [rustup](https://rustup.rs).

## Installation

**One-liner** (downloads prebuilt binary or builds from source):

```sh
curl -sSf https://raw.githubusercontent.com/purpleneutral/dotsmith/main/install.sh | sh
```

**With cargo:**

```sh
cargo install --git https://github.com/purpleneutral/dotsmith.git
```

**From source:**

```sh
git clone https://github.com/purpleneutral/dotsmith.git
cd dotsmith
make install
```

The binary installs to `~/.local/bin` by default. Override with `PREFIX`:

```sh
make install PREFIX=/usr/local
```

### Shell Completions

Generate and install tab completions for your shell:

```sh
# Bash
dotsmith completions bash > ~/.local/share/bash-completion/completions/dotsmith

# Zsh
dotsmith completions zsh > ~/.local/share/zsh/site-functions/_dotsmith

# Fish
dotsmith completions fish > ~/.config/fish/completions/dotsmith.fish
```

### Man Page

A man page is installed automatically with `make install`. To generate it separately:

```sh
make man              # generates dotsmith.1
man ./dotsmith.1      # preview locally
```

## First Run

### 1. Add tools to track

```sh
dotsmith add tmux
dotsmith add zsh
dotsmith add git
```

dotsmith auto-initializes on first use -- no separate `init` step is needed. Running `dotsmith init` explicitly is also fine (it's idempotent).

When you add a tool, dotsmith:

- Detects whether it's a Tier 1 (curated option database) or Tier 2 (auto-detected) tool
- Finds config file paths automatically (e.g., `~/.config/tmux/tmux.conf`, `~/.tmux.conf`)
- Records existing symlinks (follows the user-facing path, not the target)
- Detects existing plugin managers (TPM, zinit, oh-my-zsh, etc.) without replacing them

### 2. Check your setup

```sh
dotsmith list     # see tracked tools with tier, paths, plugin status
dotsmith status   # verify all tracked configs still exist
```

### 3. Launch the TUI

```sh
dotsmith          # opens the interactive dashboard
```

From the dashboard you can also press `a` to add tools interactively, or jump straight into exploring a tool's config options:

```sh
dotsmith explore tmux
```

See the [TUI Guide](tui.md) for navigation and keybindings.

## How It Works

dotsmith tracks your configs **in-place**. It never copies, moves, or re-symlinks your config files.

- `add` detects existing symlinks and records the user-facing path
- Existing plugin managers are detected but never replaced
- Snapshots are stored in a local SQLite database with SHA-256 content deduplication
- All write operations (`deploy`, `rollback`, `profile load`) create backups first and support `--dry-run`
- Config files are never modified unless you explicitly run a write command

## Data Directory

All dotsmith data is stored at `~/.config/dotsmith/`:

| Path | Purpose |
|------|---------|
| `manifest.toml` | Which tools are tracked, their config paths, and plugin entries |
| `config.toml` | dotsmith settings (repo path, configs directory) |
| `snapshots.db` | Snapshot history (SQLite, WAL mode) |
| `backups/` | Automatic backups from rollback and deploy operations |
| `plugins/` | Cloned plugin repositories and loader files |
| `generated/` | Config snippet files from TUI explore (`g` key) |
| `profiles/` | Saved configuration profiles (manifest + file contents) |

All dotsmith-created files use `0600`/`0700` permissions.

## Next Steps

- [Command Reference](commands.md) -- full CLI documentation
- [TUI Guide](tui.md) -- interactive dashboard and explorer
- [Snapshots & History](snapshots-and-history.md) -- track and roll back changes
- [Plugin Management](plugins.md) -- manage zsh and tmux plugins
