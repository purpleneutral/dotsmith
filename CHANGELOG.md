# Changelog

All notable changes to dotsmith will be documented in this file.

## [0.1.0-alpha.8] - 2026-02-10

### Added

- **Phase 7: Configuration profiles + remote deploy**
  - `dotsmith profile save <name>` — save current tracked tools and config file contents as a named profile
  - `dotsmith profile load <name>` — restore config files from a saved profile (backs up existing files first)
  - `dotsmith profile load --add-untracked` — also add tools from the profile that aren't currently tracked
  - `dotsmith profile load --dry-run` — preview what loading would change without modifying anything
  - `dotsmith profile list` — list saved profiles with creation date, tool count, and file count
  - `dotsmith profile delete <name>` — delete a saved profile
  - Profiles stored at `~/.config/dotsmith/profiles/<name>/` with full manifest and file contents
  - SHA-256 checksums for profile integrity verification
  - `dotsmith deploy-remote <host>` — deploy tracked configs to a remote host via SSH/SCP
  - `dotsmith deploy-remote --dry-run` — preview what would be copied
  - `dotsmith deploy-remote --tool <name>` — deploy only specific tools
  - `dotsmith deploy-remote --user <user>` — specify SSH user
  - Remote files are backed up before overwriting (`.dotsmith-bak.<timestamp>`)
  - Uses system `ssh`/`scp` commands — respects `~/.ssh/config` (aliases, ProxyJump, agent forwarding)

## [0.1.0-alpha.7] - 2026-02-10

### Added

- **Phase 5b: edit, watch, validate, man page**
  - `dotsmith edit <tool>` — open config in $EDITOR with automatic pre-edit snapshot and change detection
  - `dotsmith watch [tool]` — poll-based file watching with auto-snapshot on save (2s interval, Ctrl-C to stop)
  - Config syntax validation in `dotsmith doctor` — validates TOML, key-value (kitty), git INI, and tmux formats
  - Man page generation via `clap_mangen` — hidden `mangen` subcommand, `make man` target
  - `make install` now automatically installs the man page to `$PREFIX/share/man/man1/`
  - `install.sh` now installs the man page alongside the binary

## [0.1.0-alpha.6] - 2026-02-10

### Added

- **Quick Wins: doctor, search, config generation**
  - `dotsmith doctor [tool]` — deep health check: installation status, config paths, snapshot freshness, actionable hints
  - `dotsmith search <query>` — search across all 220+ Tier 1 options from the CLI (matches names, descriptions, categories, tags)
  - Config generation in TUI explore: press `g` to generate a commented config snippet file at `~/.config/dotsmith/generated/<tool>.<ext>`
  - Generated files include all visible options with descriptions, types, defaults, and examples (all commented)
  - Filter by category or search first to generate focused config snippets

## [0.1.0-alpha.5] - 2026-02-10

### Added

- **Phase 5a: Shell Completions + New Tier 1 Modules**
  - `dotsmith completions <shell>` command for bash, zsh, and fish via `clap_complete`
  - kitty Tier 1 module: 31 options across appearance, fonts, cursor, scrollback, mouse, performance, tabs, keybindings
  - neovim Tier 1 module: 31 options across ui, editing, search, indentation, completion, lsp, performance, files
  - alacritty Tier 1 module: 31 options across window, font, colors, cursor, scrolling, shell, keybindings, hints
  - awesomewm Tier 1 module: 31 options across general, tags, layouts, keybindings, rules, wibar, themes, notifications
  - Starship prompt integration: 2 options added to zsh module (STARSHIP_CONFIG, starship init)
  - Reload support: alacritty (auto-reloads), awesomewm (awesome-client), neovim (interactive message)
  - Neovim plugin detection: `dotsmith add neovim` now detects lazy.nvim

### Fixed

- Replaced `unwrap()` calls in CLI reload and diff commands with proper error handling
- Simplified lifetime workaround in diff command using `get_key_value()`

### Changed

- Release workflow now builds for Linux x86_64, Linux aarch64, macOS x86_64, and macOS aarch64
- Tier 1 modules expanded from 3 to 7 (93 → 220 curated options)

## [0.1.0-alpha.4] - 2026-02-08

### Added

- **Phase 4b: Full TUI Integration**
  - Status bar with mode indicator, tool name, and toast notifications (auto-expire after 3s)
  - Dashboard quick actions: `s` snapshot all, `r` reload, `d` diff, `h` history, `p` plugins, `g` sync repo
  - Diff view: scrollable colored unified diff (j/k scroll, d/u page, g/G top/bottom)
  - History view: snapshot table with `Enter` to view diff, `r` to rollback
  - Plugin view: list/add/remove/update plugins from TUI (supports add input mode)
  - Explore quick actions: `s` snapshot, `r` reload current tool
  - Git repo management: `dotsmith repo init <path>`, `dotsmith repo sync`, `dotsmith repo status`
  - Config file gains `repo_path` setting for persistent repo location
  - `DotsmithConfig::load()/save()` methods for config file management

## [0.1.0-alpha.3] - 2026-02-08

### Added

- **Phase 4: TUI Dashboard and Option Explorer**
  - Interactive TUI built with ratatui 0.29 + crossterm 0.28
  - Dashboard view: overview of all tracked tools with tier, paths, plugins, last snapshot
  - Option explorer: three-panel layout (categories | options | details) for Tier 1 tools
  - `dotsmith` with no subcommand launches the dashboard
  - `dotsmith explore <tool>` opens the explorer directly for a tool
  - Keyboard navigation: j/k, Tab panel cycling, / for search, Esc to go back, q to quit
  - Search filters options by name, description, and tags (case-insensitive)
  - Category filtering narrows options to a specific group
  - Panic hook ensures terminal is always restored cleanly
  - git Tier 1 module: 31 curated options across 8 categories

### Fixed

- Integration tests that require tool-specific configs (tmux) now skip gracefully on CI

## [0.1.0-alpha.2] - 2026-02-08

### Added

- **Phase 3: Plugin Management**
  - Built-in plugin management for zsh and tmux (replaces tpm, zinit, etc.)
  - `plugins <tool> add <repo>` -- clone with `--depth 1`, auto-detect init file, register in manifest
  - `plugins <tool> remove <name>` -- remove plugin directory and manifest entry
  - `plugins <tool> list` -- show installed plugins with repo and init file
  - `plugins <tool> update [name]` -- `git pull --ff-only` with zwc recompile for zsh
  - GitHub shorthand support (`user/repo`) and full HTTPS/file URL support
  - Auto-generated loader files (`loader.zsh` / `loader.conf`) -- source once in your rc file
  - Init file detection: `*.plugin.zsh`, `*.zsh-theme`, `init.zsh`, `*.tmux`
  - Backward-compatible manifest extension (`PluginEntry` under `[tools.<tool>.plugins]`)
  - Cleanup on failure (removes cloned directory if init file detection fails)

## [0.1.0-alpha.1] - 2026-02-08

### Added

- **Phase 1: Foundation**
  - CLI skeleton with clap: `init`, `add`, `remove`, `list`, `status`
  - Manifest system for tracking tools (`manifest.toml`)
  - Module system with `include_str!` embedded Tier 1 data
  - Symlink-aware config detection (records user-facing paths, not targets)
  - Plugin manager detection (TPM, zinix-mgr, oh-my-zsh, lazy.nvim)
  - Tier 2 auto-detection for any tool
  - tmux Tier 1 module with 31 curated options
  - Path safety checks (symlinks must resolve within `$HOME`)
  - Atomic file writes with `0600` permissions

- **Phase 2: Snapshots and Management**
  - SQLite snapshot engine with SHA-256 content dedup and WAL mode
  - `snapshot` command with per-tool or all-tools support
  - `history` command with snapshot listing
  - `diff` command with colored unified output (via `similar` crate)
  - `rollback` command with automatic backup and `--dry-run`
  - `deploy` command for symlink creation with backup and `--dry-run`
  - `reload` command (tmux source-file, kitty SIGUSR1, i3/sway reload)
  - zsh Tier 1 module with 31 curated options
  - Shared `atomic_write` utility

- **Install System**
  - Makefile with `install`, `uninstall`, `test`, `check`, `clean` targets
  - Smart `install.sh` (prebuilt binary download with source build fallback)
  - GitHub Actions CI (clippy + tests on push/PR)
  - GitHub Actions release workflow (binary on tag push)
