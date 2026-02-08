# dotsmith — The Dotfile Workbench

## Complete Project Plan

**Language:** Rust
**Interface:** CLI-first + TUI for exploration
**License:** MIT
**Repository:** github.com/[you]/dotsmith

---

## Table of Contents

1. [What Is dotsmith](#1-what-is-dotsmith)
2. [Why It Exists — The Gap](#2-why-it-exists--the-gap)
3. [Core Concepts](#3-core-concepts)
4. [Architecture](#4-architecture)
5. [Project Structure](#5-project-structure)
6. [Crate Dependencies](#6-crate-dependencies)
7. [Module System — How Tools Are Supported](#7-module-system--how-tools-are-supported)
8. [Option Database — The Secret Sauce](#8-option-database--the-secret-sauce)
9. [Plugin Management — Built-in for zsh & tmux](#9-plugin-management--built-in-for-zsh--tmux)
10. [CLI Interface — Commands & Subcommands](#10-cli-interface--commands--subcommands)
11. [TUI Interface — Exploration Mode](#11-tui-interface--exploration-mode)
12. [Snapshot & History Engine](#12-snapshot--history-engine)
13. [Reload Engine — Live Config Application](#13-reload-engine--live-config-application)
14. [Config Deployment — Symlink Management](#14-config-deployment--symlink-management)
15. [Tiered Support Model](#15-tiered-support-model)
16. [MVP Scope — What Ships First](#16-mvp-scope--what-ships-first)
17. [Post-MVP Roadmap](#17-post-mvp-roadmap)
18. [Data Flow Diagrams](#18-data-flow-diagrams)
19. [File Formats — All Specs](#19-file-formats--all-specs)
20. [Build, Test & Distribution](#20-build-test--distribution)
21. [Community & Growth Strategy](#21-community--growth-strategy)
22. [Open Questions & Decisions](#22-open-questions--decisions)

---

## 1. What Is dotsmith

dotsmith is a **TUI workbench for understanding, editing, and mastering your dotfiles.**

It is NOT another symlink manager. It does four things no other tool does:

1. **Explores** — shows you what config options exist for your tools, which ones you're
   using, and which ones you're missing
2. **Manages** — handles config deployment (symlinks), plugin management (zsh, tmux),
   change tracking (snapshots, diffs, rollback)
3. **Teaches** — every option has a description, a "why you'd want this", an example,
   pulled from man pages and curated databases
4. **Reloads** — apply changes live without restarting your session

Think of it as: **an IDE for your dotfiles.**

---

## 2. Why It Exists — The Gap

### What Already Exists

| Tool | What it does | What it doesn't do |
|------|-------------|-------------------|
| chezmoi | Sync dotfiles across machines | Help you understand or improve them |
| GNU Stow | Symlink dotfiles | Anything else |
| yadm | Git wrapper for dotfiles | Config discovery or education |
| DotState | TUI for managing symlinks | Option exploration, no intelligence |
| lazy.nvim | Manage nvim plugins | Anything outside nvim |
| TPM | Manage tmux plugins | Anything outside tmux |
| zsh_unplugged | DIY zsh plugin pattern | Not a tool, just a blog post |
| tldr | Simplified man pages | Config-focused knowledge |

### What Nobody Does

- "What tmux options am I not using?"
- "Show me all zsh options, categorized, with descriptions"
- "I just changed this setting — reload it live"
- "What did I change in my configs this week?"
- "Set up my new machine with my full environment in one command"
- "Manage my zsh AND tmux plugins from one place"

dotsmith does all of this.

---

## 3. Core Concepts

### Manifest
A TOML file (`~/.config/dotsmith/manifest.toml`) that tracks every tool dotsmith manages:
which config files, what tier of support, whether plugins are managed, last snapshot time.

### Modules
Each supported tool (tmux, zsh, kitty, etc.) is a **module** — a directory containing:
- `module.toml` — metadata (config paths, reload command, man page reference)
- `options.toml` — the option database (all known options with descriptions)
- Optional: validation rules, linter rules, default config template

### Tiers
Four levels of support:
- **Tier 1:** Full support, curated option DB ships with dotsmith
- **Tier 2:** Auto-detected, basic tracking (snapshots, diffs), no option discovery
- **Tier 3:** User-enriched, option DB generated from online docs or man pages
- **Tier 4:** Community-contributed modules via GitHub PR

### Snapshots
Point-in-time copies of a config file stored in SQLite with timestamps, optional
annotations, and diff capability against any other snapshot.

### Option Database
Per-tool structured data about every available config option: name, type, default,
description, category, example, "why you'd want this." The core differentiator.

---

## 4. Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        dotsmith binary                            │
│                                                                    │
│  ┌──────────┐    ┌─────────────────────────────────────────────┐ │
│  │   CLI    │    │               TUI (ratatui)                 │ │
│  │  (clap)  │    │  ┌─────────┐ ┌──────────┐ ┌─────────────┐  │ │
│  │          │    │  │ Tool    │ │ Config   │ │ Option      │  │ │
│  │ add      │    │  │ Tree    │ │ Editor   │ │ Discovery   │  │ │
│  │ remove   │    │  │         │ │ (syntax  │ │ Panel       │  │ │
│  │ diff     │    │  │         │ │  hl,help)│ │ (unused     │  │ │
│  │ reload   │    │  │         │ │          │ │  options)   │  │ │
│  │ plugins  │    │  └─────────┘ └──────────┘ └─────────────┘  │ │
│  │ snapshot │    │  ┌──────────────────────────────────────┐   │ │
│  │ status   │    │  │ Status: changes, reload, warnings   │   │ │
│  │ explore  │───▶│  └──────────────────────────────────────┘   │ │
│  │ deploy   │    └─────────────────────────────────────────────┘ │
│  └──────────┘                                                    │
│       │                          │                               │
│       ▼                          ▼                               │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Core Services                         │    │
│  │                                                          │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐ │    │
│  │  │ Module   │ │ Snapshot │ │ Reload   │ │ Plugin     │ │    │
│  │  │ Registry │ │ Engine   │ │ Engine   │ │ Manager    │ │    │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬──────┘ │    │
│  │       │             │            │              │        │    │
│  │  ┌────┴─────┐  ┌────┴─────┐     │         ┌────┴─────┐ │    │
│  │  │ Option   │  │ SQLite   │     │         │ Git      │ │    │
│  │  │ DB       │  │ Store    │     │         │ (clone/  │ │    │
│  │  │ (TOML)   │  │          │     │         │  pull)   │ │    │
│  │  └────┬─────┘  └──────────┘     │         └──────────┘ │    │
│  │       │                          │                       │    │
│  │  ┌────┴──────────────────┐  ┌────┴──────────────────┐   │    │
│  │  │ Parsers              │  │ Reloaders              │   │    │
│  │  │ - man page parser    │  │ - tmux source-file     │   │    │
│  │  │ - config file parser │  │ - source ~/.zshrc      │   │    │
│  │  │ - --help parser      │  │ - nvim --remote        │   │    │
│  │  │ - HTML docs scraper  │  │ - (tool-specific)      │   │    │
│  │  └──────────────────────┘  └────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Config Layer                           │    │
│  │  ~/.config/dotsmith/                                     │    │
│  │  ├── config.toml          (dotsmith's own settings)      │    │
│  │  ├── manifest.toml        (tracked tools)                │    │
│  │  ├── modules/             (option databases per tool)    │    │
│  │  │   ├── tmux/                                           │    │
│  │  │   │   ├── module.toml                                 │    │
│  │  │   │   └── options.toml                                │    │
│  │  │   ├── zsh/                                            │    │
│  │  │   └── ...                                             │    │
│  │  ├── snapshots.db         (SQLite — history)             │    │
│  │  └── plugins/             (managed plugins)              │    │
│  │      ├── zsh/                                            │    │
│  │      │   ├── zsh-autosuggestions/                        │    │
│  │      │   └── zsh-syntax-highlighting/                    │    │
│  │      └── tmux/                                           │    │
│  │          └── tmux-online-status/                         │    │
│  └─────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

---

## 5. Project Structure

```
dotsmith/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE                         # MIT
├── CHANGELOG.md
│
├── src/
│   ├── main.rs                     # Entry point, clap CLI dispatch
│   ├── lib.rs                      # Public API re-exports
│   │
│   ├── cli/                        # CLI subcommand handlers
│   │   ├── mod.rs                  # clap derive structs
│   │   ├── add.rs                  # dotsmith add <tool>
│   │   ├── remove.rs               # dotsmith remove <tool>
│   │   ├── deploy.rs               # dotsmith deploy
│   │   ├── diff.rs                 # dotsmith diff <tool>
│   │   ├── explore.rs              # launches TUI explore mode
│   │   ├── list.rs                 # dotsmith list
│   │   ├── plugins.rs              # dotsmith plugins <tool> <action>
│   │   ├── reload.rs               # dotsmith reload <tool>
│   │   ├── snapshot.rs             # dotsmith snapshot [annotation]
│   │   └── status.rs               # dotsmith status
│   │
│   ├── tui/                        # TUI views (ratatui)
│   │   ├── mod.rs                  # App state, event loop, render dispatch
│   │   ├── app.rs                  # Main TUI app struct and state management
│   │   ├── event.rs                # Input event handling (keyboard, mouse)
│   │   ├── views/
│   │   │   ├── mod.rs
│   │   │   ├── dashboard.rs        # Overview: all tracked tools, status, recent changes
│   │   │   ├── explore.rs          # Option discovery: browse, search, toggle options
│   │   │   ├── editor.rs           # Inline config editor with context tooltips
│   │   │   ├── diff.rs             # Visual diff between snapshots
│   │   │   ├── plugins.rs          # Plugin management TUI view
│   │   │   └── help.rs             # Keybinding reference overlay
│   │   ├── widgets/
│   │   │   ├── mod.rs
│   │   │   ├── file_tree.rs        # Left sidebar: tool/file navigator
│   │   │   ├── option_panel.rs     # Right sidebar: option details
│   │   │   ├── search_bar.rs       # Fuzzy search across options
│   │   │   ├── status_bar.rs       # Bottom bar: mode, tool, change count
│   │   │   └── toast.rs            # Notification popups (reload success, etc.)
│   │   └── theme.rs                # TUI color scheme and styling
│   │
│   ├── core/                       # Core business logic (no UI dependency)
│   │   ├── mod.rs
│   │   ├── manifest.rs             # Read/write manifest.toml
│   │   ├── config.rs               # dotsmith's own config (config.toml)
│   │   ├── module.rs               # Module struct, loading, registration
│   │   ├── option_db.rs            # Option database queries (search, filter, diff)
│   │   ├── snapshot.rs             # Snapshot creation, storage, diffing
│   │   ├── reload.rs               # Per-tool reload command execution
│   │   ├── deploy.rs               # Symlink creation, backup, restore
│   │   ├── detect.rs               # Auto-detect installed tools and config paths
│   │   └── plugin.rs               # Plugin management (clone, update, remove)
│   │
│   ├── parsers/                    # Config file & documentation parsers
│   │   ├── mod.rs
│   │   ├── manpage.rs              # Parse man pages via mandoc/groff
│   │   ├── help_flag.rs            # Parse --help output into structured options
│   │   ├── commented_config.rs     # Parse self-documenting config files
│   │   ├── toml_config.rs          # Parse TOML configs (alacritty, starship)
│   │   ├── ini_config.rs           # Parse INI-style configs (git)
│   │   ├── lua_config.rs           # Parse Lua configs (nvim, awesome)
│   │   ├── shell_config.rs         # Parse shell configs (zsh, bash, tmux)
│   │   └── html_docs.rs            # Scrape online docs (Tier 3, stretch)
│   │
│   ├── modules/                    # Built-in module definitions (Tier 1)
│   │   ├── mod.rs                  # Module registry, lookup
│   │   ├── tmux.rs                 # tmux-specific logic (reload, plugin paths)
│   │   ├── zsh.rs                  # zsh-specific logic (plugin sourcing, compiling)
│   │   ├── nvim.rs                 # nvim-specific logic (lazy.nvim awareness)
│   │   ├── git.rs                  # git-specific logic
│   │   ├── ssh.rs                  # ssh-specific logic
│   │   ├── kitty.rs                # kitty-specific logic
│   │   └── alacritty.rs            # alacritty-specific logic
│   │
│   └── util/                       # Shared utilities
│       ├── mod.rs
│       ├── paths.rs                # XDG path resolution, home dir expansion
│       ├── git.rs                  # Git clone, pull, status helpers
│       ├── diff.rs                 # Diff algorithm and formatting
│       ├── fs.rs                   # File system helpers (symlink, backup, watch)
│       └── shell.rs                # Shell command execution, output capture
│
├── data/                           # Ships with the binary (embedded via include_str!)
│   ├── modules/
│   │   ├── tmux/
│   │   │   ├── module.toml         # metadata
│   │   │   └── options.toml        # all tmux options, curated
│   │   ├── zsh/
│   │   │   ├── module.toml
│   │   │   └── options.toml        # zsh setopt options + key zshrc patterns
│   │   ├── nvim/
│   │   │   ├── module.toml
│   │   │   └── options.toml        # vim.opt.* options
│   │   ├── git/
│   │   │   ├── module.toml
│   │   │   └── options.toml        # git config options
│   │   ├── ssh/
│   │   │   ├── module.toml
│   │   │   └── options.toml        # ssh_config options
│   │   ├── kitty/
│   │   │   ├── module.toml
│   │   │   └── options.toml
│   │   └── alacritty/
│   │       ├── module.toml
│   │       └── options.toml
│   └── defaults/                   # Default config templates
│       ├── tmux.conf               # Sensible defaults (your refined config)
│       ├── .zshrc                  # Sensible defaults
│       └── kitty.conf              # Sensible defaults
│
├── tests/
│   ├── integration/
│   │   ├── cli_add_test.rs
│   │   ├── cli_deploy_test.rs
│   │   ├── snapshot_test.rs
│   │   ├── plugin_test.rs
│   │   └── explore_test.rs
│   └── unit/
│       ├── manifest_test.rs
│       ├── option_db_test.rs
│       ├── parser_test.rs
│       └── module_test.rs
│
└── docs/
    ├── CONTRIBUTING.md             # How to contribute modules
    └── MODULE_SPEC.md              # How to write a module.toml + options.toml
```

---

## 6. Crate Dependencies

```toml
# Cargo.toml

[package]
name = "dotsmith"
version = "0.1.0"
edition = "2021"
description = "The dotfile workbench — explore, manage, and master your configs"
license = "MIT"
repository = "https://github.com/[you]/dotsmith"
keywords = ["dotfiles", "config", "tui", "terminal", "cli"]
categories = ["command-line-utilities", "config"]

[dependencies]
# CLI
clap = { version = "4", features = ["derive", "env"] }

# TUI
ratatui = "0.29"
crossterm = "0.28"

# Data
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"

# File system
notify = "7"                    # file watching
dirs = "6"                      # XDG/home directory resolution
walkdir = "2"                   # recursive directory traversal
glob = "0.3"                    # glob pattern matching

# Utilities
chrono = { version = "0.4", features = ["serde"] }
similar = "2"                   # diff algorithm (used by git delta)
syntect = "5"                   # syntax highlighting in editor
anyhow = "1"                    # error handling
thiserror = "2"                 # typed errors for library code
colored = "2"                   # CLI colored output
indicatif = "0.17"              # progress bars for CLI
fuzzy-matcher = "0.3"           # fuzzy search in TUI

# HTTP (optional, for Tier 3 docs scraping)
reqwest = { version = "0.12", features = ["blocking"], optional = true }
scraper = { version = "0.21", optional = true }

# Git operations (plugin management)
# Using shell git commands rather than libgit2 for simplicity.
# Avoids the heavy libgit2-sys dependency. git is already installed
# on every target system.

[features]
default = []
docs-scraper = ["reqwest", "scraper"]   # opt-in online docs parsing

[profile.release]
lto = true
strip = true
codegen-units = 1
```

---

## 7. Module System — How Tools Are Supported

### Module Definition

Each tool is defined by a `module.toml`:

```toml
# data/modules/tmux/module.toml

[metadata]
name = "tmux"
display_name = "tmux"
description = "Terminal multiplexer"
homepage = "https://github.com/tmux/tmux"

# Where to find config files (in priority order)
config_paths = [
    "~/.config/tmux/tmux.conf",
    "~/.tmux.conf",
]

# How to detect if the tool is installed
detect_command = "which tmux"

# How to reload config without restarting
reload_command = "tmux source-file {config_path}"
reload_description = "Source tmux config"

# Man page for automated option parsing
man_page = "tmux"

# Config file format (determines which parser to use)
config_format = "tmux"

# Whether dotsmith can manage plugins for this tool
plugins_supported = true
plugin_dir = "~/.config/dotsmith/plugins/tmux"

# Default config file shipped with dotsmith
default_config = "defaults/tmux.conf"

# Categories for organizing options in the TUI
categories = [
    "appearance",
    "behavior",
    "interaction",
    "keybindings",
    "performance",
    "status-bar",
    "windows-panes",
]
```

### Module Registration Flow

```
User runs: dotsmith add tmux

1. Check if tmux is installed
   → which tmux → /usr/bin/tmux ✓

2. Look for built-in module
   → data/modules/tmux/ exists → Tier 1 ✓

3. Find config files
   → ~/.config/tmux/tmux.conf exists ✓
   → Scan for additional sourced files (parse `source-file` directives)
   → Found: conf/settings.conf, conf/theme.conf, conf/keys.conf

4. Take initial snapshot
   → Store all config file contents in SQLite with timestamp

5. Load option database
   → Parse data/modules/tmux/options.toml
   → Cross-reference with user's actual config
   → Identify: 23 options set, 124 options available but unused

6. Update manifest
   → Add [tools.tmux] entry to manifest.toml

7. Report
   → "Added tmux (Tier 1 — full support)"
   → "Tracking 4 config files, 23 options configured, 124 to explore"
```

### Adding an Unknown Tool (Tier 2)

```
User runs: dotsmith add waybar

1. Check if waybar is installed
   → which waybar → /usr/bin/waybar ✓

2. Look for built-in module
   → data/modules/waybar/ does NOT exist

3. Auto-detect config files
   → Search: ~/.config/waybar/, ~/waybar, /etc/waybar/
   → Found: ~/.config/waybar/config.jsonc, ~/.config/waybar/style.css

4. Ask user to confirm
   → "Found config files for waybar:"
   → "  ~/.config/waybar/config.jsonc"
   → "  ~/.config/waybar/style.css"
   → "Track these? [Y/n]"

5. Take initial snapshot (same as Tier 1)

6. Update manifest with tier = 2

7. Report
   → "Added waybar (Tier 2 — basic tracking, no option database)"
   → "Tracking 2 config files"
   → "Tip: contribute a module at github.com/[you]/dotsmith"
```

---

## 8. Option Database — The Secret Sauce

### Data Structure (Rust)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionEntry {
    /// Option name as it appears in the config file
    pub name: String,

    /// Data type: boolean, string, integer, enum, color, path, list
    pub r#type: OptionType,

    /// Default value if not set
    pub default: Option<String>,

    /// Valid values for enum types
    pub values: Option<Vec<String>>,

    /// Category for TUI grouping (appearance, behavior, performance, etc.)
    pub category: String,

    /// One-line description of what this option does
    pub description: String,

    /// Longer explanation of why a user might want this
    pub why: Option<String>,

    /// Example usage in config file syntax
    pub example: Option<String>,

    /// Version this option was introduced
    pub since: Option<String>,

    /// Version this option was deprecated
    pub deprecated: Option<String>,

    /// Replacement option if deprecated
    pub replaced_by: Option<String>,

    /// Related options that are often set together
    pub related: Option<Vec<String>>,

    /// Tags for fuzzy search (e.g. "mouse", "scroll", "click")
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OptionType {
    Boolean,
    String,
    Integer,
    Float,
    Enum,
    Color,
    Path,
    List,
    KeyBinding,
}
```

### Option Database Query API

```rust
impl OptionDb {
    /// Get all options for a tool
    fn all_options(&self, tool: &str) -> Vec<&OptionEntry>;

    /// Get options the user IS using (present in their config)
    fn used_options(&self, tool: &str, config: &ParsedConfig) -> Vec<UsedOption>;

    /// Get options the user is NOT using (the discovery feature)
    fn unused_options(&self, tool: &str, config: &ParsedConfig) -> Vec<&OptionEntry>;

    /// Search options by name, description, tags (fuzzy)
    fn search(&self, tool: &str, query: &str) -> Vec<&OptionEntry>;

    /// Get options by category
    fn by_category(&self, tool: &str, category: &str) -> Vec<&OptionEntry>;

    /// Get deprecated options the user is still using
    fn deprecated_in_use(&self, tool: &str, config: &ParsedConfig) -> Vec<DeprecatedWarning>;
}
```

### Data Sources (Priority Order)

1. **Curated TOML files** — highest quality, ship with dotsmith for Tier 1 tools.
   Hand-written with clean descriptions, categories, "why" explanations, examples.

2. **Commented default configs** — many tools ship self-documenting configs
   (e.g. `/usr/share/doc/kitty/kitty.conf`). dotsmith parses comments as descriptions,
   uncommented values as defaults. Automated, surprisingly good quality.

3. **Man page parser** — run `mandoc -T ascii man_page | parse`. Extract option names
   and descriptions from structured man page sections. Automated, rougher quality.

4. **`--help` parser** — extract flags and descriptions. Better for CLI tools than
   config-file tools. Automated.

5. **Online docs scraper** (Tier 3, feature-gated) — user provides URL, dotsmith fetches
   HTML, parses into structured option data. Requires `docs-scraper` feature.

6. **LLM-assisted enrichment** (offline, one-time) — developer tool for bootstrapping
   option databases. Run LLM against raw docs, generate TOML, review and commit.
   End users never need an API key.

### Building Option Databases (Developer Workflow)

For bootstrapping Tier 1 modules, the developer (you) would:

```bash
# Step 1: Auto-generate from man page
dotsmith dev parse-man tmux > data/modules/tmux/options_raw.toml

# Step 2: Review, clean up, add "why" descriptions, categorize
# (This is where the human curation happens)
nvim data/modules/tmux/options.toml

# Step 3: Validate
dotsmith dev validate-module tmux

# Step 4: Test against your own config
dotsmith dev test-module tmux ~/.config/tmux/tmux.conf
```

---

## 9. Plugin Management — Built-in for zsh & tmux

### Philosophy

Inspired by [zsh_unplugged](https://github.com/mattmc3/zsh_unplugged): plugin managers are
over-engineered for what they do. A plugin is:

1. A git repository
2. That contains a file to source
3. Clone it, source it, done

dotsmith embeds this logic directly. No external plugin manager needed for zsh or tmux.

### For zsh

dotsmith replaces zplug/zinit/antidote/zap with ~100 lines of Rust logic:

```
dotsmith plugins zsh add zsh-users/zsh-autosuggestions
dotsmith plugins zsh add zsh-users/zsh-syntax-highlighting
dotsmith plugins zsh add sindresorhus/pure        # prompt theme
dotsmith plugins zsh list
dotsmith plugins zsh update
dotsmith plugins zsh remove zsh-autosuggestions
```

**What happens on `add`:**

```
1. git clone --depth 1 https://github.com/zsh-users/zsh-autosuggestions \
     ~/.config/dotsmith/plugins/zsh/zsh-autosuggestions

2. Detect init file:
   - *.plugin.zsh
   - *theme.zsh-theme
   - init.zsh
   - *.zsh (if only one)

3. Register in manifest:
   [tools.zsh.plugins]
   zsh-autosuggestions = { repo = "zsh-users/zsh-autosuggestions", init = "zsh-autosuggestions.plugin.zsh" }

4. Generate/update the plugin loader snippet that gets sourced in .zshrc:
   # ~/.config/dotsmith/plugins/zsh/loader.zsh (auto-generated by dotsmith)
   source ~/.config/dotsmith/plugins/zsh/zsh-autosuggestions/zsh-autosuggestions.plugin.zsh
   source ~/.config/dotsmith/plugins/zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.plugin.zsh

5. User adds ONE line to .zshrc:
   source ~/.config/dotsmith/plugins/zsh/loader.zsh
```

**What happens on `update`:**

```
For each registered plugin:
1. cd plugin_dir && git pull --depth 1
2. If plugin has *.zwc files, recompile with zcompile
3. Report: "Updated 3 plugins, 1 already up to date"
```

### For tmux

Same pattern, replaces TPM:

```
dotsmith plugins tmux add tmux-plugins/tmux-sensible
dotsmith plugins tmux add tmux-plugins/tmux-resurrect
dotsmith plugins tmux list
dotsmith plugins tmux update
```

**What happens on `add`:**

```
1. git clone --depth 1 https://github.com/tmux-plugins/tmux-sensible \
     ~/.config/dotsmith/plugins/tmux/tmux-sensible

2. Detect init file (tmux plugins always use *.tmux)

3. Register in manifest

4. Generate loader snippet:
   # ~/.config/dotsmith/plugins/tmux/loader.conf (auto-generated by dotsmith)
   run-shell ~/.config/dotsmith/plugins/tmux/tmux-sensible/sensible.tmux
   run-shell ~/.config/dotsmith/plugins/tmux/tmux-resurrect/resurrect.tmux

5. User adds ONE line to tmux.conf:
   source-file ~/.config/dotsmith/plugins/tmux/loader.conf
```

### For nvim — Integration, Not Replacement

dotsmith does NOT manage nvim plugins. lazy.nvim won that battle.

Instead, dotsmith **observes** lazy.nvim:
- Reads `lazy-lock.json` to know what plugins are installed
- Can show plugin list in the TUI dashboard
- Tracks changes to nvim config files like any other tool
- Option discovery covers `vim.opt.*` settings, not plugin settings

---

## 10. CLI Interface — Commands & Subcommands

```
dotsmith — The Dotfile Workbench

USAGE:
    dotsmith [COMMAND]
    dotsmith                        # No args → launch TUI dashboard

COMMANDS:
    add <tool> [--docs <url>]       Add a tool to dotsmith management
    remove <tool>                   Remove a tool from dotsmith management
    list                            List all managed tools with status
    status                          Show recent changes, warnings, plugin updates

    deploy                          Symlink all managed configs to their targets
    deploy <tool>                   Symlink a specific tool's config

    explore <tool>                  Launch TUI option explorer for a tool

    diff <tool>                     Show changes since last snapshot
    diff <tool> --between <a> <b>   Diff between two specific snapshots
    snapshot [--message "msg"]      Snapshot all tracked configs now
    history <tool>                  Show snapshot history for a tool
    rollback <tool> <snapshot-id>   Restore a tool's config to a snapshot

    reload <tool>                   Reload a tool's config live
    reload --all                    Reload all tools that support live reload

    plugins <tool> add <repo>       Add a plugin (zsh, tmux)
    plugins <tool> remove <name>    Remove a plugin
    plugins <tool> list             List installed plugins
    plugins <tool> update           Update all plugins for a tool
    plugins update                  Update ALL plugins across all tools

    init                            First-time setup wizard
    init --from <git-repo>          Clone and deploy an existing dotfiles repo

    completions <shell>             Generate shell completions (zsh, bash, fish)

OPTIONS:
    -v, --verbose                   Verbose output
    -q, --quiet                     Suppress non-essential output
    --config <path>                 Custom config file path
    -h, --help                      Show help
    -V, --version                   Show version
```

### Example Workflows

**New machine setup:**
```bash
# Install dotsmith
cargo install dotsmith

# Clone your dotfiles and deploy everything
dotsmith init --from https://github.com/you/dotfiles

# Or start fresh with sensible defaults
dotsmith init
dotsmith add tmux zsh kitty git ssh
dotsmith deploy
```

**Daily use:**
```bash
# Explore what tmux options you're missing
dotsmith explore tmux

# After tweaking tmux.conf
dotsmith reload tmux
dotsmith snapshot --message "added vim-style pane navigation"

# Check what changed today
dotsmith diff tmux

# Update all plugins
dotsmith plugins update
```

**Adding a new tool:**
```bash
# Tool with full support
dotsmith add kitty
# → "Added kitty (Tier 1 — 147 options tracked)"

# Tool without a module
dotsmith add waybar
# → "Added waybar (Tier 2 — basic tracking)"

# Tool with online docs
dotsmith add awesomewm --docs https://awesomewm.org/doc/api/
# → "Parsed 89 options from docs (Tier 3)"
```

---

## 11. TUI Interface — Exploration Mode

### Dashboard View (`dotsmith` with no args)

```
┌─ dotsmith ──────────────────────────────────────────────────────────┐
│                                                                      │
│  Managed Tools                              Recent Changes           │
│  ─────────────                              ──────────────           │
│  ● tmux     Tier 1  23/147 opts  2 plugins  today  theme.conf       │
│  ● zsh      Tier 1  18/89 opts   3 plugins  today  aliases.zsh      │
│  ● nvim     Tier 1  31/200 opts  (lazy.nvim) 3d ago init.lua        │
│  ● kitty    Tier 1  12/147 opts  —          1w ago kitty.conf        │
│  ● git      Tier 1   8/45 opts   —          2w ago .gitconfig        │
│  ● ssh      Tier 1   5/32 opts   —          1mo    config            │
│  ○ waybar   Tier 2  —            —          3d ago config.jsonc      │
│                                                                      │
│  Plugin Status                              Warnings                 │
│  ─────────────                              ────────                 │
│  zsh: 3 plugins, all up to date             ⚠ tmux: deprecated      │
│  tmux: 2 plugins, 1 update available          option 'utf8' (L14)   │
│                                             ⚠ zsh: duplicate alias  │
│                                               'gs' (L23, L47)       │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│ [a]dd tool  [e]xplore  [d]iff  [r]eload  [p]lugins  [s]napshot  [q]│
└──────────────────────────────────────────────────────────────────────┘
```

### Explore View (`dotsmith explore tmux`)

```
┌─ dotsmith explore: tmux ────────────────────────────────────────────┐
│                                                                      │
│  Categories        Options (interaction)      Details                │
│  ──────────        ────────────────────       ───────                │
│  appearance   (8)  ┌─ Your settings ──────┐   mouse                  │
│  behavior     (12) │ ✓ mouse          on  │   Type: boolean          │
│ >interaction  (6)  │ ✓ mouse-select.. on  │   Default: off           │
│  keybindings  (15) │                      │   Your value: on         │
│  performance  (4)  ├─ Unexplored ─────────┤                         │
│  status-bar   (18) │ ○ mouse-resize  off  │   Enable mouse support   │
│  windows      (14) │ ○ mouse-select.. on  │   for pane selection,    │
│  panes        (9)  │ ○ set-clipboard ext  │   resizing, and          │
│                    │ ○ exit-empty    on   │   scrolling.             │
│  ── Search ─────   └──────────────────────┘                         │
│  > _                                        Why: Essential for       │
│                    Status: 23 set, 124      beginners, still useful  │
│                    unexplored               for pros. Click panes,   │
│                                             scroll with mousewheel,  │
│                                             resize with drag.        │
│                                                                      │
│                                             Example:                 │
│                                             set -g mouse on          │
│                                                                      │
│                                             Since: tmux 2.1          │
│                                             Related: mouse-select-.. │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│ [↑↓] navigate  [enter] toggle/edit  [/] search  [a] apply  [r] rel │
│ [tab] switch pane  [e] edit config  [d] diff  [?] help  [q] back   │
└──────────────────────────────────────────────────────────────────────┘
```

### Key TUI Interactions

| Key | Action |
|-----|--------|
| `↑/↓` or `j/k` | Navigate options list |
| `Enter` | Toggle boolean option / edit value for other types |
| `/` | Fuzzy search across all options |
| `Tab` | Switch focus between panels |
| `a` | Apply: write pending changes to config file |
| `r` | Reload: execute tool's reload command |
| `e` | Open config file in inline editor |
| `d` | Show diff of pending changes |
| `u` | Undo last change |
| `?` | Help overlay with all keybindings |
| `q` / `Esc` | Go back / exit |

### TUI State Machine

```
Dashboard ──[e]──▶ Explore (tool) ──[e]──▶ Editor
    │                    │                     │
    │                    │◀──────[q]───────────┘
    │                    │
    │◀──────[q]──────────┘
    │
    ├──[d]──▶ Diff view ──[q]──▶ back
    ├──[p]──▶ Plugins view ──[q]──▶ back
    └──[q]──▶ exit
```

---

## 12. Snapshot & History Engine

### SQLite Schema

```sql
CREATE TABLE snapshots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tool        TEXT NOT NULL,           -- "tmux", "zsh", etc.
    file_path   TEXT NOT NULL,           -- absolute path of the config file
    content     TEXT NOT NULL,           -- full file content at snapshot time
    hash        TEXT NOT NULL,           -- sha256 of content (for dedup)
    message     TEXT,                    -- optional annotation
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),

    -- Index for fast queries
    UNIQUE(tool, file_path, hash)       -- prevent duplicate snapshots
);

CREATE INDEX idx_snapshots_tool ON snapshots(tool);
CREATE INDEX idx_snapshots_created ON snapshots(created_at);

CREATE TABLE plugin_state (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tool        TEXT NOT NULL,           -- "zsh", "tmux"
    plugin_name TEXT NOT NULL,
    repo        TEXT NOT NULL,           -- "zsh-users/zsh-autosuggestions"
    commit_hash TEXT,                    -- current git commit
    init_file   TEXT,                    -- relative path to init file
    added_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(tool, plugin_name)
);
```

### Snapshot Operations

```rust
impl SnapshotEngine {
    /// Take a snapshot of all config files for a tool
    fn snapshot(&self, tool: &str, message: Option<&str>) -> Result<SnapshotId>;

    /// Take snapshots of ALL tracked tools
    fn snapshot_all(&self, message: Option<&str>) -> Result<Vec<SnapshotId>>;

    /// Get diff between current state and last snapshot
    fn diff_current(&self, tool: &str) -> Result<Vec<FileDiff>>;

    /// Get diff between two specific snapshots
    fn diff_between(&self, a: SnapshotId, b: SnapshotId) -> Result<Vec<FileDiff>>;

    /// List snapshot history for a tool
    fn history(&self, tool: &str, limit: usize) -> Result<Vec<SnapshotSummary>>;

    /// Rollback to a specific snapshot
    fn rollback(&self, snapshot_id: SnapshotId) -> Result<()>;

    /// Auto-snapshot on file change (called by file watcher)
    fn auto_snapshot(&self, tool: &str, changed_file: &Path) -> Result<()>;
}
```

### Auto-Snapshot Behavior

When dotsmith is running (TUI mode), it watches tracked config files via the `notify`
crate. On any change:

1. Wait 2 seconds for debounce (user might be mid-edit)
2. Compare file content hash to last snapshot
3. If different, create auto-snapshot with message "auto: [filename] changed"
4. Update TUI status bar: "tmux.conf changed (auto-saved)"

This means the user never loses a config state, even without manually snapshotting.

---

## 13. Reload Engine — Live Config Application

### Per-Tool Reload Commands

| Tool | Reload Command | Notes |
|------|---------------|-------|
| tmux | `tmux source-file {path}` | Works if tmux is running |
| zsh | `source {path}` | Tricky — needs to run in the user's shell. See below. |
| nvim | `nvim --server /tmp/nvim.sock --remote-send ':source $MYVIMRC<CR>'` | Requires `--listen` on nvim |
| git | (none needed) | Git reads config on every command |
| ssh | (none needed) | SSH reads config per connection |
| kitty | `kill -SIGUSR1 $(pgrep kitty)` | Kitty reloads on SIGUSR1 |
| alacritty | (auto-reloads on file change) | No action needed |

### The zsh Reload Problem

You can't `source ~/.zshrc` from a subprocess — it would run in dotsmith's shell, not
the user's. Solutions:

1. **Print instructions:** "Run `source ~/.zshrc` in your terminal to apply changes"
2. **Write a flag file:** dotsmith writes `~/.config/dotsmith/.zsh-reload-pending`,
   a zsh hook checks for it on each prompt and auto-sources
3. **Use zsh's SIGUSR1:** send signal to the user's zsh process, which has a trap
   that re-sources the config

Option 2 is recommended — requires a one-line addition to the user's .zshrc:

```zsh
# Add to .zshrc (done automatically by dotsmith init)
precmd() {
    if [[ -f ~/.config/dotsmith/.zsh-reload-pending ]]; then
        source ~/.zshrc
        rm ~/.config/dotsmith/.zsh-reload-pending
    fi
}
```

### Reload Result Feedback

```rust
pub enum ReloadResult {
    Success { tool: String, message: String },
    NotRunning { tool: String },           // tmux not running, etc.
    NotSupported { tool: String },         // git, ssh — no reload needed
    Pending { tool: String, instruction: String },  // zsh — user needs to act
    Error { tool: String, error: String },
}
```

---

## 14. Config Deployment — Symlink Management

### How Deployment Works

dotsmith manages a **source directory** (where your config files live, version-controlled)
and creates **symlinks** to where tools expect them.

```
Source (version-controlled):          Target (where tools read from):
~/.config/dotsmith/configs/           symlinked into place
├── tmux/                             ~/.config/tmux/ → symlink
│   ├── tmux.conf
│   └── conf/
│       ├── settings.conf
│       ├── theme.conf
│       └── keys.conf
├── zsh/                              ~/.config/zsh/ → symlink
│   ├── .zshrc
│   └── utils/
├── kitty/                            ~/.config/kitty/ → symlink
│   └── kitty.conf
└── nvim/                             ~/.config/nvim/ → symlink
    ├── init.lua
    └── lua/
```

### Deploy Command

```
dotsmith deploy
```

1. For each tool in manifest:
   - Check if target path exists
   - If it's a regular file/dir (not a symlink): back it up to `~/.config/dotsmith/backups/`
   - Create symlink: target → source
   - Verify symlink works
2. Report: "Deployed 7 tools, backed up 2 existing configs"

### Integration with Existing dotfile Repos

```
dotsmith init --from https://github.com/you/dotfiles
```

1. Clone the repo to `~/.config/dotsmith/configs/`
2. Auto-detect tool directories inside
3. Register each as a managed tool
4. Run `dotsmith deploy` to symlink everything
5. Run `dotsmith plugins zsh install` / `dotsmith plugins tmux install` if plugins are listed

This means your existing `~/.config/oz/dots/` repo structure would work — dotsmith
just needs to know where the source files are.

---

## 15. Tiered Support Model

### Tier Summary

```
Tier 1: FULL SUPPORT (ships with dotsmith)
├── Curated option database (options.toml with descriptions, "why", examples)
├── Config file parser (understands the syntax)
├── Reload command
├── Plugin management (if applicable)
├── Linting / validation rules
├── Default config template
└── Tools: tmux, zsh, nvim, git, ssh, kitty, alacritty

Tier 2: AUTO-DETECTED (works immediately)
├── Config file tracking (snapshots, diffs, rollback)
├── Change annotations
├── No option discovery (no option database)
├── No reload (user reloads manually)
└── Tools: anything with config files in ~/.config/ or ~/

Tier 3: USER-ENRICHED (user provides docs)
├── Everything in Tier 2, PLUS:
├── Option database generated from:
│   ├── Online docs URL (--docs flag)
│   ├── Man page (auto-detected)
│   └── Commented default config (auto-detected)
├── Generated options.toml placed in ~/.config/dotsmith/modules/<tool>/
├── User can review and edit the generated data
└── Tools: anything with documentation somewhere

Tier 4: COMMUNITY-CONTRIBUTED (PR to repo)
├── Someone writes a module.toml + options.toml
├── Submitted via GitHub PR
├── Reviewed, merged → ships in next release as Tier 1
└── Like tldr pages: community builds the knowledge base
```

### Tier Promotion Path

```
Tool starts at Tier 2 (auto-detected)
    │
    ├── User runs: dotsmith enrich <tool> --man
    │   → parses man page → promotes to Tier 3
    │
    ├── User runs: dotsmith enrich <tool> --docs <url>
    │   → scrapes docs → promotes to Tier 3
    │
    ├── User runs: dotsmith enrich <tool> --default-config /usr/share/doc/tool/tool.conf
    │   → parses commented config → promotes to Tier 3
    │
    └── Community submits module PR → merged → promoted to Tier 1
```

---

## 16. MVP Scope — What Ships First

### MVP = version 0.1.0

**Goal:** Usable daily by you. Covers your actual setup.

#### Must Have (MVP)

- [ ] `dotsmith init` — create config directory structure
- [ ] `dotsmith add <tool>` — register a tool (Tier 1 and Tier 2)
- [ ] `dotsmith remove <tool>` — unregister a tool
- [ ] `dotsmith list` — show all managed tools
- [ ] `dotsmith status` — show recent changes and warnings
- [ ] `dotsmith deploy` — symlink configs to target locations
- [ ] `dotsmith diff <tool>` — show changes since last snapshot (CLI output)
- [ ] `dotsmith snapshot` — take a manual snapshot with optional message
- [ ] `dotsmith history <tool>` — show snapshot history
- [ ] `dotsmith rollback <tool> <id>` — restore a snapshot
- [ ] `dotsmith reload <tool>` — reload config for tmux, kitty
- [ ] `dotsmith plugins zsh add/remove/list/update` — zsh plugin management
- [ ] `dotsmith plugins tmux add/remove/list/update` — tmux plugin management
- [ ] `dotsmith explore <tool>` — TUI option explorer (the killer feature)
- [ ] `dotsmith` (no args) — TUI dashboard
- [ ] Tier 1 modules: tmux, zsh, git (3 tools with full option databases)
- [ ] Tier 2 auto-detection for any other tool
- [ ] SQLite snapshot storage
- [ ] Shell completions (zsh, bash)

#### Won't Have (MVP)

- Online docs scraper (Tier 3)
- LLM enrichment
- Config linting / validation
- Config profiles (work vs personal)
- Cross-machine sync
- Inline TUI editor (just the explorer)
- Default config templates
- `dotsmith init --from <repo>`
- File watching / auto-snapshots

### MVP Development Phases

```
Phase 1: Foundation (week 1-2)
├── Cargo project setup with clap CLI skeleton
├── Config directory structure (~/.config/dotsmith/)
├── Manifest read/write (manifest.toml)
├── Module system (load module.toml + options.toml)
├── `dotsmith init`, `dotsmith add`, `dotsmith list`
└── First module: tmux (module.toml + options.toml)

Phase 2: Core Engine (week 3-4)
├── SQLite setup + snapshot engine
├── `dotsmith snapshot`, `dotsmith history`, `dotsmith diff`
├── `dotsmith rollback`
├── `dotsmith deploy` (symlink management)
├── `dotsmith reload` (tmux, kitty)
├── Config file parsers: tmux, shell (zsh), ini (git)
└── Second module: zsh (module.toml + options.toml)

Phase 3: Plugin Management (week 5)
├── Git clone/pull helpers
├── Plugin init file detection
├── Loader file generation (zsh, tmux)
├── `dotsmith plugins zsh add/remove/list/update`
├── `dotsmith plugins tmux add/remove/list/update`
└── Third module: git (module.toml + options.toml)

Phase 4: TUI (week 6-8)
├── ratatui app skeleton + event loop
├── Dashboard view (overview of all tools)
├── Explore view (option discovery with categories)
├── Diff view (visual diff between snapshots)
├── Search (fuzzy option search)
├── Keybinding system
└── Theme / styling

Phase 5: Polish & Ship (week 9-10)
├── Shell completions (zsh, bash)
├── Error handling and user-facing messages
├── README with screenshots / GIF demos
├── AUR PKGBUILD
├── cargo publish
├── Testing (unit + integration)
└── CHANGELOG.md
```

---

## 17. Post-MVP Roadmap

### v0.2.0 — More Tools, More Intelligence

- [ ] Tier 1 modules: ssh, kitty, alacritty, nvim
- [ ] Config linting / validation (deprecated options, conflicts, typos)
- [ ] Inline TUI editor with context tooltips
- [ ] File watching + auto-snapshots
- [ ] `dotsmith init --from <repo>` (clone and deploy existing dotfiles)
- [ ] Tier 2 auto-detection improvements (scan ~/.config/ for all tools)

### v0.3.0 — Enrichment & Community

- [ ] Tier 3: `dotsmith enrich <tool> --docs <url>` (online docs scraper)
- [ ] Tier 3: `dotsmith enrich <tool> --man` (man page parser)
- [ ] Tier 3: `dotsmith enrich <tool> --default-config <path>` (commented config parser)
- [ ] Community module contribution workflow (CONTRIBUTING.md, CI validation)
- [ ] More Tier 1 modules from community PRs

### v0.4.0 — Profiles & Sync

- [ ] Config profiles (work, personal, minimal, presentation)
- [ ] `dotsmith profile <name>` — switch profiles
- [ ] Git-based sync (push/pull dotfiles repo)
- [ ] Cross-machine comparison ("this machine has X, that one doesn't")

### v0.5.0 — Wizards & Onboarding

- [ ] Guided setup wizards ("you installed tmux, here are the 10 most popular settings")
- [ ] Config snippets (curated, community-contributed, importable)
- [ ] "What's new in tmux 3.5" — version-aware option discovery
- [ ] Default config templates (deploy sensible defaults for new tools)

### v1.0.0 — Stable Release

- [ ] All Tier 1 modules covering top 15 tools
- [ ] Stable CLI interface (no breaking changes after 1.0)
- [ ] Stable module spec (community can rely on it)
- [ ] Stable TUI interface
- [ ] Comprehensive documentation
- [ ] Homebrew, Nix flake, Debian package distribution

---

## 18. Data Flow Diagrams

### `dotsmith add tmux`

```
User ──▶ CLI (add.rs)
           │
           ├── detect.rs: is tmux installed? ──▶ `which tmux` ──▶ yes
           │
           ├── module.rs: do we have a module?
           │   └── check data/modules/tmux/ ──▶ yes (Tier 1)
           │
           ├── module.rs: find config files
           │   └── check config_paths from module.toml
           │   └── ~/.config/tmux/tmux.conf exists ──▶ yes
           │   └── scan for source-file directives ──▶ conf/*.conf
           │
           ├── snapshot.rs: take initial snapshot
           │   └── read all config files
           │   └── store in SQLite with hash + timestamp
           │
           ├── option_db.rs: load option database
           │   └── parse data/modules/tmux/options.toml
           │   └── cross-reference with user's config
           │   └── result: 23 used, 124 unused
           │
           ├── manifest.rs: update manifest.toml
           │   └── add [tools.tmux] entry
           │
           └── output: "Added tmux (Tier 1 — 23/147 options configured)"
```

### `dotsmith explore tmux`

```
User ──▶ CLI (explore.rs) ──▶ TUI app.rs
           │
           ├── Load module + option DB
           │   └── All 147 options with metadata
           │
           ├── Parse user's tmux config
           │   └── Extract all set options with values
           │
           ├── Cross-reference
           │   ├── Used options (with current values)
           │   └── Unused options (available to explore)
           │
           ├── Render TUI
           │   ├── Left: categories
           │   ├── Center: option list (used ✓ / unused ○)
           │   └── Right: selected option details
           │
           └── User interaction loop
               ├── Navigate, search, toggle
               ├── On toggle: stage change (don't write yet)
               ├── On [a]pply: write changes to config file
               ├── On [r]eload: execute reload command
               └── On [q]: exit, snapshot if changes were made
```

### `dotsmith plugins zsh add zsh-users/zsh-autosuggestions`

```
User ──▶ CLI (plugins.rs)
           │
           ├── Validate: is zsh a managed tool? ──▶ check manifest ──▶ yes
           │
           ├── git.rs: clone plugin
           │   └── git clone --depth 1 https://github.com/zsh-users/zsh-autosuggestions
           │       ~/.config/dotsmith/plugins/zsh/zsh-autosuggestions
           │
           ├── plugin.rs: detect init file
           │   └── scan for *.plugin.zsh, *.zsh-theme, init.zsh
           │   └── found: zsh-autosuggestions.plugin.zsh
           │
           ├── plugin.rs: update manifest
           │   └── add to [tools.zsh.plugins]
           │
           ├── plugin.rs: regenerate loader.zsh
           │   └── write source lines for all registered plugins
           │
           ├── SQLite: record plugin state
           │   └── store repo, commit hash, init file, timestamp
           │
           └── output: "Added zsh-autosuggestions"
               "Reminder: add `source ~/.config/dotsmith/plugins/zsh/loader.zsh` to .zshrc"
```

---

## 19. File Formats — All Specs

### dotsmith Config (`~/.config/dotsmith/config.toml`)

```toml
# dotsmith's own configuration

[general]
# Where dotsmith stores its data
data_dir = "~/.config/dotsmith"

# Where managed config source files live (for deployment)
configs_dir = "~/.config/dotsmith/configs"

# Auto-snapshot when changes are detected (requires file watcher)
auto_snapshot = true

# Debounce time for auto-snapshots (seconds)
auto_snapshot_debounce = 2

[tui]
# Color scheme: "default", "catppuccin", "tokyonight", "gruvbox"
theme = "default"

# Keybinding preset: "default", "vim", "emacs"
keybindings = "vim"

# Show line numbers in editor
line_numbers = true

[deploy]
# Backup existing configs before symlinking
backup = true
backup_dir = "~/.config/dotsmith/backups"

[plugins]
# Default plugin directory
plugin_dir = "~/.config/dotsmith/plugins"

# Shallow clone plugins (--depth 1)
shallow_clone = true
```

### Manifest (`~/.config/dotsmith/manifest.toml`)

```toml
# Tracks all tools managed by dotsmith
# Auto-managed by `dotsmith add/remove`, safe to edit by hand

[tools.tmux]
tier = 1
config_paths = ["~/.config/tmux/tmux.conf", "~/.config/tmux/conf/"]
plugins_managed = true
added_at = "2026-02-07T14:30:00Z"
last_snapshot = "2026-02-07T18:45:00Z"

[tools.tmux.plugins]
tmux-online-status = { repo = "tmux-plugins/tmux-online-status", init = "online_status.tmux" }

[tools.zsh]
tier = 1
config_paths = ["~/.config/zsh/.zshrc", "~/.config/zsh/utils/"]
plugins_managed = true
added_at = "2026-02-07T14:30:00Z"
last_snapshot = "2026-02-07T18:45:00Z"

[tools.zsh.plugins]
zsh-autosuggestions = { repo = "zsh-users/zsh-autosuggestions", init = "zsh-autosuggestions.plugin.zsh" }
zsh-syntax-highlighting = { repo = "zsh-users/zsh-syntax-highlighting", init = "zsh-syntax-highlighting.plugin.zsh" }

[tools.nvim]
tier = 1
config_paths = ["~/.config/nvim/"]
plugins_managed = false
plugin_manager = "lazy"
added_at = "2026-02-07T14:30:00Z"
last_snapshot = "2026-02-07T16:00:00Z"

[tools.git]
tier = 1
config_paths = ["~/.gitconfig", "~/.config/git/config"]
plugins_managed = false
added_at = "2026-02-07T14:30:00Z"
last_snapshot = "2026-02-07T14:30:00Z"

[tools.waybar]
tier = 2
config_paths = ["~/.config/waybar/config.jsonc", "~/.config/waybar/style.css"]
plugins_managed = false
added_at = "2026-02-08T10:00:00Z"
last_snapshot = "2026-02-08T10:00:00Z"

[tools.awesomewm]
tier = 3
config_paths = ["~/.config/awesome/rc.lua"]
docs_url = "https://awesomewm.org/doc/api/"
plugins_managed = false
added_at = "2026-02-09T12:00:00Z"
last_snapshot = "2026-02-09T12:00:00Z"
```

### Module Definition (`data/modules/<tool>/module.toml`)

```toml
# See Section 7 for full spec
[metadata]
name = "tmux"
display_name = "tmux"
description = "Terminal multiplexer"
homepage = "https://github.com/tmux/tmux"
config_paths = ["~/.config/tmux/tmux.conf", "~/.tmux.conf"]
detect_command = "which tmux"
reload_command = "tmux source-file {config_path}"
man_page = "tmux"
config_format = "tmux"
plugins_supported = true
plugin_dir = "~/.config/dotsmith/plugins/tmux"
categories = ["appearance", "behavior", "interaction", "keybindings",
              "performance", "status-bar", "windows-panes"]
```

### Option Database (`data/modules/<tool>/options.toml`)

```toml
# See Section 8 for full spec
# Each [[options]] entry defines one config option

[[options]]
name = "mouse"
type = "boolean"
default = "off"
category = "interaction"
description = "Enable mouse support for pane selection, resizing, and scrolling"
why = "Essential for beginners, still useful for pros."
example = "set -g mouse on"
since = "2.1"
tags = ["mouse", "scroll", "click", "resize"]
related = ["mouse-select-pane", "mouse-select-window", "mouse-resize-pane"]
```

### Plugin Loader (`~/.config/dotsmith/plugins/zsh/loader.zsh`)

```zsh
# Auto-generated by dotsmith — do not edit manually
# Managed plugins for zsh
# Last updated: 2026-02-07T18:45:00Z

source ~/.config/dotsmith/plugins/zsh/zsh-autosuggestions/zsh-autosuggestions.plugin.zsh
source ~/.config/dotsmith/plugins/zsh/zsh-syntax-highlighting/zsh-syntax-highlighting.plugin.zsh
```

### Plugin Loader (`~/.config/dotsmith/plugins/tmux/loader.conf`)

```tmux
# Auto-generated by dotsmith — do not edit manually
# Managed plugins for tmux
# Last updated: 2026-02-07T18:45:00Z

run-shell ~/.config/dotsmith/plugins/tmux/tmux-online-status/online_status.tmux
```

---

## 20. Build, Test & Distribution

### Building

```bash
# Development build
cargo build

# Release build (optimized, stripped)
cargo build --release

# With docs scraper feature
cargo build --release --features docs-scraper

# Cross-compile for other targets
cargo build --release --target x86_64-unknown-linux-musl  # static binary
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests (requires test fixtures)
cargo test --test integration

# Run with test config directory (doesn't touch real configs)
DOTSMITH_CONFIG_DIR=/tmp/dotsmith-test cargo run -- add tmux
```

### Test Strategy

- **Unit tests:** parsers, option DB queries, manifest read/write, snapshot diffing
- **Integration tests:** CLI commands with temp directories, TUI rendering (snapshot tests)
- **Fixture data:** sample config files for each supported tool in `tests/fixtures/`
- **No external dependencies in tests:** mock git, mock file system where needed

### Distribution

| Method | Command | Notes |
|--------|---------|-------|
| **cargo install** | `cargo install dotsmith` | Primary distribution |
| **AUR** | `yay -S dotsmith` | Your distro, priority |
| **Homebrew** | `brew install dotsmith` | macOS + Linux |
| **Nix** | `nix profile install dotsmith` | NixOS users |
| **GitHub Releases** | Download binary | Pre-built for linux-x86_64, linux-aarch64 |
| **Docker** | (not needed) | This is a local tool, not a service |

### CI/CD (GitHub Actions)

```yaml
# On push to main:
- cargo fmt --check
- cargo clippy -- -D warnings
- cargo test
- cargo build --release

# On tag (v*):
- Build release binaries (x86_64, aarch64)
- Create GitHub Release with binaries
- Publish to crates.io
- Update AUR PKGBUILD
```

---

## 21. Community & Growth Strategy

### Launch Plan

1. **Dogfood** — use it yourself for 2+ weeks before announcing
2. **README** — clear, concise, with GIF demos of the TUI explore mode
3. **Post to:**
   - r/commandline ("I built a TUI for exploring your dotfile options")
   - r/linux
   - r/unixporn (screenshot of the TUI — this audience loves pretty terminals)
   - r/neovim, r/zsh, r/tmux (tool-specific audiences)
   - Hacker News
   - Lobste.rs
   - Dev.to
4. **GIF/video demo** — the TUI explore view is the hook. Show someone discovering
   tmux options they didn't know existed.

### Community Module Contributions

The #1 way dotsmith grows is through community-contributed modules:

1. Write `MODULE_SPEC.md` — clear, simple spec for writing a module
2. Provide a template: `dotsmith dev new-module <tool>`
3. CI validates module PRs automatically (valid TOML, required fields, etc.)
4. Reviewers check quality of descriptions and categorization
5. Merged modules ship in next release as Tier 1

This is exactly the tldr-pages model — 55k+ stars, 2000+ contributors, built entirely
on community-contributed content.

### Funding

- GitHub Sponsors (primary)
- Buy Me a Coffee
- Ko-fi
- No paid tiers, no premium features — everything is free and open source
- Funding goes to: your time maintaining, reviewing module PRs, building new features

---

## 22. Open Questions & Decisions

### Resolved

- [x] **Language:** Rust
- [x] **TUI library:** ratatui
- [x] **CLI library:** clap
- [x] **Storage:** SQLite
- [x] **Interface model:** CLI-first + TUI for exploration
- [x] **Plugin management:** Built-in for zsh/tmux, integrate with lazy.nvim
- [x] **Module system:** TOML-based, tiered support

### To Decide During Implementation

- [ ] **Config source directory:** Should dotsmith manage its own copy of configs
  (like chezmoi) or just track configs in-place (like yadm)?
  - Option A: Copy configs to `~/.config/dotsmith/configs/`, symlink out (safer, git-friendly)
  - Option B: Track configs where they are (simpler, less duplication)
  - Leaning: Option A for deployed configs, but support both

- [ ] **zsh reload mechanism:** precmd hook vs SIGUSR1 vs print instructions
  - Leaning: precmd hook (simplest, most reliable)

- [ ] **How to handle config includes/sources:** tmux has `source-file`, zsh has `source`,
  nvim has `require`. Should dotsmith follow these and track the included files?
  - Leaning: Yes, recursively discover included files. Essential for your setup where
    tmux.conf sources conf/*.conf

- [ ] **Option DB format:** TOML vs SQLite
  - Leaning: TOML files in repo (human-editable, git-diffable), cached in SQLite at runtime

- [ ] **Should `dotsmith explore` allow direct editing or just show info?**
  - Leaning: Start with read-only exploration, add editing in v0.2.0

- [ ] **Binary size budget:** How large is acceptable?
  - ratatui + clap + rusqlite(bundled) + syntect → probably 10-15MB release
  - Acceptable for a cargo install tool
  - Can strip with `strip = true` in release profile

---

## Summary

dotsmith is buildable, fills a genuine gap, and your existing dotfiles setup
(`~/.config/oz/dots/`) is the prototype. The MVP is focused: 3 Tier 1 tools
(tmux, zsh, git), CLI + TUI, plugin management, snapshots, and the option
explorer. Everything else is post-MVP.

Start with Phase 1: `cargo init dotsmith`, clap skeleton, manifest system, first module.
